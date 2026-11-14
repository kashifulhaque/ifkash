//! The move system: per-actor move state, frame advance, chains, cancel windows, the
//! input buffer, root motion, tracking toward the lock target, hyperarmor and i-frame
//! flags, and stamina and FP costs (see [`super::stamina`]), dodges (see
//! [`super::dodge`]), and the guard (see [`super::guard`]). See docs/game/03-combat.md, "Combat verbs", "Input
//! buffer", and "Move data schema".
//!
//! # Frames
//!
//! Frame 0 of a move executes on the tick the move starts, and frame `f` on the `f`-th
//! tick after that. A move ends on the tick it would advance to `total_frames`. Every
//! window in the move data (hitboxes, hyperarmor, i-frames, cancels) uses these frame
//! numbers, so a straight-sword light attack with 10 startup frames has its hitbox on
//! frames 10 to 12.
//!
//! # Per-tick order
//!
//! [`step`] runs once per actor in tick system 4 (`systems::move_state`), after player
//! intent and before locomotion:
//!
//! 1. Read the actor's input into the buffer (the player only, for now).
//! 2. Advance the current move one frame, or hold it while a chargeable move charges.
//!    Free movement ends a move once it reaches `cancel_any_from`.
//! 3. Fire the buffered action if the current move allows that kind of cancel.
//! 4. Set the flags, turn toward the lock target, and set the velocity from root motion.
//!    Locomotion then moves the actor with the character controller, so root motion
//!    respects walls.
//!
//! While a move locks movement, `actor::controller::intent` leaves the velocity and
//! facing alone (see [`MoveState::locks_movement`]).

use lf_content::{Content, GuardStats, Locomotion, Move, NumId};
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::actor::targeting::{bearing, wrap_pi};
use crate::combat::dodge::{self, LoadTier};
use crate::combat::guard::GuardState;
use crate::combat::stamina;
use crate::input::{ActionFrame, buttons};
use crate::items::table::FlaskRules;
use crate::math::dmath::{atan2, cos, sin};
use crate::world::DT;

/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;

/// A move's stable numeric ID from `registry.ron`.
pub type MoveNum = NumId;

/// Bits of [`MoveState::flags`], set for the frame executed this tick.
pub mod flags {
    /// Inside the move's `hyperarmor` window: hits don't stagger (WP-023 reads it).
    pub const HYPERARMOR: u8 = 1 << 0;
    /// Inside the move's `iframes` window: hits don't land (WP-021 reads it).
    pub const IFRAMES: u8 = 1 << 1;
    /// The move is held at its `charge.charge_from` frame, charging.
    pub const CHARGING: u8 = 1 << 2;
}

/// An input that starts a move. See docs/game/03-combat.md, "Combat verbs".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// `LIGHT` pressed: the next light in the chain, a running, rolling, or backstep
    /// attack, or the first light.
    Light,
    /// `HEAVY` pressed: the heavy attack, which becomes the charged heavy if `HEAVY` is
    /// still held when it reaches the charged heavy's `charge_from` frame.
    Heavy,
    /// `DODGE` tapped (released before sprint starts): a roll toward `dir`, or a
    /// backstep when `dir` is zero.
    Dodge,
    /// `USE_ITEM` pressed (or a `Command::UseItem`): drinks the selected flask. See
    /// `crate::items`.
    UseItem,
    /// `GUARD` pressed with a catalyst in the left hand: casts the selected spell
    /// (docs/game/03-combat.md, "Spells", "Casting input"). See `combat::spells`.
    Cast,
    /// `ART` pressed: the right-hand weapon's Art (docs/game/04-arsenal-and-builds.md,
    /// "Arts"). See `combat::arts`.
    Art,
}

/// The most recent action that couldn't fire yet. See docs/game/03-combat.md,
/// "Input buffer".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buffered {
    pub action: Action,
    /// Ticks since the press. Age 0 is the press tick; the action is dropped once its
    /// age passes `Locomotion::input_buffer_ticks`.
    pub age: u16,
    /// The stick (`move_x`, `move_z`) when the action was pressed: the roll direction.
    pub dir: [i8; 2],
}

/// The move an actor is performing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveMove {
    pub num: MoveNum,
    /// The frame executed this tick.
    pub frame: u16,
    /// Ticks spent holding at `charge.charge_from`, up to `charge.max_frames`. WP-021
    /// scales the hitbox motion value with it (see [`MoveTable::charge_frac`]).
    pub charge_ticks: u16,
    /// The button that started the move. Holding it charges a chargeable move.
    pub hold_button: u16,
    /// The first frame free movement can interrupt: `cancel_any_from`, or
    /// `total_frames` when the move has none.
    pub free_from: u16,
}

/// Per-actor move state. Simulation state: part of the state hash.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveState {
    pub current: Option<ActiveMove>,
    pub buffer: Option<Buffered>,
    /// See [`flags`]. Describes the frame executed this tick; 0 with no move.
    pub flags: u8,
    /// Consecutive ticks `DODGE` has been held, to tell a tap (dodge) from a hold
    /// (sprint).
    pub dodge_hold: u16,
    /// Buttons of the previous tick, for press edges.
    pub prev_buttons: u16,
    /// Guard, guard counter window, and guard-break stagger. See `combat::guard`.
    pub guard: GuardState,
    /// While above 0 the current move lets the actor walk at this percent of the walk
    /// speed instead of locking movement (a flask drink). Set at the end of each `step`.
    pub slow_walk_percent: u8,
}

