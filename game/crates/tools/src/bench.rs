//! `lf-tools bench`: a minimal native simulation benchmark.
//!
//! Runs one player for `ticks` ticks in two scenarios: the `test_field` golden route
//! (walk, sprint up the slope, jump the ledge, fall the drop), restarting from a fresh
//! world each time the route ends, and the M0 random input on `m0_flat`.

use std::process::ExitCode;
use std::time::Instant;

use lf_content::Content;
use lf_core::input::{ActionFrame, TickInput};
use lf_core::replay::{Replay, expand};
use lf_core::{World, tick};

/// Frames of a golden replay, or one idle frame if the file is missing.
fn frames(file: &str) -> Vec<ActionFrame> {
    let path = crate::replay::replays_dir().join(file);
    std::fs::read(&path)
        .ok()
        .and_then(|b| Replay::decode(&b).ok())
        .map(|r| expand(&r.frames))
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| vec![ActionFrame::default()])
}

fn bench(content: &Content, scenario: &str, file: &str, ticks: u64) -> Result<(), String> {
    let frames = frames(file);
    let fresh = World::new(42, content, scenario).map_err(|e| e.to_string())?;
    let mut world = fresh.clone();
    let mut input = TickInput::default();
    let start = Instant::now();
    for i in 0..ticks {
        let k = (i as usize) % frames.len();
        if k == 0 && i > 0 {
            world = fresh.clone();
        }
        input.frame = frames[k];
        tick(&mut world, &input);
    }
    let secs = start.elapsed().as_secs_f64();
    println!(
        "{scenario} ({file}): {ticks} ticks in {secs:.3} s: {:.0} ticks/s, {:.3} us/tick (final hash {:016x})",
        ticks as f64 / secs,
        secs * 1e6 / ticks as f64,
        world.state_hash()
    );
    Ok(())
}

pub fn run(ticks: u64) -> ExitCode {
    let content = match Content::load_dir(crate::default_data()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let result = bench(&content, "test_field", "m1_movement.lfr", ticks)
        .and_then(|()| bench(&content, "m0_flat", "m0_random_3000.lfr", ticks));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
