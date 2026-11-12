//! The menus' exports (WP-046): the character's read-only view, the equipment and level-up
//! previews, the Lantern names, and a replay encoder that includes the queued commands.
//! `LfWorld::queue_command` (in `lib.rs`) is the only way the menus change the world, and
//! it queues a core `Command`.

use lf_content::Content;
use lf_core::{ActionFrame, Command, Replay};
use wasm_bindgen::prelude::*;

use crate::LfWorld;

/// Builds the `Command` for `queue_command`'s `kind`, `a`, and `b`; `None` when the kind is
/// unknown or an operand doesn't fit.
pub(crate) fn command_from_words(kind: u32, a: u32, b: u32) -> Option<Command> {
    let byte = |v: u32| u8::try_from(v).ok();
    Some(match kind {
        0 => Command::Rest,
        1 => Command::LevelUp {
            attribute: byte(a)?,
        },
        2 => Command::Equip {
            slot: byte(a)?,
            item: b,
        },
        3 => Command::Unequip { slot: byte(a)? },
        4 => Command::UseItem { slot: byte(a)? },
        5 => Command::Gesture(u16::try_from(a).ok()?),
        6 => Command::SplitFlasks { vigil: byte(a)? },
        7 => Command::Travel { lantern: byte(a)? },
        // The dialogue commands (WP-068) start at 40, clear of the kinds other work packages
        // add to the menu range.
        40 => Command::Ask { topic: byte(a)? },
        41 => Command::EndTalk,
        42 => Command::Appearance {
            preset: byte(a)?,
            dye: byte(b)?,
        },
        // The map commands (WP-069): 60 TravelFar (`a` is the place), 61 MapMarker (`a` and `b`
        // are the position in decimeters, plus 32768).
        60 => Command::TravelFar {
            place: u16::try_from(a).ok()?,
        },
        61 => Command::MapMarker {
            x: i16::try_from(i64::from(a) - 32_768).ok()?,
            z: i16::try_from(i64::from(b) - 32_768).ok()?,
        },
        _ => return None,
    })
}

/// Like `encode_replay`, plus the menu commands the browser queued: `commands` holds four
/// words per command, `[tick, kind, a, b]` (the kinds of `LfWorld::queue_command`, and the
/// tick is `tick_count()` when it was queued). Trailing words short of a command, and
/// commands of an unknown kind, are ignored.
#[wasm_bindgen]
pub fn encode_replay_with_commands(
    seed: u64,
    scenario: &str,
    frames: &[u8],
    commands: &[u32],
) -> Result<Vec<u8>, JsError> {
    let content = Content::load_embedded().map_err(|e| JsError::new(&e.to_string()))?;
    let (chunks, _) = frames.as_chunks::<4>();
    let frames: Vec<ActionFrame> = chunks.iter().map(|&b| ActionFrame::from_bytes(b)).collect();
    let (quads, _) = commands.as_chunks::<4>();
    let commands: Vec<(u64, Command)> = quads
        .iter()
        .filter_map(|&[t, k, a, b]| Some((u64::from(t), command_from_words(k, a, b)?)))
        .collect();
    let replay = Replay::record(seed, scenario, &content, &frames, commands)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(replay.encode())
}

/// Words in the fixed part of `LfWorld::menu_state`, so the host can check its reader.
const MENU_WORDS: usize = 45;

/// Length of the fixed part of `LfWorld::menu_state`.
#[wasm_bindgen]
pub fn menu_words() -> u32 {
    MENU_WORDS as u32
}

/// Words in a `preview_equip` or `loadout_stats` record.
const LOADOUT_WORDS: usize = 8;

/// Length of a `preview_equip` record.
#[wasm_bindgen]
pub fn loadout_words() -> u32 {
    LOADOUT_WORDS as u32
}

