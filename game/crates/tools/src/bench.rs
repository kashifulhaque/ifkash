//! `lf-tools bench`: a minimal native simulation benchmark.

use std::process::ExitCode;
use std::time::Instant;

use lf_core::input::TickInput;
use lf_core::replay::{Replay, expand};
use lf_core::{World, tick};

pub fn run(ticks: u64) -> ExitCode {
    // Loop the golden replay's frames when present; otherwise run with idle input.
    let path = crate::replay::replays_dir().join("m0_random_3000.lfr");
    let frames = std::fs::read(&path)
        .ok()
        .and_then(|b| Replay::decode(&b).ok())
        .map(|r| expand(&r.frames))
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| vec![Default::default()]);
    let mut world = World::new(42);
    let mut input = TickInput::default();
    let start = Instant::now();
    for i in 0..ticks {
        input.frame = frames[(i as usize) % frames.len()];
        tick(&mut world, &input);
    }
    let secs = start.elapsed().as_secs_f64();
    println!(
        "{ticks} ticks in {secs:.3} s: {:.0} ticks/s, {:.3} us/tick (final hash {:016x})",
        ticks as f64 / secs,
        secs * 1e6 / ticks as f64,
        world.state_hash()
    );
    ExitCode::SUCCESS
}