impl MoveState {
    /// True while the current move ignores the stick: until the frame about to execute
    /// next tick reaches `cancel_any_from`, or while a broken guard staggers. Player
    /// intent runs before this system, so it asks about next tick's frame.
    pub fn locks_movement(&self) -> bool {
        self.guard.stagger > 0
            || (self.slow_walk_percent == 0
                && self
                    .current
                    .is_some_and(|c| c.frame.saturating_add(1) < c.free_from))
    }

    /// True if the flag in [`flags`] is set for this tick's frame.
    pub fn has(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    /// Ends the current move and clears the buffer. The hook for stagger, guard break
    /// (`guard::apply_incoming_hit` calls it), and death (WP-023).
    pub fn interrupt(&mut self) {
        self.current = None;
        self.buffer = None;
        self.flags = 0;
        self.slow_walk_percent = 0;
    }

    /// Queues `action`, replacing any older buffered action (only the most recent is
    /// kept). The entry point for AI (WP-030 and later) as well as player input.
    pub fn press(&mut self, action: Action, dir: [i8; 2]) {
        self.buffer = Some(Buffered {
            action,
            age: 0,
            dir,
        });
    }
}

/// One move from the content with its references resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoveDef {
    pub num: MoveNum,
    pub data: Move,
    /// `next_chain` as a numeric ID.
    pub next_chain: Option<MoveNum>,
    /// Tracking turns the actor on frames before this one: the startup, which ends at
    /// the first hitbox frame (docs/game/03-combat.md, "Move data schema",
    /// `track_deg_per_tick`). 0 for a move with no hitbox.
    pub track_until: u16,
    /// For a spell's cast move: the first frame that does anything (a projectile, hitbox,
    /// effect, or the hold of a channel). Finesse skips a share of the frames before it
    /// ([`MoveTable::cast_skip`]). 0 for every other move.
    pub cast_startup: u16,
}

/// A weapon moveset with every move as a numeric ID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moveset {
    pub light_chain: Vec<MoveNum>,
    pub heavy: MoveNum,
    pub charged_heavy: MoveNum,
    pub running: MoveNum,
    pub rolling: MoveNum,
    pub backstep: MoveNum,
    /// Jump and plunge moves are resolved but not started yet: they need airborne
    /// attacks.
    pub jump_light: MoveNum,
    pub jump_heavy: MoveNum,
    pub plunge: MoveNum,
    /// Started by `HEAVY` within `counter_window_ticks` after a block.
    pub guard_counter: MoveNum,
    /// The cast move `GUARD` starts, when a catalyst with a matching spell selected is in
    /// the left hand (`combat::spells` sets it; a weapon's moveset never has one).
    pub left_cast: Option<MoveNum>,
    /// The move `ART` starts: the right-hand weapon's Art (`combat::arts`). Set by
    /// `World::sync_player_moveset`; `None` for a weapon with no Art.
    pub art: Option<MoveNum>,
    /// The roll for each equip load tier (there is none when overloaded).
    pub roll_light: MoveNum,
    pub roll_medium: MoveNum,
    pub roll_heavy: MoveNum,
    pub dodge_backstep: MoveNum,
    /// Started by `LIGHT` while the guard is raised, and keeps the guard up while it plays
    /// (a spear's thrust, a shield's bash). `None`: `LIGHT` drops the guard as usual.
    pub guard_attack: Option<MoveNum>,
    /// The wielder's equip load tier. A stand-in until equipment exists: the weapon's
    /// weight plus `stub_armor_weight_tenths`, over the max equip load at
    /// `stub_endurance`.
    pub load_tier: LoadTier,
    /// How the weapon blocks.
    pub guard: GuardStats,
    /// The weapon's critical hit multiplier in percent. See `combat::crit`.
    pub crit_mult_percent: u16,
}

/// Every move in the content, by numeric ID, plus the player's moveset. Built from
/// content, so it's not hashed: the content hash covers it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MoveTable {
    /// Sorted by `num`.
    defs: Vec<MoveDef>,
    /// The player's moveset. Until equipment lands, the player wields the first weapon
    /// in the content (in file and list order). `None` if the content has no weapon.
    /// The player's active moveset. `World::sync_player_moveset` sets it from the
    /// equipped right-hand weapon; it starts as the default (first) weapon's.
    pub player_moveset: Option<Moveset>,
    /// The moveset of every weapon, sorted by weapon number. See [`MoveTable::moveset_of`].
    movesets: Vec<(NumId, Moveset)>,
    /// The moveset of an empty right hand: the Fist class's, which every Fist weapon shares
    /// (docs/game/04-arsenal-and-builds.md, "Weapon classes"), with no Art. `None` if the
    /// content has no Fist weapon.
    pub unarmed: Option<Moveset>,
    /// Flask numbers, copied from `consumables.ron` (`None` if the content has none).
    pub flask: Option<FlaskRules>,
    /// The critical hit moves, if the content has them. See `combat::crit`.
    pub crits: Option<super::crit::CritMoves>,
    /// How much Finesse shortens a cast's startup, from the spell files (WP-055).
    pub cast_speed: Option<lf_content::spells::CastSpeed>,
    /// The player's gear shortens a spell's startup by this many percent (Shrike-Lite,
    /// WP-057). Derived from the equipment by `items::gear::refresh`, so it isn't hashed.
    pub cast_reduce_percent: u32,
    /// Every Art, and each weapon's Art facts (`combat::arts`). Built from content.
    pub arts: super::arts::ArtTable,
}

