//! World mechanics: ladders, lifts, doors, levers, hollow walls, pickups, chests, authored
//! messages, and the water and snow modifiers (WP-042). See docs/game/02-world.md,
//! "Traversal and world mechanics". Fog gates are the boss framework's (`ai::boss`).
//!
//! # Where state lives
//!
//! Almost everything is a world flag (see [`super::flags`]): a door is open, a lever is
//! pulled, a wall is revealed, a pickup is taken, a chest is opened, a message is read, a
//! kick-down ladder has dropped, a pool's water is raised. Flags are saved, so a loaded
//! game keeps them, and [`sync_colliders`] rebuilds the moving colliders from them every
//! tick. What is not a flag is [`MechState`]: each lift's travel and the ladder the player
//! is on. It is in the state hash, but only once it is not at rest, so scenarios without
//! mechanics hash as they did before WP-042.
//!
//! # Where things happen
//!
//! - [`run`] is the third part of tick system 1 (`systems::apply_commands`): `INTERACT`
//!   (pickups, chests, levers, doors, messages, ladders), climbing, and lifts.
//! - [`mask_input`] strips the buttons that can't be used on a ladder from the input the
//!   later systems see.
//! - [`before_locomotion`] (tick system 5) applies water and snow to the player.
//! - [`after_hits`] (tick system 10) reveals hollow walls the player's swing touches.
//!
//! Doors and revealed walls are box colliders ("movers", see `PhysicsState::add_mover`)
//! that sink into the floor when open. A lift is a mover that rises and carries the actors
//! standing on it.
//!
//! The numbers (climb speed, lift speed, reach, slowdowns) are in `locomotion.ron`,
//! `mechanics`.

use lf_content::moves::StatusKind;
use lf_content::scenario::{DoorKind, Loot};
use serde::{Deserialize, Serialize};

use super::{Layout, flags};
use crate::combat::hitbox::{Capsule, Pose};
use crate::events::{Event, LootEvent, MechanicEvent, MessageEvent};
use crate::input::{TickInput, buttons};
use crate::math::dmath::{cos, sin};
use crate::world::{DT, World};

/// Buttons that do nothing while the player is on a ladder: no attacks, guard, dodge, jump,
/// items, or lock-on (docs/game/02-world.md, Ladder: "can't attack").
const LADDER_BLOCKED: u16 = buttons::LIGHT
    | buttons::HEAVY
    | buttons::GUARD
    | buttons::DODGE
    | buttons::ART
    | buttons::JUMP
    | buttons::USE_ITEM
    | buttons::LOCK
    | buttons::TWO_HAND;

/// Samples along a swing's capsule when testing it against a hollow wall. A cost cap, not
/// a gameplay number.
const WALL_SAMPLES: u32 = 8;

/// One lift's travel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LiftRun {
    /// Height of the platform above its bottom stop, in meters.
    pub rise: f32,
    /// The stop it is going to or waiting at: 0 bottom, 1 top.
    pub target: u8,
}

/// The ladder the player is climbing.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Climb {
    /// Index into `Layout::ladders`.
    pub ladder: u16,
}

/// Mechanics state that isn't a flag. See the module docs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MechState {
    /// One per `Layout::lifts` entry.
    pub lifts: Vec<LiftRun>,
    pub climb: Option<Climb>,
    /// One per `Layout::carts` entry (WP-062; see `mine_cart`).
    #[serde(default)]
    pub carts: Vec<mine_cart::CartRun>,
    /// The tick a Lantern Flare burns out at, 0 for none (WP-062; see `darkness`).
    #[serde(default)]
    pub flare_until: u64,
    /// The tick B08's lava began to creep at, 0 while it is dry (WP-076; see
    /// `ai::bosses::b08`).
    #[serde(default)]
    pub b08_lava_from: u64,
    /// B10's beat, tolls, and shockwave (WP-078; see `ai::bosses::b10`). Not serialized here:
    /// the state hash reads it as a field of its own (`HashView::b10`), left out while it is at
    /// rest, so no other scenario's hash changes (a field here would change the hash of every
    /// world whose mechanics aren't at rest).
    #[serde(skip)]
    pub b10: crate::ai::bosses::b10::B10State,
    /// The player let go of a ladder this tick, so the buttons that did it stay masked
    /// for the rest of the tick. Recomputed every tick, so not saved or hashed.
    #[serde(skip)]
    pub suppress: bool,
}

impl MechState {
    pub fn new(layout: &Layout) -> Self {
        Self {
            lifts: vec![LiftRun::default(); layout.lifts.len()],
            climb: None,
            carts: vec![mine_cart::CartRun::default(); layout.carts.len()],
            flare_until: 0,
            b08_lava_from: 0,
            b10: Default::default(),
            suppress: false,
        }
    }

    /// At rest: no ladder, every lift waiting at its bottom stop. Such a state is left out
    /// of the state hash.
    pub fn is_inert(&self) -> bool {
        self.climb.is_none()
            && self.lifts.iter().all(|l| l.rise == 0.0 && l.target == 0)
            && self
                .carts
                .iter()
                .all(|c| *c == mine_cart::CartRun::default())
            && self.flare_until == 0
            && self.b08_lava_from == 0
    }
}

