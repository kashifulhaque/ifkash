//! Hit resolution, tick system 7 (docs/game/01-architecture.md, "Tick schedule"): tests
//! every active hitbox against every other actor's hurtbox and emits `Hit` events.
//!
//! # Per-tick order
//!
//! [`run`] runs after the move state machines and locomotion, so moves, flags, and
//! positions are this tick's.
//!
//! 1. Start a fresh struck list for every actor whose move changed since last tick.
//! 2. Collect candidates, reading the world without changing it: for each attacker in
//!    arena index order, each hitbox active on this tick's frame, in list order, and each
//!    other living actor the move hasn't struck yet, in arena index order, sweep the
//!    hitbox from the attacker's previous-tick pose to its current one against the
//!    target's hurtbox (see [`hitbox::swept_overlap`]).
//! 3. Sort the candidates by attacker, hitbox index, and target, then decide each one
//!    with [`hit_outcome`]. A move hits each target at most once: the first hitbox in
//!    that order wins, and later ticks skip the target. A landing hit also gets its
//!    damage here (`combat::damage`), its counter-hit flag, and its guard inputs.
//! 4. Apply the landings in that same order: each goes through
//!    [`guard::apply_incoming_hit`], takes the HP it reports, emits its `Hit` event, and
//!    marks the target struck. Targets left at 0 HP are marked `dead` after the last
//!    landing.
//! 5. Record every actor's pose as the next tick's sweep start.
//!
//! # Determinism of the apply step
//!
//! Which hits land, and how much each deals before the guard, is decided in step 3 from
//! the state at the start of the system. Step 4 changes only the stamina, guard state,
//! and move of the targets (through the guard) and their HP, none of which step 3 reads
//! again, and `dead` is set only after the last landing, so an earlier hit never turns
//! a later one into [`guard::HitOutcome::Ignored`]. The one thing the apply order does
//! decide is how the stamina of a guarding target is shared out when several hits reach
//! it in one tick (the first blocks, a later one can break the guard). The order is the
//! sorted one, so it is the same on every run.

use lf_content::moves::Hitbox;
use serde::{Deserialize, Serialize};

use lf_content::GuardStats;
use lf_content::moves::HitTag;

use super::damage::{self, Damage};
use super::dodge;
use super::guard::{self, IncomingHit};
use super::hitbox::{self, Capsule, Pose};
use super::moves::{MoveNum, MoveTable};
use super::{crit, poise};
use crate::actor::Actor;
use crate::arena::Id;
use crate::events::{Event, Hit, HitResult};
use crate::math::dmath::{lerp, sqrt};
use crate::world::World;

/// Per-actor hit-resolution state. Simulation state: part of the state hash.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HitState {
    /// The actor's pose at the end of the previous tick's hit resolution: where its
    /// hitboxes and hurtbox sweep from. `None` before its first tick (no sweep).
    pub prev: Option<Pose>,
    /// The move and frame the `struck` list belongs to, as of the last tick.
    pub move_num: Option<MoveNum>,
    pub move_frame: u16,
    /// Targets the current move has already hit, sorted.
    pub struck: Vec<Id>,
}

impl HitState {
    /// Clears `struck` when the actor's move isn't the one it was last tick. A move
    /// advances at most one frame a tick and holds while charging, so a lower frame
    /// means the same move started again.
    fn sync(&mut self, a: &Actor) {
        let cur = a.moves.current;
        let num = cur.map(|c| c.num);
        let frame = cur.map_or(0, |c| c.frame);
        if num != self.move_num || frame < self.move_frame {
            self.struck.clear();
        }
        self.move_num = num;
        self.move_frame = frame;
    }
}

/// Whether an incoming hit lands on its target. The decision half of the hit rule:
/// [`guard::apply_incoming_hit`] then says what a landed hit does (block, break, or
/// connect) and returns a [`guard::HitOutcome`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitOutcome {
    /// The hit passes through (i-frames, or a dead target). The target isn't marked
    /// struck, so a later active frame of the same move can still land.
    Ignored,
    /// The hit lands: the target is marked struck, whatever the guard then does.
    Connect,
}

