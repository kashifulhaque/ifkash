<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade, fly } from "svelte/transition";

  // --- Configuration ---
  const REFRESH_WEATHER_MS = 30 * 60 * 1000; // 30 mins
  const REFRESH_F1_MS = 60 * 60 * 1000; // 1 hour

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

  // Formatters (initialized on mount to ensure client-side execution)
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

  // --- API 1: Location (The Driver) ---
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

      // Format times to LOCAL device time
      const formatLocal = (dateStr: string, timeStr: string) => {
        const d = new Date(`${dateStr}T${timeStr}`);
        return d.toLocaleString("default", {
          weekday: "short",
          hour: "numeric",
          minute: "2-digit",
        });
      };

      f1.next = {
        name: nextRace.raceName,
        round: nextRace.round,
        country: nextRace.Circuit.Location.country,
        raceTime: formatLocal(nextRace.date, nextRace.time),
        qualiTime: nextRace.Qualifying
          ? formatLocal(nextRace.Qualifying.date, nextRace.Qualifying.time)
          : "TBC",
      };

      f1.last = {
        name: lastRace.raceName,
        podium: lastRace.Results.slice(0, 3).map((r: any) => ({
          pos: r.position,
          driver: r.Driver.code || r.Driver.familyName, // Use code (e.g. VER) if avail
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

      // Fetch today and tomorrow to handle "Next is tomorrow morning" case
      const fetchDate = (d: Date) =>
        fetch(
          `https://api.aladhan.com/v1/timings/${d.getDate()}-${d.getMonth() + 1}-${d.getFullYear()}?latitude=${lat}&longitude=${lon}&method=3`,
        ).then((r) => r.json());

      const tmrw = new Date(today);
      tmrw.setDate(tmrw.getDate() + 1);
      const [todayData, tmrwData] = await Promise.all([
        fetchDate(today),
        fetchDate(tmrw),
      ]);

      const t1 = todayData.data.timings;
      const t2 = tmrwData.data.timings;

      const prayers = [
        { name: "Fajr", time: t1.Fajr, date: today },
        { name: "Dhuhr", time: t1.Dhuhr, date: today },
        { name: "Asr", time: t1.Asr, date: today },
        { name: "Maghrib", time: t1.Maghrib, date: today },
        { name: "Isha", time: t1.Isha, date: today },
        { name: "Fajr", time: t2.Fajr, date: tmrw }, // Tomorrow's Fajr
      ];

      let nextP = null;
      const nowMs = new Date().getTime();

      for (const p of prayers) {
        const [h, m] = p.time.split(":");
        const pDate = new Date(p.date);
        pDate.setHours(parseInt(h), parseInt(m), 0);

        if (pDate.getTime() > nowMs) {
          nextP = { ...p, fullDate: pDate };
          break;
        }
      }

      if (nextP) {
        prayer.nextName = nextP.name;
        // Format: 5:30 PM
        prayer.nextTime = nextP.fullDate.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
        });
        // Diff in minutes
        prayer.countdown = Math.ceil(
          (nextP.fullDate.getTime() - nowMs) / 60000,
        );
        prayer.loading = false;
      }
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

    // Update prayer calculation every minute (doesn't need full API fetch usually, but this is safer for day rollovers)
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
      0: "Clear Sky",
      1: "Mainly Clear",
      2: "Partly Cloudy",
      3: "Overcast",
      45: "Fog",
      48: "Fog",
      51: "Drizzle",
      53: "Drizzle",
      55: "Drizzle",
      61: "Rain",
      63: "Rain",
      65: "Rain",
      71: "Snow",
      73: "Snow",
      75: "Snow",
      95: "Thunderstorm",
    };
    return map[code] ?? "Unknown";
  }
</script>

<svelte:head>
  <title>Dashboard</title>
</svelte:head>

<div
  class="relative min-h-screen w-full overflow-hidden bg-slate-950 text-slate-50 font-sans selection:bg-cyan-500/30"
