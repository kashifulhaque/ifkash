"""PPO smoke test on `Latticefall/BossArena-v0` against B01, the Hollow Sentry (WP-039).

A single-file PPO in the style of CleanRL, for a laptop CPU. It trains on the ``proprio``
and ``target`` observations with the default ``MultiDiscrete`` actions, action masking,
and the default rewards, for a wall-clock budget. It then evaluates four policies on the
same held-out seeds:

- ``random``: uniform over the masked action heads.
- ``idle``: does nothing (the player still starts inside the arena).
- ``scripted``: a hand-written policy that reads only the observation vectors (a port
  of the WP-034 test bot). It shows what the observations make possible.
- ``ppo``: the trained policy, sampling, and ``ppo_greedy``, taking each head's argmax.

Every iteration logs the steps per second and the recent episode returns. Results go to
``game/python/runs/<name>/``: ``log.txt``, ``summary.json``, and ``policy.pt``.

Run it from ``game/python`` with the ``train`` extra installed:

    .venv/bin/python examples/ppo_smoke.py --minutes 30
"""

from __future__ import annotations

import argparse
import json
import time
from collections import deque
from dataclasses import asdict, dataclass
from pathlib import Path

import gymnasium as gym
import numpy as np
import torch
from torch import nn

import latticefall  # noqa: F401  (registers the environments)
from latticefall.spaces import HEADS

RUNS = Path(__file__).resolve().parents[1] / "runs"


@dataclass
class Config:
    minutes: float = 30.0
    seed: int = 1
    num_envs: int = 16
    num_steps: int = 256
    epochs: int = 4
    minibatches: int = 4
    lr: float = 3e-4
    gamma: float = 0.99
    gae_lambda: float = 0.95
    clip: float = 0.2
    ent_coef: float = 0.01
    vf_coef: float = 0.5
    max_grad_norm: float = 0.5
    hidden: int = 256
    eval_episodes: int = 50
    threads: int = 4
    name: str = ""
    # Feature columns to zero out, as "proprio:I" or "target:I-J" (for ablations).
    ablate: str = ""


def parse() -> Config:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    for k, v in asdict(Config()).items():
        p.add_argument(f"--{k.replace('_', '-')}", type=type(v), default=v)
    return Config(**vars(p.parse_args()))


def ablation_mask(spec: str) -> np.ndarray:
    """A 0/1 multiplier over the concatenated observation for ``--ablate``."""
    keep = np.ones(48 + 32, dtype=np.float32)
    for part in filter(None, spec.split(",")):
        key, rng = part.split(":")
        lo, _, hi = rng.partition("-")
        base = 0 if key == "proprio" else 48
        keep[base + int(lo) : base + int(hi or lo) + 1] = 0.0
    return keep


def flat(obs: dict) -> np.ndarray:
    return np.concatenate([obs["proprio"], obs["target"]])


def flat_mask(info: dict) -> np.ndarray:
    return np.concatenate(info["action_mask"])


class Agent(nn.Module):
    def __init__(self, n_obs: int, hidden: int) -> None:
        super().__init__()

        def layer(i, o, std=np.sqrt(2)):
            lin = nn.Linear(i, o)
            nn.init.orthogonal_(lin.weight, std)
            nn.init.constant_(lin.bias, 0.0)
            return lin

        self.critic = nn.Sequential(
            layer(n_obs, hidden), nn.Tanh(), layer(hidden, hidden), nn.Tanh(), layer(hidden, 1, 1.0)
        )
        self.actor = nn.Sequential(
            layer(n_obs, hidden),
            nn.Tanh(),
            layer(hidden, hidden),
            nn.Tanh(),
            layer(hidden, sum(HEADS), 0.01),
        )

    def value(self, x: torch.Tensor) -> torch.Tensor:
        return self.critic(x).squeeze(-1)

    def act(self, x, mask, action=None, greedy=False):
        logits = self.actor(x).masked_fill(~mask, -1e8)
        dists = [torch.distributions.Categorical(logits=lg) for lg in torch.split(logits, HEADS, -1)]
        if action is None:
            if greedy:
                action = torch.stack([d.probs.argmax(-1) for d in dists], -1)
            else:
                action = torch.stack([d.sample() for d in dists], -1)
        logp = torch.stack([d.log_prob(a) for d, a in zip(dists, action.T)], -1).sum(-1)
        ent = torch.stack([d.entropy() for d in dists], -1).sum(-1)
        return action, logp, ent, self.value(x)


