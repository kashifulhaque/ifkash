<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  // --- Configuration ---
  const REFRESH_WEATHER_MS = 30 * 60 * 1000; // 30 mins
  const REFRESH_F1_MS = 60 * 60 * 1000;      // 1 hour

  // --- State ---
  let now = new Date();
  let timeFormatter: Intl.DateTimeFormat;
  let dateFormatter: Intl.DateTimeFormat;

  // Weather State
  let weather = {
    temp: "--",
    min: "--",
    max: "--",
    desc: "Loading...",
    iconCode: -1,
  };
  let weatherError = "";

  // F1 State
  let f1 = {
    loading: true,
    next: null as any,
    last: null as any,
  };

  // Prayer State
  let prayer = {
    nextName: "--",
    nextTime: "--",
    countdown: 0, // minutes
    prevName: "--",
    prevTime: "--",
    loading: true,
  };

  // Timers
  let clockTimer: number;
  let weatherTimer: number;
  let f1Timer: number;
  let prayerTimer: number;

  // --- Helpers ---
  function pad(n: number) {
    return n < 10 ? `0${n}` : `${n}`;
  }

  // Formatters
  function initFormatters() {
    timeFormatter = new Intl.DateTimeFormat("default", {
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });
    dateFormatter = new Intl.DateTimeFormat("default", {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }

  // --- API 1: Location ---
  async function getLoc(): Promise<{ lat: number; lon: number }> {
    return new Promise((resolve, reject) => {
      if (!navigator.geolocation) reject("No Geo");
      navigator.geolocation.getCurrentPosition(
        (pos) =>
          resolve({ lat: pos.coords.latitude, lon: pos.coords.longitude }),
        (err) => reject(err),
        { enableHighAccuracy: false, timeout: 10000 },
      );
    });
  }

  // --- API 2: Weather (Open-Meteo) ---
  async function fetchWeather() {
    try {
      const { lat, lon } = await getLoc();
      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
      const url = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current_weather=true&daily=temperature_2m_max,temperature_2m_min&timezone=${encodeURIComponent(tz)}`;

      const res = await fetch(url);
      const data = await res.json();
      if (!data.current_weather) throw new Error("No Data");

      weather = {
        temp: Math.round(data.current_weather.temperature).toString(),
        desc: codeToDesc(data.current_weather.weathercode),
        min: data.daily?.temperature_2m_min?.[0]
          ? Math.round(data.daily.temperature_2m_min[0]).toString()
          : "--",
        max: data.daily?.temperature_2m_max?.[0]
          ? Math.round(data.daily.temperature_2m_max[0]).toString()
          : "--",
        iconCode: data.current_weather.weathercode,
      };
      weatherError = "";
    } catch (e) {
      weatherError = "Weather unavailable";
    }
  }

  // --- API 3: F1 (Ergast/Jolpi) ---
  async function fetchF1() {
    try {
      // 1. Next Race
      const nextRes = await fetch(
        "https://api.jolpi.ca/ergast/f1/current/next.json",
      );
      const nextData = await nextRes.json();
      const nextRace = nextData.MRData.RaceTable.Races[0];

      // 2. Last Race (Results)
      const lastRes = await fetch(
        "https://api.jolpi.ca/ergast/f1/current/last/results.json",
      );
      const lastData = await lastRes.json();
      const lastRace = lastData.MRData.RaceTable.Races[0];

      // Format times to LOCAL device time with full date
      const formatLocalWithDate = (dateStr: string, timeStr: string) => {
        const d = new Date(`${dateStr}T${timeStr}`);
        const dayMonth = d.toLocaleString("default", {
          month: "short",
          day: "numeric",
        });
        const weekdayTime = d.toLocaleString("default", {
          weekday: "short",
          hour: "numeric",
          minute: "2-digit",
        });
        return { dayMonth, weekdayTime, fullDate: d };
      };

      const raceInfo = formatLocalWithDate(nextRace.date, nextRace.time);
      const qualiInfo = nextRace.Qualifying
        ? formatLocalWithDate(nextRace.Qualifying.date, nextRace.Qualifying.time)
        : null;

      f1.next = {
        name: nextRace.raceName,
        round: nextRace.round,
        country: nextRace.Circuit.Location.country,
        race: raceInfo,
        quali: qualiInfo,
      };
      f1.last = {
        name: lastRace.raceName,
        podium: lastRace.Results.slice(0, 3).map((r: any) => ({
          pos: r.position,
          driver: r.Driver.code || r.Driver.familyName,
          team: r.Constructor.name,
        })),
      };
      f1.loading = false;
    } catch (e) {
      console.error("F1 Error", e);
    }
  }

  // --- API 4: Prayer (Aladhan) ---
  async function fetchPrayer() {
    try {
      const { lat, lon } = await getLoc();
      const today = new Date();

      // Fetch yesterday, today and tomorrow
      const fetchDate = (d: Date) =>
        fetch(
          `https://api.aladhan.com/v1/timings/${d.getDate()}-${d.getMonth() + 1}-${d.getFullYear()}?latitude=${lat}&longitude=${lon}&method=3`,
        ).then((r) => r.json());
      
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);
      const tmrw = new Date(today);
      tmrw.setDate(tmrw.getDate() + 1);
      
      const [yesterdayData, todayData, tmrwData] = await Promise.all([
        fetchDate(yesterday),
        fetchDate(today),
        fetchDate(tmrw),
      ]);
      const t0 = yesterdayData.data.timings;
      const t1 = todayData.data.timings;
      const t2 = tmrwData.data.timings;

      const prayers = [
        { name: "Fajr", time: t0.Fajr, date: yesterday },
        { name: "Dhuhr", time: t0.Dhuhr, date: yesterday },
        { name: "Asr", time: t0.Asr, date: yesterday },
        { name: "Maghrib", time: t0.Maghrib, date: yesterday },
        { name: "Isha", time: t0.Isha, date: yesterday },
        { name: "Fajr", time: t1.Fajr, date: today },
        { name: "Dhuhr", time: t1.Dhuhr, date: today },
        { name: "Asr", time: t1.Asr, date: today },
        { name: "Maghrib", time: t1.Maghrib, date: today },
        { name: "Isha", time: t1.Isha, date: today },
        { name: "Fajr", time: t2.Fajr, date: tmrw },
      ];

      let nextP = null;
      let prevP = null;
      const nowMs = new Date().getTime();
      
      for (let i = 0; i < prayers.length; i++) {
        const p = prayers[i];
        const [h, m] = p.time.split(":");
        const pDate = new Date(p.date);
        pDate.setHours(parseInt(h), parseInt(m), 0);

        if (pDate.getTime() > nowMs) {
          nextP = { ...p, fullDate: pDate };
          // Previous prayer is the one before this
          if (i > 0) {
            const prevPrayer = prayers[i - 1];
            const [ph, pm] = prevPrayer.time.split(":");
            const prevDate = new Date(prevPrayer.date);
            prevDate.setHours(parseInt(ph), parseInt(pm), 0);
            prevP = { ...prevPrayer, fullDate: prevDate };
          }
          break;
        }
      }

      if (nextP) {
        prayer.nextName = nextP.name;
        prayer.nextTime = nextP.fullDate.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
        });
        prayer.countdown = Math.ceil(
          (nextP.fullDate.getTime() - nowMs) / 60000,
        );
      }
      
      if (prevP) {
        prayer.prevName = prevP.name;
        prayer.prevTime = prevP.fullDate.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
        });
      }
      
      prayer.loading = false;
    } catch (e) {
      console.error("Prayer error", e);
    }
  }

  // --- Lifecycle ---
  onMount(() => {
    initFormatters();

    // Initial Fetches
    fetchWeather();
    fetchF1();
    fetchPrayer();

    // Intervals
    clockTimer = window.setInterval(() => (now = new Date()), 1000);
    weatherTimer = window.setInterval(fetchWeather, REFRESH_WEATHER_MS);
    f1Timer = window.setInterval(fetchF1, REFRESH_F1_MS);
    prayerTimer = window.setInterval(fetchPrayer, 60 * 1000);
  });

  onDestroy(() => {
    clearInterval(clockTimer);
    clearInterval(weatherTimer);
    clearInterval(f1Timer);
    clearInterval(prayerTimer);
  });

  // --- Utils ---
  function codeToDesc(code: number): string {
    const map: Record<number, string> = {
      0: "Clear", 1: "Clear", 2: "Partly Cloudy", 3: "Overcast",
      45: "Fog", 48: "Fog", 51: "Drizzle", 53: "Drizzle", 55: "Drizzle",
      61: "Rain", 63: "Rain", 65: "Rain", 71: "Snow", 73: "Snow",
      75: "Snow", 95: "Thunderstorm",
    };
    return map[code] ?? "Unknown";
  }
