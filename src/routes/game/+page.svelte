<svelte:head>
  <title>Void Runner — Kashif</title>
  <meta name="description" content="A minimalist 3D endless runner game." />
</svelte:head>

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import * as THREE from 'three';

  let container: HTMLDivElement;
  let score = 0;
  let highScore = 0;
  let gameOver = false;
  let started = false;
  let frameId: number;

  // Audio context & nodes
  let audioCtx: AudioContext;
  let masterGain: GainNode;

  function initAudio() {
    audioCtx = new AudioContext();
    masterGain = audioCtx.createGain();
    masterGain.gain.value = 0.3;
    masterGain.connect(audioCtx.destination);
  }

  function playTone(freq: number, duration: number, type: OscillatorType = 'sine', vol = 0.15) {
    if (!audioCtx) return;
    const osc = audioCtx.createOscillator();
    const gain = audioCtx.createGain();
    osc.type = type;
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(vol, audioCtx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + duration);
    osc.connect(gain);
    gain.connect(masterGain);
    osc.start();
    osc.stop(audioCtx.currentTime + duration);
  }

  function playCollision() {
    playTone(80, 0.4, 'sawtooth', 0.25);
    playTone(60, 0.5, 'square', 0.15);
    // noise burst
    if (!audioCtx) return;
    const bufferSize = audioCtx.sampleRate * 0.15;
    const buffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) data[i] = (Math.random() * 2 - 1) * 0.3;
    const noise = audioCtx.createBufferSource();
    const noiseGain = audioCtx.createGain();
    noise.buffer = buffer;
    noiseGain.gain.setValueAtTime(0.2, audioCtx.currentTime);
    noiseGain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.15);
    noise.connect(noiseGain);
    noiseGain.connect(masterGain);
    noise.start();
  }

  function playDodge() {
    playTone(600 + Math.random() * 400, 0.08, 'sine', 0.06);
  }

  function playMilestone() {
    const now = audioCtx?.currentTime ?? 0;
    [523, 659, 784].forEach((f, i) => {
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      osc.type = 'sine';
      osc.frequency.value = f;
      gain.gain.setValueAtTime(0.12, now + i * 0.1);
      gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.1 + 0.3);
      osc.connect(gain);
      gain.connect(masterGain);
      osc.start(now + i * 0.1);
      osc.stop(now + i * 0.1 + 0.3);
    });
  }

  function playStartSound() {
    playTone(330, 0.15, 'sine', 0.1);
    setTimeout(() => playTone(440, 0.15, 'sine', 0.1), 100);
    setTimeout(() => playTone(660, 0.25, 'sine', 0.12), 200);
  }

  // ─── Game state ──────────────────────────────────────────────
  let scene: THREE.Scene;
  let camera: THREE.PerspectiveCamera;
  let renderer: THREE.WebGLRenderer;
  let player: THREE.Group;
  let obstacles: THREE.Mesh[] = [];
  let particles: THREE.Points;
  let ground: THREE.GridHelper;
  let clock: THREE.Clock;

  const LANE_WIDTH = 2.5;
  const LANES = [-LANE_WIDTH, 0, LANE_WIDTH];
  let currentLane = 1; // middle
  let targetX = 0;
  let speed = 8;
  let spawnTimer = 0;
  let spawnInterval = 1.2;
  let lastMilestone = 0;
  let moveLeft = false;
  let moveRight = false;
  let shakeAmount = 0;

  // Touch controls
  let touchStartX = 0;

  function createScene() {
    scene = new THREE.Scene();
    scene.fog = new THREE.FogExp2(0x000000, 0.035);

    camera = new THREE.PerspectiveCamera(70, container.clientWidth / container.clientHeight, 0.1, 100);
    camera.position.set(0, 4, 8);
    camera.lookAt(0, 0, -5);

    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setSize(container.clientWidth, container.clientHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000);
    container.appendChild(renderer.domElement);

    // Ground grid
    ground = new THREE.GridHelper(200, 100, 0x1a1a1a, 0x0a0a0a);
    ground.position.y = -0.5;
    scene.add(ground);

    // Side walls (subtle lane markers)
    for (const x of [-LANE_WIDTH * 2, LANE_WIDTH * 2]) {
      const wallGeo = new THREE.BoxGeometry(0.05, 0.6, 200);
      const wallMat = new THREE.MeshBasicMaterial({ color: 0x1a1a1a });
      const wall = new THREE.Mesh(wallGeo, wallMat);
      wall.position.set(x, -0.2, -80);
      scene.add(wall);
    }

    // Lane divider dots
    for (const x of [-LANE_WIDTH, LANE_WIDTH]) {
      for (let z = 0; z > -100; z -= 3) {
        const dotGeo = new THREE.BoxGeometry(0.05, 0.02, 0.4);
        const dotMat = new THREE.MeshBasicMaterial({ color: 0x222222 });
        const dot = new THREE.Mesh(dotGeo, dotMat);
        dot.position.set(x, -0.48, z);
        scene.add(dot);
      }
    }

    // Player
    player = new THREE.Group();
    
    // Main cube (wireframe)
    const cubeGeo = new THREE.BoxGeometry(1, 1, 1);
    const cubeWire = new THREE.LineSegments(
      new THREE.EdgesGeometry(cubeGeo),
      new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 2 })
    );
    player.add(cubeWire);

    // Inner glow cube
    const innerGeo = new THREE.BoxGeometry(0.6, 0.6, 0.6);
    const innerMat = new THREE.MeshBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.08 });
    const innerCube = new THREE.Mesh(innerGeo, innerMat);
    player.add(innerCube);

    // Point light on player
    const pLight = new THREE.PointLight(0xffffff, 1.5, 8);
    pLight.position.set(0, 0.5, 0);
    player.add(pLight);

    player.position.set(0, 0.2, 0);
    scene.add(player);

    // Background particles
    const pCount = 500;
    const pGeo = new THREE.BufferGeometry();
    const pPositions = new Float32Array(pCount * 3);
    for (let i = 0; i < pCount * 3; i += 3) {
      pPositions[i] = (Math.random() - 0.5) * 40;
      pPositions[i + 1] = Math.random() * 15;
      pPositions[i + 2] = (Math.random() - 0.5) * 100 - 20;
    }
    pGeo.setAttribute('position', new THREE.BufferAttribute(pPositions, 3));
    const pMat = new THREE.PointsMaterial({ color: 0x333333, size: 0.08, sizeAttenuation: true });
    particles = new THREE.Points(pGeo, pMat);
    scene.add(particles);

    // Ambient light
    scene.add(new THREE.AmbientLight(0xffffff, 0.15));

    clock = new THREE.Clock();
  }

  function spawnObstacle() {
    const lane = LANES[Math.floor(Math.random() * LANES.length)];
    
    // Variety: tall thin, wide short, or normal
    const variant = Math.random();
    let w: number, h: number, d: number;
    if (variant < 0.33) {
      w = 0.6; h = 2.5; d = 0.6; // tall pillar
    } else if (variant < 0.66) {
      w = 2; h = 0.8; d = 0.8; // wide barrier
    } else {
      w = 1; h = 1.2; d = 1; // normal
    }

    const geo = new THREE.BoxGeometry(w, h, d);
    const edges = new THREE.EdgesGeometry(geo);
    
    // Wireframe obstacle
    const wireframe = new THREE.LineSegments(
      edges,
      new THREE.LineBasicMaterial({ color: 0x444444 })
    );
    
    // Solid fill (very faint)
    const solidMat = new THREE.MeshBasicMaterial({ 
      color: 0xffffff, transparent: true, opacity: 0.02 
    });
    const solidMesh = new THREE.Mesh(geo, solidMat);
    
    // Group them
    const group = new THREE.Mesh(geo, solidMat);
    group.add(wireframe);
    group.position.set(lane, h / 2 - 0.3, -60);
    
    // Store metadata
    (group as any)._height = h;
    (group as any)._width = w;
    
    scene.add(group);
    obstacles.push(group);
  }

  function resetGame() {
    // Clean up obstacles
    for (const obs of obstacles) {
      scene.remove(obs);
      obs.geometry.dispose();
      (obs.material as THREE.Material).dispose();
    }
    obstacles = [];
    
    score = 0;
    speed = 8;
    spawnTimer = 0;
    spawnInterval = 1.2;
    lastMilestone = 0;
    currentLane = 1;
    targetX = 0;
    gameOver = false;
    shakeAmount = 0;
    if (player) {
      player.position.set(0, 0.2, 0);
      player.rotation.set(0, 0, 0);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      if (!started) {
        startGame();
      } else if (gameOver) {
        resetGame();
        started = true;
        playStartSound();
        gameLoop();
      }
      return;
    }

    if (!started || gameOver) return;

    if (e.key === 'ArrowLeft' || e.key === 'a' || e.key === 'A') {
      e.preventDefault();
      if (currentLane > 0) {
        currentLane--;
        targetX = LANES[currentLane];
        playDodge();
      }
    }
    if (e.key === 'ArrowRight' || e.key === 'd' || e.key === 'D') {
      e.preventDefault();
      if (currentLane < 2) {
        currentLane++;
        targetX = LANES[currentLane];
        playDodge();
      }
    }
  }

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.touches[0].clientX;
  }

  function handleTouchEnd(e: TouchEvent) {
    if (!started) {
      startGame();
      return;
    }
    if (gameOver) {
      resetGame();
      started = true;
      playStartSound();
      gameLoop();
      return;
    }

    const dx = e.changedTouches[0].clientX - touchStartX;
    if (Math.abs(dx) > 30) {
      if (dx < 0 && currentLane > 0) {
        currentLane--;
        targetX = LANES[currentLane];
        playDodge();
      } else if (dx > 0 && currentLane < 2) {
        currentLane++;
        targetX = LANES[currentLane];
        playDodge();
      }
    }
  }

  function checkCollision(obs: THREE.Mesh): boolean {
    const px = player.position.x;
    const pz = player.position.z;
    const ox = obs.position.x;
    const oz = obs.position.z;
    const ow = ((obs as any)._width ?? 1) * 0.45;
    const halfPlayer = 0.45;

    return (
      Math.abs(px - ox) < (halfPlayer + ow) &&
      Math.abs(pz - oz) < 1.0
    );
  }

  function gameLoop() {
    if (gameOver) return;

    const delta = clock.getDelta();
    const elapsed = clock.getElapsedTime();

    // Increase difficulty
    speed = 8 + score * 0.04;
    spawnInterval = Math.max(0.4, 1.2 - score * 0.008);

    // Move player toward target lane
    player.position.x += (targetX - player.position.x) * 0.15;
    
    // Player bob and rotation
    player.position.y = 0.2 + Math.sin(elapsed * 3) * 0.05;
    player.rotation.y += delta * 0.8;
    player.rotation.z = (targetX - player.position.x) * 0.3;

    // Camera shake
    if (shakeAmount > 0) {
      camera.position.x = (Math.random() - 0.5) * shakeAmount;
      camera.position.y = 4 + (Math.random() - 0.5) * shakeAmount;
      shakeAmount *= 0.9;
      if (shakeAmount < 0.01) {
        shakeAmount = 0;
        camera.position.x = 0;
        camera.position.y = 4;
      }
    }

    // Move ground
    ground.position.z = (ground.position.z - speed * delta) % 2;

    // Spawn obstacles
    spawnTimer += delta;
    if (spawnTimer >= spawnInterval) {
      spawnTimer = 0;
      spawnObstacle();
      // Sometimes spawn two side by side
      if (Math.random() < 0.3 && score > 10) {
        spawnObstacle();
      }
    }

    // Move obstacles
    for (let i = obstacles.length - 1; i >= 0; i--) {
      const obs = obstacles[i];
      obs.position.z += speed * delta;

      // Glow as they approach
      const dist = obs.position.z;
      if (dist > -10) {
        const wire = obs.children[0] as THREE.LineSegments;
        if (wire?.material) {
          (wire.material as THREE.LineBasicMaterial).color.setHex(
            dist > -3 ? 0xffffff : 0x666666
          );
        }
      }

      // Check collision
      if (checkCollision(obs)) {
        gameOver = true;
        if (score > highScore) highScore = score;
        playCollision();
        shakeAmount = 0.5;
        cancelAnimationFrame(frameId);

        // Flash the collided obstacle red briefly
        const wire = obs.children[0] as THREE.LineSegments;
        if (wire?.material) {
          (wire.material as THREE.LineBasicMaterial).color.setHex(0xff0000);
        }
        
        // Render one more frame to show the red flash
        renderer.render(scene, camera);
        return;
      }

      // Remove if past camera
      if (obs.position.z > 12) {
        scene.remove(obs);
        obs.geometry.dispose();
        (obs.material as THREE.Material).dispose();
        obstacles.splice(i, 1);
      }
    }

    // Move particles
    const pPositions = (particles.geometry.attributes.position as THREE.BufferAttribute).array as Float32Array;
    for (let i = 2; i < pPositions.length; i += 3) {
      pPositions[i] += speed * delta * 0.5;
      if (pPositions[i] > 10) pPositions[i] = -80;
    }
    particles.geometry.attributes.position.needsUpdate = true;

    // Score
    score += delta * 5;
    score = Math.round(score * 10) / 10;

    // Milestones
    if (Math.floor(score / 25) > lastMilestone) {
      lastMilestone = Math.floor(score / 25);
      playMilestone();
    }

    renderer.render(scene, camera);
    frameId = requestAnimationFrame(gameLoop);
  }

  function startGame() {
    if (!audioCtx) initAudio();
    if (audioCtx.state === 'suspended') audioCtx.resume();
    resetGame();
    started = true;
    playStartSound();
    clock.start();
    gameLoop();
  }

  function handleResize() {
    if (!camera || !renderer || !container) return;
    camera.aspect = container.clientWidth / container.clientHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(container.clientWidth, container.clientHeight);
  }

  onMount(() => {
    createScene();

    // Initial render
    renderer.render(scene, camera);

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('resize', handleResize);
    container.addEventListener('touchstart', handleTouchStart, { passive: true });
    container.addEventListener('touchend', handleTouchEnd, { passive: true });

    // Load high score
    try {
      const saved = localStorage.getItem('voidrunner-highscore');
      if (saved) highScore = parseFloat(saved);
    } catch {}

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(frameId);
      renderer?.dispose();
    };
  });

  onDestroy(() => {
    if (browser) {
      cancelAnimationFrame(frameId);
      audioCtx?.close();
    }
  });

  // Save high score reactively
  $: if (highScore > 0 && typeof localStorage !== 'undefined') {
    try { localStorage.setItem('voidrunner-highscore', highScore.toString()); } catch {}
  }
