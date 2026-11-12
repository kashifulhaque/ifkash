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

/// A discrete, menu-level action. Commands are no-ops in the simulation until the
/// systems that consume them land.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Rest,
    LevelUp { attribute: u8 },
    Equip { slot: u8, item: u32 },
    Unequip { slot: u8 },
    UseItem { slot: u8 },
    Gesture(u16),
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
