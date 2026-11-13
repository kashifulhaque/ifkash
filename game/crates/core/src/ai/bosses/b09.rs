//! B09, the Lamplight Matriarch (WP-077): her flight, the drained light, and the lamps. See
//! docs/game/05-enemies-and-bosses.md, "B09 The Lamplight Matriarch", and
//! `data/bosses/b09_lamplight_matriarch.ron`, where every number below lives (`params`).
//!
//! # Flight
//!
//! She is one AI actor with the `flying` rule, which keeps her feet near the floor. [`run`]
//! decides how high she is and what she may do, every tick of the Fight stage:
//!
//! - **Air phase.** She rises to `air_height_m`. Only the options whose move is named
//!   `b09_air_*` can start: the module sets the cooldown of every `b09_gnd_*` option to a tick or
//!   two, which the AI's choice already respects. A dive move takes her down and up by the
//!   move's frame (`dive_*`); counting each dive, she lands after `dives_per_air` of them, once
//!   her move is over. A stance break lands her at once. At `air_height_m` her capsule is out of
//!   a sword's reach, so in the air only a ranged hit can fill her stance meter. A `Retreat`
//!   behavior option is an air option as well (the ground phase masks it).
//! - **Ground phase.** She settles to `ground_height_m` and the `b09_gnd_*` options replace the
//!   air ones. After `ground_ticks`, once her move is over and she isn't kneeling, she takes off.
//!
//! The module sets her height before the locomotion system runs; the `flying` rule's own
//! correction toward the floor is a centimeter a tick (`climb_mps` in her definition), so it
//! doesn't fight this.
//!
//! # The drained light and the lamps
//!
//! When phase 2 starts (the framework's HP phase; she plays `b09_drain` first) the arena goes
//! dark: [`light_at`] answers for the area's `lit_at`, so only the ground within `lamp_light_m`
//! of a lit lamp is lit, and the renderer fogs the rest. Six lamps ring the middle. A lamp lights
//! (by [`light_lamps`]) when
//!
//! - the player's Fire reaches it: an active hitbox of the player's move that deals Fire (its own
//!   element, the weapon's, or an oil's) overlaps the lamp, or a Fire projectile of theirs
//!   passes it;
//! - the player carries the smoltorch (the darkness rule's `torch_flag`) and stays within
//!   `torch_m` of it for `torch_hold_ticks`; or
//! - the player casts Lantern Flare (the tick `Mech::flare_until` rises) within `flare_m`.
//!
//! A lit lamp lights the ground around it and, while she flies within `lamp_pass_m` of it,
//! stuns her: she crashes to the floor and kneels for `lamp_stun_ticks`, open to a riposte, and
//! it can't happen again for `lamp_stun_cooldown_ticks`. Lamps stay lit until the fight ends or
//! resets.
//!
//! State: [`State`], kept in `World::b09` (hashed only while it is not at rest). It is back to
//! its default whenever the fight isn't on, so a reset or a defeat relights the cavern.

use lf_content::BossDef;
use lf_content::enemy::Behavior;
use lf_content::moves::Element;
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::ai::boss::Stage;
use crate::ai::{Doing, OptionKind, Token};
use crate::arena::Id;
use crate::combat::hitbox::{Capsule, Pose, closest_points};
use crate::combat::hurtbox::Hurtbox;
use crate::combat::moves::ActiveMove;
use crate::events::{Event, MechanicEvent};
use crate::world::{DT, World};

/// The boss definition this module drives.
pub const BOSS_ID: &str = "b09_lamplight_matriarch";
/// Both dives (`b09_air_dive` and `b09_air_dive_short`) take her down and up by the move's frame.
const DIVE_PREFIX: &str = "b09_air_dive";
const STUNNED_MOVE: &str = "b09_stunned";
/// The prefixes that say which mode a move belongs to (see the module docs).
const AIR_PREFIX: &str = "b09_air_";
const GROUND_PREFIX: &str = "b09_gnd_";
/// How many lamps ring the arena (docs "B09": six).
const LAMPS: usize = 6;
/// The ticks a masked option's cooldown is held at. The AI counts cooldowns down every tick, so
/// holding one at 2 each tick keeps the option unavailable without a long cooldown to undo.
const MASK_TICKS: u16 = 2;
/// How many ticks without a move count as her move being over (the gap between a combo's moves
/// is one tick).
const QUIET_TICKS: u8 = 2;
/// The `LANDED` event's `index`: why she landed.
pub const LANDED_DIVES: u16 = 0;
pub const LANDED_STANCE: u16 = 1;
pub const LANDED_LAMP: u16 = 2;