# The scripted baseline: the WP-034 bot rewritten over the observation vectors only.
# Indices follow game/crates/core/src/obs/encode.rs.
P_STAMINA, P_LOCKED, P_VIGIL, P_HP = 2, 34, 14, 0
T_DIST, T_MOVE, T_HIT_NOW, T_TO_HIT, T_LEFT, T_FINISHER = 5, 16, 18, 19, 20, 21
DODGE, LIGHT, USE_ITEM = 5, 1, 7


class Scripted:
    """Rolls through each hit and punishes the recovery after a combo's last move."""

    lead_ticks = 7

    def __init__(self) -> None:
        self.reset()

    def reset(self) -> None:
        self.seen: tuple[int, float] | None = None
        self.rolled = False

    def __call__(self, obs: dict, info: dict) -> np.ndarray:
        p, t = obs["proprio"], obs["target"]
        a = np.zeros(5, dtype=np.int64)
        if p[P_LOCKED] < 0.5:
            a[3] = 1
            return a
        dist = t[T_DIST] * 20.0
        if t[T_MOVE] > 0.5:
            key = (int(obs["move"][1]), float(t[17]))
            if self.seen is None or self.seen[0] != key[0] or self.seen[1] > key[1]:
                self.rolled = False
            self.seen = key
        else:
            self.seen, self.rolled = None, False
        to_hit = t[T_TO_HIT] * 30.0
        if t[T_MOVE] > 0.5 and t[T_TO_HIT] < 1.0 and not self.rolled and to_hit <= self.lead_ticks:
            self.rolled = True
            a[0], a[1], a[2] = 1, 1, DODGE
            return a
        left = t[T_LEFT] * 60.0
        if t[T_FINISHER] > 0.5 and t[T_HIT_NOW] < 0.5 and t[T_TO_HIT] >= 1.0 and left >= 14:
            if p[P_HP] < 0.45 and p[P_VIGIL] > 0 and left >= 30:
                a[2] = USE_ITEM
            elif dist > 2.0:
                a[0], a[1] = 1, 1
            elif p[P_STAMINA] > 0.3:
                a[2] = LIGHT
            return a
        if t[T_MOVE] < 0.5 and dist > 3.0:
            a[0], a[1] = 1, 1
        return a


def evaluate(policy, seeds, reset_hook=None) -> dict:
    env = gym.make("Latticefall/BossArena-v0")
    rets, wins, dealt, lens = [], 0, [], []
    for s in seeds:
        obs, info = env.reset(seed=int(s))
        if reset_hook:
            reset_hook()
        total, dmg, n = 0.0, 0.0, 0
        while True:
            obs, r, term, trunc, info = env.step(policy(obs, info))
            total += r
            dmg += info["reward_components"]["damage_dealt"]
            n += 1
            if term or trunc:
                break
        rets.append(total)
        wins += info["boss_defeated"]
        dealt.append(dmg)
        lens.append(n)
    env.close()
    return {
        "return_mean": float(np.mean(rets)),
        "return_std": float(np.std(rets)),
        "win_rate": wins / len(seeds),
        "boss_hp_removed_mean": float(np.mean(dealt)),
        "steps_mean": float(np.mean(lens)),
    }


def random_policy(rng):
    def pick(obs, info):
        return np.array([rng.choice(np.flatnonzero(m)) for m in info["action_mask"]])

    return pick