/// What the player's `INTERACT` would do now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Pickup(usize),
    Chest(usize),
    Lever(usize),
    Door(usize),
    Message(usize),
    LadderFoot(usize),
    LadderTop(usize),
    LadderKick(usize),
    Cart(usize),
    Trapdoor(usize),
    Mirror(usize),
    Brazier(usize),
    Sigil(usize),
}

impl Target {
    /// Tie-break order between targets at the same distance.
    fn order(self) -> u8 {
        match self {
            Self::Pickup(_) => 0,
            Self::Chest(_) => 1,
            Self::Lever(_) => 2,
            Self::Door(_) => 3,
            Self::Message(_) => 4,
            Self::LadderFoot(_) => 5,
            Self::LadderTop(_) => 6,
            Self::LadderKick(_) => 7,
            Self::Cart(_) => 8,
            Self::Trapdoor(_) => 9,
            Self::Mirror(_) => 10,
            Self::Brazier(_) => 11,
            Self::Sigil(_) => 12,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Pickup(i)
            | Self::Chest(i)
            | Self::Lever(i)
            | Self::Door(i)
            | Self::Message(i)
            | Self::LadderFoot(i)
            | Self::LadderTop(i)
            | Self::LadderKick(i)
            | Self::Cart(i)
            | Self::Trapdoor(i)
            | Self::Mirror(i)
            | Self::Brazier(i)
            | Self::Sigil(i) => i,
        }
    }
}

// ---------------------------------------------------------------------------------------
// Geometry

fn forward(yaw: f32) -> [f32; 2] {
    [sin(yaw), cos(yaw)]
}

/// Half extents of a door's slab: it spans x when it faces z, and z when it faces x.
fn door_half(d: &lf_content::scenario::Door, t: &lf_content::locomotion::Mechanics) -> [f32; 3] {
    let h = d.height_m * 0.5;
    if sin(d.yaw).abs() > 0.5 {
        [t.door_half_thickness_m, h, d.half_width_m]
    } else {
        [d.half_width_m, h, t.door_half_thickness_m]
    }
}

fn door_center(d: &lf_content::scenario::Door, open: bool, sink_extra: f32) -> [f32; 3] {
    let sink = if open { d.height_m + sink_extra } else { 0.0 };
    [d.pos[0], d.pos[1] + d.height_m * 0.5 - sink, d.pos[2]]
}

fn wall_center(w: &lf_content::scenario::HollowWall, revealed: bool, sink_extra: f32) -> [f32; 3] {
    let sink = if revealed {
        w.half_extents[1] * 2.0 + sink_extra
    } else {
        0.0
    };
    [w.center[0], w.center[1] - sink, w.center[2]]
}

fn lift_center(l: &lf_content::scenario::Lift, rise: f32, thick: f32) -> [f32; 3] {
    [l.pos[0], l.bottom_y + rise - thick * 0.5, l.pos[1]]
}

fn lift_rise_max(l: &lf_content::scenario::Lift) -> f32 {
    l.top_y - l.bottom_y
}

fn mover_door(layout: &Layout, i: usize) -> usize {
    layout.lifts.len() + i
}

fn mover_wall(layout: &Layout, i: usize) -> usize {
    layout.lifts.len() + layout.doors.len() + i
}

// ---------------------------------------------------------------------------------------
// Setup

/// True if the scenario has anything this module does.
fn has_any(layout: &Layout) -> bool {
    !(layout.ladders.is_empty()
        && layout.lifts.is_empty()
        && layout.doors.is_empty()
        && layout.levers.is_empty()
        && layout.hollow_walls.is_empty()
        && layout.chests.is_empty()
        && layout.messages.is_empty()
        && layout.pools.is_empty()
        && layout.snow.is_empty()
        && layout.carts.is_empty()
        && layout.trapdoors.is_empty()
        && layout.mirrors.is_empty()
        && layout.beam_sources.is_empty()
        && layout.receptors.is_empty()
        && layout.braziers.is_empty()
        && layout.sigils.is_empty()
        && layout.pickups.iter().all(|p| !p.interact))
}

/// Adds the movers to the physics world: lifts, then doors, then hollow walls (see
/// `mover_door` and `mover_wall`). Called once by `World::new`, after the navigation grid
/// is baked, so closed doors don't wall off the AI's grid.
pub fn install(world: &mut World) {
    let t = world.locomotion.mechanics;
    for l in &world.layout.lifts {
        let half = [
            l.half_extents[0],
            t.lift_thickness_m * 0.5,
            l.half_extents[1],
        ];
        world
            .physics
            .add_mover(lift_center(l, 0.0, t.lift_thickness_m), half);
    }
    for d in &world.layout.doors {
        world
            .physics
            .add_mover(door_center(d, false, t.sink_extra_m), door_half(d, &t));
    }
    for w in &world.layout.hollow_walls {
        world
            .physics
            .add_mover(wall_center(w, false, t.sink_extra_m), w.half_extents);
    }
    // Trapdoor hatches come last (WP-062).
    trapdoor::install(world);
    sync_colliders(world);
}

