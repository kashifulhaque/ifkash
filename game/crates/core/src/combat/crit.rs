//! Critical hits: the riposte and the backstab (docs/game/03-combat.md, "Parry and
//! critical hits").
//!
//! A crit is a pair of moves, defined in `data/moves/crits.ron` and found by their IDs
//! ([`CritMoves`]): the attacker plays `crit_riposte` or `crit_backstab`, and the victim
//! plays the matching `crit_receive_*`. Both are locked until the moves end (the victim
//! through the same lockout a stagger uses, the attacker through its move). The damage
//! comes from the attacker's crit hitbox through normal hit resolution: the weapon's
//! attack rating times the hitbox's motion value 3.0, times the weapon's crit multiplier
//! ([`scale_motion`]).
//!
//! # Triggers
//!
//! `LIGHT` (buffered) while free to act, with a living actor in range:
//!
//! - **Riposte**: within `riposte_range_m` of a parried or stance-broken actor, with the
//!   attacker in the victim's front half-plane.
//! - **Backstab**: within `backstab_range_m` of an unaware actor, with the attacker inside
//!   the `backstab_arc_deg` arc behind the victim.
//!
//! When several actors qualify the nearest wins (the lowest arena index on a tie), and a
//! riposte beats a backstab at the same distance. Starting a crit happens in two halves
//! around [`moves::step`]: [`plan`] before it (and it takes the `LIGHT` press, so the
//! step doesn't start an ordinary attack) and [`start`] after it (so the crit move's frame
//! 0 is the tick it starts, like any move).
//!
//! Not done here: a backstab on an unaware enemy is meant to add 20 percent status
//! buildup; the status system doesn't exist yet. The `Critical` event carries the kind so
//! it can.

use lf_content::Poise;

use super::guard::GuardState;
use super::moves::{Action, ActiveMove, MoveNum, MoveTable};
use crate::actor::Actor;
use crate::arena::Id;
use crate::events::{CritKind, Critical, Event};
use crate::input::{ActionFrame, buttons};
use crate::math::dmath::{atan2, cos, sin, sqrt};
use crate::world::World;

/// The crit moves, resolved from their string IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CritMoves {
    pub riposte: MoveNum,
    pub backstab: MoveNum,
    pub receive_riposte: MoveNum,
    pub receive_backstab: MoveNum,
}

impl CritMoves {
    /// Looks the four moves up by ID. `None` if any is missing (partial content).
    pub fn resolve(table: &MoveTable) -> Option<Self> {
        Some(Self {
            riposte: table.num_of("crit_riposte")?,
            backstab: table.num_of("crit_backstab")?,
            receive_riposte: table.num_of("crit_receive_riposte")?,
            receive_backstab: table.num_of("crit_receive_backstab")?,
        })
    }

    /// The `(attacker, victim)` moves of a crit kind.
    pub fn pair(&self, kind: CritKind) -> (MoveNum, MoveNum) {
        match kind {
            CritKind::Riposte => (self.riposte, self.receive_riposte),
            CritKind::Backstab => (self.backstab, self.receive_backstab),
        }
    }

    /// True for the moves an attacker plays.
    pub fn is_attacker_move(&self, num: MoveNum) -> bool {
        num == self.riposte || num == self.backstab
    }
}

/// A crit that is about to start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CritPlan {
    pub victim: Id,
    pub kind: CritKind,
}

/// `motion_value` scaled by the attacker's weapon's crit multiplier if `num` is a crit
/// move (Critical hits: damage is `AR * 3.0 * crit_mult`; the 3.0 is the hitbox's motion
/// value). Any other move keeps its motion value.
pub fn scale_motion(motion_value: f32, attacker: &Actor, num: MoveNum, table: &MoveTable) -> f32 {
    if !table.crits.is_some_and(|c| c.is_attacker_move(num)) {
        return motion_value;
    }
    let percent = attacker
        .inventory
        .equipment
        .right_weapon()
        .and_then(|w| table.moveset_of(w))
        .map_or(100, |m| m.crit_mult_percent);
    motion_value * f32::from(percent) / 100.0
}

/// Which crit, if any, `attacker` would start against `victim`, from their positions and
/// states alone.
pub fn kind_against(attacker: &Actor, victim: &Actor, t: &Poise) -> Option<(CritKind, f32)> {
    if victim.dead {
        return None;
    }
    let (dx, dz) = (
        attacker.pos[0] - victim.pos[0],
        attacker.pos[2] - victim.pos[2],
    );
    let dist = sqrt(dx * dx + dz * dz);
    if dist <= 0.0 {
        return None;
    }
    // Unit vector from the victim to the attacker, and the victim's facing.
    let to = [dx / dist, dz / dist];
    let front = to[0] * sin(victim.yaw) + to[1] * cos(victim.yaw);
    let down = victim.poise.parried > 0 || victim.poise.kneel > 0;
    if down && dist <= t.riposte_range_m && front > 0.0 {
        return Some((CritKind::Riposte, dist));
    }
    // The rear arc is `backstab_arc_deg` wide, so its half angle's cosine is the least a
    // dot product with the victim's back direction can be. A dot product, not an angle,
    // keeps this free of inverse trig.
    let half_arc = t.backstab_arc_deg * 0.5 * (core::f32::consts::PI / 180.0);
    if victim.poise.unaware && dist <= t.backstab_range_m && -front >= cos(half_arc) {
        return Some((CritKind::Backstab, dist));
    }
    None
}