/// B09's state, in `World::b09`. All zero while the fight isn't on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Bit `i` is set while lamp `i` is lit.
    pub lamps: u8,
    /// Phase 2's drain has begun: the arena is dark.
    pub dark: bool,
    /// She is walking (the ground phase); otherwise she flies.
    pub ground: bool,
    /// The tick the current phase of the flight began at.
    pub mode_from: u64,
    /// Dives started in this air phase.
    pub dives: u8,
    /// A dive move is playing (so a dive counts once).
    pub in_dive: bool,
    /// Ticks in a row she has played no move. A combo's next move starts a tick after the last
    /// one ends, so a landing or takeoff waits for a short quiet.
    pub quiet_ticks: u8,
    /// The first tick a lamp may stun her again.
    pub stun_ready: u64,
    /// Ticks in a row the player has held the smoltorch beside an unlit lamp.
    pub torch_ticks: u16,
    /// The `flare_until` last seen, to tell a new Lantern Flare from one still burning.
    pub flare_seen: u64,
}

impl State {
    /// At rest: no fight on. Such a state is left out of the state hash.
    pub fn is_inert(&self) -> bool {
        *self == Self::default()
    }
}

/// The numbers from `BossDef::params`.
struct Params {
    body: Hurtbox,
    air_height: f32,
    ground_height: f32,
    climb: f32,
    land: f32,
    dive_down_from: f32,
    dive_down_until: f32,
    dive_up_from: f32,
    dive_up_until: f32,
    dive_low: f32,
    dives_per_air: u8,
    ground_ticks: u64,
    /// Half extents of the dark arena around her spawn: x, then z.
    arena: [f32; 2],
    lamp_radius: f32,
    lamp_height: f32,
    lamp_hit: f32,
    lamp_light: f32,
    torch: f32,
    torch_hold: u16,
    flare: f32,
    lamp_pass: f32,
    stun_ticks: u8,
    stun_cooldown: u64,
    /// Each lamp's offset from her spawn: x, z.
    lamps: [[f32; 2]; LAMPS],
}

impl Params {
    /// Reads the numbers. `None` if any is missing (the boss then has no mechanic).
    fn read(def: &BossDef) -> Option<Self> {
        let get = |key: &str| def.params.get(key).copied();
        let mut lamps = [[0.0; 2]; LAMPS];
        for (i, l) in lamps.iter_mut().enumerate() {
            *l = [
                get(&format!("lamp_{i}_dx_m"))?,
                get(&format!("lamp_{i}_dz_m"))?,
            ];
        }
        Some(Self {
            body: Hurtbox {
                radius: get("body_radius_m")?,
                half_height: get("body_half_height_m")?,
            },
            air_height: get("air_height_m")?,
            ground_height: get("ground_height_m")?,
            climb: get("climb_mps")?,
            land: get("land_mps")?,
            dive_down_from: get("dive_down_from")?,
            dive_down_until: get("dive_down_until")?,
            dive_up_from: get("dive_up_from")?,
            dive_up_until: get("dive_up_until")?,
            dive_low: get("dive_low_m")?,
            dives_per_air: get("dives_per_air")?.max(1.0) as u8,
            ground_ticks: get("ground_ticks")?.max(1.0) as u64,
            arena: [get("arena_half_x_m")?, get("arena_half_z_m")?],
            lamp_radius: get("lamp_radius_m")?,
            lamp_height: get("lamp_height_m")?,
            lamp_hit: get("lamp_hit_m")?,
            lamp_light: get("lamp_light_m")?,
            torch: get("torch_m")?,
            torch_hold: get("torch_hold_ticks")?.max(1.0) as u16,
            flare: get("flare_m")?,
            lamp_pass: get("lamp_pass_m")?,
            stun_ticks: get("lamp_stun_ticks")?.clamp(1.0, 255.0) as u8,
            stun_cooldown: get("lamp_stun_cooldown_ticks")?.max(0.0) as u64,
            lamps,
        })
    }
}

