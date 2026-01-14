<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  const REFRESH_WEATHER_MS = 30 * 60 * 1000;
  const REFRESH_F1_MS = 60 * 60 * 1000;

  let now = new Date();
  let timeFormatter: Intl.DateTimeFormat;
  let dateFormatter: Intl.DateTimeFormat;

  let weather = { temp: "--", desc: "—" };
  let weatherError = "";

  let f1 = { loading: true, next: null as any };

  let prayer = {
    prevName: "—",
    prevTime: "—",
    nextName: "—",
    nextTime: "—",
    countdown: 0,
    loading: true,
  };

  let clockTimer: number;
  let weatherTimer: number;
  let f1Timer: number;
  let prayerTimer: number;

  function initFormatters() {
    timeFormatter = new Intl.DateTimeFormat("default", {
      hour: "numeric",
      minute: "2-digit",
      hour12: false,
    });
    dateFormatter = new Intl.DateTimeFormat("default", {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }

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

  async function fetchWeather() {
    try {
      const { lat, lon } = await getLoc();
      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
      const url = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current_weather=true&timezone=${encodeURIComponent(tz)}`;
      const res = await fetch(url);
      const data = await res.json();
      if (data.current_weather) {
        weather = {
          temp: Math.round(data.current_weather.temperature).toString(),
          desc: codeToDesc(data.current_weather.weathercode),
        };
      }
    } catch {
      weatherError = "—";
    }
  }

  async function fetchF1() {
    try {
      const res = await fetch(
        "https://api.jolpi.ca/ergast/f1/current/next.json",
      );
      const data = await res.json();
      const race = data.MRData.RaceTable.Races[0];
      if (race) {
        const raceDate = new Date(`${race.date}T${race.time}`);
        f1.next = {
          name: race.Circuit.Location.country,
          date: raceDate.toLocaleDateString("default", {
            month: "short",
            day: "numeric",
          }),
          time: raceDate.toLocaleTimeString("default", {
            hour: "numeric",
            minute: "2-digit",
          }),
        };
      }
      f1.loading = false;
    } catch {
      f1.loading = false;
    }
  }

  async function fetchPrayer() {
    try {
      const { lat, lon } = await getLoc();
      const now = new Date();

      const fetchT = async (d: Date) => {
        const res = await fetch(
          `https://api.aladhan.com/v1/timings/${d.getDate()}-${d.getMonth() + 1}-${d.getFullYear()}?latitude=${lat}&longitude=${lon}&method=3`,
        );
        const data = await res.json();
        return data.data.timings;
      };

      const t = await fetchT(now);
      const todayPrayers = [
        { name: "Fajr", time: t.Fajr, date: now },
        { name: "Dhuhr", time: t.Dhuhr, date: now },
        { name: "Asr", time: t.Asr, date: now },
        { name: "Maghrib", time: t.Maghrib, date: now },
        { name: "Isha", time: t.Isha, date: now },
      ];

      const getPDate = (p: { time: string; date: Date }) => {
        const [h, m] = p.time.split(":");
        const d = new Date(p.date);
        d.setHours(parseInt(h), parseInt(m), 0, 0);
        return d;
      };

      const nowMs = now.getTime();
      let nextIdx = todayPrayers.findIndex(
        (p) => getPDate(p).getTime() > nowMs,
      );

      let next, prev;

      if (nextIdx === -1) {
        // After Isha: next is tomorrow's Fajr, prev is today's Isha
        const tomorrow = new Date(now.getTime() + 24 * 60 * 60 * 1000);
        const tTom = await fetchT(tomorrow);
        next = { name: "Fajr", time: tTom.Fajr, date: tomorrow };
        prev = todayPrayers[todayPrayers.length - 1];
      } else if (nextIdx === 0) {
        // Before Fajr: next is today's Fajr, prev is yesterday's Isha
        const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);
        const tYes = await fetchT(yesterday);
        next = todayPrayers[0];
        prev = { name: "Isha", time: tYes.Isha, date: yesterday };
      } else {
        // Normal case: both in today's list
        next = todayPrayers[nextIdx];
        prev = todayPrayers[nextIdx - 1];
      }

      const nDate = getPDate(next);
      const pDate = getPDate(prev);

      prayer.nextName = next.name;
      prayer.nextTime = nDate.toLocaleTimeString("default", {
        hour: "numeric",
        minute: "2-digit",
      });
      prayer.prevName = prev.name;
      prayer.prevTime = pDate.toLocaleTimeString("default", {
        hour: "numeric",
        minute: "2-digit",
      });
      prayer.countdown = Math.ceil((nDate.getTime() - nowMs) / 60000);
      prayer.loading = false;
    } catch (err) {
      console.error("Prayer fetch failed", err);
      prayer.loading = false;
    }
  }

  onMount(() => {
    initFormatters();
    fetchWeather();
    fetchF1();
    fetchPrayer();

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

  function codeToDesc(code: number): string {
    const map: Record<number, string> = {
      0: "Clear",
      1: "Clear",
      2: "Cloudy",
      3: "Overcast",
      45: "Fog",
      48: "Fog",
      51: "Drizzle",
      53: "Drizzle",
      61: "Rain",
      63: "Rain",
      71: "Snow",
      95: "Storm",
    };
    return map[code] ?? "—";
  }
</script>

<svelte:head>
  <title>Dashboard</title>
</svelte:head>

<div class="dashboard">
  <div class="content">
    <!-- Time -->
    <div class="time-block">
      <h1 class="time">
        {#if timeFormatter}{timeFormatter.format(now)}{:else}--:--{/if}
      </h1>
      {#if dateFormatter}
        <p class="date">{dateFormatter.format(now)}</p>
      {/if}
    </div>

    <!-- Info row -->
    <div class="info-row">
      {#if !weatherError}
        <div class="info-item">
          <span class="info-value">{weather.temp}°</span>
          <span class="info-label">{weather.desc}</span>
        </div>
      {/if}

      {#if !prayer.loading && prayer.prevName !== "—"}
        <div class="info-item prev-prayer">
          <span class="info-value">{prayer.prevName}</span>
          <span class="info-label">{prayer.prevTime}</span>
        </div>
      {/if}

      {#if !prayer.loading && prayer.nextName !== "—"}
        <div class="info-item">
          <span class="info-value">{prayer.nextName}</span>
          <span class="info-label">{prayer.nextTime}</span>
        </div>
      {/if}
    </div>
  </div>

  <!-- F1 -->
  {#if !f1.loading && f1.next}
    <div class="f1-bar">
      <span class="f1-label">F1</span>
      <span class="f1-info"
        >{f1.next.name} · {f1.next.date} · {f1.next.time}</span
      >
    </div>
  {/if}
</div>

<style>
  .dashboard {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--black);
    padding: 2rem;
    position: relative;
  }

  .content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 2rem;
  }

  .time-block {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .time {
    font-size: clamp(5rem, 18vw, 12rem);
    font-weight: 200;
    letter-spacing: -0.04em;
    color: var(--white);
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .date {
    font-size: 1rem;
    font-weight: 400;
    color: var(--gray-600);
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  .info-row {
    display: flex;
    gap: 3rem;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }

  .info-value {
    font-size: 1.25rem;
    font-weight: 500;
    color: var(--white);
  }

  .info-label {
    font-size: 0.75rem;
    color: var(--gray-600);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .prev-prayer {
    opacity: 1;
  }

  .prev-prayer .info-value {
    color: var(--gray-500);
  }

  .prev-prayer .info-label {
    color: var(--gray-700);
  }

  .f1-bar {
    position: fixed;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1.25rem;
    background: var(--gray-950);
    border: 1px solid var(--gray-800);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
  }

  .f1-label {
    font-weight: 700;
    color: var(--white);
  }

  .f1-info {
    color: var(--gray-500);
  }
</style>
