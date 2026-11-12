//! `lf-tools gen-ts`: write the TypeScript types and the UI JSON digest.

use std::path::Path;
use std::process::ExitCode;

use lf_content::Content;
use serde_json::{Value, json};
use ts_rs::{Config, TS};

/// Builds the UI digest: names and IDs for the codex and menus, plus the attribute
/// curves. The content hash is a string because a u64 doesn't fit a JS number.
fn digest(content: &Content) -> Value {
    let moves: Vec<Value> = content
        .all_moves()
        .map(|(file, m)| {
            json!({
                "id": m.id,
                "num": content.registry.num(&m.id),
                "file": file,
                "anim": m.anim,
                "totalFrames": m.total_frames,
                "stamina": m.stamina,
                "fp": m.fp,
                "stub": m.stub,
            })
        })
        .collect();
    let weapons: Vec<Value> = content
        .all_weapons()
        .map(|(file, w)| {
            json!({
                "id": w.id,
                "num": content.registry.num(&w.id),
                "file": file,
                "name": w.name,
                "class": format!("{:?}", w.class),
                "baseDamage": w.base_damage,
                "weightTenths": w.weight_tenths,
                "unique": w.unique,
                "stub": w.stub,
                "catalystSchool": w.catalyst_school.map(|s| format!("{s:?}")),
                "element": w.elemental.as_ref().map(|e| format!("{:?}", e.element)),
            })
        })
        .collect();
    // Armor pieces, for the renderer's armory and equipped-armor tints (WP-058).
    let armor: Vec<Value> = content
        .armor
        .iter()
        .map(|a| {
            json!({
                "id": a.id,
                "num": content.registry.num(&a.id),
                "name": a.name,
                "set": a.set,
                "slot": format!("{:?}", a.slot),
                "weightTenths": a.weight_tenths,
            })
        })
        .collect();
    // Spells, for the catalyst VFX (WP-058).
    let spells: Vec<Value> = content
        .spells
        .iter()
        .map(|sp| {
            json!({
                "id": sp.id,
                "num": content.registry.num(&sp.id),
                "name": sp.name,
                "school": format!("{:?}", sp.school),
            })
        })
        .collect();
    json!({
        "contentHash": content.content_hash().to_string(),
        "contentHashHex": format!("{:016x}", content.content_hash()),
        "attributes": content.attributes,
        "moves": moves,
        "weapons": weapons,
        "armor": armor,
        "spells": spells,
    })
}

/// Generates the files into `out`. Stale `.ts` and `.json` files are removed first.
pub fn run(data: &Path, out: &Path) -> ExitCode {
    match generate(data, out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn generate(data: &Path, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = Content::load_dir(data)?;
    std::fs::create_dir_all(out)?;
    for entry in std::fs::read_dir(out)? {
        let path = entry?.path();
        if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("ts" | "json")
        ) {
            std::fs::remove_file(path)?;
        }
    }
    Content::export_all(&Config::new().with_out_dir(out))?;
    let json = serde_json::to_string_pretty(&digest(&content))?;
    std::fs::write(out.join("digest.json"), json + "\n")?;
    println!(
        "wrote TypeScript types and digest.json to {}",
        out.display()
    );
    Ok(())
}