impl MoveTable {
    /// Resolves every move and the player's moveset. Fails on a reference to a move that
    /// isn't defined (the validator reports those too).
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let num = |id: &str| {
            content
                .registry
                .num(id)
                .ok_or_else(|| format!("move \"{id}\" is missing from registry.ron"))
        };
        let mut defs = Vec::new();
        for (_, m) in content.all_moves() {
            defs.push(MoveDef {
                num: num(&m.id)?,
                data: m.clone(),
                next_chain: m.next_chain.as_deref().map(num).transpose()?,
                // A move that launches projectiles aims until the first one leaves (WP-027).
                track_until: m
                    .hitboxes
                    .iter()
                    .map(|h| h.frames.0)
                    .chain(m.projectiles.iter().map(|p| p.frame))
                    .min()
                    .unwrap_or(0),
                cast_startup: 0,
            });
        }
        defs.sort_by_key(|d| d.num);
        // A spell's cast moves skip startup with Finesse (docs/game/03-combat.md, "Spells").
        for sp in &content.spells {
            for (m, effects) in std::iter::once((&sp.cast, &sp.effects))
                .chain(sp.charged.as_ref().map(|m| (m, &sp.charged_effects)))
            {
                let first = m
                    .hitboxes
                    .iter()
                    .map(|h| h.frames.0)
                    .chain(m.projectiles.iter().map(|p| p.frame))
                    .chain(effects.iter().map(|e| e.frame))
                    .chain(m.charge.map(|c| c.charge_from))
                    .min()
                    .unwrap_or(0);
                if let Some(d) = num(&m.id)
                    .ok()
                    .and_then(|n| defs.binary_search_by_key(&n, |d| d.num).ok())
                {
                    defs[d].cast_startup = first;
                }
            }
        }
        let mut table = Self {
            defs,
            cast_speed: content.spell_rules.map(|r| r.cast_speed),
            ..Self::default()
        };
        let defined = |id: &str| {
            let n = num(id)?;
            if table.get(n).is_some() {
                Ok(n)
            } else {
                Err(format!("move \"{id}\" is not defined"))
            }
        };
        let arts = super::arts::ArtTable::from_content(content)?;
        let mut movesets = Vec::new();
        for (_, w) in content.all_weapons() {
            let s = &w.moveset;
            // Equip load stub: the armor is a constant until equipment has weight.
            let load_tier = match content.locomotion {
                Some(t) => dodge::load_tier(
                    u32::from(w.weight_tenths) + u32::from(t.stub_armor_weight_tenths),
                    content
                        .attributes
                        .max_equip_load_tenths(t.stub_endurance)
                        .max(0) as u32,
                    &t.equip_load,
                ),
                None => LoadTier::Medium,
            };
            let set = Moveset {
                light_chain: s
                    .light_chain
                    .iter()
                    .map(|id| defined(id.as_str()))
                    .collect::<Result<_, _>>()?,
                heavy: defined(&s.heavy)?,
                charged_heavy: defined(&s.charged_heavy)?,
                running: defined(&s.running)?,
                rolling: defined(&s.rolling)?,
                backstep: defined(&s.backstep)?,
                jump_light: defined(&s.jump_light)?,
                jump_heavy: defined(&s.jump_heavy)?,
                plunge: defined(&s.plunge)?,
                guard_counter: defined(&s.guard_counter)?,
                left_cast: None,
                // The default Art (a weapon's Art scroll is applied at sync).
                art: arts.default_move(num(&w.id)?, w.elemental.map(|e| e.element)),
                roll_light: defined(&s.roll_light)?,
                roll_medium: defined(&s.roll_medium)?,
                roll_heavy: defined(&s.roll_heavy)?,
                dodge_backstep: defined(&s.dodge_backstep)?,
                guard_attack: s.guard_attack.as_deref().map(defined).transpose()?,
                load_tier,
                guard: w.guard,
                crit_mult_percent: w.crit_mult_percent,
            };
            movesets.push((num(&w.id)?, set));
        }
        // The default is the first weapon in the content (file and list order).
        let player_moveset = match content.all_weapons().next() {
            Some((_, w)) => {
                let n = num(&w.id)?;
                movesets
                    .iter()
                    .find(|(k, _)| *k == n)
                    .map(|(_, s)| s.clone())
            }
            None => None,
        };
        // An empty hand fights with the Fist class's moves (issue #20).
        let unarmed = content
            .all_weapons()
            .find(|(_, w)| w.class == lf_content::weapons::WeaponClass::Fist)
            .and_then(|(_, w)| {
                let n = num(&w.id).ok()?;
                movesets
                    .iter()
                    .find(|(k, _)| *k == n)
                    .map(|(_, s)| Moveset {
                        art: None,
                        ..s.clone()
                    })
            });
        movesets.sort_by_key(|(n, _)| *n);
        table.movesets = movesets;
        table.unarmed = unarmed;
        table.flask = FlaskRules::from_content(content)?;
        table.crits = super::crit::CritMoves::resolve(&table);
        table.arts = arts;
        table.player_moveset = player_moveset;
        Ok(table)
    }

    /// The moveset of the weapon with registry number `weapon`.
    pub fn moveset_of(&self, weapon: NumId) -> Option<&Moveset> {
        self.movesets
            .binary_search_by_key(&weapon, |(n, _)| *n)
            .ok()
            .map(|i| &self.movesets[i].1)
    }

    pub fn get(&self, num: MoveNum) -> Option<&MoveDef> {
        self.defs
            .binary_search_by_key(&num, |d| d.num)
            .ok()
            .map(|i| &self.defs[i])
    }

    /// Finds a move by string ID. A linear scan, for tools and tests.
    pub fn num_of(&self, id: &str) -> Option<MoveNum> {
        self.defs.iter().find(|d| d.data.id == id).map(|d| d.num)
    }

    /// How many frames of a spell's startup `finesse` skips: the cast speed rule's saving
    /// (up to `max_percent` at `at_finesse`, in proportion below it) of the frames before
    /// the first one that does anything. 0 for a move that isn't a cast.
    ///
    /// Gear then shortens what is left by `cast_reduce_percent`, so a startup of `n` frames
    /// becomes `n * (100 - reduce) / 100`, and the skip is the difference.
    pub fn cast_skip(&self, d: &MoveDef, finesse: u16) -> u16 {
        let Some(rule) = self.cast_speed.filter(|r| r.at_finesse > 0) else {
            return 0;
        };
        let saved = u32::from(rule.max_percent) * u32::from(finesse.min(rule.at_finesse))
            / u32::from(rule.at_finesse);
        let startup = u32::from(d.cast_startup);
        let left = startup - startup * saved / 100;
        let gear = left * (100 - self.cast_reduce_percent.min(100)) / 100;
        (startup - gear) as u16
    }

    /// How far a charge has gone, from 0 to 1: WP-021 lerps the hitbox motion value from
    /// its listed value to `charge.motion_value_max` by this.
    pub fn charge_frac(&self, m: &ActiveMove) -> f32 {
        match self.get(m.num).and_then(|d| d.data.charge) {
            Some(c) if c.max_frames > 0 => f32::from(m.charge_ticks) / f32::from(c.max_frames),
            _ => 0.0,
        }
    }
}

