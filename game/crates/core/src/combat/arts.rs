//! Arts: the weapon skills that cost FP (docs/game/04-arsenal-and-builds.md, "Arts"). See
//! `lf_content::arts` for the data.
//!
//! # How an Art runs
//!
//! `ART` starts the Art of the right-hand weapon (`combat::moves::Action::Art`). The Art is a
//! move like any other: it costs its FP through `Move::fp` (`combat::stamina::try_spend`),
//! lands hitboxes and projectiles, and cancels as its frame data says. This module adds the
//! parts particular to Arts:
//!
//! - **Which Art a weapon has.** A weapon's Art is its `default_art`, unless an Art Scroll has
//!   changed it. A scroll's Art is a world flag keyed by the weapon's registry number
//!   (`weapon.art.<num>`), so a game that never reads a scroll hashes as it did before.
//! - **Elemental variants.** A move can replace the Art's base move for a weapon whose
//!   elemental part is a given element (the Choir Toll for a Radiance weapon).
//! - **Buffs.** An Art can apply a buff on one of its move's frames ([`run`]).
//! - **Stances.** An Art can hold the guard up while it plays (Bastion and Windward Parry),
//!   with an auto counter or a parry window of its own ([`ArtRow::stance`]).
//!
//! The per-weapon table ([`ArtTable`]) is built from content, so it is not hashed.

use lf_content::moves::Element;
use lf_content::weapons::WeaponClass;
use lf_content::{Content, NumId};
use serde::{Deserialize, Serialize};

use super::moves::MoveNum;
use super::status;
use crate::arena::Id;
use crate::world::World;

/// The flag prefix for a weapon's Art, set by an Art Scroll: the Art's registry number, or
/// absent for the weapon's default.
const ART_PREFIX: &str = "weapon.art.";

/// One Art with its references resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtRow {
    pub num: NumId,
    pub id: String,
    pub classes: Vec<WeaponClass>,
    pub fp: u16,
    /// The base move the Art plays.
    pub move_num: MoveNum,
    /// The move for a weapon whose elemental part is the element.
    pub variants: Vec<(Element, MoveNum)>,
    /// The buff the move applies, and the move frame it applies on.
    pub buff: Option<(u16, NumId)>,
    pub stance: bool,
    pub auto_counter: bool,
    pub parry_ticks: u8,
}

/// A weapon's Art-relevant facts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponRow {
    pub num: NumId,
    pub class: WeaponClass,
    pub unique: bool,
    /// The Art the weapon starts with. `None` for a catalyst, which casts spells instead.
    pub default_art: Option<NumId>,
}

/// Every Art and every weapon's Art facts. Built from content, so it isn't hashed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ArtTable {
    /// Sorted by registry number.
    arts: Vec<ArtRow>,
    /// Every move an Art plays (the base and the variants), sorted by move number, with the
    /// index of its Art in `arts`.
    by_move: Vec<(MoveNum, usize)>,
    /// Sorted by registry number.
    weapons: Vec<WeaponRow>,
}