/// Puts every mover where the flags and lift state say it is. Does nothing for a mover
/// that is already there.
pub fn sync_colliders(world: &mut World) {
    let t = world.locomotion.mechanics;
    for (i, l) in world.layout.lifts.iter().enumerate() {
        let rise = world.mech.lifts.get(i).map_or(0.0, |r| r.rise);
        world
            .physics
            .set_mover(i, lift_center(l, rise, t.lift_thickness_m));
    }
    for (i, d) in world.layout.doors.iter().enumerate() {
        let open = world.state.flags.is_set(&flags::door_open(&d.id));
        world.physics.set_mover(
            mover_door(&world.layout, i),
            door_center(d, open, t.sink_extra_m),
        );
    }
    for (i, w) in world.layout.hollow_walls.iter().enumerate() {
        let revealed = world.state.flags.is_set(&flags::wall_revealed(&w.id));
        world.physics.set_mover(
            mover_wall(&world.layout, i),
            wall_center(w, revealed, t.sink_extra_m),
        );
    }
    trapdoor::sync(world);
}

// ---------------------------------------------------------------------------------------
// Input

/// The input the systems after the mechanics see: while the player is on a ladder (or just
/// let go of one) the attack, guard, dodge, jump, item, and lock-on buttons are cleared.
/// `None` means use the input as it is.
pub fn mask_input(world: &World, input: &TickInput) -> Option<TickInput> {
    // A player riding a mine cart is held in place (WP-062).
    if let Some(masked) = mine_cart::mask(world, input) {
        return Some(masked);
    }
    // A safe zone (Hollowhearth, WP-068) clears the combat buttons and caps the stick.
    let safe = crate::world_state::quests::safe_zone_input(world, input);
    let input = safe.as_ref().unwrap_or(input);
    if world.mech.climb.is_none() && !world.mech.suppress {
        return safe;
    }
    let mut masked = input.clone();
    masked.frame.buttons &= !LADDER_BLOCKED;
    Some(masked)
}

/// Whether the player is on a ladder.
pub fn climbing(world: &World) -> bool {
    // A mine cart carries the player along its rail the way a ladder does (WP-062).
    world.mech.climb.is_some() || mine_cart::riding(world)
}

// ---------------------------------------------------------------------------------------
// Tick system 1

/// Interaction, climbing, and lifts. Runs after the Lantern and item commands.
pub fn run(world: &mut World, input: &TickInput) {
    if !has_any(&world.layout) {
        return;
    }
    world.mech.suppress = false;
    sync_colliders(world);
    let dead = world
        .actors
        .get(world.player)
        .is_none_or(|p| p.dead || p.hp == 0);
    if dead {
        if world.mech.climb.is_some() {
            release(world, Release::Drop);
        }
    } else if world.mech.climb.is_some() {
        climb_step(world, input);
    } else {
        let pressed = input.frame.buttons & !prev_buttons(world);
        if pressed & buttons::INTERACT != 0 {
            interact(world);
        }
    }
    lifts_step(world);
    mine_cart::step(world);
    // WP-063: light beams light the receptors they reach, after this tick's turns and doors.
    light_beam::step(world);
    // WP-066: a sealing sigil holds its doors open or shut.
    brazier::step(world);
    sync_colliders(world);
}

/// The buttons of the previous tick (the player's `Motor` records them in tick system 2).
fn prev_buttons(world: &World) -> u16 {
    world
        .actors
        .get(world.player)
        .map_or(0, |p| p.motor.prev_buttons)
}

fn mechanic(world: &mut World, kind: u8, index: usize) {
    world.events.push(Event::Mechanic(MechanicEvent {
        kind,
        index: index as u16,
    }));
}

/// The horizontal distance squared from the player to a point, if the point is within
/// `radius` horizontally and `height` vertically.
fn reach(feet: [f32; 3], at: [f32; 3], radius: f32, height: f32) -> Option<f32> {
    let (dx, dz) = (at[0] - feet[0], at[2] - feet[2]);
    let d2 = dx * dx + dz * dz;
    (d2 <= radius * radius && (at[1] - feet[1]).abs() <= height).then_some(d2)
}

fn ladder_usable(world: &World, i: usize) -> bool {
    let l = &world.layout.ladders[i];
    !l.kick_down || world.state.flags.is_set(&flags::ladder_down(&l.id))
}

/// Where a climber steps off the top of ladder `i`.
fn ladder_top_landing(world: &World, i: usize) -> [f32; 3] {
    let l = &world.layout.ladders[i];
    let f = forward(l.yaw);
    let e = world.locomotion.mechanics.ladder_top_exit_m;
    [
        l.pos[0] + f[0] * e,
        l.pos[1] + l.height_m,
        l.pos[2] + f[1] * e,
    ]
}