/// The member actor `i` of boss `b`.
fn member(world: &World, b: usize, i: usize) -> Option<Id> {
    let spawn = *world.layout.bosses.get(b)?.members.get(i)?;
    world.state.ai_enemies.get(usize::from(spawn)).copied()
}

fn is_down(a: &Actor) -> bool {
    a.dead || a.hp == 0
}

/// Where her spawn is: the arena's middle.
fn home(world: &World, b: usize) -> Option<[f32; 3]> {
    world
        .layout
        .bosses
        .get(b)?
        .members
        .first()
        .and_then(|&s| world.layout.enemies.get(usize::from(s)))
        .map(|s| s.pos)
}

/// Lamp `i`'s foot, in world space.
fn lamp_pos(p: &Params, home: [f32; 3], i: usize) -> [f32; 3] {
    [home[0] + p.lamps[i][0], home[1], home[2] + p.lamps[i][1]]
}

fn flat_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (a[0] - b[0], a[2] - b[2]);
    crate::math::dmath::sqrt(dx * dx + dz * dz)
}

fn emit(world: &mut World, kind: u8, index: u16) {
    world
        .events
        .push(Event::Mechanic(MechanicEvent { kind, index }));
}

/// One tick of B09's mechanic, for every placed B09 (the hook is [`super::run`]).
pub fn run(world: &mut World) {
    for b in 0..world.bosses.len() {
        let Some(lay) = world.layout.bosses.get(b) else {
            continue;
        };
        if lay.def.id != BOSS_ID {
            continue;
        }
        let Some(p) = Params::read(&lay.def) else {
            continue;
        };
        tick(world, b, &p);
    }
}

fn tick(world: &mut World, b: usize, p: &Params) {
    let Some(body) = member(world, b, 0) else {
        return;
    };
    if let Some(a) = world.actors.get_mut(body) {
        a.hurtbox = p.body;
    }
    if world.bosses[b].stage != Stage::Fight {
        // Dormant, in the intro, or defeated: the cavern is lit and the lamps are out.
        if world.b09.lamps != 0 {
            emit(world, MechanicEvent::MATRIARCH_LAMPS_DOUSED, b as u16);
        }
        world.b09 = State::default();
        return;
    }
    if world.bosses[b].transition_left > 0 {
        // She plays `b09_drain`, invulnerable, where she is.
        return;
    }
    let Some(at) = home(world, b) else {
        return;
    };
    if world.bosses[b].phase >= 2 && !world.b09.dark {
        let st = &mut world.b09;
        st.dark = true;
        st.ground = false;
        st.dives = 0;
        st.in_dive = false;
        st.mode_from = world.tick;
        emit(world, MechanicEvent::MATRIARCH_DRAINED, b as u16);
    }
    if world.b09.dark {
        light_lamps(world, p, at);
    }
    flight(world, b, body, p, at);
}

/// The move id of `a`'s current move, if it plays one.
fn current_move<'w>(world: &'w World, a: &Actor) -> Option<(&'w str, u16)> {
    let c = a.moves.current?;
    Some((world.moves.get(c.num)?.data.id.as_str(), c.frame))
}

/// Her height above the floor `frame` frames into a dive.
fn dive_height(p: &Params, frame: u16) -> f32 {
    let f = f32::from(frame);
    let lerp =
        |a: f32, b: f32, from: f32, to: f32| a + (b - a) * ((f - from) / (to - from).max(1.0));
    if f < p.dive_down_from {
        p.air_height
    } else if f < p.dive_down_until {
        lerp(
            p.air_height,
            p.dive_low,
            p.dive_down_from,
            p.dive_down_until,
        )
    } else if f < p.dive_up_from {
        p.dive_low
    } else if f < p.dive_up_until {
        lerp(p.dive_low, p.air_height, p.dive_up_from, p.dive_up_until)
    } else {
        p.air_height
    }
}