/// Decides whether `id` starts a crit this tick, before its move state runs. If so, takes
/// the actor's `LIGHT` press (so [`moves::step`] won't start an attack with it) and
/// returns the plan for [`start`].
pub fn plan(world: &mut World, id: Id, input: Option<ActionFrame>) -> Option<CritPlan> {
    world.moves.crits?;
    let t = world.locomotion.poise;
    let a = world.actors.get(id)?;
    // A LIGHT press is buffered inside the step, which hasn't run yet, so this tick's
    // press edge counts too.
    let edge = input.is_some_and(|f| f.buttons & buttons::LIGHT != 0)
        && a.moves.prev_buttons & buttons::LIGHT == 0;
    let free = !a.dead
        && a.moves.current.is_none()
        && a.moves.guard.can_act()
        && a.motor.grounded
        && (edge || a.moves.buffer.is_some_and(|b| b.action == Action::Light));
    if !free {
        return None;
    }
    let mut best: Option<(CritPlan, f32)> = None;
    for (vid, v) in world.actors.iter() {
        if vid == id {
            continue;
        }
        if let Some((kind, dist)) = kind_against(a, v, &t)
            && best.is_none_or(|(_, d)| dist < d)
        {
            best = Some((CritPlan { victim: vid, kind }, dist));
        }
    }
    let plan = best?.0;
    if let Some(a) = world.actors.get_mut(id) {
        a.moves.buffer = None;
        // Count the press as already seen, so the step doesn't buffer it again.
        a.moves.prev_buttons |= buttons::LIGHT;
    }
    Some(plan)
}