/// The nearest thing `INTERACT` would act on, or `None`. The player must be free to act:
/// alive, standing, not mid-move, and not at a Lantern.
fn nearest(world: &World) -> Option<Target> {
    let p = world.actors.get(world.player)?;
    if p.dead
        || !p.motor.grounded
        || p.moves.current.is_some()
        || world.state.resting.is_some()
        || world.mech.climb.is_some()
        || mine_cart::riding(world)
    {
        return None;
    }
    let t = world.locomotion.mechanics;
    let (r, h) = (t.interact_radius_m, t.interact_height_m);
    let feet = p.pos;
    let mut best: Option<(f32, Target)> = None;
    let mut offer = |d2: Option<f32>, target: Target| {
        let Some(d2) = d2 else { return };
        let better = best.is_none_or(|(bd, bt)| {
            d2.total_cmp(&bd)
                .then(target.order().cmp(&bt.order()))
                .then(target.index().cmp(&bt.index()))
                .is_lt()
        });
        if better {
            best = Some((d2, target));
        }
    };
    let flags = &world.state.flags;
    for (i, s) in world.layout.pickups.iter().enumerate() {
        if s.interact && !flags.is_set(&flags::pickup_taken(&s.id)) {
            offer(reach(feet, s.pos, r, h), Target::Pickup(i));
        }
    }
    for (i, c) in world.layout.chests.iter().enumerate() {
        if !flags.is_set(&flags::chest_opened(&c.id)) {
            offer(reach(feet, c.pos, r, h), Target::Chest(i));
        }
    }
    for (i, l) in world.layout.levers.iter().enumerate() {
        offer(reach(feet, l.pos, r, h), Target::Lever(i));
    }
    for (i, d) in world.layout.doors.iter().enumerate() {
        // A sigil's seal door answers to the sigil, not to a hand (WP-066).
        if !flags.is_set(&flags::door_open(&d.id)) && !brazier::is_seal_door(world, &d.id) {
            offer(reach(feet, d.pos, r + d.half_width_m, h), Target::Door(i));
        }
    }
    for (i, m) in world.layout.messages.iter().enumerate() {
        offer(reach(feet, m.pos, r, h), Target::Message(i));
    }
    for (i, l) in world.layout.ladders.iter().enumerate() {
        let gr = t.ladder_grab_radius_m;
        let top = ladder_top_landing(world, i);
        if ladder_usable(world, i) {
            offer(reach(feet, l.pos, gr, h), Target::LadderFoot(i));
            offer(reach(feet, top, gr, h), Target::LadderTop(i));
        } else {
            offer(reach(feet, top, gr, h), Target::LadderKick(i));
        }
    }
    for i in 0..world.layout.carts.len() {
        if let Some(at) = mine_cart::spot(world, i) {
            offer(
                reach(feet, at, r + mine_cart::CART_HALF_LENGTH_M, h),
                Target::Cart(i),
            );
        }
    }
    for i in 0..world.layout.trapdoors.len() {
        if let Some(at) = trapdoor::spot(world, i) {
            offer(reach(feet, at, r, h), Target::Trapdoor(i));
        }
    }
    // WP-063: a crystal mirror, turned from beside it.
    for (i, m) in world.layout.mirrors.iter().enumerate() {
        offer(reach(feet, m.pos, r, h), Target::Mirror(i));
    }
    // WP-066: a Warden Brazier not yet lit, and a seal sigil not yet sealed.
    for (i, b) in world.layout.braziers.iter().enumerate() {
        if !brazier::is_lit(world, &b.id) {
            offer(reach(feet, b.pos, r, h), Target::Brazier(i));
        }
    }
    for (i, g) in world.layout.sigils.iter().enumerate() {
        if brazier::sigil_touchable(world, &g.id) {
            offer(reach(feet, g.pos, r, h), Target::Sigil(i));
        }
    }
    best.map(|(_, t)| t)
}

/// What the HUD should offer: 0 nothing, 2 take or open, 3 pull a lever, 4 open a door,
/// 5 read a message, 6 climb a ladder, 7 kick a ladder down. (1 is the Lantern's.)
pub fn prompt(world: &World) -> u32 {
    if !has_any(&world.layout) {
        return 0;
    }
    match nearest(world) {
        None => 0,
        Some(Target::Pickup(_) | Target::Chest(_)) => 2,
        Some(Target::Lever(_)) => 3,
        Some(Target::Door(_)) => 4,
        Some(Target::Message(_)) => 5,
        Some(Target::LadderFoot(_) | Target::LadderTop(_)) => 6,
        Some(Target::LadderKick(_)) => 7,
        // 30 ride a mine cart, 31 push one, 32 push a trapdoor open (WP-062).
        Some(Target::Cart(i)) => mine_cart::prompt(world, i),
        Some(Target::Trapdoor(_)) => trapdoor::PROMPT_OPEN,
        // 40 turn a crystal mirror (WP-063).
        Some(Target::Mirror(_)) => light_beam::PROMPT_TURN,
        // 55 light a Warden Brazier, 56 touch a seal sigil (WP-066).
        Some(Target::Brazier(_)) => brazier::PROMPT_LIGHT,
        Some(Target::Sigil(_)) => brazier::PROMPT_TOUCH,
    }
}

