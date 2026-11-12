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
//!    that order wins, and later ticks skip the target.
//! 4. Record every actor's pose as the next tick's sweep start.
//!
//! Because every outcome is decided from the state at the start of the system, the
//! order hits are applied in can't change which hits land.

use lf_content::moves::Hitbox;
use serde::{Deserialize, Serialize};

use super::hitbox::{self, Capsule, Pose};
use super::moves::{MoveNum, flags};
use crate::actor::Actor;
use crate::arena::Id;
use crate::events::{Event, Hit};
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

/// What an incoming hit does to its target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitOutcome {
    /// The hit passes through (i-frames). The target isn't marked struck, so a later
    /// active frame of the same move can still land.
    Ignored,
    /// The hit lands.
    Connect,
}

/// Decides what a hit from `attacker`'s hitbox `hb` does to `target`. The single place
/// that rule lives: guarding, guard breaks, and parries (WP-022, WP-027) add outcomes
/// here, and every outcome except `Ignored` marks the target struck for the move.
pub fn hit_outcome(attacker: &Actor, target: &Actor, hb: &Hitbox) -> HitOutcome {
    let _ = (attacker, hb);
    if target.moves.has(flags::IFRAMES) {
        HitOutcome::Ignored
    } else {
        HitOutcome::Connect
    }
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
    let mut landed: Vec<(Id, Id)> = Vec::new();
    for cand in candidates {
        if landed.contains(&(cand.attacker, cand.target)) {
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
        let charge_frac = world.moves.charge_frac(&c);
        let motion_value = match def.data.charge {
            // docs/game/03-combat.md, "Move data schema": the motion value rises from
            // its listed value to `motion_value_max` over the charge.
            Some(ch) => lerp(hb.motion_value, ch.motion_value_max, charge_frac),
            None => hb.motion_value,
        };
        let event = Hit {
            attacker: cand.attacker,
            target: cand.target,
            move_num: c.num,
            hitbox: cand.hitbox,
            motion_value,
            charge_frac,
            point: cand.point,
            dir: dir_xz(a.pos, t.pos, pose(a).forward()),
        };
        landed.push((cand.attacker, cand.target));
        world.events.push(Event::Hit(event));
    }
    // Damage is WP-023: a landed hit only marks its target struck for now.
    for (attacker, target) in landed {
        if let Some(a) = world.actors.get_mut(attacker)
            && let Err(at) = a.hit.struck.binary_search(&target)
        {
            a.hit.struck.insert(at, target);
        }
    }

    // 4. This tick's pose is the next tick's sweep start.
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
            .map(|e| match e {
                Event::Hit(h) => h,
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
        // The dummy takes no damage until WP-023.
        assert_eq!(w.actors.get(dummy).unwrap().hp, 100);
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
}