impl ArtTable {
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let num = |id: &str| {
            content
                .registry
                .num(id)
                .ok_or_else(|| format!("\"{id}\" is missing from registry.ron"))
        };
        let mut t = Self::default();
        for a in &content.arts {
            let variants = a
                .variants
                .iter()
                .map(|v| Ok((v.element, num(&v.move_id)?)))
                .collect::<Result<Vec<_>, String>>()?;
            let buff = a
                .buff
                .as_ref()
                .map(|b| Ok::<_, String>((b.frame, num(&b.buff)?)))
                .transpose()?;
            t.arts.push(ArtRow {
                num: num(&a.id)?,
                id: a.id.clone(),
                classes: a.classes.clone(),
                fp: a.fp,
                move_num: num(&a.move_id)?,
                variants,
                buff,
                stance: a.stance,
                auto_counter: a.auto_counter,
                parry_ticks: a.parry_ticks,
            });
        }
        t.arts.sort_by_key(|a| a.num);
        for (i, a) in t.arts.iter().enumerate() {
            t.by_move.push((a.move_num, i));
            t.by_move.extend(a.variants.iter().map(|&(_, m)| (m, i)));
        }
        t.by_move.sort_unstable();
        for (_, w) in content.all_weapons() {
            t.weapons.push(WeaponRow {
                num: num(&w.id)?,
                class: w.class,
                unique: w.unique,
                default_art: w.default_art.as_deref().map(num).transpose()?,
            });
        }
        t.weapons.sort_by_key(|w| w.num);
        Ok(t)
    }

    /// The Art with registry number `num`.
    pub fn get(&self, num: NumId) -> Option<&ArtRow> {
        self.arts
            .binary_search_by_key(&num, |a| a.num)
            .ok()
            .map(|i| &self.arts[i])
    }

    /// Finds an Art's number by string ID. A linear scan, for tools and tests.
    pub fn num_of(&self, id: &str) -> Option<NumId> {
        self.arts.iter().find(|a| a.id == id).map(|a| a.num)
    }

    /// The Art whose move (base or variant) is `m`.
    pub fn of_move(&self, m: MoveNum) -> Option<&ArtRow> {
        self.by_move
            .binary_search_by_key(&m, |e| e.0)
            .ok()
            .map(|i| &self.arts[self.by_move[i].1])
    }

    fn weapon(&self, w: NumId) -> Option<&WeaponRow> {
        self.weapons
            .binary_search_by_key(&w, |r| r.num)
            .ok()
            .map(|i| &self.weapons[i])
    }

    /// The move a weapon's default Art plays, for a weapon whose elemental part is `element`.
    /// `None` for a weapon with no default Art (a catalyst).
    pub fn default_move(&self, weapon: NumId, element: Option<Element>) -> Option<MoveNum> {
        let art = self.weapon(weapon)?.default_art?;
        Some(self.move_for(self.get(art)?, element))
    }

    /// The move `art` plays for a weapon whose elemental part is `element`.
    pub fn move_for(&self, art: &ArtRow, element: Option<Element>) -> MoveNum {
        element
            .and_then(|e| art.variants.iter().find(|(v, _)| *v == e).map(|&(_, m)| m))
            .unwrap_or(art.move_num)
    }
}

/// The Art weapon `weapon` uses now: an Art Scroll's if one changed it, else its default.
pub fn weapon_art(world: &World, weapon: NumId) -> Option<&ArtRow> {
    let row = world.moves.arts.weapon(weapon)?;
    let scrolled = world.state.flags.get(&format!("{ART_PREFIX}{weapon}"));
    let num = if scrolled > 0 {
        scrolled
    } else {
        row.default_art?
    };
    world.moves.arts.get(num)
}

/// The move the player's Art starts, with the right-hand weapon `weapon` held. `None` for a
/// weapon with no Art.
pub fn player_art_move(world: &World, weapon: NumId) -> Option<MoveNum> {
    let art = weapon_art(world, weapon)?;
    let element = world
        .damage
        .weapon(weapon)
        .and_then(|w| w.elem.map(|(e, _)| e));
    Some(world.moves.arts.move_for(art, element))
}

/// The parry window a stance of `id` gives, or 0 when `id` isn't in a parry stance.
pub fn stance_parry_ticks(world: &World, id: Id) -> u8 {
    world
        .actors
        .get(id)
        .and_then(|a| a.moves.current)
        .and_then(|c| world.moves.arts.of_move(c.num))
        .filter(|r| r.stance)
        .map_or(0, |r| r.parry_ticks)
}