/// A loadout as `LOADOUT_WORDS` floats: 1 if the change was allowed, physical attack
/// rating (rounded), 1 if unwieldy, defense, equip load, maximum equip load (both in
/// tenths), the roll tier (0 Light, 1 Medium, 2 Heavy, 3 Overloaded), and 1 for valid.
fn loadout_record(ok: bool, s: lf_core::items::preview::LoadoutStats) -> Vec<f32> {
    use lf_core::combat::dodge::LoadTier;
    vec![
        f32::from(u8::from(ok)),
        ((s.ar_phys_milli + 500) / 1000) as f32,
        f32::from(u8::from(s.unwieldy)),
        s.defense as f32,
        s.load_tenths as f32,
        s.max_load_tenths as f32,
        match s.tier {
            LoadTier::Light => 0.0,
            LoadTier::Medium => 1.0,
            LoadTier::Heavy => 2.0,
            LoadTier::Overloaded => 3.0,
        },
        1.0,
    ]
}

#[wasm_bindgen]
impl LfWorld {
    /// The menus' read-only view of the character. `menu_words()` fixed floats, then the
    /// variable part. Fixed layout:
    ///
    /// - 0 to 15: the item in each equipment slot by slot number (0 for empty; the slot
    ///   numbers are `lf_core::items::slot`: 0 to 3 hands, 4 head, 5 body, 6 to 9 charms,
    ///   10 to 14 quick items, 15 arrows).
    /// - 16, 17, 18: selected right-hand, left-hand, and quick slot (0-based within the group).
    /// - 19: open charm slots.
    /// - 20: Glimmer held.
    /// - 21 to 27: attributes: Vitality, Focus, Endurance, Might, Finesse, Insight, Devotion.
    /// - 28: character level. 29: Glimmer the next level costs. 30: the attribute maximum.
    /// - 31 to 35: flasks: total charges, the Vigil Flask's share, Vigil charges, Lucid
    ///   charges, potency level. 36: the most charges the flasks can hold.
    /// - 37: 1 while resting at a Lantern (the Lantern menu is open).
    /// - 38: 1 while a boss fight is running (Quit to title is disabled). 39: 1 if dead.
    /// - 40: the index plus one of the Lantern being rested at (0 for none).
    /// - 41, 42: counts of Lanterns `n_l` and owned gear `n_g`.
    /// - 43, 44: registry numbers of the Vigil and Lucid flasks (0 if the content has none),
    ///   so the menu can put a flask back in a quick slot.
    ///
    /// Then `n_l` words (1 if that Lantern is lit), `n_g` registry numbers of owned
    /// weapons, armor, and charms, and pairs `(item, count)` for every consumable stack
    /// until the end.
    pub fn menu_state(&self) -> Vec<f32> {
        use lf_core::ai::boss::Stage;
        use lf_core::world_state::flags;
        let w = &self.world;
        let mut out = vec![0.0f32; MENU_WORDS];
        let Some(p) = w.actors.get(w.player) else {
            return out;
        };
        let inv = &p.inventory;
        for (i, s) in inv.equipment.slots.iter().enumerate() {
            out[i] = s.unwrap_or(0) as f32;
        }
        out[16] = f32::from(inv.equipment.right_active);
        out[17] = f32::from(inv.equipment.left_active);
        out[18] = f32::from(inv.equipment.quick_active);
        out[19] = f32::from(inv.equipment.charm_slots);
        out[20] = inv.glimmer as f32;
        let a = &p.stats.attrs;
        let attrs = [
            a.vitality,
            a.focus,
            a.endurance,
            a.might,
            a.finesse,
            a.insight,
            a.devotion,
        ];
        for (i, v) in attrs.iter().enumerate() {
            out[21 + i] = f32::from(*v);
        }
        let level = a.level(&w.attributes);
        out[28] = level as f32;
        out[29] = lf_core::actor::level::level_cost(&w.attributes, level) as f32;
        out[30] = f32::from(w.attributes.max);
        let f = &inv.flasks;
        out[31] = f32::from(f.max_charges);
        out[32] = f32::from(f.vigil_max);
        out[33] = f32::from(f.vigil);
        out[34] = f32::from(f.lucid);
        out[35] = f32::from(f.level);
        out[36] = w
            .items
            .flask
            .as_ref()
            .map_or(0.0, |r| f32::from(r.max_charges));
        out[37] = f32::from(u8::from(w.state.resting.is_some()));
        out[38] = f32::from(u8::from(
            w.bosses
                .iter()
                .any(|r| matches!(r.stage, Stage::Intro { .. } | Stage::Fight)),
        ));
        out[39] = f32::from(u8::from(p.dead));
        out[40] = w
            .state
            .resting
            .as_deref()
            .and_then(|id| w.layout.lantern(id))
            .map_or(0.0, |(i, _)| (i + 1) as f32);
        out[41] = w.layout.lanterns.len() as f32;
        out[42] = inv.gear.len() as f32;
        if let Some(r) = w.items.flask.as_ref() {
            out[43] = r.vigil as f32;
            out[44] = r.lucid as f32;
        }
        out.extend(
            w.layout
                .lanterns
                .iter()
                .map(|l| f32::from(u8::from(w.state.flags.is_set(&flags::lantern_lit(&l.id))))),
        );
        out.extend(inv.gear.iter().map(|&n| n as f32));
        for &(item, count) in &inv.stacks {
            out.extend([item as f32, f32::from(count)]);
        }
        out
    }

