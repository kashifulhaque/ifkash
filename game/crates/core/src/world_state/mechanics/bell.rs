//! Bells and flag levers (WP-061; docs/game/02-world.md, "Mini-dungeons" D2 and D3).
//!
//! A bell is a lever whose target is [`LeverTarget::Bell`]: `INTERACT` rings it. Ringing
//! stuns every living enemy within the bell's radius: it breaks their stance, so they kneel
//! for `stun_ticks` and a riposte can start. The Drowned Belfry's bells use it to open its
//! gallery door and the bell-shaft lift's gate, and to stun the elite Choir Knight in its
//! loft. A bell that names a door in `opens` opens it for good.
//!
//! A flag lever ([`LeverTarget::Flag`]) raises a world flag once: the Sunken Ossuary's
//! drain levers (`lowered_by` on its pools, `wakes_on` on its Drowned) and the Belfry's
//! sluice lever (`sluice.mirrormere.open`, see [`super::water_level`]).
//!
//! Both are world flags and kneel counters, so they are saved and deterministic: the
//! radius and duration are level data, not constants.

use lf_content::scenario::LeverTarget;

use super::{flags, mechanic, set_door};
use crate::events::{Event, StanceBreak};
use crate::world::World;

/// The `MechanicEvent` kind of a bell ringing; its `index` is the lever's. (WP-061 reserves
/// kinds 30-39.)
pub const BELL_RUNG: u8 = 30;

/// How far above or below the bell an enemy still feels it, in meters. About one storey.
const STUN_HEIGHT_M: f32 = 4.0;

/// Works a lever whose target is a flag or a bell; true if it was one. Called by
/// `pull_lever` after it flips the lever's own flag.
pub fn pull(world: &mut World, lever: usize) -> bool {
    let l = world.layout.levers[lever].clone();
    match &l.target {
        LeverTarget::Flag(flag) => {
            world.state.flags.raise(flag);
            true
        }
        LeverTarget::Bell {
            radius_m,
            stun_ticks,
            opens,
        } => {
            ring(world, lever, l.pos, *radius_m, *stun_ticks);
            if let Some(id) = opens
                && let Some(d) = world.layout.doors.iter().position(|d| &d.id == id)
                && !world.state.flags.is_set(&flags::door_open(id))
            {
                set_door(world, d, true);
            }
            true
        }
        _ => false,
    }
}

/// Stuns the enemies within `radius` of `at`.
fn ring(world: &mut World, lever: usize, at: [f32; 3], radius: f32, ticks: u8) {
    mechanic(world, BELL_RUNG, lever);
    let hit: Vec<_> = world
        .actors
        .iter()
        .filter(|(id, a)| {
            *id != world.player
                && a.ai.is_some()
                && !a.is_ally()
                && !a.dead
                && a.hp > 0
                && a.poise.kneel == 0
        })
        .filter(|(_, a)| {
            let (dx, dz) = (a.pos[0] - at[0], a.pos[2] - at[2]);
            dx * dx + dz * dz <= radius * radius && (a.pos[1] - at[1]).abs() <= STUN_HEIGHT_M
        })
        .map(|(id, _)| id)
        .collect();
    for id in hit {
        let Some(a) = world.actors.get_mut(id) else {
            continue;
        };
        a.moves.interrupt();
        a.moves.guard.drop_guard();
        a.moves.guard.counter = 0;
        a.moves.guard.stagger = a.moves.guard.stagger.max(ticks);
        a.poise.kneel = ticks;
        a.poise.accum = 0;
        a.poise.unaware = false;
        world
            .events
            .push(Event::StanceBreak(StanceBreak { target: id, ticks }));
    }
}
