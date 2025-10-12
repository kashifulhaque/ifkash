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

  type Weather = { tempC: number | null; description: string | null };
  let weather: Weather = { tempC: null, description: null };
  let weatherError: string | null = null;
  let weatherTimer: number;

  // F1 Data
  let nextGP: any = null;
  let lastGP: any = null;
  let driversStandings: any[] = [];
  let constructorsStandings: any[] = [];
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

      // Fetch driver standings (top 3)
      const driversRes = await fetch('https://api.jolpi.ca/ergast/f1/current/driverStandings.json');
      const driversData = await driversRes.json();
      driversStandings = driversData.MRData.StandingsTable.StandingsLists[0].DriverStandings.slice(0, 3).map((standing: any) => ({
        driver: standing.Driver.familyName,
        points: parseInt(standing.points)
      }));

      // Fetch constructor standings (top 3)
      const constructorsRes = await fetch('https://api.jolpi.ca/ergast/f1/current/constructorStandings.json');
      const constructorsData = await constructorsRes.json();
      constructorsStandings = constructorsData.MRData.StandingsTable.StandingsLists[0].ConstructorStandings.slice(0, 3).map((standing: any) => ({
        team: standing.Constructor.name,
        points: parseInt(standing.points)
      }));

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

  onMount(() => {
    loadPrefs();
    timer = window.setInterval(() => (now = new Date()), 1000);
    loadWeather();
    loadF1Data();
    weatherTimer = window.setInterval(() => loadWeather(), 30 * 60 * 1000);
  });
  onDestroy(() => {
    clearInterval(timer);
    clearInterval(weatherTimer);
  });

  function toggle24h() { use24h = !use24h; savePrefs(); }
  function toggleSeconds() { showSeconds = !showSeconds; savePrefs(); }
  function toggleWeather() { showWeather = !showWeather; savePrefs(); }
  function toggleF1() { showF1 = !showF1; savePrefs(); }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
  <link href="https://fonts.googleapis.com/css2?family=Manrope:wght@300;400&display=swap" rel="stylesheet">
</svelte:head>

<!-- Background with subtle vignette and grain -->
<div class="relative isolate min-h-screen w-full overflow-hidden bg-neutral-950 text-neutral-200">
  <div class="pointer-events-none absolute inset-0 opacity-70 [background:radial-gradient(60%_40%_at_50%_0%,rgba(255,255,255,0.06)_0%,rgba(255,255,255,0)_70%)]"></div>
  <div class="pointer-events-none absolute inset-0 mix-blend-overlay opacity-[0.08] grain-overlay"></div>

  <!-- Controls -->
  <div class="absolute right-4 top-4 flex items-center gap-2 text-sm text-neutral-400">
    <button type="button" class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggle24h} aria-label="Toggle 24h">
      {use24h ? '24h' : '12h'}
    </button>
    <button type="button" class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggleSeconds} aria-label="Toggle seconds">
      {showSeconds ? 'sec on' : 'sec off'}
    </button>
    <button type="button" class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggleWeather} aria-label="Toggle weather">
      {showWeather ? 'wx on' : 'wx off'}
    </button>
    <button type="button" class="rounded-xl border border-neutral-800 px-3 py-1.5 hover:border-neutral-700 hover:text-neutral-200" on:click={toggleF1} aria-label="Toggle F1">
      {showF1 ? 'f1 on' : 'f1 off'}
    </button>
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
        <div class="flex items-end gap-3">
          <div class="text-3xl font-light leading-none">{Math.round(weather.tempC)}°C</div>
          <div class="pb-[2px] text-sm text-neutral-400">{weather.description}</div>
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
          <div class="border-b border-neutral-800/50 pb-2.5">
            <div class="text-xs text-neutral-500 mb-1">Last GP: {lastGP.name}</div>
            <div class="text-neutral-400 text-xs">
              🥇 {lastGP.podium[0]} • 🥈 {lastGP.podium[1]} • 🥉 {lastGP.podium[2]}
            </div>
          </div>
        {/if}

        <!-- Standings -->
        <div class="grid grid-cols-2 gap-3 pt-0.5">
          <div>
            <div class="text-xs text-neutral-500 mb-1.5">Drivers</div>
            {#each driversStandings as driver, i}
              <div class="text-xs text-neutral-400 leading-relaxed">
                {i + 1}. {driver.driver} <span class="text-neutral-600">({driver.points})</span>
              </div>
            {/each}
          </div>
          <div>
            <div class="text-xs text-neutral-500 mb-1.5">Constructors</div>
            {#each constructorsStandings as team, i}
              <div class="text-xs text-neutral-400 leading-relaxed">
                {i + 1}. {team.team} <span class="text-neutral-600">({team.points})</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </aside>
  {/if}

  <!-- Tiny footer hint -->
  <footer class="pointer-events-none absolute bottom-3 left-0 right-0 flex items-center justify-center text-xs text-neutral-600">
    <span class="rounded-full border border-neutral-800/70 px-3 py-1">Press F11 / full-screen for kiosk vibes</span>
  </footer>
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
