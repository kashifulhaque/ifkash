//! The render manifest: what the renderer needs to draw an area (WP-041).
//!
//! JSON, one object, format 1:
//!
//! - `format`: 1.
//! - `area`: `id`, `name`, `bounds`, `kill_y`, `ambient`, `music`, `regions`,
//!   `connections`, and `cells` (the 32-meter activity cells the bounds cover, as
//!   inclusive `min` and `max` cell indices; docs/game/01-architecture.md, "Activity
//!   cells").
//! - `terrain`: the heightfield (`center`, `rows`, `cols`, `size_x`, `size_z`, and
//!   row-major `heights`), or null. Same layout as the core's `Heightfield`.
//! - `pieces`: every kit piece the area uses, by ID: `model` (a path relative to
//!   `static/latticefall/`, or null for greybox), `tags`, and `colliders` (piece frame;
//!   greybox pieces are drawn from these).
//! - `instances`: layout pieces, landmarks, and hand-placed props, each with `name`,
//!   `piece`, `pos`, `yaw` (radians from +Z toward +X), `scale` (x, y, z), and `source`
//!   (`piece`, `landmark`, or `prop`).
//! - `scatter`: one batch per (rule, piece), ready for instancing: `rule` (index in
//!   `dressing.ron`), `tag`, `piece`, `collider`, and `transforms`, each
//!   `[x, y, z, yaw, scale]`.
//! - `spawn`: the player start (`pos`, `yaw`).
//! - `entities`: the `entities.ron` list as externally tagged objects, for example
//!   `{"Lantern": {"id": ..., "pos": ..., "yaw": ...}}`. New entity kinds appear here
//!   without a format change.

use std::collections::BTreeMap;

use lf_content::level::{Placement, Source};
use lf_content::{Kits, Level, Scenario};
use serde_json::{Value, json};

/// The format version written to `format`.
pub const FORMAT: u32 = 1;

/// Activity cell size in meters (docs/game/01-architecture.md, "Activity cells").
const ACTIVITY_CELL_M: f32 = 32.0;

/// Builds the manifest JSON text for a compiled level.
pub fn build(level: &Level, kits: &Kits<'_>, scenario: &Scenario) -> Result<String, String> {
    let area = &level.area;
    let cell = |v: f32| (v / ACTIVITY_CELL_M).floor() as i32;
    // The max edge belongs to the cell below it when it sits exactly on a boundary.
    let cell_max = |v: f32| (v / ACTIVITY_CELL_M).ceil() as i32 - 1;
    let mut instances = level.placements();
    for (i, p) in level.dressing.props.iter().enumerate() {
        instances.push(Placement {
            name: format!("prop#{i}"),
            piece: p.kit.clone(),
            pos: p.pos,
            yaw: p.yaw,
            scale: [p.scale; 3],
            source: Source::Prop,
        });
    }
    let scatter = level.scatter(kits, scenario.heightfield.as_ref(), false)?;
    let mut batches: BTreeMap<(usize, &str), Vec<[f32; 5]>> = BTreeMap::new();
    for s in &scatter {
        batches
            .entry((s.rule, s.piece.as_str()))
            .or_default()
            .push([s.pos[0], s.pos[1], s.pos[2], s.yaw, s.scale]);
    }
    let mut pieces = BTreeMap::new();
    for id in instances
        .iter()
        .map(|p| p.piece.as_str())
        .chain(batches.keys().map(|(_, p)| *p))
    {
        let piece = kits
            .get(id)
            .ok_or_else(|| format!("unknown kit piece {id:?}"))?;
        pieces.insert(
            id,
            json!({ "model": piece.model, "tags": piece.tags, "colliders": piece.colliders }),
        );
    }
    let scatter: Vec<Value> = batches
        .into_iter()
        .map(|((rule, piece), transforms)| {
            let r = &level.dressing.rules[rule];
            json!({
                "rule": rule,
                "tag": r.tag,
                "piece": piece,
                "collider": r.collider,
                "transforms": transforms,
            })
        })
        .collect();
    let manifest = json!({
        "format": FORMAT,
        "area": {
            "id": area.id,
            "name": area.name,
            "bounds": area.bounds,
            "cells": {
                "size_m": ACTIVITY_CELL_M,
                "min": [cell(area.bounds.min[0]), cell(area.bounds.min[1])],
                "max": [cell_max(area.bounds.max[0]), cell_max(area.bounds.max[1])],
            },
            "kill_y": area.kill_y,
            "ambient": area.ambient,
            "music": area.music,
            "regions": area.regions,
            "connections": area.connections,
        },
        "terrain": scenario.heightfield,
        "pieces": pieces,
        "instances": instances,
        "scatter": scatter,
        "spawn": level.entities.spawn,
        "entities": level.entities.entities,
    });
    let mut text = manifest.to_string();
    text.push('\n');
    Ok(text)
}