</script>

<svelte:head>
  <title>Dashboard</title>
</svelte:head>

<div class="min-h-screen w-full flex flex-col items-center justify-center bg-black text-neutral-200 font-sans p-6 overflow-hidden">
  
  <div class="flex flex-col items-center text-center z-10 space-y-2">
    
    <h1 class="text-[25vw] sm:text-[14rem] font-bold leading-none tracking-tighter text-white select-none">
      {#if timeFormatter}
        {timeFormatter.format(now).replace(/\s[AP]M/, "")}
      {:else}
        --:--
      {/if}
    </h1>

    {#if dateFormatter}
      <div class="text-xl sm:text-2xl font-light tracking-widest uppercase text-neutral-500">
        {dateFormatter.format(now)}
      </div>
    {/if}

    <div class="flex flex-wrap justify-center items-center gap-4 sm:gap-6 mt-4 text-sm sm:text-lg font-mono text-neutral-400">
      
      {#if !weatherError}
        <div class="flex items-center gap-2">
          <span class="text-white">{weather.temp}°</span>
          <span class="opacity-60">{weather.desc}</span>
        </div>
      {/if}

      {#if !weatherError && !prayer.loading}<span class="hidden sm:inline text-neutral-800">|</span>{/if}

      {#if !prayer.loading && prayer.prevName !== "--"}
        <div class="flex items-center gap-2">
          <span class="text-neutral-600 text-xs">prev</span>
          <span class="text-neutral-500">{prayer.prevName}</span>
          <span class="opacity-60">{prayer.prevTime}</span>
        </div>
      {/if}

      {#if !prayer.loading && prayer.prevName !== "--" && prayer.nextName !== "--"}<span class="hidden sm:inline text-neutral-800">|</span>{/if}

      {#if !prayer.loading && prayer.nextName !== "--"}
        <div class="flex items-center gap-2">
          <span class="text-neutral-600 text-xs">next</span>
          <span class="text-white">{prayer.nextName}</span>
          <span>{prayer.nextTime}</span>
          {#if prayer.countdown > 0}
            <span class="text-neutral-600 text-xs">(-{prayer.countdown}m)</span>
          {/if}
        </div>
      {/if}
    </div>

  </div>

  <div class="fixed bottom-10 left-0 right-0 flex justify-center text-xs sm:text-sm font-mono text-neutral-500">
    {#if !f1.loading && f1.next}
      <div class="flex flex-col sm:flex-row items-center gap-2 sm:gap-3 px-4 py-2 rounded-2xl border border-neutral-900 bg-neutral-950/50 backdrop-blur-sm">
        <div class="flex items-center gap-2">
          <span class="font-bold text-neutral-300">F1</span>
          <span>{f1.next.country} GP</span>
        </div>
        <div class="flex items-center gap-3">
          {#if f1.next.quali}
            <div class="flex items-center gap-1.5">
              <span class="text-neutral-600 text-xs">Q</span>
              <span class="text-neutral-400">{f1.next.quali.dayMonth}</span>
              <span class="opacity-50 text-xs">{f1.next.quali.weekdayTime}</span>
            </div>
            <span class="opacity-20">•</span>
          {/if}
          <div class="flex items-center gap-1.5">
            <span class="text-neutral-600 text-xs">R</span>
            <span class="text-neutral-300">{f1.next.race.dayMonth}</span>
            <span class="opacity-70">{f1.next.race.weekdayTime}</span>
          </div>
        </div>
      </div>
    {/if}
  </div>

</div>

<style>
  :global(body) {
    background-color: #000000;
    margin: 0;
  }
</style>