/// The flight cycle: air phase, landing, ground phase, takeoff; her height; and which options
/// may start.
fn flight(world: &mut World, b: usize, body: Id, p: &Params, home: [f32; 3]) {
    let Some(a) = world.actors.get(body) else {
        return;
    };
    if is_down(a) {
        return;
    }
    let (kneel, base, y) = (a.poise.kneel, a.motor.ground_y, a.pos[1]);
    let playing = current_move(world, a);
    let diving = playing.is_some_and(|(id, _)| id.starts_with(DIVE_PREFIX));
    let dive_frame = playing
        .filter(|(id, _)| id.starts_with(DIVE_PREFIX))
        .map(|(_, f)| f);
    let mut st = world.b09;
    st.quiet_ticks = if a.moves.current.is_none() {
        st.quiet_ticks.saturating_add(1)
    } else {
        0
    };
    let idle = st.quiet_ticks >= QUIET_TICKS;

    // Dives count once each; the air phase ends after enough of them.
    if diving && !st.in_dive {
        st.in_dive = true;
        st.dives = st.dives.saturating_add(1);
    } else if !diving {
        st.in_dive = false;
    }
    let mut landed = None;
    if !st.ground {
        if kneel > 0 {
            landed = Some(LANDED_STANCE);
        } else if st.dives >= p.dives_per_air && idle && !diving {
            landed = Some(LANDED_DIVES);
        } else if let Some(lamp) = lamp_stun(world, b, body, &st, p, home) {
            landed = Some(LANDED_LAMP);
            st.stun_ready = world.tick + p.stun_cooldown;
            emit(world, MechanicEvent::MATRIARCH_LAMP_STUN, lamp as u16);
        }
    } else if kneel == 0 && idle && world.tick.saturating_sub(st.mode_from) >= p.ground_ticks {
        st.ground = false;
        st.dives = 0;
        st.in_dive = false;
        st.mode_from = world.tick;
        emit(world, MechanicEvent::MATRIARCH_TOOK_FLIGHT, b as u16);
    }
    if let Some(reason) = landed {
        st.ground = true;
        st.dives = 0;
        st.in_dive = false;
        st.mode_from = world.tick;
        emit(world, MechanicEvent::MATRIARCH_LANDED, reason);
        if reason != LANDED_DIVES {
            crash(world, body, p, reason == LANDED_LAMP);
        }
    }
    world.b09 = st;

    // Height: a dive follows its move's frame; otherwise she climbs or settles toward the
    // phase's height.
    let want = if st.ground {
        p.ground_height
    } else {
        p.air_height
    };
    let rate = if st.ground { p.land } else { p.climb } * DT;
    let now = y - base;
    let height = match dive_frame {
        Some(f) if !st.ground => dive_height(p, f),
        _ => now + (want - now).clamp(-rate, rate),
    };
    if let Some(a) = world.actors.get_mut(body) {
        a.pos[1] = base + height;
    }

    // The other phase's moves can't start.
    let other = if st.ground { AIR_PREFIX } else { GROUND_PREFIX };
    let masked: Vec<usize> = world
        .actors
        .get(body)
        .and_then(|a| a.ai.as_ref())
        .map(|brain| {
            world
                .ai
                .def(brain.def)
                .options
                .iter()
                .enumerate()
                .filter(|(_, o)| match o.kind {
                    OptionKind::Move { num, .. } => world
                        .moves
                        .get(num)
                        .is_some_and(|m| m.data.id.starts_with(other)),
                    // A retreat is an air option too: it backs her away from a player she hangs
                    // over, so a dive has room. On the ground she stands and fights.
                    OptionKind::Behavior(behavior) => {
                        st.ground && matches!(behavior, Behavior::Retreat { .. })
                    }
                })
                .map(|(i, _)| i)
                .collect()
        })
        .unwrap_or_default();
    if let Some(brain) = world.actors.get_mut(body).and_then(|a| a.ai.as_mut()) {
        for i in masked {
            if let Some(c) = brain.cooldowns.get_mut(i) {
                *c = (*c).max(MASK_TICKS);
            }
        }
    }
}