/// Applies an Art Scroll to the right-hand weapon. The weapon must be common, and its class
/// must be one the Art lists. Returns false, changing nothing, otherwise (and the scroll is
/// kept).
pub fn apply_scroll(world: &mut World, art_id: &str) -> bool {
    let Some(art) = world.moves.arts.num_of(art_id) else {
        return false;
    };
    let Some(weapon) = world
        .actors
        .get(world.player)
        .and_then(|p| p.inventory.equipment.right_weapon())
    else {
        return false;
    };
    let Some(row) = world.moves.arts.weapon(weapon) else {
        return false;
    };
    let Some(a) = world.moves.arts.get(art) else {
        return false;
    };
    if row.unique || !a.classes.contains(&row.class) {
        return false;
    }
    world.state.flags.set(&format!("{ART_PREFIX}{weapon}"), art);
    world.sync_player_moveset();
    true
}

/// Tick system 6, after the move system: applies the buff of the Art move the player is on,
/// on the move's buff frame.
pub fn run(world: &mut World) {
    let id = world.player;
    let Some(c) = world
        .actors
        .get(id)
        .filter(|a| !a.dead)
        .and_then(|a| a.moves.current)
    else {
        return;
    };
    let Some(buff) = world
        .moves
        .arts
        .of_move(c.num)
        .and_then(|r| r.buff)
        .filter(|&(frame, _)| frame == c.frame)
        .map(|(_, buff)| buff)
    else {
        return;
    };
    if let Some(a) = world.actors.get_mut(id) {
        status::apply_buff(a, &world.status, buff);
    }
}

#[cfg(test)]
mod tests {
    use lf_content::Content;

    use super::*;
    use crate::items::slot;

    fn world(scenario: &str) -> (World, Content) {
        let content = Content::load_embedded().expect("embedded content");
        let w = World::new(5, &content, scenario).expect("scenario exists");
        (w, content)
    }

    /// Puts `weapon` in the right hand, as a player who owns it would.
    fn equip(w: &mut World, content: &Content, weapon: &str) {
        let num = content.registry.num(weapon).expect("a weapon");
        let player = w.player;
        if let Some(p) = w.actors.get_mut(player) {
            p.inventory.give_gear(num);
            p.inventory.equip(slot::RIGHT_1, num, &w.items);
        }
        w.sync_player_moveset();
    }

    fn art_move(content: &Content, id: &str) -> MoveNum {
        content.registry.num(id).expect("a move")
    }

    /// The move `ART` starts with the player's moveset as it is now.
    fn player_art(w: &World) -> Option<MoveNum> {
        w.moves.player_moveset.as_ref().and_then(|m| m.art)
    }

    #[test]
    fn a_scroll_changes_the_art_only_for_a_class_the_art_lists() {
        // The castle kite is a shield, which Feint Step doesn't list, and Windward Parry does.
        let (mut w, c) = world("m5_art_bastion");
        assert!(!apply_scroll(&mut w, "art_feint_step"));
        assert_eq!(player_art(&w), Some(art_move(&c, "amv_bastion")));
        assert!(apply_scroll(&mut w, "art_windward_parry"));
        assert_eq!(player_art(&w), Some(art_move(&c, "amv_windward_parry")));
    }

    #[test]
    fn a_unique_weapon_refuses_a_scroll() {
        let (mut w, c) = world("m5_art_bastion");
        equip(&mut w, &c, "wpn_nopokedb");
        assert!(!apply_scroll(&mut w, "art_windward_parry"));
    }

    #[test]
    fn a_radiance_weapon_plays_the_choir_toll() {
        // The Bellwright's Mallet is Aether, so its Toll is the Bell Toll; the Choir Glaive is
        // Radiance, so its Toll is the Choir Toll.
        let (mut w, c) = world("m5_art_toll");
        assert_eq!(player_art(&w), Some(art_move(&c, "amv_bell_toll")));
        equip(&mut w, &c, "wpn_choir_glaive");
        assert_eq!(player_art(&w), Some(art_move(&c, "amv_choir_toll")));
    }
}