/// Decides whether a hit from `attacker`'s hitbox `hb` lands on `target`. The single
/// place that rule lives: every outcome except `Ignored` marks the target struck for
/// the move. It reads the same state `guard::apply_incoming_hit` does to ignore a hit,
/// so a landing hit is never ignored at apply time (unless the world changed in between).
pub fn hit_outcome(attacker: &Actor, target: &Actor, hb: &Hitbox) -> HitOutcome {
    let _ = (attacker, hb);
    if target.dead || dodge::is_invulnerable(&target.moves) {
        HitOutcome::Ignored
    } else {
        HitOutcome::Connect
    }
}

/// True when `target` is in the startup or active frames of one of its own attacks: it
/// has a move with hitboxes and hasn't passed the last active frame of the last one
/// (docs/game/03-combat.md, "Damage dealt", `counter_mult`).
pub(crate) fn is_counter(target: &Actor, moves: &MoveTable) -> bool {
    let Some(c) = target.moves.current else {
        return false;
    };
    moves
        .get(c.num)
        .and_then(|d| d.data.hitboxes.iter().map(|h| h.frames.1).max())
        .is_some_and(|last_active| c.frame <= last_active)
}

/// A hit that landed, decided from the world at the start of the system. Projectiles
/// (`combat::projectile`) build these too and apply them with [`apply_landing`].
pub(crate) struct Landing {
    pub(crate) attacker: Id,
    pub(crate) target: Id,
    pub(crate) move_num: MoveNum,
    pub(crate) hitbox: u8,
    pub(crate) motion_value: f32,
    pub(crate) charge_frac: f32,
    pub(crate) point: [f32; 3],
    pub(crate) dir: [f32; 3],
    /// Damage before the guard.
    pub(crate) damage: Damage,
    pub(crate) counter: bool,
    pub(crate) poise_damage: u32,
    pub(crate) unblockable: bool,
}

/// A hitbox touching a hurtbox this tick, before its outcome is decided.
struct Candidate {
    attacker: Id,
    hitbox: u8,
    target: Id,
    point: [f32; 3],
}

/// The hurtbox an actor sweeps through this tick.
struct Body {
    id: Id,
    hurt0: Capsule,
    hurt1: Capsule,
}

fn pose(a: &Actor) -> Pose {
    Pose {
        pos: a.pos,
        yaw: a.yaw,
    }
}

/// Unit XZ direction from `from` to `to`, or `fallback` if they're on the same spot.
fn dir_xz(from: [f32; 3], to: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    let len = sqrt(dx * dx + dz * dz);
    if len > 0.0 {
        [dx / len, 0.0, dz / len]
    } else {
        fallback
    }
}

/// Applies one landing: through the target's guard, to its HP, and out as a `Hit` event.
/// The apply step shared by melee ([`run`]) and projectiles (`combat::projectile`).
/// Returns false if the hit passed through (the target is gone or can't be hit), and then
/// changes nothing. The caller marks the target struck, because what that means differs:
/// a move's struck list for melee, a projectile's for a shot.
pub(crate) fn apply_landing(world: &mut World, l: &Landing) -> bool {
    // Only the player has a moveset until AI lands; anyone else can't be guarding.
    let guard_stats = if l.target == world.player {
        world.moves.player_moveset.as_ref().map(|m| m.guard)
    } else {
        None
    }
    .unwrap_or(GuardStats {
        absorb_phys_percent: 0,
        absorb_elem_percent: 0,
        guard_boost: 1,
        parry_window_ticks: 0,
    });
    let Some(t) = world.actors.get_mut(l.target) else {
        return false;
    };
    let incoming = IncomingHit {
        damage: l.damage.total,
        poise_damage: l.poise_damage,
        elemental: l.damage.mostly_elemental(),
        unblockable: l.unblockable,
    };
    let (taken, result) =
        match guard::apply_incoming_hit(t, incoming, &guard_stats, &world.locomotion.guard) {
            // Decided as a landing from the same state, so this can't happen; if it
            // ever does, the hit passes through like an i-frame hit.
            guard::HitOutcome::Ignored => return false,
            guard::HitOutcome::Connected { damage } => (damage, HitResult::Connected),
            guard::HitOutcome::Guarded { damage, .. } => (damage, HitResult::Guarded),
            guard::HitOutcome::GuardBroken { damage } => (damage, HitResult::GuardBroken),
        };
    let hp_before = t.hp;
    t.hp = hp_before.saturating_sub(taken);
    world.events.push(Event::Hit(Hit {
        attacker: l.attacker,
        target: l.target,
        move_num: l.move_num,
        hitbox: l.hitbox,
        motion_value: l.motion_value,
        charge_frac: l.charge_frac,
        point: l.point,
        dir: l.dir,
        damage: hp_before - t.hp,
        result,
        counter: l.counter,
        killed: hp_before > 0 && t.hp == 0,
        hp: t.hp,
    }));
    // A hit that wasn't blocked adds poise and stance damage (see `poise`). Projectile
    // hits count too.
    if result == HitResult::Connected {
        poise::apply_landing(world, l.target, l.move_num, l.poise_damage);
    }
    true
}

