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