/// Meters forward at `frame` on a root-motion curve: linear between `(frame, meters)`
/// keys, held flat before the first key and after the last.
pub fn root_motion_at(keys: &[(u16, f32)], frame: u16) -> f32 {
    let Some(&(first_f, first_m)) = keys.first() else {
        return 0.0;
    };
    if frame <= first_f {
        return first_m;
    }
    for w in keys.windows(2) {
        let (f0, m0) = w[0];
        let (f1, m1) = w[1];
        if frame <= f1 {
            if f1 == f0 {
                return m1;
            }
            let t = f32::from(frame - f0) / f32::from(f1 - f0);
            return crate::math::dmath::lerp(m0, m1, t);
        }
    }
    keys.last().map_or(0.0, |k| k.1)
}

/// Meters forward the actor moves while executing `frame`: the curve from `frame` to
/// `frame + 1`. Summed over a whole move, this is the last key's distance.
pub fn root_motion_delta(keys: &[(u16, f32)], frame: u16) -> f32 {
    root_motion_at(keys, frame.saturating_add(1)) - root_motion_at(keys, frame)
}

/// The world-space direction the left stick points in `input`, or `None` when it's at rest
/// (the controller's dead zone) or there's no input (an AI actor).
fn stick_yaw(input: Option<ActionFrame>) -> Option<f32> {
    let f = input?;
    let (x, z) = (f32::from(f.move_x), f32::from(f.move_z));
    (x * x + z * z >= 1.0).then(|| atan2(x, z))
}

/// Turns `yaw` toward `target` by at most `max_step` radians, the short way round.
pub fn turn_toward(yaw: f32, target: f32, max_step: f32) -> f32 {
    let diff = wrap_pi(target - yaw);
    wrap_pi(yaw + diff.clamp(-max_step, max_step))
}

fn in_window(w: Option<lf_content::moves::FrameWindow>, frame: u16) -> bool {
    w.is_some_and(|w| (w.from..=w.to).contains(&frame))
}

/// Records press edges into the buffer. Only the most recent action is kept; when
/// several land on one tick, a dodge beats a light and a light beats a heavy.
fn read_input(ms: &mut MoveState, f: ActionFrame, t: &Locomotion) {
    let pressed = f.buttons & !ms.prev_buttons;
    ms.prev_buttons = f.buttons;
    let dir = [f.move_x, f.move_z];
    if pressed & buttons::HEAVY != 0 {
        ms.press(Action::Heavy, dir);
    }
    if pressed & buttons::LIGHT != 0 {
        ms.press(Action::Light, dir);
    }
    if pressed & buttons::USE_ITEM != 0 {
        ms.press(Action::UseItem, dir);
    }
    if pressed & buttons::ART != 0 {
        ms.press(Action::Art, dir);
    }
    // A tap is a release before sprint could start (docs/game/03-combat.md, "Combat
    // verbs": Roll and Backstep are a `DODGE` tap, Sprint a hold of more than
    // `sprint_delay_ticks`).
    if f.pressed(buttons::DODGE) {
        ms.dodge_hold = ms.dodge_hold.saturating_add(1);
    } else {
        if ms.dodge_hold > 0 && ms.dodge_hold <= t.sprint_delay_ticks {
            ms.press(Action::Dodge, dir);
        }
        ms.dodge_hold = 0;
    }
}

/// The move `b` would start now, or `None` if the current move doesn't allow that kind
/// of cancel yet (docs/game/03-combat.md, "Input buffer").
fn resolve(
    b: Buffered,
    current: Option<ActiveMove>,
    sprinting: bool,
    counter: bool,
    guarding: bool,
    set: &Moveset,
    table: &MoveTable,
) -> Option<MoveNum> {
    let cur = current.and_then(|c| table.get(c.num).map(|d| (c, d)));
    if let Some((c, d)) = cur {
        let reached = |w: Option<u16>| w.is_some_and(|from| c.frame >= from);
        let window = match b.action {
            Action::Light | Action::Heavy => d.data.cancel_attack_from,
            Action::Dodge => d.data.cancel_dodge_from,
            Action::Cast | Action::Art => d.data.cancel_attack_from,
            Action::UseItem => None,
        };
        if !reached(window) && !reached(d.data.cancel_any_from) {
            return None;
        }
    }
    let num = match b.action {
        // A raised guard and a guard attack: `LIGHT` thrusts or bashes from behind the guard.
        Action::Light if guarding && set.guard_attack.is_some() => set.guard_attack?,
        Action::Light => match cur {
            Some((c, _)) if dodge::is_roll(set, c.num) => set.rolling,
            Some((c, _)) if c.num == set.dodge_backstep => set.backstep,
            Some((_, d)) if d.next_chain.is_some() => d.next_chain?,
            _ if sprinting => set.running,
            _ => *set.light_chain.first()?,
        },
        Action::Heavy if counter => set.guard_counter,
        Action::Heavy => set.heavy,
        Action::Cast => set.left_cast?,
        Action::Art => set.art?,
        Action::Dodge => dodge::select(set, set.load_tier, b.dir != [0, 0])?,
        // Items are resolved by `crate::items::drink_move` before this is called.
        Action::UseItem => return None,
    };
    Some(num)
}