/// Tick system 7. See the module docs for the order.
pub fn run(world: &mut World) {
    let ids: Vec<Id> = world.actors.iter().map(|(id, _)| id).collect();
    // 1. Struck lists follow the current move.
    for &id in &ids {
        if let Some(a) = world.actors.get_mut(id) {
            let mut hit = core::mem::take(&mut a.hit);
            hit.sync(a);
            a.hit = hit;
        }
    }

    // 2. Candidates, from an unchanged world.
    let bodies: Vec<Body> = world
        .actors
        .iter()
        .filter(|(_, a)| !a.dead)
        .map(|(id, a)| {
            let prev = a.hit.prev.unwrap_or_else(|| pose(a));
            Body {
                id,
                hurt0: a.hurtbox.capsule(prev.pos),
                hurt1: a.hurtbox.capsule(a.pos),
            }
        })
        .collect();
    let mut candidates = Vec::new();
    for (aid, a) in world.actors.iter() {
        let Some(c) = a.moves.current.filter(|_| !a.dead) else {
            continue;
        };
        let Some(def) = world.moves.get(c.num) else {
            continue;
        };
        let now = pose(a);
        let before = a.hit.prev.unwrap_or(now);
        for (i, hb) in def.data.hitboxes.iter().enumerate() {
            if !(hb.frames.0..=hb.frames.1).contains(&c.frame) {
                continue;
            }
            let hit0 = Capsule::from_shape(&hb.shape, &before);
            let hit1 = Capsule::from_shape(&hb.shape, &now);
            for body in &bodies {
                if body.id == aid || a.hit.struck.binary_search(&body.id).is_ok() {
                    continue;
                }
                if let Some((_, point)) =
                    hitbox::swept_overlap(&hit0, &hit1, &body.hurt0, &body.hurt1)
                {
                    candidates.push(Candidate {
                        attacker: aid,
                        hitbox: u8::try_from(i).unwrap_or(u8::MAX),
                        target: body.id,
                        point,
                    });
                }
            }
        }
    }

    // 3. Outcomes, in attacker, hitbox, target order.
    candidates.sort_by_key(|c| (c.attacker, c.hitbox, c.target));
    let mut landings: Vec<Landing> = Vec::new();
    let mut parries: Vec<poise::Parry> = Vec::new();
    for cand in candidates {
        if landings
            .iter()
            .any(|l| (l.attacker, l.target) == (cand.attacker, cand.target))
            || parries
                .iter()
                .any(|p| (p.attacker, p.target) == (cand.attacker, cand.target))
        {
            continue;
        }
        let (Some(a), Some(t)) = (
            world.actors.get(cand.attacker),
            world.actors.get(cand.target),
        ) else {
            continue;
        };
        let Some(c) = a.moves.current else {
            continue;
        };
        let Some(def) = world.moves.get(c.num) else {
            continue;
        };
        let Some(hb) = def.data.hitboxes.get(usize::from(cand.hitbox)) else {
            continue;
        };
        match hit_outcome(a, t, hb) {
            HitOutcome::Ignored => continue,
            HitOutcome::Connect => {}
        }
        // A parry turns the hit aside before it lands (see `poise`).
        if poise::is_parry(
            a,
            t,
            poise::parry_window_of(world, cand.target),
            hb.tags.contains(&HitTag::Unparryable),
            &world.locomotion.poise,
        ) {
            parries.push(poise::Parry {
                attacker: cand.attacker,
                target: cand.target,
                move_num: c.num,
            });
            continue;
        }
        let charge_frac = world.moves.charge_frac(&c);
        let motion_value = match def.data.charge {
            // docs/game/03-combat.md, "Move data schema": the motion value rises from
            // its listed value to `motion_value_max` over the charge.
            Some(ch) => lerp(hb.motion_value, ch.motion_value_max, charge_frac),
            None => hb.motion_value,
        };
        let motion_value = crit::scale_motion(motion_value, a, c.num, &world.moves);
        // Damage, from the attacker's equipped weapon and stats. An actor with no
        // weapon has no attack rating and deals the minimum of 1.
        let ar = a
            .inventory
            .equipment
            .right_weapon()
            .and_then(|w| {
                world.damage.attack_rating(
                    w,
                    &a.stats.attrs,
                    a.stats.weapon_upgrade,
                    a.stats.two_handing,
                )
            })
            .unwrap_or_default();
        let counter = is_counter(t, &world.moves);
        let hit = damage::Hit {
            ar,
            phys: damage::phys_type(hb.phys),
            element: hb.element.map(damage::element_type),
            // Thousandths. Rounding the f32 is exact for the listed values (1.3 -> 1300).
            motion_milli: (motion_value * 1000.0).round().max(0.0) as u32,
            counter,
            buff_percent: a.stats.buff_percent,
        };
        landings.push(Landing {
            attacker: cand.attacker,
            target: cand.target,
            move_num: c.num,
            hitbox: cand.hitbox,
            motion_value,
            charge_frac,
            point: cand.point,
            dir: dir_xz(a.pos, t.pos, pose(a).forward()),
            damage: damage::damage(&world.damage.rules, &hit, &t.stats.defense),
            counter,
            poise_damage: u32::from(hb.poise_damage),
            unblockable: hb.tags.contains(&HitTag::Unblockable),
        });
    }

    // 4. Apply the parries, then the landings in the same order. See the module docs.
    poise::apply_parries(world, &parries);
    for l in &landings {
        if !apply_landing(world, l) {
            continue;
        }
        if let Some(a) = world.actors.get_mut(l.attacker)
            && let Err(at) = a.hit.struck.binary_search(&l.target)
        {
            a.hit.struck.insert(at, l.target);
        }
    }
    // An actor at 0 HP is dead. Set after the last landing so that two hits in one tick
    // both land (see the module docs). Respawn and drops are WP-029's.
    for l in &landings {
        if let Some(t) = world.actors.get_mut(l.target)
            && t.hp == 0
        {
            t.dead = true;
        }
    }

    // 5. This tick's pose is the next tick's sweep start.
    for id in ids {
        if let Some(a) = world.actors.get_mut(id) {
            a.hit.prev = Some(pose(a));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::hurtbox::Hurtbox;
    use crate::combat::moves::ActiveMove;
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::tick::tick;
    use lf_content::Content;

    const FRAC_PI_2: f32 = core::f32::consts::FRAC_PI_2;

    fn field() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(7, &content, "test_field").expect("test_field exists")
    }

    fn step(w: &mut World, buttons: u16) -> Vec<Hit> {
        let input = TickInput {
            frame: ActionFrame {
                move_x: 0,
                move_z: 0,
                buttons,
            },
            commands: Vec::new(),
        };
        tick(w, &input);
        w.events
            .drain()
            .into_iter()
            .filter_map(|e| match e {
                Event::Hit(h) => Some(h),
                _ => None,
            })
            .collect()
    }

    /// Moves the player to `z` on the center lane, facing +Z, as if it had stood there.
    fn place_player(w: &mut World, z: f32) {
        let p = w.actors.get_mut(w.player).unwrap();
        p.pos = [0.0, 0.0, z];
        p.yaw = 0.0;
        p.hit.prev = None;
    }

    fn dummy_ahead(w: &World) -> Id {
        // test_field lists dummy_ahead first; the player is index 0.
        let (id, d) = w.actors.iter().nth(1).unwrap();
        assert_eq!(d.pos, [0.0, 0.0, 6.0]);
        id
    }

    #[test]
    fn dummies_and_player_have_the_controller_capsule() {
        let w = field();
        let want = Hurtbox::from_locomotion(&w.locomotion);
        assert!(w.actors.iter().all(|(_, a)| a.hurtbox == want));
        assert_eq!(w.actors.len(), 4);
    }

    #[test]
    fn a_light_hits_the_dummy_once() {
        let mut w = field();
        let dummy = dummy_ahead(&w);
        place_player(&mut w, 4.0);
        let mut hits = step(&mut w, buttons::LIGHT);
        for _ in 0..40 {
            hits.extend(step(&mut w, 0));
        }
        assert_eq!(hits.len(), 1, "{hits:?}");
        let h = &hits[0];
        assert_eq!(h.attacker, w.player);
        assert_eq!(h.target, dummy);
        assert_eq!(h.move_num, w.moves.num_of("ss_light_1").unwrap());
        assert_eq!((h.hitbox, h.motion_value, h.charge_frac), (0, 1.0, 0.0));
        assert!((h.dir[2] - 1.0).abs() < 1e-6, "{:?}", h.dir);
        // The Wayfarer's Blade in the stub loadout: AR 135.24; raw 135.24 against the
        // dummy's flat 500 is 135.24^2 / 635.24 = 28.79, so 29 damage.
        assert_eq!((h.damage, h.hp, h.result), (29, 71, HitResult::Connected));
        assert!(!h.counter && !h.killed);
        assert_eq!(w.actors.get(dummy).unwrap().hp, 71);
    }

    #[test]
    fn iframes_ignore_the_hit_without_using_it_up() {
        let mut w = field();
        let dummy = dummy_ahead(&w);
        place_player(&mut w, 4.0);
        let light = w.moves.num_of("ss_light_1").unwrap();
        let (from, to) = w.moves.get(light).unwrap().data.hitboxes[0].frames;
        // The dummy rolls in place (only the player moves through locomotion), timed so
        // its i-frames cover every active frame of the light but the last.
        let roll = w.moves.player_moveset.as_ref().unwrap().roll_medium;
        let iframes = w.moves.get(roll).unwrap().data.iframes.unwrap();
        let offset = iframes.to - (to - 1);
        assert!(offset + from >= iframes.from, "i-frames start too late");
        w.actors.get_mut(dummy).unwrap().moves.current = Some(ActiveMove {
            num: roll,
            frame: offset - 1,
            charge_ticks: 0,
            hold_button: 0,
            free_from: u16::MAX,
        });
        let mut hits = Vec::new();
        for i in 0..40 {
            let new = step(&mut w, if i == 0 { buttons::LIGHT } else { 0 });
            if !new.is_empty() {
                assert_eq!(player_frame(&w), to, "landed on the last active frame");
            }
            hits.extend(new);
        }
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].target, dummy);
    }

    #[test]
    fn a_fast_swing_does_not_tunnel_through_a_thin_target() {
        let mut w = field();
        // Clear the field: the only target is a 2 cm pole.
        let others: Vec<Id> = w
            .actors
            .iter()
            .map(|(id, _)| id)
            .filter(|&id| id != w.player)
            .collect();
        for id in others {
            w.actors.remove(id);
        }
        place_player(&mut w, 0.0);
        let light = w.moves.num_of("ss_light_1").unwrap();
        let shape = w.moves.get(light).unwrap().data.hitboxes[0].shape;
        let first = w.moves.get(light).unwrap().data.hitboxes[0].frames.0;
        step(&mut w, buttons::LIGHT);
        for _ in 1..first {
            step(&mut w, 0);
        }
        // The player is at its frame-9 pose. Next tick it whips 90 degrees to face +X:
        // a swing far faster than the blade's width per tick.
        let p = w.actors.get(w.player).unwrap();
        let before = pose(p);
        let after = Pose {
            yaw: FRAC_PI_2,
            ..before
        };
        let (b0, b1) = (
            Capsule::from_shape(&shape, &before),
            Capsule::from_shape(&shape, &after),
        );
        // A pole halfway along the swing (on the chord the sweep interpolates along).
        let mid = b0.lerp(&b1, 0.5);
        let at = [
            0.5 * (mid.a[0] + mid.b[0]),
            0.0,
            0.5 * (mid.a[2] + mid.b[2]),
        ];
        let pole = w.actors.insert(Actor {
            pos: at,
            hp: 1,
            max_hp: 1,
            hurtbox: Hurtbox {
                radius: 0.02,
                half_height: 0.9,
            },
            ..Actor::default()
        });
        let hurt = w.actors.get(pole).unwrap().hurtbox.capsule(at);
        assert_eq!(
            hitbox::overlap(&b0, &hurt),
            None,
            "touches before the swing"
        );
        assert_eq!(hitbox::overlap(&b1, &hurt), None, "touches after the swing");
        w.actors.get_mut(w.player).unwrap().yaw = FRAC_PI_2;
        let hits = step(&mut w, 0);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].target, pole);
        assert_eq!(player_frame(&w), first);
    }

    fn player_frame(w: &World) -> u16 {
        w.actors.get(w.player).unwrap().moves.current.unwrap().frame
    }

    #[test]
    fn candidates_resolve_in_attacker_hitbox_target_order() {
        // Two dummies stacked on the same spot: both are hit on the same tick, in arena
        // order, and each only once.
        let mut w = field();
        place_player(&mut w, 4.0);
        let twin = w.actors.insert(Actor {
            pos: [0.0, 0.0, 6.0],
            hp: 100,
            max_hp: 100,
            hurtbox: Hurtbox::from_locomotion(&w.locomotion),
            ..Actor::default()
        });
        let mut hits = step(&mut w, buttons::LIGHT);
        for _ in 0..40 {
            hits.extend(step(&mut w, 0));
        }
        let targets: Vec<Id> = hits.iter().map(|h| h.target).collect();
        assert_eq!(targets, vec![dummy_ahead(&w), twin]);
    }

    // --- WP-023: damage, the guard, and counter hits ---

    /// An actor with the player's loadout and stats standing 1.2 m in front of the
    /// player, facing it, `frame` frames into a light attack.
    fn attacker_at_player(w: &mut World, frame: u16) -> Id {
        let p = w.actors.get(w.player).unwrap();
        let (inventory, stats) = (p.inventory.clone(), p.stats.clone());
        let light = w.moves.num_of("ss_light_1").unwrap();
        w.actors.insert(Actor {
            pos: [0.0, 0.0, 1.2],
            yaw: core::f32::consts::PI,
            hp: 100,
            max_hp: 100,
            hurtbox: Hurtbox::from_locomotion(&w.locomotion),
            inventory,
            stats,
            moves: crate::combat::moves::MoveState {
                current: Some(ActiveMove {
                    num: light,
                    frame,
                    charge_ticks: 0,
                    hold_button: 0,
                    free_from: u16::MAX,
                }),
                ..Default::default()
            },
            ..Actor::default()
        })
    }

    /// Steps until the first hit on `target`, with the player's input `buttons` held
    /// the whole time, and returns it.
    fn first_hit_on(w: &mut World, target: Id, buttons: u16) -> Hit {
        for _ in 0..40 {
            if let Some(h) = step(w, buttons).into_iter().find(|h| h.target == target) {
                return h;
            }
        }
        panic!("no hit on the target");
    }

    fn player_hp(w: &World) -> u32 {
        w.actors.get(w.player).unwrap().hp
    }

    /// The stamina a block of the light costs: `poise_damage * 100 / guard_boost`.
    fn light_block_cost(w: &World) -> f32 {
        let light = w.moves.num_of("ss_light_1").unwrap();
        let poise = w.moves.get(light).unwrap().data.hitboxes[0].poise_damage;
        let boost = w.moves.player_moveset.as_ref().unwrap().guard.guard_boost;
        f32::from(poise) * 100.0 / f32::from(boost)
    }

    #[test]
    fn an_unguarded_hit_takes_full_damage_from_the_player() {
        let mut w = field();
        let player = w.player;
        attacker_at_player(&mut w, 0);
        let hp = player_hp(&w);
        let h = first_hit_on(&mut w, player, 0);
        // AR 135.24 against the player's flat 40: 135.24^2 / 175.24 = 104.4.
        assert_eq!((h.damage, h.result), (104, HitResult::Connected));
        assert_eq!(player_hp(&w), hp - 104);
        assert_eq!(h.hp, hp - 104);
    }

    #[test]
    fn a_guarded_hit_costs_stamina_and_takes_half_the_damage() {
        let mut w = field();
        let player = w.player;
        for _ in 0..3 {
            step(&mut w, buttons::GUARD);
        }
        assert!(w.actors.get(w.player).unwrap().moves.guard.active);
        attacker_at_player(&mut w, 0);
        let (hp, max_stamina) = {
            let p = w.actors.get(w.player).unwrap();
            (p.hp, p.max_stamina)
        };
        let h = first_hit_on(&mut w, player, buttons::GUARD);
        // The blade absorbs 50 percent of the 104: 52.
        assert_eq!((h.damage, h.result), (52, HitResult::Guarded));
        assert_eq!(player_hp(&w), hp - 52);
        let cost = light_block_cost(&w);
        assert_eq!(cost, 30.0);
        assert_eq!(w.actors.get(w.player).unwrap().stamina, max_stamina - cost);
    }

    #[test]
    fn a_hit_that_overruns_the_stamina_breaks_the_guard() {
        let mut w = field();
        let player = w.player;
        for _ in 0..3 {
            step(&mut w, buttons::GUARD);
        }
        w.actors.get_mut(w.player).unwrap().stamina = 10.0;
        attacker_at_player(&mut w, 0);
        let hp = player_hp(&w);
        let h = first_hit_on(&mut w, player, buttons::GUARD);
        // Stamina covers 10 / 30 of the block: 35 of the 104 is blocked at 50 percent
        // (18, halves up) and the other 69 lands in full.
        assert_eq!((h.damage, h.result), (87, HitResult::GuardBroken));
        assert_eq!(player_hp(&w), hp - 87);
        let p = w.actors.get(w.player).unwrap();
        assert_eq!(p.stamina, 0.0);
        assert!(p.moves.guard.stagger > 0 && !p.moves.guard.active);
    }

    #[test]
    fn the_dummy_loses_hp_and_dies_at_zero() {
        let mut w = field();
        let dummy = dummy_ahead(&w);
        place_player(&mut w, 4.0);
        w.actors.get_mut(dummy).unwrap().hp = 20;
        let hit = {
            step(&mut w, buttons::LIGHT);
            first_hit_on(&mut w, dummy, 0)
        };
        // The 29 damage is cut to the 20 HP it had.
        assert_eq!((hit.damage, hit.hp, hit.killed), (20, 0, true));
        let d = w.actors.get(dummy).unwrap();
        assert_eq!((d.hp, d.dead), (0, true));
        // A dead actor takes no more hits.
        for _ in 0..60 {
            assert!(step(&mut w, 0).iter().all(|h| h.target != dummy));
        }
    }

    #[test]
    fn two_hits_in_one_tick_both_land_on_a_target_the_first_kills() {
        let mut w = field();
        let a = attacker_at_player(&mut w, 0);
        let b = attacker_at_player(&mut w, 0);
        w.actors.get_mut(w.player).unwrap().hp = 1;
        let mut hits = Vec::new();
        for _ in 0..40 {
            hits.extend(step(&mut w, 0).into_iter().filter(|h| h.target == w.player));
        }
        assert_eq!(hits.len(), 2, "{hits:?}");
        assert_eq!((hits[0].attacker, hits[1].attacker), (a, b));
        assert_eq!((hits[0].killed, hits[0].damage), (true, 1));
        assert_eq!((hits[1].killed, hits[1].damage, hits[1].hp), (false, 0, 0));
        assert!(w.actors.get(w.player).unwrap().dead);
    }

    #[test]
    fn a_target_in_its_own_attack_takes_the_counter_multiplier() {
        let mut w = field();
        let dummy = dummy_ahead(&w);
        place_player(&mut w, 4.0);
        let light = w.moves.num_of("ss_light_1").unwrap();
        // The dummy starts its own light on the same tick: startup when the player's
        // blade arrives.
        w.actors.get_mut(dummy).unwrap().moves.current = Some(ActiveMove {
            num: light,
            frame: 0,
            charge_ticks: 0,
            hold_button: 0,
            free_from: u16::MAX,
        });
        step(&mut w, buttons::LIGHT);
        let h = first_hit_on(&mut w, dummy, 0);
        assert!(h.counter, "{h:?}");
        // Raw 135.24 * 1.15 = 155.526; 155.526^2 / 655.526 = 36.9.
        assert_eq!((h.damage, h.hp), (37, 63));
    }
}