/// Starts the crit `plan` made for `attacker`, after its move state ran: the attacker
/// turns to the victim and plays its crit move, and the victim is locked in the matching
/// receive move.
pub fn start(world: &mut World, attacker: Id, plan: CritPlan) {
    let Some(crits) = world.moves.crits else {
        return;
    };
    let (att_num, vic_num) = crits.pair(plan.kind);
    let total_of = |n: MoveNum| world.moves.get(n).map_or(0, |d| d.data.total_frames);
    let (att_total, vic_total) = (total_of(att_num), total_of(vic_num));
    let Some(v_pos) = world.actors.get(plan.victim).map(|v| v.pos) else {
        return;
    };
    if let Some(a) = world.actors.get_mut(attacker) {
        a.yaw = atan2(v_pos[0] - a.pos[0], v_pos[2] - a.pos[2]);
        a.moves.guard.drop_guard();
        a.moves.interrupt();
        a.moves.current = Some(ActiveMove {
            num: att_num,
            frame: 0,
            charge_ticks: 0,
            hold_button: 0,
            free_from: att_total,
        });
    }
    if let Some(v) = world.actors.get_mut(plan.victim) {
        v.moves.interrupt();
        v.moves.current = Some(ActiveMove {
            num: vic_num,
            frame: 0,
            charge_ticks: 0,
            hold_button: 0,
            free_from: vic_total,
        });
        // Locked for the whole receive move, like a stagger. The parried and stance-broken
        // states are used up by the crit.
        v.moves.guard = GuardState {
            stagger: u8::try_from(vic_total).unwrap_or(u8::MAX),
            ..GuardState::default()
        };
        v.poise.parried = 0;
        v.poise.kneel = 0;
        v.poise.stance_dmg = 0;
        v.poise.accum = 0;
    }
    world.events.push(Event::Critical(Critical {
        attacker,
        target: plan.victim,
        kind: plan.kind,
        move_num: att_num,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::tick::tick;
    use lf_content::Content;

    fn tuning() -> Poise {
        Content::load_embedded().unwrap().locomotion.unwrap().poise
    }

    fn at(pos: [f32; 3], yaw: f32) -> Actor {
        Actor {
            pos,
            yaw,
            hp: 100,
            max_hp: 100,
            ..Actor::default()
        }
    }

    #[test]
    fn riposte_needs_a_downed_victim_in_range_and_in_front() {
        let t = tuning();
        let attacker = at([0.0, 0.0, 0.0], 0.0);
        // The victim stands 1.4 m ahead, facing the attacker (yaw pi).
        let mut victim = at([0.0, 0.0, 1.4], core::f32::consts::PI);
        assert_eq!(kind_against(&attacker, &victim, &t), None, "standing");
        victim.poise.parried = 10;
        assert_eq!(
            kind_against(&attacker, &victim, &t).map(|k| k.0),
            Some(CritKind::Riposte)
        );
        victim.poise.parried = 0;
        victim.poise.kneel = 10;
        assert!(kind_against(&attacker, &victim, &t).is_some(), "kneeling");
        // Out of range: just over 1.5 m.
        victim.pos = [0.0, 0.0, 1.6];
        assert_eq!(kind_against(&attacker, &victim, &t), None);
        // Facing away: the attacker is behind it, so no riposte (and it isn't unaware).
        victim.pos = [0.0, 0.0, 1.4];
        victim.yaw = 0.0;
        assert_eq!(kind_against(&attacker, &victim, &t), None);
        victim.dead = true;
        victim.yaw = core::f32::consts::PI;
        assert_eq!(kind_against(&attacker, &victim, &t), None, "dead");
    }

    #[test]
    fn backstab_needs_an_unaware_victim_in_its_rear_arc() {
        let t = tuning();
        let attacker = at([0.0, 0.0, 0.0], 0.0);
        // The victim is 1.0 m ahead facing +Z, away from the attacker.
        let mut victim = at([0.0, 0.0, 1.0], 0.0);
        assert_eq!(kind_against(&attacker, &victim, &t), None, "aware");
        victim.poise.unaware = true;
        assert_eq!(
            kind_against(&attacker, &victim, &t).map(|k| k.0),
            Some(CritKind::Backstab)
        );
        // Out of range.
        victim.pos = [0.0, 0.0, 1.3];
        assert_eq!(kind_against(&attacker, &victim, &t), None);
        // Inside the arc at 40 degrees off the back, outside it at 50.
        let spot = |deg: f32| {
            let r = deg * core::f32::consts::PI / 180.0;
            // The attacker's offset from the victim: 1.0 m, `deg` off straight behind it.
            [sin(r), 0.0, -cos(r)]
        };
        victim.pos = [0.0, 0.0, 5.0];
        for (deg, hit) in [(40.0, true), (50.0, false)] {
            let o = spot(deg);
            let a = at([victim.pos[0] + o[0], 0.0, victim.pos[2] + o[2]], 0.0);
            assert_eq!(
                kind_against(&a, &victim, &t).is_some(),
                hit,
                "{deg} degrees"
            );
        }
        // In front of an unaware actor is no backstab.
        victim.yaw = core::f32::consts::PI;
        let a = at([0.0, 0.0, 4.0], 0.0);
        assert_eq!(kind_against(&a, &victim, &t), None);
    }

    #[test]
    fn crit_motion_uses_the_weapons_multiplier() {
        let w = World::new(1, &Content::load_embedded().unwrap(), "duel_backstab").unwrap();
        let p = w.actors.get(w.player).unwrap();
        let crit = w.moves.num_of("crit_backstab").unwrap();
        let light = w.moves.num_of("ss_light_1").unwrap();
        // The parrying dagger's crit multiplier is 1.4.
        assert!((scale_motion(3.0, p, crit, &w.moves) - 4.2).abs() < 1e-4);
        assert_eq!(scale_motion(1.0, p, light, &w.moves), 1.0);
    }

    fn press_light(w: &mut World) {
        tick(
            w,
            &TickInput {
                frame: ActionFrame {
                    move_x: 0,
                    move_z: 0,
                    buttons: buttons::LIGHT,
                },
                commands: Vec::new(),
            },
        );
    }

    #[test]
    fn light_behind_an_unaware_actor_starts_the_paired_crit() {
        let mut w = World::new(1, &Content::load_embedded().unwrap(), "duel_backstab").unwrap();
        let sentry = w.actors.iter().find(|(id, _)| *id != w.player).unwrap().0;
        press_light(&mut w);
        let p = w.actors.get(w.player).unwrap();
        let crits = w.moves.crits.unwrap();
        assert_eq!(
            p.moves.current.map(|c| (c.num, c.frame)),
            Some((crits.backstab, 0))
        );
        let v = w.actors.get(sentry).unwrap();
        assert_eq!(v.moves.current.map(|c| c.num), Some(crits.receive_backstab));
        assert!(v.moves.guard.stagger > 0);
    }

    #[test]
    fn light_in_front_of_an_aware_actor_is_an_ordinary_attack() {
        let mut w = World::new(1, &Content::load_embedded().unwrap(), "duel_backstab").unwrap();
        let sentry = w.actors.iter().find(|(id, _)| *id != w.player).unwrap().0;
        // Turn the sentry to face the player and make it aware.
        let s = w.actors.get_mut(sentry).unwrap();
        s.yaw = core::f32::consts::PI;
        s.poise.unaware = false;
        press_light(&mut w);
        let p = w.actors.get(w.player).unwrap();
        assert_eq!(p.moves.current.map(|c| c.num), w.moves.num_of("ss_light_1"));
    }
}