fn interact(world: &mut World) {
    let Some(target) = nearest(world) else {
        return;
    };
    match target {
        Target::Pickup(i) => {
            crate::pickups::take(world, i);
        }
        Target::Chest(i) => open_chest(world, i),
        Target::Lever(i) => pull_lever(world, i),
        Target::Door(i) => try_door(world, i),
        Target::Message(i) => {
            let id = world.layout.messages[i].id.clone();
            world.state.flags.raise(&flags::message_read(&id));
            world
                .events
                .push(Event::Message(MessageEvent { index: i as u16 }));
        }
        Target::LadderFoot(i) => grab(world, i, false),
        Target::LadderTop(i) => grab(world, i, true),
        Target::LadderKick(i) => {
            let id = world.layout.ladders[i].id.clone();
            world.state.flags.raise(&flags::ladder_down(&id));
            mechanic(world, MechanicEvent::LADDER_KICKED, i);
        }
        Target::Cart(i) => mine_cart::interact(world, i),
        Target::Trapdoor(i) => trapdoor::interact(world, i),
        Target::Mirror(i) => light_beam::turn(world, i),
        Target::Brazier(i) => brazier::light(world, i),
        Target::Sigil(i) => brazier::touch(world, i),
    }
}

/// Gives the player a chest's loot. False, giving nothing, if the item won't fit.
fn give_loot(world: &mut World, loot: &Loot) -> Option<(u32, u16)> {
    let player = world.player;
    let mut given = (0, 0);
    if let Some(id) = &loot.item {
        let num = world.items.num_of(id)?;
        let n = loot.count.max(1);
        let p = world.actors.get_mut(player)?;
        let took = if world.items.is_weapon(num) {
            u16::from(p.inventory.give_gear(num))
        } else {
            p.inventory.pick_up(num, n, &world.items)
        };
        if took == 0 {
            return None;
        }
        given = (num, took);
    }
    if let Some(p) = world.actors.get_mut(player) {
        p.inventory.glimmer = p.inventory.glimmer.saturating_add(loot.glimmer);
    }
    Some(given)
}

fn open_chest(world: &mut World, i: usize) {
    let (id, loot) = {
        let c = &world.layout.chests[i];
        (c.id.clone(), c.loot.clone())
    };
    let Some((item, count)) = give_loot(world, &loot) else {
        return;
    };
    world.state.flags.raise(&flags::chest_opened(&id));
    world.events.push(Event::Loot(LootEvent {
        source: 1,
        index: i as u16,
        item,
        count,
        glimmer: loot.glimmer,
    }));
}

fn set_door(world: &mut World, i: usize, open: bool) {
    let id = world.layout.doors[i].id.clone();
    world
        .state
        .flags
        .set(&flags::door_open(&id), u32::from(open));
    let kind = if open {
        MechanicEvent::DOOR_OPENED
    } else {
        MechanicEvent::DOOR_CLOSED
    };
    mechanic(world, kind, i);
}

fn try_door(world: &mut World, i: usize) {
    let d = world.layout.doors[i].clone();
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    match &d.kind {
        DoorKind::Locked => mechanic(world, MechanicEvent::DOOR_LOCKED, i),
        DoorKind::OneWay => {
            let f = forward(d.yaw);
            let side = (p.pos[0] - d.pos[0]) * f[0] + (p.pos[2] - d.pos[2]) * f[1];
            if side > 0.0 {
                set_door(world, i, true);
            } else {
                mechanic(world, MechanicEvent::DOOR_ONE_WAY, i);
            }
        }
        DoorKind::Held { items, need } => {
            if held_door::holds(world, items, *need) {
                set_door(world, i, true);
            } else {
                mechanic(world, MechanicEvent::DOOR_NEEDS_KEY, i);
            }
        }
        DoorKind::Keyed(key) => {
            let player = world.player;
            let taken = world.items.num_of(key).is_some_and(|num| {
                world
                    .actors
                    .get_mut(player)
                    .is_some_and(|p| p.inventory.remove(num, 1) > 0)
            });
            if taken {
                set_door(world, i, true);
            } else {
                mechanic(world, MechanicEvent::DOOR_NEEDS_KEY, i);
            }
        }
    }
}

