<script lang="ts">
  import { onMount } from "svelte";
  import { getApiBase } from "$lib/apiBase";

  type WeatherPayload = {
    status: string;
    message?: string;
    device_type?: number;
    locality_weather_data?: {
      temperature?: number | null;
      humidity?: number | null;
      wind_speed?: number | null;
      wind_direction?: number | null;
      rain_intensity?: number | null;
      rain_accumulation?: number | null;
      aqi_pm_10?: number | null;
      aqi_pm_2_point_5?: number | null;
    };
  };

  const REFRESH_MS = 10 * 60 * 1000;

  let data: WeatherPayload | null = null;
  let loaded = false;
  let isOpen = false;
  let containerEl: HTMLDivElement;

  async function refresh() {
    try {
      const res = await fetch(`${getApiBase()}/api/home_weather`, {
        headers: { Accept: "application/json" },
      });
      if (!res.ok) throw new Error(`status ${res.status}`);
      const json = (await res.json()) as WeatherPayload;
      const ok =
        json.status === "200" ||
        (json.status as unknown as number) === 200 ||
        !!json.locality_weather_data;
      if (!ok) throw new Error("upstream not ok");
      data = json;
      loaded = true;
    } catch {
      data = null;
      loaded = false;
      isOpen = false;
    }
  }

  function toggle() {
    isOpen = !isOpen;
  }

  function close() {
    isOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isOpen) close();
  }

  function handleClickOutside(e: MouseEvent) {
    if (
      isOpen &&
      containerEl &&
      !containerEl.contains(e.target as Node)
    ) {
      close();
    }
  }

  function bearingLabel(deg: number | null | undefined): string {
    if (deg == null || Number.isNaN(deg)) return "";
    const dirs = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    return dirs[Math.round(((deg % 360) / 45)) % 8];
  }

  function fmt(value: number | null | undefined, digits = 1): string {
    if (value == null || Number.isNaN(value)) return "—";
    return value.toFixed(digits);
  }

  $: lwd = data?.locality_weather_data ?? null;
  $: temp = lwd?.temperature ?? null;
  $: humidity = lwd?.humidity ?? null;
  $: windSpeed = lwd?.wind_speed ?? null;
  $: windDir = lwd?.wind_direction ?? null;
  $: rainIntensity = lwd?.rain_intensity ?? null;
  $: rainAcc = lwd?.rain_accumulation ?? null;
  $: aqi10 = lwd?.aqi_pm_10 ?? null;
  $: aqi25 = lwd?.aqi_pm_2_point_5 ?? null;
  $: isRaining = (rainIntensity ?? 0) > 0;
  $: roundedTemp = temp == null || Number.isNaN(temp) ? null : Math.round(temp);
  $: aqiReading = aqi25 ?? aqi10;

  onMount(() => {
    refresh();
    const handle = setInterval(refresh, REFRESH_MS);
    window.addEventListener("keydown", handleKeydown);
    window.addEventListener("mousedown", handleClickOutside);
    return () => {
      clearInterval(handle);
      window.removeEventListener("keydown", handleKeydown);
      window.removeEventListener("mousedown", handleClickOutside);
    };
  });
</script>