    /// The ID of the Lantern at `index` in the scenario (for example `lantern_field`);
    /// empty if there is none.
    pub fn lantern_name(&self, index: u32) -> String {
        self.world
            .layout
            .lanterns
            .get(index as usize)
            .map(|l| l.id.clone())
            .unwrap_or_default()
    }

    /// The stats after putting `item` in `slot` (or emptying `slot` when `item` is 0),
    /// without changing the world: `loadout_words()` floats. Word 0 is 1 if the change is
    /// allowed (the command would do it); when it isn't, the rest are the current stats.
    /// An out-of-range slot gives the current stats. Empty if the player is gone.
    pub fn preview_equip(&self, slot: u32, item: u32) -> Vec<f32> {
        let slot = u8::try_from(slot).unwrap_or(u8::MAX);
        lf_core::items::preview::preview_equip(&self.world, slot, item)
            .map(|(ok, s)| loadout_record(ok, s))
            .unwrap_or_default()
    }

    /// The current stats, in the layout of `preview_equip` (word 0 is 0).
    pub fn loadout_stats(&self) -> Vec<f32> {
        let Some(p) = self.world.actors.get(self.world.player) else {
            return Vec::new();
        };
        lf_core::items::preview::loadout_stats(&self.world, &p.inventory)
            .map(|s| loadout_record(false, s))
            .unwrap_or_default()
    }

    /// What raising `attribute` (0 to 6) by one does: eight floats, the value now and
    /// after for maximum HP, FP, stamina, and equip load (tenths). Empty if the attribute
    /// doesn't exist or is at the maximum. Read-only.
    pub fn attribute_preview(&self, attribute: u32) -> Vec<f32> {
        let Ok(attribute) = u8::try_from(attribute) else {
            return Vec::new();
        };
        let Some((a, b)) =
            lf_core::world_state::lantern_menu::level_preview(&self.world, attribute)
        else {
            return Vec::new();
        };
        vec![
            a.max_hp as f32,
            b.max_hp as f32,
            a.max_fp,
            b.max_fp,
            a.max_stamina,
            b.max_stamina,
            a.max_equip_load_tenths as f32,
            b.max_equip_load_tenths as f32,
        ]
    }
}

#[wasm_bindgen]
impl LfWorld {
    /// The scenario (area) the world is in now. It changes when the player travels to
    /// another area (WP-069), which replaces the world's level.
    pub fn scenario_name(&self) -> String {
        self.world.physics.scenario.clone()
    }

    /// The map as JSON (WP-069): the area, the cells the player has visited, the Lanterns of
    /// the whole realm and whether each is lit, the defeated bosses, the Remnant, the
    /// markers, and where the player is. See `lf_core::world_state::map::view_json`.
    /// Read-only.
    pub fn map_json(&self) -> String {
        let bounds = self
            .content
            .scenario(&self.world.physics.scenario)
            .and_then(|s| s.heightfield.as_ref())
            .map(|h| {
                [
                    h.center[0] - h.size_x / 2.0,
                    h.center[2] - h.size_z / 2.0,
                    h.center[0] + h.size_x / 2.0,
                    h.center[2] + h.size_z / 2.0,
                ]
            });
        lf_core::world_state::map::view_json(&self.world, &self.places, bounds)
    }
}
