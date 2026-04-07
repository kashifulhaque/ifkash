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

  // Ramadan state
  let ramadan = {
    isRamadan: false,
    suhoorTime: "—",
    iftaarTime: "—",
    suhoorMs: 0,
    iftaarMs: 0,
    loading: true,
    dayNumber: 0,
  };

  let clockTimer: number;
  let weatherTimer: number;
  let f1Timer: number;
  let prayerTimer: number;
  let ramadanTimer: number;

  // Reactive countdowns
  $: suhoorCountdown =
    ramadan.suhoorMs > 0
      ? formatCountdown(ramadan.suhoorMs - now.getTime())
      : null;
  $: iftaarCountdown =
    ramadan.iftaarMs > 0
      ? formatCountdown(ramadan.iftaarMs - now.getTime())
      : null;

  // Which is next: suhoor or iftaar?
  $: nextRamadanEvent = (() => {
    if (!ramadan.isRamadan) return null;
    const nowMs = now.getTime();
    const sMs = ramadan.suhoorMs - nowMs;
    const iMs = ramadan.iftaarMs - nowMs;
    if (sMs > 0 && (iMs <= 0 || sMs < iMs)) return "suhoor";
    if (iMs > 0) return "iftaar";
    return null;
  })();

  function formatCountdown(ms: number): string | null {
    if (ms <= 0) return null;
    const totalSec = Math.floor(ms / 1000);
    const h = Math.floor(totalSec / 3600);
    const m = Math.floor((totalSec % 3600) / 60);
    const s = totalSec % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }

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
        const tomorrow = new Date(now.getTime() + 24 * 60 * 60 * 1000);
        const tTom = await fetchT(tomorrow);
        next = { name: "Fajr", time: tTom.Fajr, date: tomorrow };
        prev = todayPrayers[todayPrayers.length - 1];
      } else if (nextIdx === 0) {
        const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);
        const tYes = await fetchT(yesterday);
        next = todayPrayers[0];
        prev = { name: "Isha", time: tYes.Isha, date: yesterday };
      } else {
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

  async function fetchRamadan() {
    try {
      const { lat, lon } = await getLoc();
      const currentDate = new Date();

      const fetchTimingsWithHijri = async (d: Date) => {
        const res = await fetch(
          `https://api.aladhan.com/v1/timings/${d.getDate()}-${d.getMonth() + 1}-${d.getFullYear()}?latitude=${lat}&longitude=${lon}&method=3`,
        );
        const data = await res.json();
        return data.data;
      };

      const todayData = await fetchTimingsWithHijri(currentDate);
      const hijriMonth = parseInt(todayData.date.hijri.month.number);
      const hijriDay = parseInt(todayData.date.hijri.day);

      if (hijriMonth !== 9) {
        ramadan.isRamadan = false;
        ramadan.loading = false;
        return;
      }

      ramadan.isRamadan = true;
      ramadan.dayNumber = hijriDay;

      const timings = todayData.timings;
      const nowMs = currentDate.getTime();

      const [fajrH, fajrM] = timings.Fajr.split(":");
      const suhoorDate = new Date(currentDate);
      suhoorDate.setHours(parseInt(fajrH), parseInt(fajrM), 0, 0);

      const [maghribH, maghribM] = timings.Maghrib.split(":");
      const iftaarDate = new Date(currentDate);
      iftaarDate.setHours(parseInt(maghribH), parseInt(maghribM), 0, 0);

      if (suhoorDate.getTime() <= nowMs) {
        const tomorrow = new Date(currentDate.getTime() + 24 * 60 * 60 * 1000);
        const tomorrowData = await fetchTimingsWithHijri(tomorrow);
        const [tFajrH, tFajrM] = tomorrowData.timings.Fajr.split(":");
        const tomorrowSuhoor = new Date(tomorrow);
        tomorrowSuhoor.setHours(parseInt(tFajrH), parseInt(tFajrM), 0, 0);
        ramadan.suhoorMs = tomorrowSuhoor.getTime();
        ramadan.suhoorTime = tomorrowSuhoor.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
          hour12: true,
        });
      } else {
        ramadan.suhoorMs = suhoorDate.getTime();
        ramadan.suhoorTime = suhoorDate.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
          hour12: true,
        });
      }

      if (iftaarDate.getTime() <= nowMs) {
        const tomorrow = new Date(currentDate.getTime() + 24 * 60 * 60 * 1000);
        const tomorrowData = await fetchTimingsWithHijri(tomorrow);
        const [tMaghribH, tMaghribM] = tomorrowData.timings.Maghrib.split(":");
        const tomorrowIftaar = new Date(tomorrow);
        tomorrowIftaar.setHours(parseInt(tMaghribH), parseInt(tMaghribM), 0, 0);
        ramadan.iftaarMs = tomorrowIftaar.getTime();
        ramadan.iftaarTime = tomorrowIftaar.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
          hour12: true,
        });
      } else {
        ramadan.iftaarMs = iftaarDate.getTime();
        ramadan.iftaarTime = iftaarDate.toLocaleTimeString("default", {
          hour: "numeric",
          minute: "2-digit",
          hour12: true,
        });
      }

      ramadan.loading = false;
    } catch (err) {
      console.error("Ramadan fetch failed", err);
      ramadan.loading = false;
    }
  }

  onMount(() => {
    initFormatters();
    fetchWeather();
    fetchF1();
    fetchPrayer();
    fetchRamadan();

    clockTimer = window.setInterval(() => (now = new Date()), 1000);
    weatherTimer = window.setInterval(fetchWeather, REFRESH_WEATHER_MS);
    f1Timer = window.setInterval(fetchF1, REFRESH_F1_MS);
    prayerTimer = window.setInterval(fetchPrayer, 60 * 1000);
    ramadanTimer = window.setInterval(fetchRamadan, 5 * 60 * 1000);
  });

  onDestroy(() => {
    clearInterval(clockTimer);
    clearInterval(weatherTimer);
    clearInterval(f1Timer);
    clearInterval(prayerTimer);
    clearInterval(ramadanTimer);
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

    <!-- Ramadan Card -->
    {#if !ramadan.loading && ramadan.isRamadan}
      <div class="ramadan-bar">
        <span class="ramadan-label"
          >☽ Ramadan <span class="ramadan-day"
            >· Day {ramadan.dayNumber - 1}</span
          ></span
        >
        <span class="ramadan-sep"></span>
        <span
          class="ramadan-item"
          class:ramadan-active={nextRamadanEvent === "suhoor"}
        >
          <span class="ramadan-item-name">Suhoor</span>
          <span class="ramadan-item-time">{ramadan.suhoorTime}</span>
          {#if nextRamadanEvent === "suhoor" && suhoorCountdown}
            <span class="ramadan-item-countdown">{suhoorCountdown}</span>
          {/if}
        </span>
        <span class="ramadan-dot">·</span>
        <span
          class="ramadan-item"
          class:ramadan-active={nextRamadanEvent === "iftaar"}
        >
          <span class="ramadan-item-name">Iftaar</span>
          <span class="ramadan-item-time">{ramadan.iftaarTime}</span>
          {#if nextRamadanEvent === "iftaar" && iftaarCountdown}
            <span class="ramadan-item-countdown">{iftaarCountdown}</span>
          {/if}
        </span>
      </div>
    {/if}
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
    background: var(--paper);
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
    color: var(--text-primary);
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .date {
    font-size: 1rem;
    font-weight: 400;
    color: var(--text-faint);
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
    color: var(--text-primary);
  }

  .info-label {
    font-size: 0.75rem;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .prev-prayer {
    opacity: 1;
  }

  .prev-prayer .info-value {
    color: var(--text-tertiary);
  }

  .prev-prayer .info-label {
    color: var(--text-faint);
  }

  .ramadan-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    animation: fade-up 0.6s var(--ease-out-quart) both;
  }

  .ramadan-label {
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.03em;
    white-space: nowrap;
  }

  .ramadan-day {
    font-weight: 400;
    color: var(--text-tertiary);
  }

  .ramadan-sep {
    width: 1px;
    height: 0.875rem;
    background: var(--border);
    flex-shrink: 0;
  }

  .ramadan-dot {
    color: var(--text-faint);
  }

  .ramadan-item {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    color: var(--text-tertiary);
    transition: color 0.3s ease;
  }

  .ramadan-item.ramadan-active {
    color: var(--text-secondary);
  }

  .ramadan-item-name {
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.75rem;
  }

  .ramadan-item-time {
    font-variant-numeric: tabular-nums;
  }

  .ramadan-item-countdown {
    font-size: 0.6875rem;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
    background: var(--surface-sunken);
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
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
    background: var(--surface-raised);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
  }

  .f1-label {
    font-weight: 700;
    color: var(--text-primary);
  }

  .f1-info {
    color: var(--text-tertiary);
  }

  @keyframes fade-up {
    from {
      opacity: 0;
      transform: translateY(16px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
