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
  $: roundedTemp = temp == null || Number.isNaN(temp) ? null : temp;

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
      <span>{roundedTemp}°</span>
    </button>

    {#if isOpen}
      <section
        id="local-weather-panel"
        class="weather-panel"
        aria-label="Local weather details"
      >
        <header class="weather-panel-header">
          <div>
            <p class="weather-kicker">Live conditions</p>
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

  .weather-trigger {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    min-width: 42px;
    height: 28px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-2);
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
    transition:
      color 160ms var(--ease-inout),
      background-color 160ms var(--ease-inout);
  }

  .weather-trigger:hover,
  .weather-trigger.active {
    background: var(--hover);
    color: var(--foreground);
  }

  .weather-trigger svg,
  .weather-close svg {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .weather-panel {
    position: absolute;
    top: calc(100% + 10px);
    right: 0;
    width: 280px;
    padding: 20px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--agent-bg);
    color: var(--foreground);
    box-shadow: 0 16px 40px var(--agent-shadow);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    animation: weather-enter 160ms var(--ease-out);
  }

  @keyframes weather-enter {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .weather-panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .weather-kicker {
    margin: 0 0 4px;
    color: var(--ink-3);
    font-size: 12px;
    line-height: 1.4;
  }

  .weather-panel h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    line-height: 1.3;
    letter-spacing: -0.015em;
  }

  .weather-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    width: 28px;
    height: 28px;
    margin: -6px -6px 0 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-3);
  }

  .weather-close:hover {
    background: var(--hover);
    color: var(--foreground);
  }

  .weather-reading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    margin-top: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }

  .weather-reading strong {
    color: var(--foreground);
    font-size: 24px;
    font-weight: 500;
    letter-spacing: -0.03em;
  }

  .weather-reading span {
    color: var(--ink-3);
    font-size: 12px;
  }

  .weather-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px 20px;
    margin: 16px 0 0;
  }

  .weather-stats div {
    min-width: 0;
  }

  .weather-stats dt {
    margin-bottom: 3px;
    color: var(--ink-3);
    font-size: 12px;
  }

  .weather-stats dd {
    margin: 0;
    color: var(--foreground);
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
  }

  .weather-stats dd span {
    margin-left: 3px;
    color: var(--ink-3);
    font-weight: 400;
  }

  @media (max-width: 560px) {
    .weather-panel {
      position: fixed;
      top: 132px;
      right: 24px;
      left: 24px;
      width: auto;
    }
  }
</style>