/// She crashes to the floor: her move is broken, and she plays `b09_stunned`. A lamp's stun
/// also sets her kneel (a stance break has already done it).
fn crash(world: &mut World, body: Id, p: &Params, set_kneel: bool) {
    let stunned = world.moves.num_of(STUNNED_MOVE).and_then(|num| {
        let d = world.moves.get(num)?;
        let (total, free) = (d.data.total_frames, d.data.cancel_any_from);
        Some(ActiveMove {
            num,
            frame: 0,
            charge_ticks: 0,
            hold_button: 0,
            free_from: free.unwrap_or(total),
        })
    });
    let Some(a) = world.actors.get_mut(body) else {
        return;
    };
    a.moves.interrupt();
    a.vel[0] = 0.0;
    a.vel[2] = 0.0;
    if set_kneel {
        a.poise.kneel = p.stun_ticks;
        a.poise.accum = 0;
    }
    a.moves.current = stunned;
    if let (Some(m), Some(brain)) = (stunned, a.ai.as_mut()) {
        brain.doing = Doing::Attack {
            option: 0,
            move_num: m.num,
        };
        brain.doing_ticks = 0;
        brain.token = Token::None;
        brain.token_target = None;
    }
}

/// The lamp that stuns her now, if one does: she flies, isn't kneeling, the cooldown is over,
/// and she is within `lamp_pass_m` of a lit lamp.
fn lamp_stun(
    world: &World,
    b: usize,
    body: Id,
    st: &State,
    p: &Params,
    home: [f32; 3],
) -> Option<usize> {
    if !st.dark || st.lamps == 0 || world.tick < st.stun_ready || world.bosses[b].phase < 2 {
        return None;
    }
    let a = world.actors.get(body)?;
    if a.poise.kneel > 0 {
        return None;
    }
    (0..LAMPS)
        .filter(|&i| st.lamps & (1 << i) != 0)
        .find(|&i| flat_dist(a.pos, lamp_pos(p, home, i)) <= p.lamp_pass)
}

/// Phase 2: lights the lamps the player's Fire, smoltorch, or Lantern Flare reaches.
fn light_lamps(world: &mut World, p: &Params, home: [f32; 3]) {
    let Some(player) = world.actors.get(world.player) else {
        return;
    };
    if is_down(player) {
        world.b09.torch_ticks = 0;
        return;
    }
    let (pos, unlit) = (
        player.pos,
        (0..LAMPS)
            .filter(|&i| world.b09.lamps & (1 << i) == 0)
            .collect::<Vec<_>>(),
    );
    let mut lit: Vec<usize> = Vec::new();

    // Fire: the player's active hitboxes and projectiles.
    let lamp = |i: usize| {
        let at = lamp_pos(p, home, i);
        (at, [at[0], at[1] + p.lamp_height, at[2]])
    };
    let fire_hitboxes: Vec<Capsule> = fire_hitboxes(world, player);
    for &i in &unlit {
        let (foot, top) = lamp(i);
        let reach = p.lamp_radius + p.lamp_hit;
        let by_hitbox = fire_hitboxes.iter().any(|c| {
            let (_, _, d2) = closest_points(c.a, c.b, foot, top);
            crate::math::dmath::sqrt(d2) <= c.radius + reach
        });
        let by_projectile = world.projectiles.iter().any(|(_, pr)| {
            if pr.shooter != world.player {
                return false;
            }
            let Some(def) = world.projectile_defs.get(pr.def) else {
                return false;
            };
            if def.element != Some(Element::Fire) {
                return false;
            }
            let (_, _, d2) = closest_points(pr.pos, pr.pos, foot, top);
            crate::math::dmath::sqrt(d2) <= def.radius + reach
        });
        if by_hitbox || by_projectile {
            lit.push(i);
        }
    }

    // Lantern Flare: a new one lights every unlit lamp within reach of the player.
    let flare_until = world.mech.flare_until;
    if flare_until > world.b09.flare_seen {
        world.b09.flare_seen = flare_until;
        for &i in &unlit {
            if flat_dist(pos, lamp_pos(p, home, i)) <= p.flare && !lit.contains(&i) {
                lit.push(i);
            }
        }
    }

    // The smoltorch: standing beside an unlit lamp for a while.
    let torch = world
        .layout
        .darkness
        .as_ref()
        .is_some_and(|r| world.state.flags.is_set(&r.torch_flag));
    let near = unlit
        .iter()
        .copied()
        .filter(|&i| flat_dist(pos, lamp_pos(p, home, i)) <= p.torch)
        .min_by(|&x, &y| {
            flat_dist(pos, lamp_pos(p, home, x)).total_cmp(&flat_dist(pos, lamp_pos(p, home, y)))
        });
    match (torch, near) {
        (true, Some(i)) => {
            world.b09.torch_ticks += 1;
            if world.b09.torch_ticks >= p.torch_hold {
                world.b09.torch_ticks = 0;
                if !lit.contains(&i) {
                    lit.push(i);
                }
            }
        }
        _ => world.b09.torch_ticks = 0,
    }

    lit.sort_unstable();
    for i in lit {
        world.b09.lamps |= 1 << i;
        emit(world, MechanicEvent::MATRIARCH_LAMP_LIT, i as u16);
    }
}

