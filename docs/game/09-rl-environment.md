# RL environment

This document defines how *Latticefall* becomes a reinforcement-learning environment:
scenarios, observation and action spaces, rewards, the Python API, vectorization,
human demonstrations, interfaces for LLM and VLM agents, and a first experiment plan.
Read it before any WP in the `rl` lane.

Most of this ships in [milestone M9](10-roadmap.md#m9-rl-environment), late in the
plan. But the constraints in [RL readiness from day one](#rl-readiness-from-day-one)
apply to every WP from M0 onward, because retrofitting them is far more expensive than
respecting them early.

## Goals

- **One environment for every agent family.** Classic deep RL, LLM and VLM agents, and
  sequence models all run on the same deterministic core with the same dynamics.
- **Fast enough for real training.** At least 5,000 steps per second per CPU core with
  vector observations in the boss-arena scenario, and near-linear scaling across cores.
- **Reproducible.** The same seed, scenario, content version, and actions give
  bit-identical results on macOS, Linux, and the browser.
- **Demonstrations for free.** Human play in the browser records as tiny input logs that
  replay into any observation format later.
- **Shown on the site.** A trained policy can play in the browser, as a portfolio piece.

## Agent families

| Family | Examples | How it plugs in | Decision rate |
|---|---|---|---|
| Deep RL | PPO, PPO with LSTM, DQN variants, SAC on the continuous variant | Gymnasium and vector APIs, vector observations | 15 Hz (frame skip 2) |
| LLM and VLM agents | Open-weights instruct models, served locally | Text environment with macro-actions; turn-based | 0.5-2 Hz of game time; the game pauses between decisions |
| Sequence models | Decision Transformer, behavior cloning transformers | Offline datasets exported from replays; then the Gymnasium API for evaluation | 15 Hz |
| Fast decision models | Liquid AI's d1 family (see the following section) | Candidate-scoring interface: the environment proposes candidate macro-actions, the model picks one | 5-15 Hz |

### A note on Liquid AI's d1

The owner named "Liquid D1" as an example. Research for this plan found that Liquid AI
published open-weight models called d1 (`d1-3B`, built from LFM2.5-VL-3B, and
`d1-omni-600M`, marked experimental). The vendor describes them as decision models that
return an answer in one forward pass without generating tokens, with single-request
latency of about 8 ms on an RTX 4090 for `d1-3B`.

Treat the following as unverified until checked on the model cards:

- The license terms (the blog calls the models open-weight; secondary sources disagree
  on the license name).
- Whether the models were trained on any control or game data.
- The exact input format for choosing among options.

The safest way to support d1 and similar models is the **candidate-scoring interface**:
each decision, the environment renders the state as text (and optionally an image) plus
a numbered list of legal candidate actions, and the model returns one index. The same
interface works for any LLM used as a classifier. See [Text environment](#text-environment).

## RL readiness from day one

Every WP, starting with M0, must respect the following constraints:

- **No gameplay logic outside the core.** If it affects what happens, it's in `lf-core`.
- **One input type.** Humans, scripts, and policies all produce `ActionFrame` and
  `Command` values. See [the architecture spec](01-architecture.md#inputs).
- **Determinism contract.** See [the architecture spec](01-architecture.md#determinism-contract).
- **Clonable worlds.** `World: Clone` must stay cheap (under 200 microseconds for a
  boss arena). Don't put large immutable data in `World`; put it behind
  `Arc<Content>`.
- **Headless by default.** The core never needs a renderer, a window, a GPU, or a
  browser. Anything visual is optional.
- **Scenario-friendly content.** Any boss, enemy, or area can be loaded on its own
  through a scenario file, without the rest of the world.

## Scenarios

A scenario is a RON file in `game/data/scenarios/` that says what to load, where the
player starts, with what build, and when the episode ends. The following scenarios
ship with the first RL release:

| Gymnasium ID | Scenario | Episode ends when | Truncation |
|---|---|---|---|
| `Latticefall/Duel-v0` | Player and one enemy (choose any) in a flat arena | Either dies | 90 s |
| `Latticefall/BossArena-v0` | Player and one boss in its arena (choose any of the 12) | Either dies | 300 s |
| `Latticefall/Gauntlet-v0` | A room with a mixed enemy group | Player dies or all enemies die | 180 s |
| `Latticefall/Navigate-v0` | Reach a named Lantern or landmark; no enemies | Goal reached | 240 s |
| `Latticefall/Dungeon-v0` | A mini-dungeon from entrance to its end encounter | End encounter beaten or player dies with no lives left | 900 s |
| `Latticefall/World-v0` | The full game from a save state or a new game | B12 defeated | 3 hours |

Every scenario accepts `options` in `reset()`:

- `boss`, `enemy`, `area`, `start_lantern`: what to load.
- `build`: one of the nine [builds](04-arsenal-and-builds.md#builds), at a chosen
  level, or a custom loadout.
- `difficulty`: the same settings humans have.
- `randomize`: domain-randomization ranges (see [Curriculum](#curriculum-and-randomization)).
- `lives`: respawn count for dungeon and world scenarios.

## Python API

### Install

The Python package lives in `game/python/` and is built with maturin. The following
commands install it for development:

```sh
cd game/python
maturin develop --release
pip install -e ".[train]"
```

### Single environment

The following code sample shows the Gymnasium API:

```python
import gymnasium as gym
import latticefall  # registers the Latticefall/* environments

env = gym.make(
    "Latticefall/BossArena-v0",
    obs="vector",            # "vector", "vector+grid", "pixels", or "text"
    frame_skip=2,            # 30 ticks per second / 2 = 15 decisions per second
)
obs, info = env.reset(seed=7, options={"boss": "b07_castellan", "build": "bloodletter"})
done = False
while not done:
    action = env.action_space.sample(mask=info["action_mask"])
    obs, reward, terminated, truncated, info = env.step(action)
    done = terminated or truncated
```

`terminated` is true when the player or the episode's goal dies or is reached.
`truncated` is true on timeout. Training code must bootstrap on `truncated` and not on
`terminated`.

### The info dictionary

| Key | Type | Meaning |
|---|---|---|
| `action_mask` | tuple of bool arrays | Legal values per action head |
| `reward_components` | dict | Each reward term, before weighting |
| `player_hp_frac`, `target_hp_frac` | float | Convenience values for logging |
| `boss_phase` | int | Current boss phase, or -1 |
| `tick` | int | Simulation tick |
| `state_hash` | int | Only when `debug_hash=True` |
| `events` | list | Hits, deaths, parries, and so on, since the last step |

### Vector environment

`latticefall.VecEnv(num_envs, scenario, **kwargs)` steps many worlds in one call. The
Rust side runs them on a rayon pool with the GIL released and writes observations,
rewards, and flags into preallocated NumPy arrays without copying. It follows the
Gymnasium vector API with same-step autoreset, and a PufferLib emulation wrapper is
provided.

### State control

These methods support search-based agents, curriculum tools, and debugging:

- `env.unwrapped.clone_state()` and `restore_state(state)`: snapshot and restore.
- `env.unwrapped.save(path)` and `load(path)`: a save file, the same format the browser
  uses.
- `env.unwrapped.replay(path)`: load a replay and step through it.

## Observations

All encoders live in `lf-core` under the `obs` feature, so the browser can compute the
exact same observations for a policy running on the site.

| Key | Shape | Content |
|---|---|---|
| `proprio` | `(48,)` float32 | HP, FP, stamina (fractions); equip load tier; current move ID (embedded as an index into a separate `move` key), move frame fraction; i-frame, hyperarmor, and guard flags; flask charges; status buildups; buff timers; velocity; grounded flag; lock-on flag |
| `move` | `(2,)` int32 | Own current move ID and the lock target's current move ID, for embedding tables |
| `target` | `(32,)` float32 | Lock target (or nearest enemy): relative position and velocity in the player's frame, facing, HP and stance fractions, move frame fraction, move timing (hitbox active now, ticks to the next hit frame, ticks left, last attack of its combo; see [ADR 0006](decisions/0006-rl-smoke-test.md)), distance, angle, phase, status flags |
| `entities` | `(16, 24)` float32, plus a `(16,)` mask | Nearest actors and projectiles: kind, relative position and velocity, facing, HP fraction, hostility, move ID, move frame fraction |
| `rays` | `(2, 32)` float32 | 32 horizontal rays at knee and head height: distance and a hit-class code (wall, ledge, hazard, actor) |
| `grid` | `(6, 32, 32)` uint8 | Egocentric top-down semantic grid at 0.5 m: walkable, height, hazards, enemies, projectiles, items |
| `pixels` | `(2, 64, 64)` uint8 | Optional ray-cast depth and semantic class from the player's eye, rendered in the core with Rapier rays (no GPU) |

The exact layout of each vector is documented on its encoder in
`game/crates/core/src/obs/encode.rs`, and every feature is scaled into `[-1, 1]`. Two
notes on observations:

- **The boss fight isn't Markov from one frame.** Phase timers and the Overfit Warden's
  action counts are hidden state. Use frame stacking or a recurrent policy, or turn on
  `privileged=True` to add hidden state to `target` for debugging and teacher policies.
- **RGB pixels.** True rendered frames come from the browser renderer through a
  headless-browser render server (`latticefall.render.BrowserRenderer`), at tens of frames
  per second. Use it for VLM evaluation and demos, not for PPO training.

## Actions

The default action space is `MultiDiscrete` with five heads. Movement is relative to the
lock target if locked, or to the player's facing if not, so the policy never needs
camera control.

| Head | Size | Values |
|---|---|---|
| `move_dir` | 9 | None, forward, forward-right, right, back-right, back, back-left, left, forward-left |
| `move_speed` | 2 | Walk, run |
| `action` | 10 | None, light, heavy (tap), heavy (hold), guard (hold), dodge (tap; becomes sprint while held across steps), Art, use item, jump, interact. Light and heavy cast spells when a catalyst is active |
| `lock` | 3 | Keep, toggle, switch to the next target |
| `quick` | 4 | None, cycle item, cycle spell, two-hand toggle |

Each head has a mask in `info["action_mask"]`. For example, `action` masks attacks when
stamina is 0 and the move can't be buffered, and masks `use item` when no flask charges
remain. Masking is advisory: unmasked illegal actions do nothing, as they would for a
human.

Every button is a press edge in the core, so a step decodes its taps (light, heavy tap,
dodge, Art, use item, jump, interact, and the `lock` and `quick` heads) as a press on the
step's first tick and a release on the rest. Guard and heavy (hold) are held for the whole
step, and dodge chosen on consecutive steps is held, so it becomes a sprint. A step must
therefore be at least 2 ticks for the same tap on two steps in a row to count twice (see
[ADR 0006](decisions/0006-rl-smoke-test.md)).

Variants:

- `action_space="flat"`: a single `Discrete(63)` over the useful combinations.
- `action_space="continuous"`: `Box` movement in the player's frame plus discrete
  buttons, for agents that want analog control.
- `reaction_delay=(0, 3)`: delays applied actions by a random 0-3 ticks, so a policy
  can't exploit superhuman reaction speed. On by default for leaderboard evaluation.

## Rewards

Rewards are a weighted sum of named components. Each scenario sets default weights, and
`reward_weights={...}` overrides them. Every component is reported unweighted in
`info["reward_components"]`.

| Component | Signal | Default weight (boss arena) |
|---|---|---|
| `damage_dealt` | Target HP fraction removed this step | 1.0 |
| `damage_taken` | Own HP fraction lost this step | -0.5 |
| `win` | Target or goal defeated (terminal) | 10.0 |
| `death` | Player died (terminal) | -5.0 |
| `stance_break` | Broke a boss or elite's stance | 0.5 |
| `phase` | Boss entered a new phase | 1.0 |
| `time` | Per step | -0.001 |
| `heal_waste` | Healed above 90% HP | -0.1 |
| `explore` | New 16-meter map cell visited | 0.0 (0.05 in world and navigation scenarios) |
| `lantern` | New Lantern lit | 0.0 (1.0 in world scenarios) |
| `progress` | Distance-to-goal reduction | 0.0 (shaped in navigation scenarios) |

### Reward hacking safeguards

Souls-like bosses are easy to cheese. The following safeguards are on by default:

- **Reachability gate:** If the target can't reach the player (a nav-grid query from the
  target's cell to the player's cell fails for more than 90 ticks; bosses use this query
  even though they steer directly), `damage_dealt` is zero and a log flag is set.
- **Stall penalty:** In boss scenarios, 20 seconds without either side taking damage
  adds a growing time penalty.
- **Sanity metrics:** Evaluation reports damage per second, win rate, and the fraction of
  time the target was reachable. A run with high damage and no wins gets flagged.

## Curriculum and randomization

Training typically moves through these stages:

1. `Duel-v0` against Husks, then knights.
2. `BossArena-v0` against B01, then each boss, with randomization.
3. `Gauntlet-v0` and `Dungeon-v0`.
4. `World-v0` from save states at each Lantern.

Randomization options (all drawn from the episode seed):

- Boss and enemy HP, damage, and speed: each within plus or minus 20 percent.
- Boss option weights: jittered so the policy can't memorize one sequence.
- Player build and level: sampled from the nine builds and a level range.
- Start position jitter and arena variants.
- A **boss league**: sample opponents from all bosses and their randomized variants, and
  report per-boss win rates.

## Text environment

`Latticefall/Text-v0` wraps any scenario for LLM agents. Time stops between decisions;
each decision runs one macro-action until it completes or is interrupted.

### Observation

The observation is compact JSON, under about 800 tokens, with units in key names. The
following example shows a boss-arena observation:

```json
{
  "tick": 1840,
  "you": {"hp_pct": 62, "fp_pct": 40, "stamina_pct": 85, "flasks": {"vigil": 2, "lucid": 1},
          "weapon": "banana.cpp", "status": [], "locked_on": "Castellan Voss"},
  "target": {"name": "Castellan Voss", "hp_pct": 48, "phase": 2, "distance_m": 3.1,
             "bearing_deg": 12, "doing": "winding up a heavy overhead swing",
             "frames_until_hit_est": 9},
  "nearby": [],
  "terrain": "open hall; chandelier marker 4 m to your left",
  "last_events": ["You landed 2 hits (210 damage).", "You rolled through a sweep."],
  "candidates": ["dodge(back_left)", "dodge(right)", "guard(15)", "attack(light, 1)",
                 "heal(vigil)", "retreat(4)"]
}
```

The `doing` field comes from per-move descriptions in the content data, so a model
reasons about telegraphs the way a human reads them.

### Macro-actions

The following list defines the macro-actions, each a scripted controller in
`lf-core::macros`:

- `move_to(target)`: a Lantern, landmark, item, or entity ID; uses the nav grid.
- `attack(kind, count)`: light, heavy, or charged, `count` times, with lock-on.
- `dodge(direction)`: one roll relative to the target.
- `guard(ticks)`, `parry()`, `art()`, `cast(spell, count)`.
- `heal(flask)`, `use_item(name)`.
- `lock_on(target)`, `interact()`, `wait(ticks)`, `retreat(meters)`.

A macro returns a short result string and an interrupt reason if one fired: damage
taken, a new enemy noticed, the target died, or a telegraphed attack is about to land.

### Interfaces

- **Gymnasium:** `Text-v0` returns the JSON string as the observation and takes a macro
  string as the action.
- **Candidate scoring:** `candidates` lists 4-8 legal macros. A model can return just an
  index. This is the interface for d1-style decision models.
- **MCP server:** `latticefall-mcp` exposes `observe`, `act`, `reset`, and `status` as
  tools, so any MCP-capable agent can play.
- **OpenEnv:** an adapter serves the environment with the OpenEnv `reset`, `step`, and
  `state` interface for use with TRL and similar trainers. Verify the protocol against
  the OpenEnv repository when building it; the project marks itself experimental.
- **Verifiers:** a `load_environment()` entry point that returns the environment, a
  dataset of scenario seeds, and a reward function, for Prime Intellect's verifiers and
  Environments Hub.

### Multi-turn RL for LLMs

For training an LLM agent with GRPO-style multi-turn RL, use terminal or calibrated
rewards (win, progress to phase, HP fraction at the end). Research reports that naive
dense per-turn rewards can hurt multi-turn RL, so dense components are off by default in
`Text-v0`.

## Human demonstrations

Because the core is deterministic, a human playthrough is a seed plus an input log.
That log replays into any observation format, at any frame skip, even formats invented
after the recording.

1. **Consent.** A visitor opts in from **Settings > Research** with a plain-language
   explanation. The default is off. The consent version is stored with every upload.
2. **Recording.** The browser records the replay format (see
   [the architecture spec](01-architecture.md#persistence)). About 50 KB per hour.
3. **Upload.** At session end, the replay goes to `POST /api/latticefall/replays`, which
   stores it in R2 with the content hash and anonymous ID.
4. **Deletion.** **Settings > Research > Delete my recordings** removes every replay for
   that anonymous ID.
5. **Export.** `lf-tools replay export --format minari --obs vector --frame-skip 2`
   replays recordings natively and writes a Minari dataset (HDF5). A Hugging Face
   datasets (Parquet) exporter is a second option.
6. **Versioning.** A replay only replays against the same content hash. The export tool
   keeps old content versions buildable from git tags, so old recordings stay useful.

## Policies in the browser

A trained policy can play on the site:

- Export the policy to ONNX.
- In the browser, the WASM core computes observations with the same encoders, and the
  policy runs through ONNX Runtime Web (or `@huggingface/transformers`, which the site
  already depends on).
- `/rl/latticefall` shows an agent fighting a chosen boss, with its win rate and a link
  to the training code. This joins the existing `/rl` section of the site.

## Benchmarks and evaluation

- **Throughput:** `lf-tools bench` reports steps per second per scenario and observation
  type, natively and in WASM. CI fails if `BossArena-v0` with vector observations drops
  below 5,000 steps per second per core on the CI runner.
- **Evaluation protocol:** a fixed set of 100 held-out seeds per scenario, with
  `reaction_delay` on. Report win rate with a 95% confidence interval, median time to
  win, and median damage taken.
- **Leaderboard:** results publish with the environment version, content hash, and seed
  set. A results table lives in `game/python/RESULTS.md` and on `/rl/latticefall`.

## Experiment plan

These experiments are suggestions for after the environment ships. Each one is sized to
be a weekend to a week for the owner.

| ID | Experiment | Setup | Question |
|---|---|---|---|
| E1 | PPO baseline | `BossArena-v0`, B01, vector observations, 64 envs | Does PPO beat the tutorial boss, and how fast? |
| E2 | Recurrent PPO against the Overfit Warden | B07, LSTM policy | Does the agent learn varied play when the boss punishes its habits? A small study of non-stationary opponents |
| E3 | Boss league generalization | Train on 8 bosses, test on 4 | Does skill transfer to unseen bosses? |
| E4 | Navigation | `Navigate-v0`, grid observations | Can a policy learn the world's layout and shortcuts? |
| E5 | LLM evaluation | `Text-v0`, 3-8B open-weights instruct models | Can LLMs clear a mini-dungeon with macros? Where do they fail? |
| E6 | Behavior cloning then PPO | Human demonstrations, then PPO fine-tuning | How much do demos speed up learning? |
| E7 | Decision Transformer | Mixed human and PPO data | Does return conditioning help on sparse boss rewards? |
| E8 | Multi-turn GRPO | Small LLM on `Text-v0` dungeons | Does RL fine-tuning close the knowing-doing gap? |
| E9 | Fast decision model | d1-style candidate scoring at 5-15 Hz | Can a one-pass model act in near real time? |

## Package layout

```text
game/python/
  pyproject.toml            # maturin build backend; extras: [env], [train], [llm], [dev]
  latticefall/
    __init__.py             # gymnasium.register calls
    envs.py                 # Gymnasium Env classes
    vector.py               # VecEnv, PufferLib emulation
    spaces.py               # observation and action space definitions
    rewards.py              # reward weights and presets
    text.py                 # Text-v0, macros, candidate scoring
    mcp_server.py           # latticefall-mcp entry point
    adapters/
      openenv.py
      verifiers.py
    datasets.py             # replay export helpers
    render.py               # BrowserRenderer (headless browser frames)
  examples/
    ppo_boss.py             # single-file PPO in the style of CleanRL
    llm_agent.py            # text-environment agent loop
  tests/
```
