<script lang="ts">
  import { onMount } from "svelte";
  import { getApiBase } from "$lib/apiBase";

  type EdgeSignal = {
    colo: string;
    city: string | null;
    country: string | null;
    transport: string;
    tls: string;
    signals: number | null;
    counted: boolean;
  };

  type Packet = {
    x: number;
    lane: number;
    speed: number;
    alpha: number;
    offset: number;
  };

  const WEBGPU_PARTICLES = 24_576;
  const MOBILE_WEBGPU_PARTICLES = 16_384;
  const FALLBACK_PARTICLES = 720;

  let canvas: HTMLCanvasElement;
  let prompt = "train small models, ship fast systems";
  let activePrompt = prompt;
  let fieldSeed = hashText(prompt);
  let energy = 1;
  let paused = false;
  let rendererLabel = "INITIALIZING";
  let particleCount = 0;
  let fps = 0;
  let edgeState: "connecting" | "live" | "unavailable" = "connecting";
  let edgeSignal: EdgeSignal | null = null;
  let reducedMotion = false;
  let frameHandle = 0;
  let destroyRenderer = () => {};

  $: edgeLocation = edgeSignal
    ? [edgeSignal.city, edgeSignal.country].filter(Boolean).join(", ") || "Unknown origin"
    : "Locating nearest edge";

  function hashText(value: string): number {
    let hash = 2166136261;
    for (let i = 0; i < value.length; i += 1) {
      hash ^= value.charCodeAt(i);
      hash = Math.imul(hash, 16777619);
    }
    return (hash >>> 0) / 4294967295;
  }

  function commitPrompt() {
    const nextPrompt = prompt.trim() || "train small models, ship fast systems";
    prompt = nextPrompt;
    activePrompt = nextPrompt;
    fieldSeed = hashText(nextPrompt);
    energy = 3.2;
  }

  function perturbField() {
    fieldSeed = (fieldSeed + 0.38196601125) % 1;
    energy = 4.2;
  }

  function formatCount(value: number | null | undefined) {
    if (value === null || value === undefined) return "Not counted";
    return new Intl.NumberFormat("en", { notation: "compact" }).format(value);
  }

  async function loadEdgeSignal(signal: AbortSignal) {
    try {
      const response = await fetch(`${getApiBase()}/api/edge-signal`, {
        cache: "no-store",
        signal,
      });
      if (!response.ok) throw new Error(`Edge signal returned ${response.status}`);
      edgeSignal = (await response.json()) as EdgeSignal;
      edgeState = "live";
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") return;
      edgeState = "unavailable";
    }
  }

  function pointerPosition(event: PointerEvent) {
    const bounds = canvas.getBoundingClientRect();
    return {
      x: ((event.clientX - bounds.left) / bounds.width) * 2 - 1,
      y: 1 - ((event.clientY - bounds.top) / bounds.height) * 2,
    };
  }

  async function startWebGpu(): Promise<boolean> {
    if (!navigator.gpu) return false;

    const adapter = await navigator.gpu.requestAdapter({
      powerPreference: "high-performance",
    });
    if (!adapter) return false;

    const device = await adapter.requestDevice();
    const context = canvas.getContext("webgpu");
    if (!context) {
      device.destroy();
      return false;
    }

    const presentationFormat = navigator.gpu.getPreferredCanvasFormat();
    const isCompact = window.matchMedia("(max-width: 720px)").matches;
    particleCount = isCompact
      ? MOBILE_WEBGPU_PARTICLES
      : WEBGPU_PARTICLES;

    context.configure({
      device,
      format: presentationFormat,
      alphaMode: "premultiplied",
    });

    const resize = () => {
      const bounds = canvas.getBoundingClientRect();
      const pixelRatio = Math.min(window.devicePixelRatio, 2);
      const width = Math.max(1, Math.round(bounds.width * pixelRatio));
      const height = Math.max(1, Math.round(bounds.height * pixelRatio));
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
    };
    const resizeObserver = new ResizeObserver(resize);
    resizeObserver.observe(canvas);
    resize();

    const initialData = new Float32Array(particleCount * 4);
    for (let i = 0; i < particleCount; i += 1) {
      const cursor = i * 4;
      const lane = ((i * 17) % 9) / 8;
      initialData[cursor] = ((i * 73) % particleCount) / particleCount * 2.24 - 1.12;
      initialData[cursor + 1] = (lane - 0.5) * 1.3;
      initialData[cursor + 2] = 0.07 + ((i * 29) % 100) / 1000;
      initialData[cursor + 3] = 0;
    }

    const particleBuffer = device.createBuffer({
      label: "Edge field particles",
      size: initialData.byteLength,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(particleBuffer, 0, initialData);

    const uniformBuffer = device.createBuffer({
      label: "Edge field uniforms",
      size: 32,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    const shader = device.createShaderModule({
      label: "Edge field compute and render shader",
      code: `
        struct Particle {
          position: vec2f,
          velocity: vec2f,
        }

        struct Uniforms {
          flow: vec4f,
          pointer: vec4f,
        }

        struct VertexOutput {
          @builtin(position) position: vec4f,
          @location(0) color: vec4f,
        }

        @group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
        @group(0) @binding(1) var<uniform> uniforms: Uniforms;
        @group(0) @binding(2) var<storage, read> render_particles: array<Particle>;

        fn hash(value: f32) -> f32 {
          return fract(sin(value * 12.9898) * 43758.5453);
        }

        fn route_y(x: f32, lane: f32, seed: f32) -> f32 {
          let primary = sin(x * 4.8 + lane * 5.2 + seed * 9.0) * 0.13;
          let attention = sin(x * 11.0 - seed * 5.0 + lane * 2.7) * 0.045;
          let layer = floor((x + 1.12) * 2.25);
          let routing = sin(layer * 2.1 + lane * 7.0 + seed * 12.0) * 0.18;
          return (lane - 0.5) * 1.12 + primary + attention + routing;
        }

        @compute @workgroup_size(256)
        fn update(@builtin(global_invocation_id) id: vec3u) {
          let index = id.x;
          if (index >= arrayLength(&particles)) {
            return;
          }

          var packet = particles[index];
          let dt = uniforms.flow.y;
          let seed = uniforms.flow.z;
          let impulse = uniforms.flow.w;
          let lane = hash(f32(index) * 0.173 + seed * 113.0);
          let target_y = route_y(packet.position.x, lane, seed);

          packet.velocity.y += (target_y - packet.position.y) * dt * (2.8 + impulse * 0.28);
          packet.velocity.y *= pow(0.12, dt);

          if (uniforms.pointer.z > 0.5) {
            let delta = packet.position - uniforms.pointer.xy;
            let distance_squared = max(dot(delta, delta), 0.004);
            packet.velocity += normalize(delta) * (0.0018 / distance_squared) * dt;
          }

          let base_speed = 0.085 + hash(f32(index) * 0.831) * 0.12;
          packet.velocity.x = base_speed * (1.0 + impulse * 0.34);
          packet.position += packet.velocity * dt;

          if (packet.position.x > 1.14 || abs(packet.position.y) > 1.2) {
            packet.position.x = -1.14 - hash(f32(index) + uniforms.flow.x) * 0.06;
            packet.position.y = route_y(-1.12, lane, seed);
            packet.velocity.y = 0.0;
          }

          particles[index] = packet;
        }

        @vertex
        fn vertex_main(
          @builtin(vertex_index) vertex_index: u32,
          @builtin(instance_index) instance_index: u32,
        ) -> VertexOutput {
          let corners = array<vec2f, 6>(
            vec2f(-1.0, -1.0),
            vec2f( 1.0, -1.0),
            vec2f(-1.0,  1.0),
            vec2f(-1.0,  1.0),
            vec2f( 1.0, -1.0),
            vec2f( 1.0,  1.0),
          );
          let packet = render_particles[instance_index];
          let speed = clamp(packet.velocity.x * 5.0, 0.0, 1.0);
          let size = vec2f(0.004 + speed * 0.008, 0.0028);
          let offset = corners[vertex_index] * size;
          let tint = hash(f32(instance_index) * 0.417 + uniforms.flow.z * 31.0);
          let cool = vec3f(0.28, 0.87, 1.0);
          let warm = vec3f(0.74, 1.0, 0.36);
          var output: VertexOutput;
          output.position = vec4f(packet.position + offset, 0.0, 1.0);
          output.color = vec4f(mix(cool, warm, tint), 0.22 + speed * 0.42);
          return output;
        }

        @fragment
        fn fragment_main(input: VertexOutput) -> @location(0) vec4f {
          return input.color;
        }
      `,
    });

    const computeBindGroupLayout = device.createBindGroupLayout({
      label: "Edge field compute bindings",
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          buffer: { type: "storage" },
        },
        {
          binding: 1,
          visibility: GPUShaderStage.COMPUTE,
          buffer: { type: "uniform" },
        },
      ],
    });

    const renderBindGroupLayout = device.createBindGroupLayout({
      label: "Edge field render bindings",
      entries: [
        {
          binding: 1,
          visibility: GPUShaderStage.VERTEX,
          buffer: { type: "uniform" },
        },
        {
          binding: 2,
          visibility: GPUShaderStage.VERTEX,
          buffer: { type: "read-only-storage" },
        },
      ],
    });

    const computePipelineLayout = device.createPipelineLayout({
      bindGroupLayouts: [computeBindGroupLayout],
    });
    const renderPipelineLayout = device.createPipelineLayout({
      bindGroupLayouts: [renderBindGroupLayout],
    });

    const computePipeline = device.createComputePipeline({
      label: "Edge field compute pipeline",
      layout: computePipelineLayout,
      compute: { module: shader, entryPoint: "update" },
    });

    const renderPipeline = device.createRenderPipeline({
      label: "Edge field render pipeline",
      layout: renderPipelineLayout,
      vertex: { module: shader, entryPoint: "vertex_main" },
      fragment: {
        module: shader,
        entryPoint: "fragment_main",
        targets: [
          {
            format: presentationFormat,
            blend: {
              color: {
                srcFactor: "src-alpha",
                dstFactor: "one",
                operation: "add",
              },
              alpha: {
                srcFactor: "one",
                dstFactor: "one-minus-src-alpha",
                operation: "add",
              },
            },
          },
        ],
      },
      primitive: { topology: "triangle-list" },
    });

    const computeBindGroup = device.createBindGroup({
      label: "Edge field compute bind group",
      layout: computeBindGroupLayout,
      entries: [
        { binding: 0, resource: { buffer: particleBuffer } },
        { binding: 1, resource: { buffer: uniformBuffer } },
      ],
    });
    const renderBindGroup = device.createBindGroup({
      label: "Edge field render bind group",
      layout: renderBindGroupLayout,
      entries: [
        { binding: 1, resource: { buffer: uniformBuffer } },
        { binding: 2, resource: { buffer: particleBuffer } },
      ],
    });

    let pointerX = 0;
    let pointerY = 0;
    let pointerActive = 0;
    let stopped = false;
    let lastTime = performance.now();
    let fpsStarted = lastTime;
    let renderedFrames = 0;

    const updatePointer = (event: PointerEvent) => {
      const position = pointerPosition(event);
      pointerX = position.x;
      pointerY = position.y;
      pointerActive = 1;
    };
    const clearPointer = () => {
      pointerActive = 0;
    };

    canvas.addEventListener("pointermove", updatePointer);
    canvas.addEventListener("pointerleave", clearPointer);

    const draw = (time: number) => {
      if (stopped) return;
      const delta = Math.min((time - lastTime) / 1000, 0.034);
      lastTime = time;

      if (!paused && !document.hidden) {
        energy += (1 - energy) * Math.min(delta * 2.4, 1);
        const values = new Float32Array([
          time / 1000,
          reducedMotion ? 0 : delta,
          fieldSeed,
          energy,
          pointerX,
          pointerY,
          pointerActive,
          canvas.width / Math.max(canvas.height, 1),
        ]);
        device.queue.writeBuffer(uniformBuffer, 0, values);

        const encoder = device.createCommandEncoder({ label: "Edge field frame" });
        if (!reducedMotion) {
          const computePass = encoder.beginComputePass();
          computePass.setPipeline(computePipeline);
          computePass.setBindGroup(0, computeBindGroup);
          computePass.dispatchWorkgroups(Math.ceil(particleCount / 256));
          computePass.end();
        }

        const renderPass = encoder.beginRenderPass({
          colorAttachments: [
            {
              view: context.getCurrentTexture().createView(),
              clearValue: { r: 0.018, g: 0.025, b: 0.021, a: 1 },
              loadOp: "clear",
              storeOp: "store",
            },
          ],
        });
        renderPass.setPipeline(renderPipeline);
        renderPass.setBindGroup(0, renderBindGroup);
        renderPass.draw(6, particleCount);
        renderPass.end();
        device.queue.submit([encoder.finish()]);

        renderedFrames += 1;
        if (time - fpsStarted >= 750) {
          fps = reducedMotion
            ? 0
            : Math.round((renderedFrames * 1000) / (time - fpsStarted));
          renderedFrames = 0;
          fpsStarted = time;
        }
      }

      frameHandle = requestAnimationFrame(draw);
    };

    rendererLabel = "WEBGPU COMPUTE";
    frameHandle = requestAnimationFrame(draw);

    destroyRenderer = () => {
      stopped = true;
      cancelAnimationFrame(frameHandle);
      resizeObserver.disconnect();
      canvas.removeEventListener("pointermove", updatePointer);
      canvas.removeEventListener("pointerleave", clearPointer);
      particleBuffer.destroy();
      uniformBuffer.destroy();
      device.destroy();
    };

    return true;
  }

  function startCanvasFallback() {
    const context = canvas.getContext("2d", { alpha: false });
    if (!context) {
      rendererLabel = "STATIC MODE";
      return;
    }

    rendererLabel = "CANVAS FALLBACK";
    particleCount = FALLBACK_PARTICLES;

    const packets: Packet[] = Array.from({ length: FALLBACK_PARTICLES }, (_, i) => ({
      x: ((i * 71) % FALLBACK_PARTICLES) / FALLBACK_PARTICLES,
      lane: ((i * 43) % 101) / 100,
      speed: 0.035 + ((i * 19) % 41) / 900,
      alpha: 0.18 + ((i * 31) % 60) / 100,
      offset: ((i * 13) % 37) / 37,
    }));

    let width = 1;
    let height = 1;
    let pointerX = -10;
    let pointerY = -10;
    let stopped = false;
    let lastTime = performance.now();
    let fpsStarted = lastTime;
    let renderedFrames = 0;

    const resize = () => {
      const bounds = canvas.getBoundingClientRect();
      const pixelRatio = Math.min(window.devicePixelRatio, 2);
      width = Math.max(1, Math.round(bounds.width * pixelRatio));
      height = Math.max(1, Math.round(bounds.height * pixelRatio));
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
    };

    const routeY = (x: number, lane: number, seed: number) => {
      const primary = Math.sin(x * 10 + lane * 5.2 + seed * 9) * 0.065;
      const layer = Math.floor(x * 5);
      const routing = Math.sin(layer * 2.1 + lane * 7 + seed * 12) * 0.09;
      return 0.16 + lane * 0.68 + primary + routing;
    };

    const updatePointer = (event: PointerEvent) => {
      const bounds = canvas.getBoundingClientRect();
      pointerX = (event.clientX - bounds.left) / bounds.width;
      pointerY = (event.clientY - bounds.top) / bounds.height;
    };
    const clearPointer = () => {
      pointerX = -10;
      pointerY = -10;
    };

    const resizeObserver = new ResizeObserver(resize);
    resizeObserver.observe(canvas);
    canvas.addEventListener("pointermove", updatePointer);
    canvas.addEventListener("pointerleave", clearPointer);
    resize();

    const draw = (time: number) => {
      if (stopped) return;
      const delta = Math.min((time - lastTime) / 1000, 0.034);
      lastTime = time;

      if (!paused && !document.hidden) {
        energy += (1 - energy) * Math.min(delta * 2.4, 1);
        context.fillStyle = "#050706";
        context.fillRect(0, 0, width, height);

        for (let layer = 1; layer < 5; layer += 1) {
          const x = (width * layer) / 5;
          context.strokeStyle = "rgba(255,255,255,0.055)";
          context.beginPath();
          context.moveTo(x, 0);
          context.lineTo(x, height);
          context.stroke();
        }

        for (let i = 0; i < packets.length; i += 1) {
          const packet = packets[i];
          const previousX = packet.x;
          if (!reducedMotion) {
            packet.x += packet.speed * delta * (1 + energy * 0.34);
            if (packet.x > 1.04) packet.x = -0.04;
          }

          const y = routeY(packet.x, packet.lane, fieldSeed);
          const distance = Math.hypot(packet.x - pointerX, y - pointerY);
          const deflection = distance < 0.14
            ? ((0.14 - distance) / 0.14) * Math.sign(y - pointerY || 1) * 0.08
            : 0;
          const px = packet.x * width;
          const py = (y + deflection) * height;
          const length = Math.max(1.5, (packet.x - previousX) * width * 3 + 2);
          const warm = (i + Math.round(fieldSeed * 100)) % 5 === 0;
          context.fillStyle = warm
            ? `rgba(190,255,94,${packet.alpha})`
            : `rgba(80,218,255,${packet.alpha * 0.82})`;
          context.fillRect(px - length, py, length, Math.max(1, height / 520));
        }

        renderedFrames += 1;
        if (time - fpsStarted >= 750) {
          fps = reducedMotion
            ? 0
            : Math.round((renderedFrames * 1000) / (time - fpsStarted));
          renderedFrames = 0;
          fpsStarted = time;
        }
      }

      frameHandle = requestAnimationFrame(draw);
    };

    frameHandle = requestAnimationFrame(draw);
    destroyRenderer = () => {
      stopped = true;
      cancelAnimationFrame(frameHandle);
      resizeObserver.disconnect();
      canvas.removeEventListener("pointermove", updatePointer);
      canvas.removeEventListener("pointerleave", clearPointer);
    };
  }

  onMount(() => {
    const controller = new AbortController();
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotion = motionQuery.matches;

    void loadEdgeSignal(controller.signal);
    void startWebGpu()
      .then((started) => {
        if (!started) startCanvasFallback();
      })
      .catch(() => startCanvasFallback());

    return () => {
      controller.abort();
      destroyRenderer();
    };
  });
</script>

<section class="edge-field" aria-labelledby="edge-field-heading">
  <header class="field-heading">
    <div>
      <p class="field-kicker">Experiment 08 / edge field</p>
      <h2 id="edge-field-heading">Watch a neural system route itself.</h2>
    </div>
    <p class="field-intro">
      Your prompt shapes a live packet field. WebGPU computes the motion locally,
      while a Rust Worker returns the Cloudflare edge that served you.
    </p>
  </header>

  <div class="field-machine">
    <div class="viewport-column">
      <div class="machine-bar">
        <div class="machine-status">
          <span class:live={rendererLabel !== "INITIALIZING"}></span>
          {rendererLabel}
        </div>
        <div class="machine-readout">
          <span>{particleCount.toLocaleString()} packets</span>
          <span>{reducedMotion ? "Reduced motion" : `${fps} fps`}</span>
        </div>
      </div>

      <div class="canvas-wrap">
        <canvas
          bind:this={canvas}
          aria-label="An interactive particle simulation of packets flowing through five neural network layers"
        ></canvas>

        <div class="layer-map" aria-hidden="true">
          <div class="layer layer-1"><i></i><span>input</span></div>
          <div class="layer layer-2"><i></i><span>embed</span></div>
          <div class="layer layer-3"><i></i><span>attn</span></div>
          <div class="layer layer-4"><i></i><span>mlp</span></div>
          <div class="layer layer-5"><i></i><span>output</span></div>
        </div>

        <div class="canvas-instruction" aria-hidden="true">
          Move your pointer to bend the field
        </div>
        <div class="seed-readout" aria-hidden="true">
          SEED {fieldSeed.toFixed(8)}
        </div>
      </div>
    </div>

    <aside class="control-column" aria-label="Edge field controls and telemetry">
      <div class="edge-block" aria-live="polite">
        <div class="block-label">
          <span class:live={edgeState === "live"}></span>
          Cloudflare edge
        </div>

        {#if edgeState === "live" && edgeSignal}
          <p class="colo">{edgeSignal.colo}</p>
          <p class="edge-location">{edgeLocation}</p>
          <dl class="edge-facts">
            <div>
              <dt>Transport</dt>
              <dd>{edgeSignal.transport}</dd>
            </div>
            <div>
              <dt>TLS</dt>
              <dd>{edgeSignal.tls}</dd>
            </div>
            <div>
              <dt>Signals</dt>
              <dd>{formatCount(edgeSignal.signals)}</dd>
            </div>
          </dl>
        {:else if edgeState === "unavailable"}
          <p class="edge-message">Edge telemetry is unavailable. The local field is still running.</p>
        {:else}
          <p class="edge-message">Negotiating a route to the nearest data center.</p>
        {/if}
      </div>

      <form class="prompt-control" on:submit|preventDefault={commitPrompt}>
        <label for="field-prompt">Shape the system</label>
        <div class="prompt-input">
          <span aria-hidden="true">&gt;</span>
          <input
            id="field-prompt"
            bind:value={prompt}
            maxlength="64"
            autocomplete="off"
            spellcheck="false"
          />
        </div>
        <button type="submit">Recompile field</button>
      </form>

      <div class="machine-actions">
        <button type="button" on:click={perturbField}>Inject noise</button>
        <button type="button" on:click={() => (paused = !paused)}>
          {paused ? "Resume" : "Pause"}
        </button>
      </div>

      <div class="active-seed">
        <span>Active prompt</span>
        <p>{activePrompt}</p>
      </div>

      <p class="privacy-note">
        D1 stores one aggregate counter per edge location. This experiment stores no
        IP address, cookie, or fingerprint.
      </p>
    </aside>
  </div>
</section>

<style>
  .edge-field {
    --field-green: #baff63;
    --field-cyan: #54d8ff;
    --field-bg: #050706;
    --field-panel: #090c0a;
    --field-line: rgb(224 255 229 / 13%);
    margin: 88px 0 8px;
    color: #f4f7f2;
  }

  .field-heading {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(280px, 0.65fr);
    align-items: end;
    gap: 32px 64px;
    margin-bottom: 28px;
  }

  .field-kicker {
    margin: 0 0 14px;
    color: var(--ink-3);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1;
    letter-spacing: 0.09em;
    text-transform: uppercase;
  }

  .field-heading h2 {
    max-width: 14ch;
    color: var(--foreground);
    font-size: clamp(30px, 4vw, 52px);
    font-weight: 500;
    line-height: 1.02;
    letter-spacing: -0.045em;
  }

  .field-intro {
    max-width: 46ch;
    margin: 0;
    color: var(--ink-2);
    font-size: 14px;
    line-height: 1.65;
  }

  .field-machine {
    display: grid;
    grid-template-columns: minmax(0, 1.7fr) minmax(260px, 0.72fr);
    min-height: 540px;
    overflow: hidden;
    border: 1px solid var(--field-line);
    border-radius: 10px;
    background: var(--field-bg);
    box-shadow: 0 32px 100px rgb(0 0 0 / 34%);
  }

  .viewport-column {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    border-right: 1px solid var(--field-line);
  }

  .machine-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 44px;
    padding: 0 16px;
    border-bottom: 1px solid var(--field-line);
    color: rgb(234 255 237 / 54%);
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.07em;
    text-transform: uppercase;
  }

  .machine-status,
  .machine-readout {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .machine-readout {
    gap: 16px;
  }

  .machine-status > span,
  .block-label > span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #6b736d;
    box-shadow: 0 0 0 3px rgb(255 255 255 / 3%);
  }

  .machine-status > span.live,
  .block-label > span.live {
    background: var(--field-green);
    box-shadow: 0 0 14px rgb(186 255 99 / 72%);
  }

  .canvas-wrap {
    position: relative;
    min-height: 494px;
    overflow: hidden;
    background:
      radial-gradient(circle at 52% 48%, rgb(84 216 255 / 7%), transparent 48%),
      var(--field-bg);
  }

  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    cursor: crosshair;
    touch-action: pan-y;
  }

  .layer-map {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .layer {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: linear-gradient(transparent, rgb(255 255 255 / 8%) 16%, rgb(255 255 255 / 8%) 84%, transparent);
  }

  .layer-1 { left: 10%; }
  .layer-2 { left: 30%; }
  .layer-3 { left: 50%; }
  .layer-4 { left: 70%; }
  .layer-5 { left: 90%; }

  .layer i {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 8px;
    height: 8px;
    border: 1px solid rgb(186 255 99 / 60%);
    border-radius: 50%;
    background: var(--field-bg);
    box-shadow: 0 0 18px rgb(186 255 99 / 30%);
    transform: translate(-50%, -50%);
  }

  .layer:nth-child(even) i {
    top: 34%;
    border-color: rgb(84 216 255 / 60%);
    box-shadow: 0 0 18px rgb(84 216 255 / 30%);
  }

  .layer:nth-child(3) i {
    top: 66%;
  }

  .layer span {
    position: absolute;
    top: 16px;
    left: 8px;
    color: rgb(234 255 237 / 28%);
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .canvas-instruction,
  .seed-readout {
    position: absolute;
    bottom: 14px;
    color: rgb(234 255 237 / 30%);
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    pointer-events: none;
  }

  .canvas-instruction {
    left: 16px;
  }

  .seed-readout {
    right: 16px;
  }

  .control-column {
    display: flex;
    min-width: 0;
    flex-direction: column;
    background: var(--field-panel);
  }

  .edge-block,
  .prompt-control,
  .active-seed,
  .privacy-note {
    padding: 22px;
    border-bottom: 1px solid var(--field-line);
  }

  .block-label,
  .prompt-control label,
  .active-seed > span {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    color: rgb(234 255 237 / 42%);
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .colo {
    margin: 18px 0 0;
    color: var(--field-green);
    font-family: var(--font-mono);
    font-size: 42px;
    font-weight: 500;
    line-height: 1;
    letter-spacing: -0.05em;
    text-shadow: 0 0 28px rgb(186 255 99 / 22%);
  }

  .edge-location {
    margin: 8px 0 0;
    color: rgb(244 247 242 / 74%);
    font-size: 12px;
  }

  .edge-facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
    margin: 24px 0 0;
  }

  .edge-facts dt {
    color: rgb(234 255 237 / 32%);
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .edge-facts dd {
    margin: 5px 0 0;
    overflow: hidden;
    color: rgb(244 247 242 / 76%);
    font-family: var(--font-mono);
    font-size: 9px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .edge-message {
    min-height: 92px;
    margin: 18px 0 0;
    color: rgb(244 247 242 / 58%);
    font-size: 12px;
    line-height: 1.65;
  }

  .prompt-control {
    display: grid;
    gap: 12px;
  }

  .prompt-input {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    border: 1px solid var(--field-line);
    background: rgb(255 255 255 / 2%);
    color: var(--field-green);
    font-family: var(--font-mono);
  }

  .prompt-input:focus-within {
    border-color: rgb(186 255 99 / 52%);
    box-shadow: 0 0 0 3px rgb(186 255 99 / 7%);
  }

  .prompt-input input {
    width: 100%;
    min-width: 0;
    height: 42px;
    padding: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: #f4f7f2;
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .prompt-control button,
  .machine-actions button {
    min-height: 38px;
    border: 1px solid var(--field-line);
    border-radius: 5px;
    background: transparent;
    color: rgb(244 247 242 / 68%);
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .prompt-control button {
    border-color: var(--field-green);
    background: var(--field-green);
    color: #071006;
    font-weight: 600;
  }

  .prompt-control button:hover,
  .prompt-control button:focus-visible {
    background: #d0ff97;
  }

  .machine-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 12px 22px;
    border-bottom: 1px solid var(--field-line);
  }

  .machine-actions button:hover,
  .machine-actions button:focus-visible {
    border-color: rgb(244 247 242 / 36%);
    color: #ffffff;
  }

  .active-seed {
    flex: 1;
  }

  .active-seed p {
    margin: 12px 0 0;
    color: rgb(244 247 242 / 74%);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.55;
  }

  .privacy-note {
    margin: 0;
    border-bottom: 0;
    color: rgb(234 255 237 / 32%);
    font-family: var(--font-mono);
    font-size: 8px;
    line-height: 1.65;
    letter-spacing: 0.02em;
  }

  @media (max-width: 900px) {
    .field-heading {
      grid-template-columns: 1fr;
      gap: 16px;
    }

    .field-machine {
      grid-template-columns: 1fr;
    }

    .viewport-column {
      border-right: 0;
      border-bottom: 1px solid var(--field-line);
    }

    .canvas-wrap {
      min-height: 440px;
    }

    .control-column {
      display: grid;
      grid-template-columns: 1fr 1fr;
    }

    .edge-block,
    .prompt-control,
    .active-seed,
    .privacy-note,
    .machine-actions {
      border-bottom: 1px solid var(--field-line);
    }

    .edge-block,
    .active-seed {
      border-right: 1px solid var(--field-line);
    }

    .machine-actions {
      padding: 12px 22px;
    }
  }

  @media (max-width: 620px) {
    .edge-field {
      margin-top: 72px;
    }

    .field-heading h2 {
      max-width: 12ch;
    }

    .field-machine {
      margin-inline: -12px;
      border-radius: 7px;
    }

    .machine-readout span:first-child,
    .canvas-instruction,
    .layer span {
      display: none;
    }

    .canvas-wrap {
      min-height: 380px;
    }

    .control-column {
      display: flex;
    }

    .edge-block,
    .active-seed {
      border-right: 0;
    }

    .edge-facts {
      gap: 8px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    canvas {
      cursor: default;
    }
  }
</style>