def main() -> None:
    cfg = parse()
    torch.set_num_threads(cfg.threads)
    torch.manual_seed(cfg.seed)
    rng = np.random.default_rng(cfg.seed)
    name = cfg.name or time.strftime("ppo_smoke-%Y%m%d-%H%M%S")
    out = RUNS / name
    out.mkdir(parents=True, exist_ok=True)
    log_file = (out / "log.txt").open("w")

    def log(msg: str) -> None:
        print(msg, flush=True)
        log_file.write(msg + "\n")
        log_file.flush()

    log(f"config {json.dumps(asdict(cfg))}")
    keep = torch.from_numpy(ablation_mask(cfg.ablate))
    envs = [gym.make("Latticefall/BossArena-v0") for _ in range(cfg.num_envs)]
    n_obs = 48 + 32
    agent = Agent(n_obs, cfg.hidden)
    opt = torch.optim.Adam(agent.parameters(), lr=cfg.lr, eps=1e-5)

    N, T = cfg.num_envs, cfg.num_steps
    obs_buf = torch.zeros(T, N, n_obs)
    mask_buf = torch.zeros(T, N, sum(HEADS), dtype=torch.bool)
    act_buf = torch.zeros(T, N, len(HEADS), dtype=torch.long)
    logp_buf = torch.zeros(T, N)
    rew_buf = torch.zeros(T, N)
    done_buf = torch.zeros(T, N)
    val_buf = torch.zeros(T, N)

    train_seed = 1_000_000 * cfg.seed
    cur_obs, cur_mask = [], []
    for i, e in enumerate(envs):
        o, info = e.reset(seed=train_seed + i)
        cur_obs.append(flat(o))
        cur_mask.append(flat_mask(info))
    next_seed = train_seed + N
    ep_ret = np.zeros(N)
    ep_dmg = np.zeros(N)
    recent = deque(maxlen=100)
    recent_win = deque(maxlen=100)
    recent_dmg = deque(maxlen=100)
    history = []

    budget = cfg.minutes * 60.0
    start = time.perf_counter()
    global_step, it = 0, 0
    sim_time = 0.0
    while time.perf_counter() - start < budget:
        it += 1
        frac_left = max(0.0, 1.0 - (time.perf_counter() - start) / budget)
        opt.param_groups[0]["lr"] = cfg.lr * frac_left
        it_start = time.perf_counter()
        for t in range(T):
            x = torch.from_numpy(np.stack(cur_obs)) * keep
            m = torch.from_numpy(np.stack(cur_mask).astype(bool))
            with torch.no_grad():
                a, logp, _, v = agent.act(x, m)
            obs_buf[t], mask_buf[t], act_buf[t], logp_buf[t], val_buf[t] = x, m, a, logp, v
            acts = a.numpy()
            s0 = time.perf_counter()
            for i, e in enumerate(envs):
                o, r, term, trunc, info = e.step(acts[i])
                ep_ret[i] += r
                ep_dmg[i] += info["reward_components"]["damage_dealt"]
                if trunc and not term:
                    # Bootstrap on truncation, not on termination.
                    with torch.no_grad():
                        r += cfg.gamma * float(agent.value(torch.from_numpy(flat(o)) * keep))
                rew_buf[t, i] = r
                done_buf[t, i] = float(term or trunc)
                if term or trunc:
                    recent.append(ep_ret[i])
                    recent_win.append(float(info["boss_defeated"]))
                    recent_dmg.append(ep_dmg[i])
                    ep_ret[i] = ep_dmg[i] = 0.0
                    o, info = e.reset(seed=next_seed)
                    next_seed += 1
                cur_obs[i] = flat(o)
                cur_mask[i] = flat_mask(info)
            sim_time += time.perf_counter() - s0
        global_step += N * T

        with torch.no_grad():
            nxt = agent.value(torch.from_numpy(np.stack(cur_obs)) * keep)
            adv = torch.zeros(T, N)
            last = torch.zeros(N)
            for t in reversed(range(T)):
                nv = nxt if t == T - 1 else val_buf[t + 1]
                nonterm = 1.0 - done_buf[t]
                delta = rew_buf[t] + cfg.gamma * nv * nonterm - val_buf[t]
                last = delta + cfg.gamma * cfg.gae_lambda * nonterm * last
                adv[t] = last
            ret = adv + val_buf

        b_obs, b_mask = obs_buf.reshape(T * N, -1), mask_buf.reshape(T * N, -1)
        b_act, b_logp = act_buf.reshape(T * N, -1), logp_buf.reshape(-1)
        b_adv, b_ret = adv.reshape(-1), ret.reshape(-1)
        mb = T * N // cfg.minibatches
        for _ in range(cfg.epochs):
            perm = torch.randperm(T * N)
            for k in range(cfg.minibatches):
                idx = perm[k * mb : (k + 1) * mb]
                _, logp, ent, v = agent.act(b_obs[idx], b_mask[idx], b_act[idx])
                ratio = (logp - b_logp[idx]).exp()
                a_ = b_adv[idx]
                a_ = (a_ - a_.mean()) / (a_.std() + 1e-8)
                pg = torch.max(-a_ * ratio, -a_ * ratio.clamp(1 - cfg.clip, 1 + cfg.clip)).mean()
                vl = 0.5 * (v - b_ret[idx]).pow(2).mean()
                loss = pg + cfg.vf_coef * vl - cfg.ent_coef * ent.mean()
                opt.zero_grad()
                loss.backward()
                nn.utils.clip_grad_norm_(agent.parameters(), cfg.max_grad_norm)
                opt.step()

        elapsed = time.perf_counter() - start
        row = {
            "iter": it,
            "steps": global_step,
            "minutes": round(elapsed / 60, 2),
            "sps": round(N * T / (time.perf_counter() - it_start)),
            "env_sps": round(global_step / max(sim_time, 1e-9)),
            "episodes": len(recent),
            "return_mean": float(np.mean(recent)) if recent else None,
            "win_rate": float(np.mean(recent_win)) if recent_win else None,
            "boss_hp_removed": float(np.mean(recent_dmg)) if recent_dmg else None,
            "entropy": float(ent.mean()),
            "value_loss": float(vl),
        }
        history.append(row)
        if it % 5 == 1 or time.perf_counter() - start >= budget:
            log(
                f"iter {it:4d} steps {global_step:>9,d} t {row['minutes']:6.2f}m "
                f"sps {row['sps']:>6,d} (env {row['env_sps']:>6,d}) "
                f"ret {row['return_mean'] if row['return_mean'] is not None else float('nan'):7.3f} "
                f"win {row['win_rate'] or 0:.2f} dmg {row['boss_hp_removed'] or 0:.3f} "
                f"ent {row['entropy']:.2f}"
            )
    train_minutes = (time.perf_counter() - start) / 60
    for e in envs:
        e.close()
    torch.save(agent.state_dict(), out / "policy.pt")

    log(f"training done: {global_step:,d} steps in {train_minutes:.1f} min "
        f"({global_step / (train_minutes * 60):,.0f} steps/s overall)")
    seeds = np.arange(cfg.eval_episodes) + 7_000_000  # held out from training seeds

    def ppo(greedy):
        def pick(obs, info):
            x = torch.from_numpy(flat(obs)).unsqueeze(0) * keep
            m = torch.from_numpy(flat_mask(info).astype(bool)).unsqueeze(0)
            with torch.no_grad():
                a, *_ = agent.act(x, m, greedy=greedy)
            return a[0].numpy()

        return pick

    bot = Scripted()
    results = {
        "random": evaluate(random_policy(np.random.default_rng(123)), seeds),
        "idle": evaluate(lambda o, i: np.zeros(5, dtype=np.int64), seeds),
        "scripted": evaluate(bot, seeds, bot.reset),
        "ppo": evaluate(ppo(False), seeds),
        "ppo_greedy": evaluate(ppo(True), seeds),
    }
    for k, r in results.items():
        log(
            f"eval {k:10s} return {r['return_mean']:7.3f} ± {r['return_std']:.3f} "
            f"win {r['win_rate']:.2f} boss_hp_removed {r['boss_hp_removed_mean']:.3f} "
            f"steps {r['steps_mean']:.0f}"
        )
    summary = {
        "config": asdict(cfg),
        "train_steps": global_step,
        "train_minutes": train_minutes,
        "steps_per_second": global_step / (train_minutes * 60),
        "env_steps_per_second": global_step / max(sim_time, 1e-9),
        "eval": results,
        "history": history,
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=1))
    log(f"wrote {out}")
    log_file.close()


if __name__ == "__main__":
    main()