>
  <!-- Background Gradient Mesh -->
  <div class="absolute inset-0 z-0 opacity-40 pointer-events-none">
    <div
      class="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] rounded-full bg-purple-900/30 blur-[120px] animate-pulse"
    ></div>
    <div
      class="absolute bottom-[-10%] right-[-10%] w-[50%] h-[50%] rounded-full bg-cyan-900/30 blur-[120px] animate-pulse"
      style="animation-delay: 2s;"
    ></div>
  </div>

  <!-- Main Content Grid -->
  <div
    class="relative z-10 h-screen w-full p-8 grid grid-cols-12 grid-rows-12 gap-6"
  >
    <!-- 1. Top Left: Weather -->
    <div class="col-span-6 sm:col-span-3 row-span-2 flex items-start">
      {#if !weatherError}
        <div
          in:fade={{ duration: 800 }}
          class="group flex flex-col gap-1 p-4 rounded-2xl bg-white/5 backdrop-blur-md border border-white/10 hover:bg-white/10 transition-all duration-500"
        >
          <div class="flex items-baseline gap-2">
            <span class="text-4xl font-light tracking-tighter"
              >{weather.temp}°</span
            >
            <span
              class="text-xs font-medium text-cyan-400 uppercase tracking-wider"
              >{weather.desc}</span
            >
          </div>
          <div class="flex gap-3 text-[10px] text-slate-400 font-mono">
            <span>L: {weather.min}°</span>
            <span>H: {weather.max}°</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- 2. Center: Clock & Date -->
    <div
      class="col-span-12 row-span-4 sm:row-span-6 sm:col-start-4 sm:col-end-10 flex flex-col items-center justify-center text-center z-20 mt-10 sm:mt-0"
    >
      <h1
        class="font-thin tracking-tighter leading-none text-[18vw] sm:text-[10rem] md:text-[12rem] text-transparent bg-clip-text bg-gradient-to-b from-white to-slate-400 drop-shadow-2xl select-none"
      >
        {#if timeFormatter}
          {timeFormatter.format(now).replace(/\s[AP]M/, "")}
        {:else}
          --:--
        {/if}
      </h1>
      <div class="mt-2 sm:mt-6 flex items-center gap-4">
        <div
          class="h-[1px] w-12 bg-gradient-to-r from-transparent via-cyan-500 to-transparent opacity-50"
        ></div>
        {#if dateFormatter}
          <span
            class="text-sm sm:text-xl font-light tracking-[0.3em] uppercase text-cyan-100/80"
          >
            {dateFormatter.format(now)}
          </span>
        {/if}
        <div
          class="h-[1px] w-12 bg-gradient-to-r from-transparent via-cyan-500 to-transparent opacity-50"
        ></div>
      </div>
    </div>

    <!-- 3. Bottom Left: F1 -->
    <div
      class="col-span-12 sm:col-span-4 row-span-4 sm:row-start-9 sm:col-start-1 flex items-end"
    >
      {#if !f1.loading && f1.next}
        <div
          in:fly={{ y: 20, duration: 800, delay: 200 }}
          class="w-full max-w-sm p-5 rounded-2xl bg-white/5 backdrop-blur-md border border-white/10 hover:border-white/20 transition-all duration-300"
        >
          <div class="flex items-center gap-2 mb-4">
            <div class="w-1 h-4 bg-red-500 rounded-full"></div>
            <span
              class="text-xs font-bold uppercase tracking-widest text-slate-400"
              >Formula 1</span
            >
          </div>

          <div class="flex justify-between items-end mb-4">
            <div>
              <div class="text-lg font-medium text-white">
                {f1.next.country} GP
              </div>
              <div class="text-[10px] uppercase text-slate-500 tracking-wider">
                Round {f1.next.round}
              </div>
            </div>
            <div class="text-right">
              <div class="text-xs text-slate-300">{f1.next.raceTime}</div>
              <div class="text-[10px] text-slate-500 uppercase">Race Start</div>
            </div>
          </div>

          {#if f1.last}
            <div class="pt-3 border-t border-white/5">
              <div class="flex justify-between items-center text-xs">
                <span class="text-[10px] uppercase text-slate-500"
                  >Last Podium</span
                >
                <div class="flex gap-2 font-mono">
                  {#each f1.last.podium as p}
                    <span
                      class:text-yellow-400={p.pos === "1"}
                      class:text-slate-300={p.pos !== "1"}>{p.driver}</span
                    >
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- 4. Bottom Right: Prayer -->
    <div
      class="col-span-12 sm:col-span-3 row-span-3 sm:row-start-10 sm:col-start-10 flex flex-col justify-end items-end"
    >
      <div
        in:fly={{ y: 20, duration: 800, delay: 400 }}
        class="w-full transition-opacity duration-500"
        class:opacity-0={prayer.loading}
      >
        <div
          class="group relative overflow-hidden p-5 rounded-2xl bg-gradient-to-br from-cyan-900/20 to-purple-900/20 backdrop-blur-md border border-white/10 text-right"
        >
          <div
            class="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000"
          ></div>

          <div class="relative z-10">
            <div
              class="text-xs font-bold uppercase tracking-widest text-cyan-400 mb-1"
            >
              Next Prayer
            </div>
            <div class="text-3xl font-light text-white mb-1">
              {prayer.nextName}
            </div>
            <div class="text-sm font-mono text-slate-400">
              {prayer.nextTime}
            </div>

            <div
              class="mt-3 inline-flex items-center gap-2 px-2 py-1 rounded bg-black/20 border border-white/5"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse"
              ></span>
              <span class="text-xs font-mono text-cyan-200"
                >{prayer.countdown}m remaining</span
              >
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* Custom font adjustments if needed, though Tailwind handles most */
  :global(body) {
    background-color: #020617; /* Ensure background matches to avoid white flash */
  }
</style>