/// The player's active hitboxes that deal Fire now, in world space. A hitbox deals Fire if it
/// has the element, or has none and the weapon (or an oil on it) does.
fn fire_hitboxes(world: &World, player: &Actor) -> Vec<Capsule> {
    let Some(c) = player.moves.current else {
        return Vec::new();
    };
    let Some(def) = world.moves.get(c.num) else {
        return Vec::new();
    };
    let weapon_fire = player
        .inventory
        .equipment
        .right_weapon()
        .and_then(|w| {
            crate::items::upgrade::weapon_rating(
                world,
                w,
                &player.stats.attrs,
                player.stats.two_handing,
            )
        })
        .is_some_and(|ar| ar.element == Some(Element::Fire));
    let oil_fire = player
        .status
        .weapon_buff
        .and_then(|buff| world.status.buff(buff.num))
        .and_then(|d| d.element)
        .is_some_and(|(e, _)| e == Element::Fire);
    let pose = Pose {
        pos: player.pos,
        yaw: player.yaw,
    };
    // The frame counter runs a tick behind the hit resolution that last used it, so the window
    // is a frame wider on both sides.
    let frame = c.frame;
    def.data
        .hitboxes
        .iter()
        .filter(|h| h.frames.0 <= frame.saturating_add(1) && frame <= h.frames.1.saturating_add(1))
        .filter(|h| match h.element {
            Some(e) => e == Element::Fire,
            None => weapon_fire || oil_fire,
        })
        .map(|h| Capsule::from_shape(&h.shape, &pose))
        .collect()
}

/// Whether the ground at (x, z) is lit, if B09 has a say: inside the drained arena only the
/// ground near a lit lamp is lit. `None` anywhere else, and whenever the light isn't drained
/// (the area's own rule decides). Called from `darkness::lit_at`.
pub fn light_at(world: &World, x: f32, z: f32) -> Option<bool> {
    if !world.b09.dark {
        return None;
    }
    for b in 0..world.bosses.len() {
        let Some(lay) = world.layout.bosses.get(b) else {
            continue;
        };
        if lay.def.id != BOSS_ID {
            continue;
        }
        let (Some(p), Some(at)) = (Params::read(&lay.def), home(world, b)) else {
            continue;
        };
        if (x - at[0]).abs() > p.arena[0] || (z - at[2]).abs() > p.arena[1] {
            continue;
        }
        let lamps = world.b09.lamps;
        return Some((0..LAMPS).any(|i| {
            let l = lamp_pos(&p, at, i);
            lamps & (1 << i) != 0 && flat_dist([x, 0.0, z], l) <= p.lamp_light
        }));
    }
    None
}
