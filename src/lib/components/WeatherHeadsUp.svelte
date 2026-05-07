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
  $: roundedTemp =
    temp == null || Number.isNaN(temp) ? null : Math.round(temp);

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
    {#if isOpen}
      <div class="weather-panel" role="dialog" aria-label="Local weather details">
        <div class="weather-panel-header">
          <span class="weather-title">weather @ kashif's place</span>
          <button class="weather-close" on:click={close} aria-label="Close">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        <dl class="weather-stats">
          <div class="weather-row">
            <dt>temp</dt>
            <dd>{fmt(temp, 1)} °C</dd>
          </div>
          <div class="weather-row">
            <dt>humidity</dt>
            <dd>{fmt(humidity, 0)}%</dd>
          </div>
          <div class="weather-row">
            <dt>wind</dt>
            <dd>
              {fmt(windSpeed, 1)} m/s
              {#if windDir != null}
                <span class="weather-faint">{bearingLabel(windDir)}</span>
              {/if}
            </dd>
          </div>
          <div class="weather-row">
            <dt>rain</dt>
            <dd>
              {fmt(rainIntensity, 2)} mm/h
              {#if rainAcc != null && rainAcc > 0}
                <span class="weather-faint">+{fmt(rainAcc, 1)} mm</span>
              {/if}
            </dd>
          </div>
          {#if aqi25 != null || aqi10 != null}
            <div class="weather-row">
              <dt>aqi</dt>
              <dd>
                {#if aqi25 != null}
                  <span>pm2.5 {fmt(aqi25, 0)}</span>
                {/if}
                {#if aqi10 != null}
                  <span class="weather-faint">pm10 {fmt(aqi10, 0)}</span>
                {/if}
              </dd>
            </div>
          {/if}
        </dl>
      </div>
    {/if}

    <button
      class="weather-pill"
      class:active={isOpen}
      on:click={toggle}
      aria-label="Local weather"
      title={`${fmt(temp, 1)}°C · ${fmt(humidity, 0)}% humidity`}
    >
      {#if isRaining}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20 16.2A4.5 4.5 0 0 0 17.5 8h-1.8A7 7 0 1 0 4 14.9"></path>
          <line x1="8" y1="19" x2="8" y2="21"></line>
          <line x1="12" y1="20" x2="12" y2="22"></line>
          <line x1="16" y1="19" x2="16" y2="21"></line>
        </svg>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
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
      <span class="weather-temp">{roundedTemp}°</span>
    </button>
  </div>
{/if}

<style>
  .weather-wrap {
    position: fixed;
    bottom: 1rem;
    left: 1rem;
    z-index: 50;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .weather-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    height: 32px;
    padding: 0 0.625rem;
    border-radius: 999px;
    border: 1px solid var(--agent-fab-border);
    background: var(--agent-fab-bg);
    color: var(--agent-fab-color);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    line-height: 1;
    box-shadow: 0 4px 18px var(--agent-fab-shadow);
    transition: color var(--dur-instant) var(--ease-out-quart),
      border-color var(--dur-instant) var(--ease-out-quart),
      transform var(--dur-fast) var(--ease-out-quart);
  }

  .weather-pill:hover,
  .weather-pill.active {
    color: var(--agent-fab-hover-color);
    border-color: var(--agent-fab-hover-border);
    transform: translateY(-1px);
  }

  .weather-temp {
    letter-spacing: -0.02em;
  }

  .weather-panel {
    width: 220px;
    background: var(--agent-bg);
    border: 1px solid var(--agent-border);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 32px var(--agent-shadow);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    overflow: hidden;
    animation: weather-enter var(--dur-fast) var(--ease-out-quart);
  }

  @keyframes weather-enter {
    from {
      opacity: 0;
      transform: translateY(6px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .weather-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--agent-border);
  }

  .weather-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--agent-text);
    letter-spacing: 0;
  }

  .weather-close {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--agent-text-faint);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: color var(--dur-instant) var(--ease-out-quart),
      background var(--dur-instant) var(--ease-out-quart);
  }

  .weather-close:hover {
    color: var(--agent-text);
    background: var(--agent-surface);
  }

  .weather-stats {
    margin: 0;
    padding: 0.5rem 0.75rem 0.625rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .weather-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.75rem;
  }

  .weather-row dt {
    color: var(--agent-text-faint);
    text-transform: lowercase;
    letter-spacing: 0;
  }

  .weather-row dd {
    margin: 0;
    color: var(--agent-text);
    font-family: var(--font-mono);
    font-weight: 600;
    text-align: right;
    display: inline-flex;
    align-items: baseline;
    gap: 0.375rem;
  }

  .weather-faint {
    color: var(--agent-text-faint);
    font-weight: 400;
    font-size: 0.6875rem;
  }
</style>