/// Advances one actor's move by a tick. `input` is the actor's action frame (`None` for
/// actors without input yet), `lock_pos` the position of its lock target, and `set` its
/// moveset (`None` means it can't start moves).
pub fn step(
    a: &mut Actor,
    input: Option<ActionFrame>,
    lock_pos: Option<[f32; 3]>,
    table: &MoveTable,
    set: Option<&Moveset>,
    t: &Locomotion,
) {
    if a.dead {
        a.moves.interrupt();
        return;
    }
    // `GUARD` casts instead of guarding when a catalyst is in the left hand.
    let cast_press = set.is_some_and(|s| s.left_cast.is_some())
        && input.is_some_and(|f| {
            f.pressed(buttons::GUARD) && a.moves.prev_buttons & buttons::GUARD == 0
        });
    if let Some(f) = input {
        read_input(&mut a.moves, f, t);
    }
    if cast_press {
        a.moves.press(Action::Cast, [0, 0]);
    }
    let held = input.map_or(0, |f| f.buttons);
    let stick = input.is_some_and(|f| f.move_x != 0 || f.move_z != 0);

    // 2. Advance, hold a charge, or end the current move.
    let mut charging = false;
    if let Some(mut c) = a.moves.current {
        let def = table.get(c.num);
        let charge = def.and_then(|d| d.data.charge);
        match charge {
            Some(ch)
                if c.frame == ch.charge_from
                    && held & c.hold_button != 0
                    && c.charge_ticks < ch.max_frames =>
            {
                c.charge_ticks += 1;
                charging = true;
            }
            _ => c.frame = c.frame.saturating_add(1),
        }
        // The heavy becomes the charged heavy if its button is still held when the move
        // reaches the charged heavy's charge frame (Combat verbs: tap versus hold).
        if let Some(s) = set
            && c.num == s.heavy
            && held & c.hold_button != 0
            && let Some(ch) = table.get(s.charged_heavy)
            && ch.data.charge.is_some_and(|x| x.charge_from == c.frame)
        {
            let extra = ch
                .data
                .stamina
                .saturating_sub(def.map_or(0, |d| d.data.stamina));
            stamina::spend(a, f32::from(extra));
            c.num = s.charged_heavy;
            c.free_from = ch.data.cancel_any_from.unwrap_or(ch.data.total_frames);
        }
        // A flask's effect lands on its effect frame; an interrupted drink never gets here.
        if let Some(f) = &table.flask
            && c.num == f.drink_move
            && c.frame == f.effect_frame
        {
            crate::items::inventory::drink_lands(a, f);
        }
        let total = table.get(c.num).map_or(0, |d| d.data.total_frames);
        let walked_out = stick && c.frame >= c.free_from;
        a.moves.current = if c.frame >= total || walked_out {
            None
        } else {
            Some(c)
        };
    }

    // The guard raises and lowers (see `combat::guard`). Whether a move may start is read
    // before the update, so a stagger of N ticks blocks exactly N ticks of starts.
    let can_act = a.moves.guard.can_act();
    // An Art stance (Bastion, Windward Parry) keeps the guard up while it plays.
    let stance = a
        .moves
        .current
        .and_then(|c| table.arts.of_move(c.num))
        .filter(|r| r.stance);
    let stance_now = stance.is_some();
    let auto_counter_now = stance.is_some_and(|r| r.auto_counter);
    // An AI actor has no buttons; its `Guard` behavior raises the guard (WP-032).
    // A catalyst can't guard (docs/game/03-combat.md, "Spells").
    let casting_left = set.is_some_and(|s| s.left_cast.is_some());
    let wants_guard = (held & buttons::GUARD != 0 && !casting_left)
        || a.ai.as_ref().is_some_and(|b| b.guarding)
        || stance_now;
    // A guard attack keeps the guard up while it plays, so the guard sees the actor as
    // free during it (see `Moveset::guard_attack`).
    let in_guard_attack = set.is_some_and(|s| {
        s.guard_attack.is_some() && a.moves.current.map(|c| c.num) == s.guard_attack
    });
    a.moves.guard.update(
        wants_guard,
        a.moves.current.is_none() || in_guard_attack || stance_now,
        &t.guard,
    );

    // A stance with an auto counter (Bastion) starts the guard counter as soon as a block
    // opens its window, without a `HEAVY`: the buffered `HEAVY` resolves to the counter below.
    if auto_counter_now && a.moves.guard.counter > 0 && a.moves.buffer.is_none() {
        a.moves.press(Action::Heavy, [0, 0]);
    }

    // 3. Fire the buffered action if the current move allows it. A broken guard or a
    // lowering one blocks starting anything.
    if let (Some(b), Some(s)) = (a.moves.buffer, set) {
        let grounded = a.motor.grounded && a.motor.jump_startup == 0 && a.motor.landing == 0;
        let counter = a.moves.guard.counter > 0;
        let guarding = a.moves.guard.active;
        let num = if b.action == Action::UseItem {
            crate::items::inventory::drink_move(a, table)
        } else {
            resolve(
                b,
                a.moves.current,
                a.motor.sprinting,
                counter,
                guarding,
                s,
                table,
            )
        };
        if grounded
            && can_act
            && let Some(d) = num.and_then(|n| table.get(n))
            && stamina::try_spend(a, &d.data)
        {
            // Starting a move drops the guard, except a guard attack and an Art stance, which
            // keep it up. It also uses up the counter window.
            let keeps_guard = s.guard_attack == Some(d.num)
                || table.arts.of_move(d.num).is_some_and(|r| r.stance);
            if !keeps_guard {
                a.moves.guard.drop_guard();
            }
            if d.num == s.guard_counter {
                a.moves.guard.counter = 0;
            }
            if b.action == Action::Dodge && b.dir != [0, 0] {
                a.yaw = atan2(f32::from(b.dir[0]), f32::from(b.dir[1]));
            } else if lock_pos.is_none()
                && let Some(y) = stick_yaw(input)
            {
                // Unlocked, every other move starts facing the stick, so each swing of a
                // combo can be aimed, as in Elden Ring (issue #78). Locked on, tracking
                // turns it toward the target instead.
                a.yaw = y;
            }
            // A spell's startup is shorter with Finesse (`MoveTable::cast_skip`).
            let skip = table.cast_skip(d, a.stats.attrs.finesse);
            a.moves.current = Some(ActiveMove {
                num: d.num,
                frame: skip,
                charge_ticks: 0,
                // Holding the button charges a chargeable move: a heavy, or a channeled
                // spell started by `LIGHT` or a left-hand `GUARD`.
                hold_button: match b.action {
                    Action::Heavy => buttons::HEAVY,
                    Action::Light if d.data.charge.is_some() => buttons::LIGHT,
                    Action::Cast if d.data.charge.is_some() => buttons::GUARD,
                    _ => 0,
                },
                free_from: d.data.cancel_any_from.unwrap_or(d.data.total_frames),
            });
            if let (Action::UseItem, Some(f)) = (b.action, &table.flask) {
                crate::items::inventory::drink_starts(a, f);
            }
            a.moves.buffer = None;
            charging = false;
        }
    }
    if let Some(b) = a.moves.buffer.as_mut() {
        b.age += 1;
        if b.age > t.input_buffer_ticks {
            a.moves.buffer = None;
        }
    }
    // The counter window counts down after the buffered action had its chance, so a block
    // on tick T allows a counter on ticks T+1 through T+`counter_window_ticks`.
    a.moves.guard.counter = a.moves.guard.counter.saturating_sub(1);

    // 4. Flags, tracking, and root motion for the frame executed this tick.
    a.moves.flags = 0;
    a.moves.slow_walk_percent = match (a.moves.current, &table.flask) {
        (Some(c), Some(f)) if c.num == f.drink_move => f.walk_percent,
        _ => 0,
    };
    let Some(c) = a.moves.current else {
        return;
    };
    let Some(d) = table.get(c.num) else {
        return;
    };
    let mut bits = 0;
    if in_window(d.data.hyperarmor, c.frame) {
        bits |= flags::HYPERARMOR;
    }
    if in_window(d.data.iframes, c.frame) {
        bits |= flags::IFRAMES;
    }
    if charging {
        bits |= flags::CHARGING;
    }
    a.moves.flags = bits;
    // A slow-walk move (a flask drink) keeps the stick's velocity, capped at its percent
    // of the walk speed; it has no root motion or tracking.
    if a.moves.slow_walk_percent > 0 {
        let cap = t.walk_speed * f32::from(a.moves.slow_walk_percent) / 100.0;
        let speed = crate::math::dmath::sqrt(a.vel[0] * a.vel[0] + a.vel[2] * a.vel[2]);
        if speed > cap {
            a.vel[0] *= cap / speed;
            a.vel[2] *= cap / speed;
        }
        return;
    }
    if c.frame < d.track_until {
        let step = d.data.track_deg_per_tick * DEG_TO_RAD;
        if let Some(p) = lock_pos {
            a.yaw = turn_toward(a.yaw, bearing(a.pos, p, a.yaw), step);
        } else if d.data.free_aim
            && let Some(f) = input
        {
            super::bow::free_aim(a, f, step);
        }
    }
    let meters = if charging {
        0.0
    } else {
        root_motion_delta(&d.data.root_motion, c.frame)
    };
    let speed = meters / DT;
    a.vel[0] = sin(a.yaw) * speed;
    a.vel[2] = cos(a.yaw) * speed;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::TickInput;
    use crate::snapshot::{self, write_snapshot};
    use crate::tick::tick;
    use crate::world::World;

    /// How far a move through the character controller may land from its root-motion
    /// distance, in meters. Not a gameplay number: the controller's skin and snapping.
    const KCC_TOLERANCE_M: f32 = 0.01;

    fn flat() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(5, &content, "m0_flat").expect("m0_flat exists")
    }

    /// Runs one tick with these buttons and stick, and returns the player's move.
    fn step(w: &mut World, buttons: u16, x: i8, z: i8) -> Option<ActiveMove> {
        let input = TickInput {
            frame: ActionFrame {
                move_x: x,
                move_z: z,
                buttons,
            },
            commands: Vec::new(),
        };
        tick(w, &input);
        player(w).moves.current
    }

    fn idle(w: &mut World, ticks: u32) {
        for _ in 0..ticks {
            step(w, 0, 0, 0);
        }
    }

    fn player(w: &World) -> &Actor {
        w.actors.get(w.player).unwrap()
    }

    fn num(w: &World, id: &str) -> MoveNum {
        w.moves.num_of(id).unwrap()
    }

    fn data<'a>(w: &'a World, id: &str) -> &'a Move {
        &w.moves.get(num(w, id)).unwrap().data
    }

    #[test]
    fn light_chain_of_three() {
        let mut w = flat();
        let ids = ["ss_light_1", "ss_light_2", "ss_light_3"];
        let c = step(&mut w, buttons::LIGHT, 0, 0).unwrap();
        assert_eq!((c.num, c.frame), (num(&w, ids[0]), 0));
        for pair in ids.windows(2) {
            let chain_from = data(&w, pair[0]).cancel_attack_from.unwrap();
            // Press early in recovery: the press is buffered until `cancel_attack_from`.
            let press_at = chain_from - 4;
            for f in 1..press_at {
                let c = step(&mut w, 0, 0, 0).unwrap();
                assert_eq!((c.num, c.frame), (num(&w, pair[0]), f));
            }
            let c = step(&mut w, buttons::LIGHT, 0, 0).unwrap();
            assert_eq!((c.num, c.frame), (num(&w, pair[0]), press_at));
            for f in press_at + 1..chain_from {
                assert_eq!(step(&mut w, 0, 0, 0).unwrap().frame, f);
            }
            let c = step(&mut w, 0, 0, 0).unwrap();
            assert_eq!((c.num, c.frame), (num(&w, pair[1]), 0), "{pair:?}");
        }
        // The chain ends at the third light: it runs to its end with no fourth.
        let total = data(&w, ids[2]).total_frames;
        for f in 1..total {
            assert_eq!(step(&mut w, 0, 0, 0).unwrap().frame, f);
        }
        assert_eq!(step(&mut w, 0, 0, 0), None);
    }

    #[test]
    fn dodge_buffered_in_recovery_fires_on_cancel_dodge_from() {
        let mut w = flat();
        let light = num(&w, "ss_light_1");
        let dodge_from = data(&w, "ss_light_1").cancel_dodge_from.unwrap();
        // The first recovery frame: startup 10 plus active 3 (Combat verbs).
        let recovery = data(&w, "ss_light_1").hitboxes[0].frames.1 + 1;
        assert_eq!(dodge_from, recovery + 4);
        step(&mut w, buttons::LIGHT, 0, 0);
        idle(&mut w, u32::from(recovery) - 1);
        // Tap DODGE with the stick to the right (+X): down, then up.
        assert_eq!(
            step(&mut w, buttons::DODGE, 127, 0).unwrap().frame,
            recovery
        );
        let c = step(&mut w, 0, 127, 0).unwrap();
        assert_eq!((c.num, c.frame), (light, recovery + 1));
        assert!(player(&w).moves.buffer.is_some());
        for f in recovery + 2..dodge_from {
            let c = step(&mut w, 0, 127, 0).unwrap();
            assert_eq!((c.num, c.frame), (light, f));
        }
        let c = step(&mut w, 0, 127, 0).unwrap();
        assert_eq!((c.num, c.frame), (num(&w, "dodge_roll_medium"), 0));
        assert!(player(&w).moves.buffer.is_none());
        // The roll faces the stick, and its i-frames start on frame 2.
        let yaw = player(&w).yaw;
        assert!((yaw - core::f32::consts::FRAC_PI_2).abs() < 1e-6, "{yaw}");
        assert!(!player(&w).moves.has(flags::IFRAMES));
        step(&mut w, 0, 0, 0);
        step(&mut w, 0, 0, 0);
        assert!(player(&w).moves.has(flags::IFRAMES));
        let mut snap = Vec::new();
        write_snapshot(&w, &mut snap);
        assert_eq!(u32::from(snap[0].move_id), num(&w, "dodge_roll_medium"));
        assert_eq!(snap[0].move_frame, 2);
        assert_ne!(snap[0].flags & snapshot::flags::INVULNERABLE, 0);
    }

    #[test]
    fn root_motion_moves_0_9_m() {
        let w = flat();
        let keys = &data(&w, "ss_light_1").root_motion;
        let sum: f32 = (0..27).map(|f| root_motion_delta(keys, f)).sum();
        assert!((sum - 0.9).abs() < 1e-6, "{sum}");

        let mut w = w;
        let start = player(&w).pos;
        step(&mut w, buttons::LIGHT, 0, 0);
        idle(&mut w, 40);
        assert_eq!(player(&w).moves.current, None);
        // Through the character controller, which keeps a 0.02 m skin from the floor
        // and can drift a few millimeters; the curve itself is exact (above).
        let end = player(&w).pos;
        let dz = end[2] - start[2];
        assert!((dz - 0.9).abs() < KCC_TOLERANCE_M, "moved {dz} m forward");
        assert!(
            (end[0] - start[0]).abs() < KCC_TOLERANCE_M,
            "{start:?} -> {end:?}"
        );
    }

    #[test]
    fn buffer_holds_a_press_for_eight_ticks() {
        for (press_at, chains) in [(13, true), (12, false)] {
            let mut w = flat();
            let chain_from = data(&w, "ss_light_1").cancel_attack_from.unwrap();
            assert_eq!(chain_from - 8, 13);
            step(&mut w, buttons::LIGHT, 0, 0);
            idle(&mut w, u32::from(press_at) - 1);
            step(&mut w, buttons::LIGHT, 0, 0);
            idle(&mut w, u32::from(chain_from - press_at));
            let c = player(&w).moves.current.unwrap();
            let want = if chains { "ss_light_2" } else { "ss_light_1" };
            assert_eq!(c.num, num(&w, want), "pressed on frame {press_at}");
        }
    }

    #[test]
    fn holding_dodge_sprints_without_rolling_and_no_stick_backsteps() {
        let mut w = flat();
        for _ in 0..30 {
            step(&mut w, buttons::DODGE, 0, 127);
        }
        assert!(player(&w).motor.sprinting);
        assert_eq!(step(&mut w, 0, 0, 0), None);

        let mut w = flat();
        let start = player(&w).pos;
        step(&mut w, buttons::DODGE, 0, 0);
        let c = step(&mut w, 0, 0, 0).unwrap();
        assert_eq!(c.num, num(&w, "dodge_backstep"));
        idle(&mut w, 30);
        let dz = player(&w).pos[2] - start[2];
        assert!((dz + 1.8).abs() < KCC_TOLERANCE_M, "moved {dz} m");
    }

    #[test]
    fn costs_and_hyperarmor() {
        let mut w = flat();
        let max = player(&w).max_stamina;
        step(&mut w, buttons::HEAVY, 0, 0);
        let cost = f32::from(data(&w, "ss_heavy").stamina);
        assert_eq!(player(&w).stamina, max - cost);
        let from = data(&w, "ss_heavy").hyperarmor.unwrap().from;
        idle(&mut w, u32::from(from) - 1);
        assert!(!player(&w).moves.has(flags::HYPERARMOR));
        idle(&mut w, 1);
        assert!(player(&w).moves.has(flags::HYPERARMOR));
    }

    #[test]
    fn holding_heavy_charges() {
        let mut w = flat();
        let charge = data(&w, "ss_heavy_charged").charge.unwrap();
        for _ in 0..=charge.charge_from {
            step(&mut w, buttons::HEAVY, 0, 0);
        }
        let c = player(&w).moves.current.unwrap();
        assert_eq!(
            (c.num, c.frame),
            (num(&w, "ss_heavy_charged"), charge.charge_from)
        );
        for i in 1..=charge.max_frames {
            let c = step(&mut w, buttons::HEAVY, 0, 0).unwrap();
            assert_eq!((c.frame, c.charge_ticks), (charge.charge_from, i));
            assert!(player(&w).moves.has(flags::CHARGING));
        }
        // Fully charged: the move goes on even with the button held.
        let c = step(&mut w, buttons::HEAVY, 0, 0).unwrap();
        assert_eq!(c.frame, charge.charge_from + 1);
        assert_eq!(w.moves.charge_frac(&c), 1.0);
    }

    #[test]
    fn walking_cancels_from_cancel_any_from() {
        let mut w = flat();
        let any = data(&w, "ss_light_1").cancel_any_from.unwrap();
        step(&mut w, buttons::LIGHT, 0, 0);
        for _ in 1..any {
            assert!(step(&mut w, 0, 0, 127).is_some());
        }
        assert_eq!(step(&mut w, 0, 0, 127), None);
        assert!(player(&w).vel[2] > 0.0);
    }

    /// Issue #78: unlocked, a swing starts facing the stick, and the next swing of the
    /// combo can face somewhere else.
    #[test]
    fn the_stick_aims_each_swing_of_a_combo() {
        use core::f32::consts::FRAC_PI_2;
        let mut w = flat();
        assert_eq!(player(&w).yaw, 0.0);
        // Light with the stick at +X: the swing faces +X.
        let first = step(&mut w, buttons::LIGHT, 127, 0).unwrap();
        assert!(
            (player(&w).yaw - FRAC_PI_2).abs() < 1e-5,
            "{}",
            player(&w).yaw
        );
        // Ride the first swing with the stick at rest until the chain can continue, then
        // press light with the stick at -X: the second swing turns round.
        let mut c = first;
        while c.num == first.num {
            c = step(&mut w, buttons::LIGHT, -127, 0).unwrap_or(c);
            if c.num == first.num {
                step(&mut w, 0, 0, 0);
            }
        }
        assert_ne!(c.num, first.num);
        assert!(
            (player(&w).yaw + FRAC_PI_2).abs() < 1e-5,
            "{}",
            player(&w).yaw
        );
    }

    #[test]
    fn tracking_turns_by_at_most_the_step() {
        let max = 12.0 * DEG_TO_RAD;
        assert_eq!(turn_toward(0.0, 1.0, max), max);
        assert_eq!(turn_toward(0.0, 0.1, max), 0.1);
        // The short way round across +-pi.
        let y = turn_toward(3.0, -3.0, max);
        assert!((y - wrap_pi(3.0 + max)).abs() < 1e-6 && y < -3.0, "{y}");

        let mut w = flat();
        let target = w.actors.insert(Actor {
            pos: [5.0, 0.0, 0.0],
            hp: 10,
            max_hp: 10,
            ..Actor::default()
        });
        let track = data(&w, "ss_light_1").track_deg_per_tick * DEG_TO_RAD;
        // Start the attack facing +Z, then lock on to the target at +X: tracking turns
        // 12 degrees a tick in startup. The lock-on snap in player intent is off while
        // the move runs.
        step(&mut w, buttons::LIGHT, 0, 0);
        assert_eq!(player(&w).yaw, 0.0);
        w.actors.get_mut(w.player).unwrap().lock_target = Some(target);
        step(&mut w, 0, 0, 0);
        assert!((player(&w).yaw - track).abs() < 1e-6);
        step(&mut w, 0, 0, 0);
        assert!((player(&w).yaw - 2.0 * track).abs() < 1e-6);
    }
}
