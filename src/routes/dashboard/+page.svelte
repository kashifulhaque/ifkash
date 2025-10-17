<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  // Props (optional): provide lat/lon to skip geolocation; set initial preferences
  export let lat: number | null = null;
  export let lon: number | null = null;
  export let use24hDefault: boolean = false;
  export let showSecondsDefault: boolean = false;

  let now = new Date();
  let timer: number;

  let use24h = use24hDefault;
  let showSeconds = showSecondsDefault;
  let showWeather = true;
  let showF1 = true;

  type Weather = { tempC: number | null; description: string | null; minC: number | null; maxC: number | null };
  let weather: Weather = { tempC: null, description: null, minC: null, maxC: null };
  let weatherError: string | null = null;
  let weatherTimer: number;

  // F1 Data
  let nextGP: any = null;
  let lastGP: any = null;
  let f1Loading = true;

  const storageKey = 'minimal-clock-prefs-v1';
  type Prefs = { use24h: boolean; showSeconds: boolean; showWeather: boolean; showF1: boolean };

  function loadPrefs() {
    try {
      const raw = localStorage.getItem(storageKey);
      if (raw) {
        const p = JSON.parse(raw) as Prefs;
        use24h = p.use24h ?? use24h;
        showSeconds = p.showSeconds ?? showSeconds;
        showWeather = p.showWeather ?? showWeather;
        showF1 = p.showF1 ?? showF1;
      }
    } catch {}
  }
  function savePrefs() {
    const p: Prefs = { use24h, showSeconds, showWeather, showF1 };
    localStorage.setItem(storageKey, JSON.stringify(p));
  }

  function pad(n: number) { return n < 10 ? `0${n}` : `${n}`; }

  function formatTime(d: Date) {
    let h = d.getHours();
    const m = pad(d.getMinutes());
    const s = pad(d.getSeconds());

    if (!use24h) {
      const ampm = h >= 12 ? 'PM' : 'AM';
      h = h % 12 || 12;
      return showSeconds ? `${h}:${m}:${s}${ampm}` : `${h}:${m}${ampm}`;
    }
    return showSeconds ? `${pad(h)}:${m}:${s}` : `${pad(h)}:${m}`;
  }

  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    weekday: 'short', year: 'numeric', month: 'long', day: 'numeric'
  });

  function formatDate(d: Date) {
    return dateFormatter.format(d);
  }

  async function loadWeather() {
    try {
      let la = lat, lo = lon;
      if (la == null || lo == null) {
        const pos = await new Promise<GeolocationPosition>((resolve, reject) => {
          if (!navigator.geolocation) return reject(new Error('no geo'));
          navigator.geolocation.getCurrentPosition(resolve, reject, { enableHighAccuracy: true, timeout: 8000 });
        });
        la = pos.coords.latitude; lo = pos.coords.longitude;
      }
      if (la == null || lo == null) throw new Error('missing coords');

      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone || 'auto';
      const url =
        `https://api.open-meteo.com/v1/forecast?latitude=${la}&longitude=${lo}` +
        `&current_weather=true&daily=temperature_2m_max,temperature_2m_min&timezone=${encodeURIComponent(tz)}`;

      const res = await fetch(url);
      if (!res.ok) throw new Error('weather http ' + res.status);
      const data = await res.json();

      const cw = data.current_weather;
      if (!cw) throw new Error('no current_weather');

      weather.tempC = cw.temperature ?? null;
      weather.description = codeToDesc(cw.weathercode);

      const d = data.daily ?? {};
      const maxArr = d.temperature_2m_max ?? [];
      const minArr = d.temperature_2m_min ?? [];
      weather.maxC = (maxArr.length ? maxArr[0] : null);
      weather.minC = (minArr.length ? minArr[0] : null);

      weatherError = null;
    } catch (e) {
      weatherError = 'Weather unavailable';
    }
  }

  async function loadF1Data() {
    try {
      // Fetch next race
      const nextRaceRes = await fetch('https://api.jolpi.ca/ergast/f1/current/next.json');
      const nextRaceData = await nextRaceRes.json();
      const race = nextRaceData.MRData.RaceTable.Races[0];

      nextGP = {
        name: race.raceName,
        location: race.Circuit.Location.country,
        raceTime: convertToIST(race.date, race.time),
        qualiTime: convertToIST(race.Qualifying.date, race.Qualifying.time)
      };

      // Fetch last race result
      const lastRaceRes = await fetch('https://api.jolpi.ca/ergast/f1/current/last/results.json');
      const lastRaceData = await lastRaceRes.json();
      const lastRaceInfo = lastRaceData.MRData.RaceTable.Races[0];

      lastGP = {
        name: lastRaceInfo.raceName,
        podium: lastRaceInfo.Results.slice(0, 3).map((result: any) =>
          `${result.Driver.familyName}`
        )
      };

      f1Loading = false;
    } catch (error) {
      console.error('F1 data error:', error);
      f1Loading = false;
    }
  }

  function convertToIST(date: string, time: string) {
    const utcDate = new Date(date + 'T' + time);
    return utcDate.toLocaleString('en-IN', {
      timeZone: 'Asia/Kolkata',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function codeToDesc(code: number): string {
    const map: Record<number, string> = {
      0: 'Clear', 1: 'Mainly clear', 2: 'Partly cloudy', 3: 'Overcast',
      45: 'Fog', 48: 'Depositing rime fog',
      51: 'Light drizzle', 53: 'Drizzle', 55: 'Dense drizzle',
      61: 'Light rain', 63: 'Rain', 65: 'Heavy rain',
      71: 'Light snow', 73: 'Snow', 75: 'Heavy snow',
      95: 'Thunderstorm'
    };
    return map[code] ?? '—';
  }

  let prayerInterval: any;
  onMount(() => {
    updatePrayerTimings();
    prayerInterval = setInterval(updatePrayerTimings, 30000);
    loadPrefs();
    timer = window.setInterval(() => (now = new Date()), 1000);
    loadWeather();
    loadF1Data();
    weatherTimer = window.setInterval(() => loadWeather(), 30 * 60 * 1000);
  });
  onDestroy(() => {
    try { clearInterval(prayerInterval); } catch {}
    clearInterval(timer);
    clearInterval(weatherTimer);
  });

  // --- Namaaz / Prayer timings ---
  let nextPrayerName: string = '';
  let nextPrayerTimeDisplay: string = '';
  let minutesRemaining: number = 0;

  async function updatePrayerTimings() {
    try {
      // geolocate or use provided props
      let la = lat, lo = lon;
      if (la == null || lo == null) {
        try {
          const pos: GeolocationPosition = await new Promise((resolve, reject) => {
            if (!navigator.geolocation) return reject(new Error('Geolocation not available'));
            navigator.geolocation.getCurrentPosition(resolve, reject, { enableHighAccuracy: true, timeout: 8000 });
          }) as GeolocationPosition;
          la = pos.coords.latitude; lo = pos.coords.longitude;
        } catch (e) {
          // fallback: Bangalore coords
          la = 12.9716; lo = 77.5946;
        }
      }

      const today = new Date();
      const yyyy = today.getFullYear();
      const mm = String(today.getMonth() + 1).padStart(2, '0');
      const dd = String(today.getDate()).padStart(2, '0');

      const buildUrl = (dateStr: string) =>
        `https://api.aladhan.com/v1/timings/${dateStr}?latitude=${la}&longitude=${lo}&method=3`; // MWL

      const [res1, res2] = await Promise.all([
        fetch(buildUrl(`${dd}-${mm}-${yyyy}`)),
        fetch(buildUrl(new Date(today.getTime()+86400000).toLocaleDateString('en-GB').split('/').reverse().join('-')))
      ]);
      const j1 = await res1.json();
      const j2 = await res2.json();
      const t1 = j1?.data?.timings || {};
      const t2 = j2?.data?.timings || {};

      function parseTime(hm: string, baseDate: Date) {
        if (!hm) return null;
        const [h, m] = hm.split(':').map(Number);
        const d = new Date(baseDate);
        d.setHours(h, m, 0, 0);
        return d;
      }

      const nowLocal = new Date();

      const order = [
        ['Fajr', t1.Fajr],
        ['Sunrise', t1.Sunrise],
        ['Dhuhr', t1.Dhuhr],
        ['Asr', t1.Asr],
        ['Maghrib', t1.Maghrib],
        ['Isha', t1.Isha],
      ] as const;

      let upcoming: {name:string, time: Date, display:string} | null = null;
      for (const [name, str] of order) {
        const dt = parseTime(String(str || ''), today);
        if (dt && dt.getTime() > nowLocal.getTime()) {
          upcoming = { name, time: dt, display: String(str) };
          break;
        }
      }
      if (!upcoming) {
        // take Fajr from tomorrow
        const t = parseTime(String(t2.Fajr || ''), new Date(today.getTime()+86400000));
        if (t) upcoming = { name: 'Fajr', time: t, display: String(t2.Fajr) };
      }
      if (upcoming) {
        nextPrayerName = upcoming.name;
        nextPrayerTimeDisplay = formatTime(upcoming.time);
        minutesRemaining = Math.max(0, Math.round((upcoming.time.getTime() - nowLocal.getTime())/60000));
      }
    } catch (err) {
      console.error('prayer timings error', err);
    }
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=Manrope:wght@300;400&display=swap" rel="stylesheet">
</svelte:head>

<!-- Background with subtle vignette and grain -->
<div class="relative isolate min-h-screen w-full overflow-hidden bg-black text-neutral-200">
  <!-- Next Namaaz badge (top-right) -->
  <div class="fixed right-4 top-4 z-50">
    <div class="rounded-2xl border border-white/10 bg-white/5 px-4 py-2 shadow-lg backdrop-blur supports-[backdrop-filter]:bg-white/10">
      <div class="text-sm text-neutral-500 mb-1 tracking-wide">Next Namaaz</div>
      <div class="flex items-baseline gap-2">
        <span class="text-medium font-medium">{nextPrayerName}</span>
        <span class="text-medium tabular-nums font-semibold">{nextPrayerTimeDisplay}</span>
        <span class="text-sm opacity-70">({minutesRemaining} min)</span>
      </div>
    </div>
  </div>

<!-- Center content -->
  <main class="relative mx-auto flex min-h-screen max-w-7xl flex-col items-center justify-center px-6 text-center">
    <h1 class="select-none font-light tracking-wide clock-font" style="font-size:clamp(3rem,14vw,11rem); line-height:0.95;">
      {formatTime(now)}
    </h1>
    <p class="mt-3 text-balance text-lg text-neutral-400 sm:text-xl">
      {formatDate(now)}
    </p>
  </main>

  <!-- Weather in corner -->
  {#if showWeather}
    <aside class="pointer-events-auto absolute bottom-6 right-6 rounded-2xl border border-neutral-800/80 bg-neutral-900/30 px-4 py-3 backdrop-blur-sm">
      {#if weather.tempC !== null}
        <div class="flex items-center gap-3">
          <div class="text-3xl font-light leading-none">{Math.round(weather.tempC)}°C</div>
          <div class="text-sm text-neutral-400 leading-tight">
            <div>{weather.description}</div>
            {#if weather.maxC !== null && weather.minC !== null}
              <div class="mt-0.5 text-xs text-neutral-500">
                H {Math.round(weather.maxC)}° • L {Math.round(weather.minC)}°
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="text-sm text-neutral-500">{weatherError ?? 'Loading weather…'}</div>
      {/if}
    </aside>
  {/if}

  <!-- F1 in left corner -->
  {#if showF1 && !f1Loading && nextGP}
    <aside class="pointer-events-auto absolute bottom-6 left-6 rounded-2xl border border-neutral-800/80 bg-neutral-900/30 px-4 py-3 backdrop-blur-sm max-w-sm">
      <div class="space-y-2.5 text-sm">
        <!-- Next GP -->
        <div class="border-b border-neutral-800/50 pb-2.5">
          <div class="text-xs text-neutral-500 mb-1">Next GP</div>
          <div class="font-light text-base">{nextGP.name}</div>
          <div class="text-neutral-400 text-xs mt-0.5">
            Race: {nextGP.raceTime} • Quali: {nextGP.qualiTime}
          </div>
        </div>

        <!-- Last GP Podium -->
        {#if lastGP}
          <div>
            <div class="text-xs text-neutral-500 mb-1">Last GP: {lastGP.name}</div>
            <div class="text-neutral-400 text-xs">
              🥇 {lastGP.podium[0]} • 🥈 {lastGP.podium[1]} • 🥉 {lastGP.podium[2]}
            </div>
          </div>
        {/if}
      </div>
    </aside>
  {/if}
</div>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Lato:ital,wght@0,100;0,300;0,400;0,700;0,900;1,100;1,300;1,400;1,700;1,900&display=swap');
  @import url('https://fonts.googleapis.com/css2?family=Zalando+Sans:ital,wght@0,200..900;1,200..900&display=swap');

  :global(body) {
    font-family: 'Zalando Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .clock-font {
    font-family: 'Zalando Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-weight: 500;
  }

  :global(html) {
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .grain-overlay {
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='120' height='120' viewBox='0 0 120 120'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='2'/></filter><rect width='120' height='120' filter='url(%23n)' opacity='0.6'/></svg>");
    background-repeat: repeat;
    background-size: 120px 120px;
  }
</style>
