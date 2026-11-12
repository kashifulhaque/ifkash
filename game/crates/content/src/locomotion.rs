//! Player locomotion tuning: speeds, jump, gravity, stamina stub, fall damage, and the
//! character controller settings. Loaded from `locomotion.ron`; each field names the
//! spec section it comes from in that file.

use serde::{Deserialize, Serialize};

/// Locomotion tuning for the player's kinematic character controller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Locomotion {
    /// Walk speed in m/s.
    pub walk_speed: f32,
    /// Run speed in m/s.
    pub run_speed: f32,
    /// Sprint speed in m/s.
    pub sprint_speed: f32,
    /// Stick magnitude (0 to 127) below which the player walks.
    pub walk_below: f32,
    /// Stick magnitude above which the player runs.
    pub run_above: f32,
    /// Ticks `DODGE` must be held before sprinting starts.
    pub sprint_delay_ticks: u16,
    /// Stamina drained per sprinting tick.
    pub sprint_stamina_per_tick: f32,
    /// Peak jump height in meters.
    pub jump_height: f32,
    /// Ticks between pressing `JUMP` and leaving the ground.
    pub jump_startup_ticks: u8,
    /// Ticks of landing recovery after a jump, with movement locked.
    pub jump_landing_ticks: u8,
    /// Stamina cost of a jump.
    pub jump_stamina: f32,
    /// Downward acceleration in m/s^2.
    pub gravity: f32,
    /// Maximum fall speed in m/s.
    pub terminal_fall_speed: f32,
    /// Stamina regained per tick once regeneration resumes.
    pub stamina_regen_per_tick: f32,
    /// Ticks without spending stamina before it regenerates.
    pub stamina_regen_delay_ticks: u16,
    /// Vitality used for the player's max HP until characters have attributes (stub).
    pub stub_vitality: u16,
    /// Endurance used for the player's max stamina until characters have attributes (stub).
    pub stub_endurance: u16,
    /// Focus used for the player's max FP until characters have attributes (stub).
    pub stub_focus: u16,
    /// Might, Finesse, Insight, and Devotion for the player until leveling (stub; WP-050).
    pub stub_might: u16,
    pub stub_finesse: u16,
    pub stub_insight: u16,
    pub stub_devotion: u16,
    /// Upgrade level of the equipped weapon until upgrades exist (stub).
    pub stub_weapon_upgrade: u8,
    /// Flat defense of the player against every damage type until armor exists (stub).
    pub stub_defense: u32,
    /// Ticks a pressed action stays buffered while the current move can't be canceled
    /// (docs/game/03-combat.md, "Input buffer").
    pub input_buffer_ticks: u16,
    /// Falls of this many meters or less deal no damage.
    pub fall_damage_from_m: f32,
    /// Fraction of max HP lost per meter fallen above `fall_damage_from_m`.
    pub fall_damage_per_m: f32,
    /// Falls of more than this many meters kill.
    pub fall_death_over_m: f32,
    /// Capsule radius in meters.
    pub capsule_radius: f32,
    /// Half the length of the capsule's cylinder segment, in meters.
    pub capsule_half_height: f32,
    /// Skin gap the controller keeps from obstacles, in meters.
    pub controller_offset: f32,
    /// Steepest slope the player can walk up, in degrees.
    pub max_slope_climb_deg: f32,
    /// Slopes steeper than this make the player slide down, in degrees.
    pub min_slope_slide_deg: f32,
    /// Tallest step the player climbs without jumping, in meters.
    pub autostep_height: f32,
    /// Minimum free width on top of a step, in meters.
    pub autostep_min_width: f32,
    /// Distance within which the player is snapped down to the ground, in meters.
    pub snap_to_ground: f32,
    /// Lock-on targeting numbers.
    pub lock_on: LockOn,
    /// Armor weight in tenths of a unit until equipment exists (stub).
    pub stub_armor_weight_tenths: u16,
    /// Equip load tier thresholds.
    pub equip_load: EquipLoad,
    /// Guard timings and numbers.
    pub guard: Guard,
    /// Death, Remnant, and Lantern numbers.
    pub death: Death,
    /// Poise, stance, parry, and critical hit numbers.
    pub poise: Poise,
    /// Ladders, lifts, doors, water, and snow (WP-042).
    pub mechanics: Mechanics,
}