fn pull_lever(world: &mut World, i: usize) {
    use lf_content::scenario::LeverTarget;
    let l = world.layout.levers[i].clone();
    let key = flags::lever_pulled(&l.id);
    let now = !world.state.flags.is_set(&key);
    world.state.flags.set(&key, u32::from(now));
    mechanic(world, MechanicEvent::LEVER_PULLED, i);
    match &l.target {
        LeverTarget::Door(id) => {
            if let Some(d) = world.layout.doors.iter().position(|d| &d.id == id) {
                let open = !world.state.flags.is_set(&flags::door_open(id));
                set_door(world, d, open);
            }
        }
        LeverTarget::Lift { id, stop } => {
            if let Some(n) = world.layout.lifts.iter().position(|l| &l.id == id) {
                let max = lift_rise_max(&world.layout.lifts[n]);
                let run = &mut world.mech.lifts[n];
                let waiting_here =
                    run.rise == if *stop == 0 { 0.0 } else { max } && run.target == *stop;
                let goes_to = if waiting_here {
                    // Already here: send it to the other stop.
                    Some(1 - *stop)
                } else if run.target != *stop {
                    // Elsewhere or on its way away: call it.
                    Some(*stop)
                } else {
                    // Already on its way here.
                    None
                };
                if let Some(to) = goes_to {
                    run.target = to;
                    mechanic(world, MechanicEvent::LIFT_STARTED, n);
                }
            }
        }
        LeverTarget::Water(name) => {
            let key = flags::pool_raised(name);
            let raised = !world.state.flags.is_set(&key);
            world.state.flags.set(&key, u32::from(raised));
        }
        // Flag levers and bells (WP-061).
        LeverTarget::Flag(_) | LeverTarget::Bell { .. } => {
            bell::pull(world, i);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Ladders

#[derive(Clone, Copy)]
enum Release {
    /// Let go where the player hangs; they fall.
    Drop,
    /// Climbed off the top onto the ledge.
    Top,
    /// Climbed down to the ground.
    Bottom,
}

fn grab(world: &mut World, i: usize, from_top: bool) {
    let t = world.locomotion.mechanics;
    let l = world.layout.ladders[i].clone();
    let y = if from_top {
        l.pos[1] + l.height_m - t.ladder_top_grab_below_m
    } else {
        l.pos[1]
    };
    if let Some(p) = world.actors.get_mut(world.player) {
        p.pos = [l.pos[0], y, l.pos[2]];
        p.yaw = l.yaw;
        hang(p);
    }
    world.mech.climb = Some(Climb { ladder: i as u16 });
    mechanic(world, MechanicEvent::LADDER_GRABBED, i);
}

/// Holds the player on the rails: no velocity, no fall.
fn hang(p: &mut crate::actor::Actor) {
    p.vel = [0.0; 3];
    let m = &mut p.motor;
    m.grounded = false;
    m.jumped = false;
    m.jump_startup = 0;
    m.landing = 0;
    m.ground_y = p.pos[1];
    m.fall_peak_y = p.pos[1];
}

fn release(world: &mut World, how: Release) {
    let Some(c) = world.mech.climb.take() else {
        return;
    };
    let i = usize::from(c.ladder);
    let t = world.locomotion.mechanics;
    let l = world.layout.ladders[i].clone();
    let f = forward(l.yaw);
    let top = ladder_top_landing(world, i);
    if let Some(p) = world.actors.get_mut(world.player) {
        match how {
            Release::Drop => hang(p),
            Release::Top => {
                p.pos = top;
                stand(p);
            }
            Release::Bottom => {
                p.pos = [
                    l.pos[0] - f[0] * t.ladder_bottom_exit_m,
                    l.pos[1],
                    l.pos[2] - f[1] * t.ladder_bottom_exit_m,
                ];
                stand(p);
            }
        }
    }
    world.mech.suppress = true;
    mechanic(world, MechanicEvent::LADDER_RELEASED, i);
}

fn stand(p: &mut crate::actor::Actor) {
    hang(p);
    p.motor.grounded = true;
}

fn climb_step(world: &mut World, input: &TickInput) {
    let Some(c) = world.mech.climb else { return };
    let i = usize::from(c.ladder);
    let t = world.locomotion.mechanics;
    let l = world.layout.ladders[i].clone();
    let pressed = input.frame.buttons & !prev_buttons(world);
    if pressed & (buttons::JUMP | buttons::DODGE) != 0 {
        release(world, Release::Drop);
        return;
    }
    let f = forward(l.yaw);
    let stick =
        (f32::from(input.frame.move_x) * f[0] + f32::from(input.frame.move_z) * f[1]) / 127.0;
    let dir = if stick > t.ladder_stick_threshold {
        1.0
    } else if stick < -t.ladder_stick_threshold {
        -1.0
    } else {
        0.0
    };
    let speed = stick.abs().min(1.0) * t.ladder_climb_speed_mps * DT * dir;
    let top_y = l.pos[1] + l.height_m;
    let Some(p) = world.actors.get_mut(world.player) else {
        return;
    };
    let y = p.pos[1] + speed;
    if speed > 0.0 && y >= top_y {
        release(world, Release::Top);
    } else if speed < 0.0 && y <= l.pos[1] {
        release(world, Release::Bottom);
    } else {
        p.pos = [l.pos[0], y, l.pos[2]];
        p.yaw = l.yaw;
        hang(p);
    }
}

// ---------------------------------------------------------------------------------------
// Lifts

fn lifts_step(world: &mut World) {
    let t = world.locomotion.mechanics;
    for n in 0..world.layout.lifts.len() {
        let lift = world.layout.lifts[n].clone();
        let run = world.mech.lifts[n];
        let goal = if run.target == 0 {
            0.0
        } else {
            lift_rise_max(&lift)
        };
        if run.rise == goal {
            continue;
        }
        let step = t.lift_speed_mps * DT;
        let next = if (goal - run.rise).abs() <= step {
            goal
        } else if goal > run.rise {
            run.rise + step
        } else {
            run.rise - step
        };
        let dy = next - run.rise;
        // Whoever stands on the platform now rides it.
        let surface = lift.bottom_y + run.rise;
        let riders: Vec<_> = world
            .actors
            .iter()
            .filter(|(_, a)| {
                !a.dead
                    && a.motor.grounded
                    && (a.pos[0] - lift.pos[0]).abs() <= lift.half_extents[0]
                    && (a.pos[2] - lift.pos[1]).abs() <= lift.half_extents[1]
                    && (a.pos[1] - surface).abs() <= t.lift_rider_tolerance_m
            })
            .map(|(id, _)| id)
            .collect();
        for id in riders {
            if let Some(a) = world.actors.get_mut(id) {
                a.pos[1] += dy;
                a.motor.ground_y += dy;
                a.motor.fall_peak_y += dy;
            }
        }
        world.mech.lifts[n].rise = next;
        if next == goal {
            mechanic(world, MechanicEvent::LIFT_ARRIVED, n);
        }
    }
}

// ---------------------------------------------------------------------------------------
// Water and snow (tick system 5)

fn inside(center: [f32; 2], half: [f32; 2], x: f32, z: f32) -> bool {
    (x - center[0]).abs() <= half[0] && (z - center[1]).abs() <= half[1]
}

/// A pool's depth now, in meters: a lever may have raised it, or a sluice lowered it
/// (`water_level`).
fn pool_depth(world: &World, i: usize) -> f32 {
    water_level::depth(world, i)
}

/// Applies water and snow to the player before the controller moves them: shallow water
/// and deep snow slow them, snow makes sprinting cost more, deep water is an invisible
/// wall at its edge, and blight water builds Blight. Only the player is affected.
pub fn before_locomotion(world: &mut World) {
    if world.layout.pools.is_empty() && world.layout.snow.is_empty() {
        return;
    }
    let t: lf_content::locomotion::Mechanics = world.locomotion.mechanics;
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    if p.dead {
        return;
    }
    let (pos, vel, grounded, sprinting) = (p.pos, p.vel, p.motor.grounded, p.motor.sprinting);
    let mut slow = 100u32;
    let mut blight = false;
    let mut block = [false, false];
    let (nx, nz) = (pos[0] + vel[0] * DT, pos[2] + vel[2] * DT);
    for i in 0..world.layout.pools.len() {
        let pool = &world.layout.pools[i];
        let (c, h) = (pool.center, pool.half_extents);
        let deep = pool_depth(world, i) > t.water_deep_over_m;
        if deep {
            // An invisible wall: refuse a step from outside the pool to inside it.
            if !inside(c, h, pos[0], pos[2]) {
                block[0] |= inside(c, h, nx, pos[2]);
                block[1] |= inside(c, h, pos[0], nz);
                if !block[0] && !block[1] && inside(c, h, nx, nz) {
                    block = [true, true];
                }
            }
        } else if inside(c, h, pos[0], pos[2]) {
            slow = slow * (100 - u32::from(t.water_slow_percent)) / 100;
            blight |= pool.blight;
            // WP-073: a pool a boss fight has set dragging slows a second time (B05's tide).
            if world.state.flags.is_set(&flags::pool_drag(&pool.name)) {
                slow = slow * (100 - u32::from(t.water_slow_percent)) / 100;
            }
        }
    }
    let in_snow = world
        .layout
        .snow
        .iter()
        .any(|z| inside(z.center, z.half_extents, pos[0], pos[2]));
    if in_snow {
        slow = slow * (100 - u32::from(t.snow_slow_percent)) / 100;
    }
    let sprint_cost = world.locomotion.sprint_stamina_per_tick;
    if let Some(p) = world.actors.get_mut(world.player) {
        if grounded && slow < 100 {
            p.vel[0] = p.vel[0] * slow as f32 / 100.0;
            p.vel[2] = p.vel[2] * slow as f32 / 100.0;
        }
        if block[0] {
            p.vel[0] = 0.0;
        }
        if block[1] {
            p.vel[2] = 0.0;
        }
        if in_snow && sprinting {
            let extra = sprint_cost * (f32::from(t.snow_sprint_stamina_percent) - 100.0) / 100.0;
            p.stamina = (p.stamina - extra).max(0.0);
        }
    }
    if blight && grounded {
        let id = world.player;
        crate::combat::status::add_buildup(
            world,
            id,
            StatusKind::Blight,
            u64::from(t.blight_buildup_per_tick_milli),
        );
    }
}

// ---------------------------------------------------------------------------------------
// Hollow walls (tick system 10)

/// Squared distance from `p` to the box.
fn point_box_d2(p: [f32; 3], center: [f32; 3], half: [f32; 3]) -> f32 {
    let mut d2 = 0.0;
    for k in 0..3 {
        let over = (p[k] - center[k]).abs() - half[k];
        if over > 0.0 {
            d2 += over * over;
        }
    }
    d2
}

/// Reveals every hollow wall the player's active hitboxes touch.
pub fn after_hits(world: &mut World) {
    if world.layout.hollow_walls.is_empty() {
        return;
    }
    let Some(p) = world.actors.get(world.player) else {
        return;
    };
    let Some(cur) = p.moves.current.filter(|_| !p.dead) else {
        return;
    };
    let Some(def) = world.moves.get(cur.num) else {
        return;
    };
    let pose = Pose {
        pos: p.pos,
        yaw: p.yaw,
    };
    let caps: Vec<Capsule> = def
        .data
        .hitboxes
        .iter()
        .filter(|hb| (hb.frames.0..=hb.frames.1).contains(&cur.frame))
        .map(|hb| Capsule::from_shape(&hb.shape, &pose))
        .collect();
    if caps.is_empty() {
        return;
    }
    let mut revealed = Vec::new();
    for (i, w) in world.layout.hollow_walls.iter().enumerate() {
        if w.sturdy || world.state.flags.is_set(&flags::wall_revealed(&w.id)) {
            continue;
        }
        let hit = caps.iter().any(|c| {
            (0..=WALL_SAMPLES).any(|s| {
                let u = s as f32 / WALL_SAMPLES as f32;
                let q = [
                    c.a[0] + (c.b[0] - c.a[0]) * u,
                    c.a[1] + (c.b[1] - c.a[1]) * u,
                    c.a[2] + (c.b[2] - c.a[2]) * u,
                ];
                point_box_d2(q, w.center, w.half_extents) <= c.radius * c.radius
            })
        });
        if hit {
            revealed.push(i);
        }
    }
    for i in revealed {
        let id = world.layout.hollow_walls[i].id.clone();
        world.state.flags.raise(&flags::wall_revealed(&id));
        mechanic(world, MechanicEvent::WALL_REVEALED, i);
    }
    sync_colliders(world);
}

// ---------------------------------------------------------------------------------------
// For the host

/// What the renderer needs that isn't in the actor snapshot, as floats, in this order:
/// one word for the ladder the player is on plus one (0 for none); then, per list in the
/// scenario's order: ladders (1 if climbable now), lifts (the platform's rise in meters),
/// doors (1 if open), levers (1 if pulled), hollow walls (1 if revealed), pickups (1 if
/// taken), chests (1 if opened), messages (1 if read), and pools (the water's depth). Then
/// WP-062's words (carts, trapdoors, and, in a dark area, the visibility), then WP-063's
/// (`light_beam::render_words`: the mirrors' facings, the receptors, and each beam's legs), then
/// WP-066's (`brazier::render_words`: each brazier, each sigil).
pub fn render_state(world: &World) -> Vec<f32> {
    let flag = |k: String| f32::from(u8::from(world.state.flags.is_set(&k)));
    let l = &world.layout;
    let mut out = vec![world.mech.climb.map_or(0.0, |c| f32::from(c.ladder) + 1.0)];
    out.extend((0..l.ladders.len()).map(|i| f32::from(u8::from(ladder_usable(world, i)))));
    out.extend(world.mech.lifts.iter().map(|r| r.rise));
    out.extend(l.doors.iter().map(|d| flag(flags::door_open(&d.id))));
    out.extend(l.levers.iter().map(|d| flag(flags::lever_pulled(&d.id))));
    out.extend(
        l.hollow_walls
            .iter()
            .map(|d| flag(flags::wall_revealed(&d.id))),
    );
    out.extend(l.pickups.iter().map(|d| flag(flags::pickup_taken(&d.id))));
    out.extend(l.chests.iter().map(|d| flag(flags::chest_opened(&d.id))));
    out.extend(l.messages.iter().map(|d| flag(flags::message_read(&d.id))));
    out.extend((0..l.pools.len()).map(|i| pool_depth(world, i)));
    // WP-062: each mine cart's distance along its rail, each trapdoor (1 if open), then, in
    // a dark area only, the player's visibility in meters (0 for no limit).
    out.extend(mine_cart::render_words(world));
    out.extend(trapdoor::render_words(world));
    if l.darkness.is_some() {
        out.push(darkness::render_word(world));
    }
    // WP-063: each mirror's facing, each receptor (1 once lit), then each beam's legs.
    out.extend(light_beam::render_words(world));
    // WP-066: each Warden Brazier (1 once lit), then each seal sigil (0 cold, 1 touched, 2 sealed).
    out.extend(brazier::render_words(world));
    out
}

pub mod darkness;
pub mod held_door;
pub mod mine_cart;
pub mod trapdoor;
pub mod water_level;
// WP-061: bells, flag levers, and sleeping enemies.
pub mod bell;
pub mod slumber;
// WP-063: light beams with mirrors and receptors, and low ceilings.
pub mod light_beam;
pub mod low_ceiling;
// WP-066: Warden Braziers and seal sigils.
pub mod brazier;

#[cfg(test)]
mod tests;
