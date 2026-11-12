//! Independent PCG32 streams, all derived from one episode seed.

use serde::{Deserialize, Serialize};

const PCG_MULT: u64 = 6_364_136_223_846_793_005;

/// PCG-XSH-RR 64/32 (O'Neill). State and increment are plain `u64`s, so it
/// serializes canonically.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    /// Standard PCG seeding: `state` is the seed, `stream` selects the sequence.
    pub fn new(state: u64, stream: u64) -> Self {
        let mut rng = Pcg32 {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(state);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(PCG_MULT).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    pub fn next_u64(&mut self) -> u64 {
        (u64::from(self.next_u32()) << 32) | u64::from(self.next_u32())
    }

    /// Uniform in `[0, 1)` with 24 bits of precision.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
}

/// splitmix64 step, used only to derive stream seeds.
fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Distinct constants mixed into the seed so each stream is independent.
const STREAM_AI: u64 = 0x0000_0000_0000_A101;
const STREAM_COMBAT: u64 = 0x0000_0000_0000_C0B2;
const STREAM_LOOT: u64 = 0x0000_0000_0000_1003;
const STREAM_WORLD: u64 = 0x0000_0000_0000_3004;

fn derive(seed: u64, stream_const: u64) -> Pcg32 {
    let mut s = seed ^ stream_const.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let state = splitmix64(&mut s);
    let inc = splitmix64(&mut s);
    Pcg32::new(state, inc)
}

/// Separate streams so adding a draw in one system never shifts another's sequence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngStreams {
    pub ai: Pcg32,
    pub combat: Pcg32,
    pub loot: Pcg32,
    pub world: Pcg32,
}

impl RngStreams {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            ai: derive(seed, STREAM_AI),
            combat: derive(seed, STREAM_COMBAT),
            loot: derive(seed, STREAM_LOOT),
            world: derive(seed, STREAM_WORLD),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcg32_reference_vector() {
        // Reference from the pcg-c demo: seed 42, stream 54.
        let mut r = Pcg32::new(42, 54);
        let got: Vec<u32> = (0..6).map(|_| r.next_u32()).collect();
        assert_eq!(
            got,
            [
                0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e
            ]
        );
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = RngStreams::from_seed(7);
        let mut b = RngStreams::from_seed(7);
        for _ in 0..100 {
            assert_eq!(a.loot.next_u32(), b.loot.next_u32());
        }
    }

    #[test]
    fn streams_are_independent() {
        let mut a = RngStreams::from_seed(42);
        let mut b = RngStreams::from_seed(42);
        // Draw heavily from `ai` in `a` only; `combat`, `loot`, `world` must be unaffected.
        for _ in 0..1000 {
            a.ai.next_u32();
        }
        for _ in 0..50 {
            assert_eq!(a.combat.next_u32(), b.combat.next_u32());
            assert_eq!(a.loot.next_u32(), b.loot.next_u32());
            assert_eq!(a.world.next_u32(), b.world.next_u32());
        }
        assert_ne!(a.ai, b.ai);
        let mut c = RngStreams::from_seed(42);
        assert_ne!(c.ai.next_u32(), c.combat.next_u32());
    }
}