/// World mechanics tuning (docs/game/02-world.md, "Traversal and world mechanics").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mechanics {
    /// `INTERACT` reaches a lever, door, pickup, chest, or message within this horizontal
    /// distance of the player's feet, in meters.
    pub interact_radius_m: f32,
    /// ...and within this vertical distance, in meters.
    pub interact_height_m: f32,
    /// The player can grab a ladder (at its foot or its top) within this horizontal
    /// distance of it, in meters.
    pub ladder_grab_radius_m: f32,
    /// Vertical climb speed at full stick, in meters per second.
    pub ladder_climb_speed_mps: f32,
    /// The stick's component along the climb direction (0 to 1) must exceed this to climb.
    pub ladder_stick_threshold: f32,
    /// Climbing off the top lands the player this far past the top rail, in meters.
    pub ladder_top_exit_m: f32,
    /// Climbing off the bottom puts the player this far back from the foot, in meters.
    pub ladder_bottom_exit_m: f32,
    /// A player grabbing the ladder from its top starts this far below the top, in meters.
    pub ladder_top_grab_below_m: f32,
    /// Lift speed, in meters per second.
    pub lift_speed_mps: f32,
    /// Lift platform thickness, in meters.
    pub lift_thickness_m: f32,
    /// An actor stands on a lift when its feet are within this height of the platform's
    /// top and its position is over the platform, in meters.
    pub lift_rider_tolerance_m: f32,
    /// Half the thickness of a door slab, in meters.
    pub door_half_thickness_m: f32,
    /// An open door or revealed wall sinks this far below the floor line past its own
    /// height, in meters.
    pub sink_extra_m: f32,
    /// Shallow water slows the player by this percent (Water: 20%).
    pub water_slow_percent: u8,
    /// Water deeper than this blocks the player with an invisible wall (Water: 1.4 m).
    pub water_deep_over_m: f32,
    /// Deep snow slows the player by this percent (Deep snow: 25%).
    pub snow_slow_percent: u8,
    /// Sprinting through deep snow costs this percent of the normal stamina (Deep snow:
    /// 1.5x).
    pub snow_sprint_stamina_percent: u16,
    /// Blight buildup per tick while wading in blight water, in thousandths of a point.
    pub blight_buildup_per_tick_milli: u32,
}

/// Death, Remnants, and Lanterns (docs/game/06-progression-and-systems.md, "Glimmer and
/// death" and "Lanterns").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Death {
    /// Ticks between the player dying and respawning at the last rested Lantern.
    pub respawn_delay_ticks: u16,
    /// A Remnant is recovered when the player's feet are within this distance of it, in
    /// meters.
    pub remnant_pickup_radius_m: f32,
    /// The player can rest at a Lantern when their feet are within this horizontal
    /// distance of it, in meters.
    pub lantern_rest_radius_m: f32,
    /// ...and within this vertical distance of it, in meters.
    pub lantern_rest_height_m: f32,
}

/// Poise, stance, parry, and critical hit tuning (docs/game/03-combat.md, "Poise and
/// stance" and "Parry and critical hits").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Poise {
    /// The player's poise until armor exists (stub).
    pub stub_poise: u16,
    /// Ticks of a light stagger.
    pub light_stagger_ticks: u8,
    /// Ticks of a heavy stagger.
    pub heavy_stagger_ticks: u8,
    /// A stagger whose margin over the poise is above this percent of it is heavy.
    pub heavy_margin_percent: u16,
    /// Ticks without poise damage before the accumulator resets.
    pub reset_ticks: u16,
    /// The poise counts as this many times larger during hyperarmor frames.
    pub hyperarmor_mult: u16,
    /// Ticks a stance-broken actor kneels.
    pub stance_break_ticks: u8,
    /// Ticks without a hit before stance damage starts to decay.
    pub stance_decay_after_ticks: u16,
    /// Percent of the stance meter that decays each tick.
    pub stance_decay_percent: u16,
    /// Ticks from raising the guard until a parry window opens.
    pub parry_startup_ticks: u8,
    /// Ticks an attacker stays in the parried state.
    pub parried_ticks: u8,
    /// Distance for a riposte, in meters.
    pub riposte_range_m: f32,
    /// Distance for a backstab, in meters.
    pub backstab_range_m: f32,
    /// Full angle of the rear arc for a backstab, in degrees.
    pub backstab_arc_deg: f32,
}

/// Equip load tiers (docs/game/03-combat.md, "Equip load and roll tiers"). Percentages
/// are of the maximum equip load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipLoad {
    /// Under this is the Light tier.
    pub light_below_percent: u16,
    /// From this up to `overloaded_over_percent` is the Heavy tier.
    pub heavy_from_percent: u16,
    /// Over this is Overloaded: no roll, no sprint.
    pub overloaded_over_percent: u16,
    /// Stamina regeneration at Heavy load, as a percent of the normal rate.
    pub heavy_regen_percent: u16,
}

/// Guard tuning (docs/game/03-combat.md, "Combat verbs", "Regeneration", and "Guard and
/// guard counter").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guard {
    /// Ticks `GUARD` is held before blocks count.
    pub raise_ticks: u8,
    /// Ticks after releasing during which the guard can't be raised and no move starts.
    pub lower_ticks: u8,
    /// Ticks a broken guard staggers.
    pub break_stagger_ticks: u8,
    /// Ticks after a block in which `HEAVY` is a guard counter.
    pub counter_window_ticks: u8,
    /// Stamina regeneration while guarding, as a percent of the normal rate.
    pub regen_percent: u16,
}

/// Lock-on tuning (docs/game/03-combat.md, "Lock-on").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockOn {
    /// Farthest a target can be to be acquired or switched to, in meters.
    pub acquire_range_m: f32,
    /// Full apex angle of the acquisition cone around the facing, in degrees. A target
    /// is inside it when it is within half this angle of the facing on either side.
    pub acquire_cone_deg: f32,
    /// The lock breaks when the target is farther than this, in meters.
    pub break_range_m: f32,
    /// Radius around the player in which a new target is picked automatically after the
    /// locked one dies, in meters.
    pub retarget_range_m: f32,
    /// Height above the feet of the point the line-of-sight ray runs between, in meters.
    pub eye_height_m: f32,
}