</script>

<div class="game-page">
  <div class="game-container" bind:this={container}>
    <!-- HUD -->
    {#if started && !gameOver}
      <div class="hud">
        <div class="hud-score">{Math.floor(score)}</div>
        <div class="hud-speed">×{(speed / 8).toFixed(1)}</div>
      </div>
    {/if}

    <!-- Start Screen -->
    {#if !started}
      <div class="overlay">
        <div class="overlay-content">
          <h1 class="game-title">VOID RUNNER</h1>
          <p class="game-subtitle">Navigate the void. Dodge the obstacles.</p>
          <div class="controls-hint">
            <div class="key-row">
              <kbd>←</kbd> <kbd>→</kbd> or <kbd>A</kbd> <kbd>D</kbd>
            </div>
            <p class="touch-hint">Swipe on mobile</p>
          </div>
          <button class="start-btn" on:click={startGame}>
            START
          </button>
          <p class="press-hint">or press SPACE</p>
          {#if highScore > 0}
            <p class="high-score-hint">Best: {Math.floor(highScore)}</p>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Game Over -->
    {#if gameOver}
      <div class="overlay">
        <div class="overlay-content">
          <h2 class="gameover-title">VOID CONSUMED</h2>
          <div class="final-score">{Math.floor(score)}</div>
          <p class="score-label">Distance</p>
          {#if score >= highScore}
            <p class="new-best">NEW BEST</p>
          {/if}
          <button class="start-btn" on:click={() => { resetGame(); started = true; playStartSound(); clock.start(); gameLoop(); }}>
            RETRY
          </button>
          <p class="press-hint">or press SPACE</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .game-page {
    position: fixed;
    inset: 0;
    background: #000;
    z-index: 1000;
  }

  .game-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
  }

  .game-container :global(canvas) {
    display: block;
  }

  /* ─── HUD ─── */
  .hud {
    position: absolute;
    top: 24px;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    gap: 32px;
    pointer-events: none;
    z-index: 10;
  }

  .hud-score {
    font-family: 'Inter', monospace;
    font-size: 2rem;
    font-weight: 700;
    color: #fff;
    letter-spacing: 0.1em;
    text-shadow: 0 0 20px rgba(255,255,255,0.3);
  }

  .hud-speed {
    font-family: 'Inter', monospace;
    font-size: 1rem;
    font-weight: 400;
    color: rgba(255,255,255,0.3);
    align-self: center;
  }

  /* ─── Overlays ─── */
  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    z-index: 20;
  }

  .overlay-content {
    text-align: center;
    color: #fff;
  }

  .game-title {
    font-family: 'Inter', sans-serif;
    font-size: 3.5rem;
    font-weight: 900;
    letter-spacing: 0.25em;
    margin: 0 0 8px 0;
    background: linear-gradient(180deg, #fff 0%, #666 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .game-subtitle {
    font-size: 0.9rem;
    color: rgba(255,255,255,0.4);
    margin: 0 0 40px 0;
    font-weight: 300;
    letter-spacing: 0.1em;
  }

  .gameover-title {
    font-family: 'Inter', sans-serif;
    font-size: 2rem;
    font-weight: 800;
    letter-spacing: 0.2em;
    margin: 0 0 24px 0;
    color: rgba(255,255,255,0.6);
  }

  .final-score {
    font-size: 5rem;
    font-weight: 900;
    line-height: 1;
    margin: 0;
    text-shadow: 0 0 40px rgba(255,255,255,0.2);
  }

  .score-label {
    font-size: 0.75rem;
    color: rgba(255,255,255,0.3);
    letter-spacing: 0.2em;
    text-transform: uppercase;
    margin: 8px 0 24px 0;
  }

  .new-best {
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.2em;
    color: #fff;
    margin: 0 0 24px 0;
    animation: pulse 1s ease infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .controls-hint {
    margin: 0 0 32px 0;
  }

  .key-row {
    display: flex;
    gap: 8px;
    justify-content: center;
    align-items: center;
    color: rgba(255,255,255,0.5);
    font-size: 0.85rem;
  }

  kbd {
    display: inline-block;
    padding: 4px 10px;
    font-family: 'Inter', monospace;
    font-size: 0.8rem;
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 4px;
    background: rgba(255,255,255,0.05);
    color: rgba(255,255,255,0.7);
  }

  .touch-hint {
    font-size: 0.75rem;
    color: rgba(255,255,255,0.25);
    margin: 8px 0 0 0;
  }

  .start-btn {
    display: inline-block;
    padding: 12px 48px;
    font-family: 'Inter', sans-serif;
    font-size: 0.9rem;
    font-weight: 700;
    letter-spacing: 0.25em;
    color: #000;
    background: #fff;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .start-btn:hover {
    background: rgba(255,255,255,0.85);
    box-shadow: 0 0 30px rgba(255,255,255,0.2);
    transform: scale(1.02);
  }

  .press-hint {
    font-size: 0.7rem;
    color: rgba(255,255,255,0.2);
    margin: 12px 0 0 0;
  }

  .high-score-hint {
    font-size: 0.8rem;
    color: rgba(255,255,255,0.3);
    margin: 16px 0 0 0;
  }

  @media (max-width: 600px) {
    .game-title {
      font-size: 2rem;
      letter-spacing: 0.15em;
    }

    .hud-score {
      font-size: 1.5rem;
    }

    .final-score {
      font-size: 3.5rem;
    }
  }
</style>
