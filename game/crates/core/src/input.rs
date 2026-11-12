//! Per-tick input: the 4-byte `ActionFrame` and the discrete `Command`s.
//! See docs/game/01-architecture.md, "Inputs".

use serde::{Deserialize, Serialize};

/// Bit positions of `ActionFrame::buttons`, in spec order.
pub mod buttons {
    pub const LIGHT: u16 = 1 << 0;
    pub const HEAVY: u16 = 1 << 1;
    pub const GUARD: u16 = 1 << 2;
    pub const DODGE: u16 = 1 << 3;
    pub const ART: u16 = 1 << 4;
    pub const JUMP: u16 = 1 << 5;
    pub const USE_ITEM: u16 = 1 << 6;
    pub const INTERACT: u16 = 1 << 7;
    pub const LOCK: u16 = 1 << 8;
    pub const SWITCH_L: u16 = 1 << 9;
    pub const SWITCH_R: u16 = 1 << 10;
    pub const CYCLE_ITEM: u16 = 1 << 11;
    pub const CYCLE_SPELL: u16 = 1 << 12;
    pub const CYCLE_RIGHT: u16 = 1 << 13;
    pub const CYCLE_LEFT: u16 = 1 << 14;
    pub const TWO_HAND: u16 = 1 << 15;
}

/// One tick of continuous input. Exactly 4 bytes on the wire (see `to_bytes`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionFrame {
    pub move_x: i8,
    pub move_z: i8,
    pub buttons: u16,
}

impl ActionFrame {
    /// `[move_x, move_z, buttons_lo, buttons_hi]`; buttons are little-endian.
    pub fn to_bytes(self) -> [u8; 4] {
        let b = self.buttons.to_le_bytes();
        [self.move_x as u8, self.move_z as u8, b[0], b[1]]
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self {
            move_x: bytes[0] as i8,
            move_z: bytes[1] as i8,
            buttons: u16::from_le_bytes([bytes[2], bytes[3]]),
        }
    }

    pub fn pressed(self, mask: u16) -> bool {
        self.buttons & mask != 0
    }
}

/// A discrete, menu-level action. `Gesture` is a no-op in the simulation until the system
/// that consumes it lands. `Rest` rests at the Lantern in reach
/// (`world_state::lantern`); it does nothing with none in reach. Variants are serialized by
/// position in replays, so only ever append.
///
/// - `Equip` and `Unequip` take an equipment slot number from `items::slot`.
/// - `UseItem` takes a quick slot index, 0 to 4: it selects that slot and uses its item.
/// - `SplitFlasks` sets the Vigil Flask's share of the charges (the Lantern menu). It
///   applies only while resting at a Lantern.
/// - `LevelUp` raises attribute `attribute` (0 to 6: Vitality, Focus, Endurance, Might,
///   Finesse, Insight, Devotion) by one, for the Glimmer the next level costs. `Travel`
///   moves the player to the lit Lantern at index `lantern` of the scenario. Both apply
///   only while resting at a Lantern (`world_state::lantern_menu`).
/// - `Respec` sets the attribute spread to `attributes` (in the order of `LevelUp`'s
///   attributes, Vitality to Devotion) and consumes a Rewrite Quill. It applies only while
///   resting at a Lantern, and only if the spread keeps the total and the starting values
///   as the floor (`actor::level::respec_allowed`).
/// - `Upgrade` raises the weapon in hand slot `slot` (0 to 3, `items::slot`) by one level
///   at the Smith, paying its Glimmer and material. `Temper` tempers the common weapon in
///   hand slot `slot` with temper `temper`, an index into `upgrades.ron`. Both apply only
///   while resting at a Lantern, since the Smith has no place of its own yet
///   (`items::upgrade`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Rest,
    LevelUp {
        attribute: u8,
    },
    Equip {
        slot: u8,
        item: u32,
    },
    Unequip {
        slot: u8,
    },
    UseItem {
        slot: u8,
    },
    Gesture(u16),
    SplitFlasks {
        vigil: u8,
    },
    Travel {
        lantern: u8,
    },
    Respec {
        attributes: [u16; 7],
    },
    Upgrade {
        slot: u8,
    },
    Temper {
        slot: u8,
        temper: u8,
    },
    /// While talking to an NPC (WP-068): ask about topic `topic`, an index into the NPC's
    /// topics; 254 is the quest line (it takes the quest up, or turns it in), and 255 goes
    /// back to the NPC's menu. The core decides whether it does anything.
    Ask {
        topic: u8,
    },
    /// Ends the conversation (WP-068).
    EndTalk,
    /// While at the Mirror (WP-068): sets the appearance preset and the dye (indexes into the
    /// Mirror's `presets` and `dyes`). A dye the player doesn't own changes nothing.
    Appearance {
        preset: u8,
        dye: u8,
    },
    /// While resting at a Lantern (WP-069): travel to place `place`, an index into
    /// `world_state::map::places`, in another area. The host finishes the trip after the
    /// tick (`world_state::map::tick_world`). Travel inside the area is `Travel`.
    TravelFar {
        place: u16,
    },
    /// Places a map marker at `(x, z)` in decimeters, or removes the one within 3 m of it
    /// (WP-069).
    MapMarker {
        x: i16,
        z: i16,
    },
}

/// Everything the simulation consumes in one tick.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TickInput {
    pub frame: ActionFrame,
    pub commands: Vec<Command>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_round_trip_and_layout() {
        let f = ActionFrame {
            move_x: -127,
            move_z: 100,
            buttons: buttons::LIGHT | buttons::TWO_HAND,
        };
        assert_eq!(f.to_bytes(), [0x81, 100, 0x01, 0x80]);
        assert_eq!(ActionFrame::from_bytes(f.to_bytes()), f);
    }
}