{#if loaded && roundedTemp != null}
  <div class="weather-wrap" bind:this={containerEl}>
    <button
      type="button"
      class="weather-trigger"
      class:active={isOpen}
      on:click={toggle}
      aria-label={`Local weather: ${fmt(temp, 1)} degrees Celsius`}
      aria-expanded={isOpen}
      aria-controls="local-weather-panel"
      title={`${fmt(temp, 1)} °C · ${fmt(humidity, 0)}% humidity`}
    >
      {#if isRaining}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M20 16.2A4.5 4.5 0 0 0 17.5 8h-1.8A7 7 0 1 0 4 14.9"></path>
          <line x1="8" y1="19" x2="8" y2="21"></line>
          <line x1="12" y1="20" x2="12" y2="22"></line>
          <line x1="16" y1="19" x2="16" y2="21"></line>
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="4"></circle>
          <line x1="12" y1="2" x2="12" y2="4"></line>
          <line x1="12" y1="20" x2="12" y2="22"></line>
          <line x1="4.93" y1="4.93" x2="6.34" y2="6.34"></line>
          <line x1="17.66" y1="17.66" x2="19.07" y2="19.07"></line>
          <line x1="2" y1="12" x2="4" y2="12"></line>
          <line x1="20" y1="12" x2="22" y2="12"></line>
          <line x1="4.93" y1="19.07" x2="6.34" y2="17.66"></line>
          <line x1="17.66" y1="6.34" x2="19.07" y2="4.93"></line>
        </svg>
      {/if}
      <span class="readout">
        <span class="temp">{roundedTemp}°</span>
        {#if aqiReading != null}
          <span class="aqi"><span class="sep" aria-hidden="true">·</span>AQI {fmt(aqiReading, 0)}</span>
        {/if}
      </span>
    </button>

    {#if isOpen}
      <section
        id="local-weather-panel"
        class="weather-panel"
        aria-label="Local weather details"
      >
        <header class="weather-panel-header">
          <div>
            <p class="weather-kicker"><span class="led" aria-hidden="true"></span>Live conditions</p>
            <h2>Weather at home</h2>
          </div>
          <button
            type="button"
            class="weather-close"
            on:click={close}
            aria-label="Close weather details"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </header>

        <div class="weather-reading">
          <strong>{fmt(temp, 1)} °C</strong>
          <span>{isRaining ? "Rain detected" : "No rain detected"}</span>
        </div>

        <dl class="weather-stats">
          <div>
            <dt>Humidity</dt>
            <dd>{fmt(humidity, 0)}%</dd>
          </div>
          <div>
            <dt>Wind</dt>
            <dd>
              {fmt(windSpeed, 1)} m/s
              {#if windDir != null}
                <span>{bearingLabel(windDir)}</span>
              {/if}
            </dd>
          </div>
          <div>
            <dt>Rain</dt>
            <dd>
              {fmt(rainIntensity, 2)} mm/h
              {#if rainAcc != null && rainAcc > 0}
                <span>+{fmt(rainAcc, 1)} mm</span>
              {/if}
            </dd>
          </div>
          {#if aqi25 != null || aqi10 != null}
            <div>
              <dt>Air quality</dt>
              <dd>
                {#if aqi25 != null}
                  PM2.5 {fmt(aqi25, 0)}
                {:else}
                  PM10 {fmt(aqi10, 0)}
                {/if}
              </dd>
            </div>
          {/if}
        </dl>
      </section>
    {/if}
  </div>
{/if}

<style>
  .weather-wrap {
    position: relative;
    z-index: 60;
    display: inline-flex;
    align-items: center;
  }

  /* Quiet mono readout in the top bar: "27° · AQI 41" */
  .weather-trigger {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    height: 32px;
    padding: 0 var(--s-2);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    line-height: 1;
    white-space: nowrap;
    cursor: pointer;
    transition:
      color var(--dur-fast) var(--ease),
      background-color var(--dur-fast) var(--ease),
      border-color var(--dur-fast) var(--ease);
  }

  .weather-trigger:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .weather-trigger.active {
    border-color: var(--line);
    color: var(--ink);
  }

  .readout {
    display: inline-flex;
    align-items: baseline;
  }

  .temp {
    color: var(--ink);
  }

  .aqi {
    color: var(--ink-3);
  }

  .sep {
    margin: 0 0.5em;
  }

  .weather-trigger svg,
  .weather-close svg {
    width: 14px;
    height: 14px;
    flex: 0 0 auto;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .weather-trigger svg {
    color: var(--ink-3);
  }

  /* Popover */
  .weather-panel {
    position: absolute;
    top: calc(100% + var(--s-3));
    right: 0;
    width: 296px;
    padding: var(--s-5);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--ink);
    box-shadow: 0 12px 32px -12px var(--shadow-color);
    animation: weather-enter var(--dur) var(--ease-out);
  }

  @keyframes weather-enter {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  .weather-panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
  }

  .weather-kicker {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin: 0 0 var(--s-2);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .weather-panel h2 {
    margin: 0;
    color: var(--ink);
    font-size: var(--text-base);
    font-weight: 500;
    line-height: 1.3;
    letter-spacing: -0.01em;
  }

  .weather-close {
    display: inline-grid;
    place-items: center;
    flex: 0 0 auto;
    width: 32px;
    height: 32px;
    margin: calc(-1 * var(--s-2)) calc(-1 * var(--s-2)) 0 0;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease), background-color var(--dur-fast) var(--ease);
  }

  .weather-close:hover {
    background: var(--hover-wash);
    color: var(--ink);
  }

  .weather-reading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    margin-top: var(--s-5);
    padding-bottom: var(--s-4);
    border-bottom: 1px solid var(--line);
  }

  .weather-reading strong {
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-xl);
    font-weight: 400;
    font-variant-numeric: tabular-nums;
    line-height: 1;
    letter-spacing: -0.03em;
  }

  .weather-reading span {
    color: var(--ink-3);
    font-size: var(--text-sm);
    text-align: right;
  }

  .weather-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--s-4) var(--s-5);
    margin: var(--s-4) 0 0;
  }

  .weather-stats div {
    min-width: 0;
  }

  .weather-stats dt {
    margin-bottom: var(--s-1);
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .weather-stats dd {
    margin: 0;
    color: var(--ink);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .weather-stats dd span {
    margin-left: var(--s-1);
    color: var(--ink-3);
  }

  @media (max-width: 640px) {
    .aqi {
      display: none;
    }

    .weather-panel {
      position: fixed;
      top: 64px;
      right: var(--gutter);
      left: var(--gutter);
      width: auto;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .weather-panel {
      animation: none;
    }
  }
</style>
