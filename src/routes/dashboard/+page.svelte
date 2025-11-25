<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  // --- Configuration ---
  const REFRESH_WEATHER_MS = 30 * 60 * 1000; // 30 mins
  const REFRESH_F1_MS = 60 * 60 * 1000;      // 1 hour

  // --- State ---
  let now = new Date();
  let timeFormatter: Intl.DateTimeFormat;
  let dateFormatter: Intl.DateTimeFormat;
  
  // Weather State
  let weather = { temp: '--', min: '--', max: '--', desc: 'Loading...', iconCode: -1 };
  let weatherError = '';

  // F1 State
  let f1 = {
    loading: true,
    next: null as any,
    last: null as any
  };

  // Prayer State
  let prayer = {
    nextName: '--',
    nextTime: '--',
    countdown: 0, // minutes
    loading: true
  };

  // Timers
  let clockTimer: number;
  let weatherTimer: number;
  let f1Timer: number;
  let prayerTimer: number;

  // --- Helpers ---
  function pad(n: number) { return n < 10 ? `0${n}` : `${n}`; }
  
  // Formatters (initialized on mount to ensure client-side execution)
  function initFormatters() {
    timeFormatter = new Intl.DateTimeFormat('default', { hour: 'numeric', minute: '2-digit', hour12: true });
    dateFormatter = new Intl.DateTimeFormat('default', { weekday: 'long', month: 'long', day: 'numeric' });
  }

  // --- API 1: Location (The Driver) ---
  async function getLoc(): Promise<{lat: number, lon: number}> {
    return new Promise((resolve, reject) => {
      if (!navigator.geolocation) reject('No Geo');
      navigator.geolocation.getCurrentPosition(
        (pos) => resolve({ lat: pos.coords.latitude, lon: pos.coords.longitude }),
        (err) => reject(err),
        { enableHighAccuracy: false, timeout: 10000 }
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

      if (!data.current_weather) throw new Error('No Data');

      weather = {
        temp: Math.round(data.current_weather.temperature).toString(),
        desc: codeToDesc(data.current_weather.weathercode),
        min: data.daily?.temperature_2m_min?.[0] ? Math.round(data.daily.temperature_2m_min[0]).toString() : '--',
        max: data.daily?.temperature_2m_max?.[0] ? Math.round(data.daily.temperature_2m_max[0]).toString() : '--',
        iconCode: data.current_weather.weathercode
      };
      weatherError = '';
    } catch (e) {
      weatherError = 'Weather unavailable';
    }
  }

  // --- API 3: F1 (Ergast/Jolpi) ---
  async function fetchF1() {
    try {
      // 1. Next Race
      const nextRes = await fetch('https://api.jolpi.ca/ergast/f1/current/next.json');
      const nextData = await nextRes.json();
      const nextRace = nextData.MRData.RaceTable.Races[0];

      // 2. Last Race (Results)
      const lastRes = await fetch('https://api.jolpi.ca/ergast/f1/current/last/results.json');
      const lastData = await lastRes.json();
      const lastRace = lastData.MRData.RaceTable.Races[0];

      // Format times to LOCAL device time
      const formatLocal = (dateStr: string, timeStr: string) => {
        const d = new Date(`${dateStr}T${timeStr}`);
        return d.toLocaleString('default', { 
            weekday: 'short', hour: 'numeric', minute: '2-digit' 
        });
      };

      f1.next = {
        name: nextRace.raceName,
        round: nextRace.round,
        country: nextRace.Circuit.Location.country,
        raceTime: formatLocal(nextRace.date, nextRace.time),
        qualiTime: nextRace.Qualifying ? formatLocal(nextRace.Qualifying.date, nextRace.Qualifying.time) : 'TBC'
      };

      f1.last = {
        name: lastRace.raceName,
        podium: lastRace.Results.slice(0, 3).map((r: any) => ({
            pos: r.position,
            driver: r.Driver.code || r.Driver.familyName, // Use code (e.g. VER) if avail
            team: r.Constructor.name
        }))
      };
      
      f1.loading = false;
    } catch (e) {
      console.error('F1 Error', e);
    }
  }

  // --- API 4: Prayer (Aladhan) ---
  async function fetchPrayer() {
    try {
      const { lat, lon } = await getLoc();
      const today = new Date();
      
      // Fetch today and tomorrow to handle "Next is tomorrow morning" case
      const fetchDate = (d: Date) => 
        fetch(`https://api.aladhan.com/v1/timings/${d.getDate()}-${d.getMonth()+1}-${d.getFullYear()}?latitude=${lat}&longitude=${lon}&method=3`).then(r => r.json());

      const tmrw = new Date(today); tmrw.setDate(tmrw.getDate() + 1);
      const [todayData, tmrwData] = await Promise.all([fetchDate(today), fetchDate(tmrw)]);

      const t1 = todayData.data.timings;
      const t2 = tmrwData.data.timings;

      const prayers = [
        { name: 'Fajr', time: t1.Fajr, date: today },
        { name: 'Dhuhr', time: t1.Dhuhr, date: today },
        { name: 'Asr', time: t1.Asr, date: today },
        { name: 'Maghrib', time: t1.Maghrib, date: today },
        { name: 'Isha', time: t1.Isha, date: today },
        { name: 'Fajr', time: t2.Fajr, date: tmrw } // Tomorrow's Fajr
      ];

      let nextP = null;
      const nowMs = new Date().getTime();

      for (const p of prayers) {
        const [h, m] = p.time.split(':');
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
        prayer.nextTime = nextP.fullDate.toLocaleTimeString('default', { hour: 'numeric', minute:'2-digit'});
        // Diff in minutes
        prayer.countdown = Math.ceil((nextP.fullDate.getTime() - nowMs) / 60000);
        prayer.loading = false;
      }

    } catch (e) { console.error("Prayer error", e); }
  }

  // --- Lifecycle ---
  onMount(() => {
    initFormatters();
    
    // Initial Fetches
    fetchWeather();
    fetchF1();
    fetchPrayer();

    // Intervals
    clockTimer = window.setInterval(() => now = new Date(), 1000);
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
      0: 'Clear Sky', 1: 'Mainly Clear', 2: 'Partly Cloudy', 3: 'Overcast',
      45: 'Fog', 48: 'Fog', 51: 'Drizzle', 53: 'Drizzle', 55: 'Drizzle',
      61: 'Rain', 63: 'Rain', 65: 'Rain', 71: 'Snow', 73: 'Snow', 75: 'Snow',
      95: 'Thunderstorm'
    };
    return map[code] ?? 'Unknown';
  }
</script>

<svelte:head>
  <title>Dashboard</title>
</svelte:head>

<div class="relative min-h-screen w-full overflow-hidden bg-[var(--color-background)] text-[var(--color-headline)] selection:bg-[var(--color-secondary)] selection:text-white font-sans">
  
  <main class="absolute inset-0 flex flex-col items-center justify-center z-10 pointer-events-none">
    <div class="flex flex-col items-center select-none">
      <h1 class="font-extralight tracking-tighter tabular-nums leading-none" 
          style="font-size: clamp(4rem, 20vw, 14rem);">
        {#if timeFormatter}
          {timeFormatter.format(now).replace(/\s[AP]M/, '')} {:else}
          --:--
        {/if}
      </h1>
      
      <div class="mt-4 flex items-center gap-3 text-[var(--color-paragraph)] text-lg sm:text-2xl font-light tracking-wide uppercase opacity-80">
        {#if dateFormatter}
          <span>{dateFormatter.format(now)}</span>
        {/if}
      </div>
    </div>
  </main>

  <div class="absolute inset-0 p-6 sm:p-10 grid grid-cols-1 sm:grid-cols-2 grid-rows-2 gap-4 pointer-events-none z-20">
    
    <div></div>

    <div class="flex justify-end items-start">
      <div class="pointer-events-auto backdrop-blur-md bg-[var(--color-surface)]/40 border border-[var(--color-border)] rounded-2xl p-5 min-w-[200px] shadow-sm transition-opacity duration-500" class:opacity-0={prayer.loading}>
        <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-bold uppercase tracking-wider text-[var(--color-secondary)]">Next Prayer</span>
            <span class="text-xs font-mono opacity-60">{prayer.countdown}m left</span>
        </div>
        <div class="flex flex-col">
            <span class="text-3xl font-light">{prayer.nextName}</span>
            <span class="text-xl opacity-80">{prayer.nextTime}</span>
        </div>
      </div>
    </div>

    <div class="flex justify-start items-end">
        {#if !f1.loading && f1.next}
        <div class="pointer-events-auto backdrop-blur-md bg-[var(--color-surface)]/40 border border-[var(--color-border)] rounded-2xl p-5 max-w-sm w-full shadow-sm">
            
            <div class="flex items-center gap-2 mb-4 border-b border-[var(--color-border)] pb-2">
                <span class="text-xs font-bold uppercase tracking-wider text-[var(--color-secondary)]">Formula 1</span>
            </div>

            <div class="flex flex-col gap-4">
                <div>
                    <div class="flex justify-between items-baseline mb-1">
                        <span class="text-sm font-semibold text-[var(--color-headline)]">{f1.next.country} GP</span>
                        <span class="text-[10px] uppercase opacity-50">Round {f1.next.round}</span>
                    </div>
                    <div class="grid grid-cols-2 gap-2 text-xs text-[var(--color-paragraph)]">
                        <div class="bg-[var(--color-background)]/50 p-2 rounded">
                            <span class="block opacity-50 text-[9px] uppercase">Quali</span>
                            {f1.next.qualiTime}
                        </div>
                        <div class="bg-[var(--color-background)]/50 p-2 rounded">
                            <span class="block opacity-50 text-[9px] uppercase">Race</span>
                            {f1.next.raceTime}
                        </div>
                    </div>
                </div>

                {#if f1.last}
                <div class="pt-2 border-t border-[var(--color-border)] border-dashed">
                    <span class="text-[10px] uppercase opacity-50 block mb-2">Last: {f1.last.name}</span>
                    <div class="flex justify-between items-end gap-1">
                        {#each f1.last.podium as p}
                            <div class="flex flex-col items-center w-full">
                                <span class="text-xs font-bold" class:text-yellow-500={p.pos==='1'} class:text-gray-400={p.pos==='2'} class:text-orange-700={p.pos==='3'}>
                                    {p.pos}
                                </span>
                                <span class="text-xs font-mono">{p.driver}</span>
                            </div>
                        {/each}
                    </div>
                </div>
                {/if}
            </div>
        </div>
        {/if}
    </div>

    <div class="flex justify-end items-end">
      {#if !weatherError}
        <div class="pointer-events-auto backdrop-blur-md bg-[var(--color-surface)]/40 border border-[var(--color-border)] rounded-2xl p-5 text-right min-w-[180px] shadow-sm">
            <div class="text-5xl font-extralight tracking-tighter mb-1">{weather.temp}°</div>
            <div class="text-sm font-medium text-[var(--color-secondary)] mb-3">{weather.desc}</div>
            
            <div class="flex justify-end gap-3 text-xs text-[var(--color-paragraph)] font-mono border-t border-[var(--color-border)] pt-2">
                <span class="flex flex-col items-center">
                    <span class="opacity-50 text-[9px]">MIN</span>
                    <span>{weather.min}°</span>
                </span>
                <span class="flex flex-col items-center">
                    <span class="opacity-50 text-[9px]">MAX</span>
                    <span>{weather.max}°</span>
                </span>
            </div>
        </div>
      {:else}
        <div class="text-xs text-red-400">{weatherError}</div>
      {/if}
    </div>

  </div>
</div>

<style>
    /* Ensure Tailwind/Variables work if not globally defined */
    :global(:root) {
        --color-background: #0f172a; /* Slate 900 */
        --color-surface: #1e293b;    /* Slate 800 */
        --color-border: #334155;     /* Slate 700 */
        --color-headline: #f8fafc;   /* Slate 50 */
        --color-paragraph: #94a3b8;  /* Slate 400 */
        --color-secondary: #38bdf8;  /* Sky 400 */
    }
</style>
