<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  // Props (optional): provide lat/lon to skip geolocation; set initial preferences
  export let lat: number | null = null;
  export let lon: number | null = null;
  export let use24hDefault: boolean = true;
  export let showSecondsDefault: boolean = false;

  let now = new Date();
  let timer: number;

  let use24h = use24hDefault;
  let showSeconds = showSecondsDefault;
  let showWeather = true;

  type Weather = { tempC: number | null; description: string | null };
  let weather: Weather = { tempC: null, description: null };
  let weatherError: string | null = null;

  const storageKey = 'minimal-clock-prefs-v1';
  type Prefs = { use24h: boolean; showSeconds: boolean; showWeather: boolean };

  function loadPrefs() {
    try {
      const raw = localStorage.getItem(storageKey);
      if (raw) {
        const p = JSON.parse(raw) as Prefs;
        use24h = p.use24h ?? use24h;
        showSeconds = p.showSeconds ?? showSeconds;
        showWeather = p.showWeather ?? showWeather;
      }
    } catch {}
  }
  function savePrefs() {
    const p: Prefs = { use24h, showSeconds, showWeather };
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
      return showSeconds ? `${h}:${m}:${s} ${ampm}` : `${h}:${m} ${ampm}`;
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

      // Open-Meteo: no API key, returns current_weather
      const url = `https://api.open-meteo.com/v1/forecast?latitude=${la}&longitude=${lo}&current_weather=true`;
      const res = await fetch(url);
      if (!res.ok) throw new Error('weather http ' + res.status);
      const data = await res.json();
      const cw = data.current_weather;
      if (!cw) throw new Error('no current_weather');
      weather.tempC = cw.temperature ?? null;
      weather.description = codeToDesc(cw.weathercode);
      weatherError = null;
    } catch (e) {
      weatherError = 'Weather unavailable';
    }
  }

  function codeToDesc(code: number): string {
    // Minimal mapping for ambient feel
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

  onMount(() => {
    loadPrefs();
    timer = window.setInterval(() => (now = new Date()), 1000);
    loadWeather();
  });
  onDestroy(() => clearInterval(timer));

  function toggle24h() { use24h = !use24h; savePrefs(); }
  function toggleSeconds() { showSeconds = !showSeconds; savePrefs(); }
  function toggleWeather() { showWeather = !showWeather; savePrefs(); }
</script>

<!-- Background with subtle vignette and grain -->
<div class="relative isolate min-h-screen w-full overflow-hidden bg-neutral-950 text-neutral-200">
  <div class="pointer-events-none absolute inset-0 opacity-70 [background:radial-gradient(60%_40%_at_50%_0%,rgba(255,255,255,0.06)_0%,rgba(255,255,255,0)_70%)]"></div>
  <div class="pointer-events-none absolute inset-0 mix-blend-overlay opacity-[0.08] grain-overlay"></div>

  <!-- Controls -->
  <div class="absolute right-4 top-4 flex items-center gap-2 text-sm text-neutral-400">
    <button class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggle24h} aria-label="Toggle 24h">
      {use24h ? '24h' : '12h'}
    </button>
    <button class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggleSeconds} aria-label="Toggle seconds">
      {showSeconds ? 'sec on' : 'sec off'}
    </button>
    <button class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggleWeather} aria-label="Toggle weather">
      {showWeather ? 'wx on' : 'wx off'}
    </button>
  </div>

  <!-- Center content -->
  <main class="relative mx-auto flex min-h-screen max-w-7xl flex-col items-center justify-center px-6 text-center">
    <h1 class="select-none font-light tracking-wide" style="font-size:clamp(3rem,14vw,11rem); line-height:0.95;">
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
        <div class="flex items-end gap-3">
          <div class="text-3xl font-light leading-none">{Math.round(weather.tempC)}°C</div>
          <div class="pb-[2px] text-sm text-neutral-400">{weather.description}</div>
        </div>
      {:else}
        <div class="text-sm text-neutral-500">{weatherError ?? 'Loading weather…'}</div>
      {/if}
    </aside>
  {/if}

  <!-- Tiny footer hint (can delete) -->
  <footer class="pointer-events-none absolute bottom-3 left-0 right-0 flex items-center justify-center text-xs text-neutral-600">
    <span class="rounded-full border border-neutral-800/70 px-3 py-1">Press F11 / full-screen for kiosk vibes</span>
  </footer>
</div>

<style>
  /* Optional: better text rendering */
  :global(html) { text-rendering: optimizeLegibility; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; }
.grain-overlay{background-image:url("data:image/svg+xml;utf8,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%22120%22 height=%22120%22 viewBox=%220 0 120 120%22><filter id=%22n%22><feTurbulence type=%22fractalNoise%22 baseFrequency=%220.65%22 numOctaves=%222%22/></filter><rect width=%22120%22 height=%22120%22 filter=%22url(%23n)%22 opacity=%220.6%22/></svg>");background-repeat:repeat;background-size:120px 120px;}
</style>

