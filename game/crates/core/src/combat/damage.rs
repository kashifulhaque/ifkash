//! The damage pipeline: attack rating, requirements, two-handing, and damage dealt. See
//! docs/game/03-combat.md, "Attack rating" and "Damage dealt".
//!
//! Everything here is integer math (`u128` for products), so a hit's damage is exact and
//! identical on every platform. Quantities carry a fixed scale:
//!
//! - Attack rating and damage, in thousandths ("milli"): 135.240 is `135_240`.
//! - Motion value, in thousandths: 1.3 is `1300`.
//! - Percents (counter, buff, absorption, weakness) are whole percents.
//! - The scaling curve is in millionths, `k_a` in hundred-thousandths.
//!
//! Every step truncates toward zero except the last, which rounds the sum of the typed
//! damages to the nearest whole point (halves up), so a result is within a thousandth of
//! the real-number formula before that rounding.
//!
//! # Where the rest of combat plugs in
//!
//! [`damage`] takes one [`Hit`] and one target [`Defense`], and `resolve` calls it once
//! per landed hitbox from the unchanged world. Poise damage (WP-024) and status buildup
//! (WP-025) come from the hitbox, not from this module. A projectile (WP-027) builds the
//! same [`Hit`] from its owner's [`attack_rating`].

use lf_content::attributes::DamageRules;
use lf_content::moves::{Element, PhysType};
use lf_content::scenario::DamageType;
use lf_content::{Content, NumId, Weapon};
use serde::{Deserialize, Serialize};

use crate::actor::stats::{Attrs, Defense};

/// Number of damage types: four physical, four elemental.
pub const TYPE_COUNT: usize = 8;

/// The damage type of a hitbox's physical subtype.
pub fn phys_type(p: PhysType) -> DamageType {
    match p {
        PhysType::Neutral => DamageType::Neutral,
        PhysType::Slash => DamageType::Slash,
        PhysType::Blunt => DamageType::Blunt,
        PhysType::Pierce => DamageType::Pierce,
    }
}

/// The damage type of a hitbox's element.
pub fn element_type(e: Element) -> DamageType {
    match e {
        Element::Aether => DamageType::Aether,
        Element::Fire => DamageType::Fire,
        Element::Storm => DamageType::Storm,
        Element::Radiance => DamageType::Radiance,
    }
}

/// A weapon's attack rating inputs, from its content entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponStats {
    /// The physical base damage. A temper can move part of it into `elem`.
    pub base_damage: u32,
    /// The elemental base damage and its element, if the weapon has one (WP-053).
    pub elem: Option<(Element, u32)>,
    /// Scaling in thousandths, in Might, Finesse, Insight, Devotion order.
    pub scaling_permille: [u32; 4],
    /// Requirements in the same order.
    pub requirements: [u16; 4],
    /// A unique or relic weapon: upgraded with Starglass and never tempered (WP-053).
    pub unique: bool,
}

impl WeaponStats {
    pub fn from_weapon(w: &Weapon) -> Self {
        // Content scaling is a decimal like 0.6; thousandths round it exactly.
        let pm = |s: f32| (s * 1000.0).round() as u32;
        Self {
            base_damage: u32::from(w.base_damage),
            elem: w.elemental.map(|e| (e.element, u32::from(e.base))),
            scaling_permille: [
                pm(w.scaling.might),
                pm(w.scaling.finesse),
                pm(w.scaling.insight),
                pm(w.scaling.devotion),
            ],
            requirements: [
                w.requirements.might,
                w.requirements.finesse,
                w.requirements.insight,
                w.requirements.devotion,
            ],
            unique: w.unique,
        }
    }
}

/// The damage rules plus every weapon's attack rating inputs. Built from content, so
/// it's not hashed: the content hash covers it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageTable {
    pub rules: DamageRules,
    /// The attribute cap (`attributes.ron`, `max`), for clamping Might.
    pub attr_max: u16,
    /// Sorted by registry number.
    weapons: Vec<(NumId, WeaponStats)>,
}

impl DamageTable {
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let mut weapons = Vec::new();
        for (_, w) in content.all_weapons() {
            let num = content
                .registry
                .num(&w.id)
                .ok_or_else(|| format!("weapon \"{}\" is missing from registry.ron", w.id))?;
            weapons.push((num, WeaponStats::from_weapon(w)));
        }
        weapons.sort_by_key(|(n, _)| *n);
        Ok(Self {
            rules: content.attributes.damage.clone(),
            attr_max: content.attributes.max,
            weapons,
        })
    }

    pub fn weapon(&self, num: NumId) -> Option<&WeaponStats> {
        self.weapons
            .binary_search_by_key(&num, |(n, _)| *n)
            .ok()
            .map(|i| &self.weapons[i].1)
    }

    /// The attack rating of the weapon `num` in `attrs`' hands.
    pub fn attack_rating(
        &self,
        num: NumId,
        attrs: &Attrs,
        upgrade: u8,
        two_handing: bool,
    ) -> Option<AttackRating> {
        let w = self.weapon(num)?;
        Some(attack_rating(
            &self.rules,
            self.attr_max,
            w,
            attrs,
            upgrade,
            two_handing,
        ))
    }
}

/// A weapon's attack rating in someone's hands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttackRating {
    /// The physical AR in thousandths, dealt as whatever physical subtype the hitbox has.
    pub phys_milli: u64,
    /// The elemental AR in thousandths, indexed Aether, Fire, Storm, Radiance. A weapon's
    /// elemental part fills its slot; the rest are 0.
    pub elem_milli: [u64; 4],
    /// The weapon's element, when it has one. A hit with no element of its own takes this
    /// one, so the weapon's elemental part is dealt (resolve.rs).
    pub element: Option<Element>,
    /// A requirement isn't met: scaling is 0, base damage is cut, and the move system
    /// lengthens recovery by `unwieldy_recovery_percent`.
    pub unwieldy: bool,
}

/// An attribute value in half points after two-handing: Might counts
/// `two_hand_might_percent` of its value, capped at `attr_max`.
fn might_halves(rules: &DamageRules, attr_max: u16, might: u16, two_handing: bool) -> u32 {
    let halves = u32::from(might) * 2;
    if two_handing {
        (halves * u32::from(rules.two_hand_might_percent) / 100).min(u32::from(attr_max) * 2)
    } else {
        halves
    }
}

/// Whether `attrs` meet every requirement of `w`, counting two-handed Might.
pub fn meets_requirements(
    rules: &DamageRules,
    attr_max: u16,
    w: &WeaponStats,
    attrs: &Attrs,
    two_handing: bool,
) -> bool {
    let halves = [
        might_halves(rules, attr_max, attrs.might, two_handing),
        u32::from(attrs.finesse) * 2,
        u32::from(attrs.insight) * 2,
        u32::from(attrs.devotion) * 2,
    ];
    halves
        .iter()
        .zip(&w.requirements)
        .all(|(&have, &need)| have >= u32::from(need) * 2)
}

/// `AR = base(U) * (1 + sum over a of k_a(U) * curve(attr_a))`, with the unwieldy
/// penalty and two-handing. See the module docs for the scales.
pub fn attack_rating(
    rules: &DamageRules,
    attr_max: u16,
    w: &WeaponStats,
    attrs: &Attrs,
    upgrade: u8,
    two_handing: bool,
) -> AttackRating {
    let u = u128::from(upgrade.min(rules.max_upgrade));
    // base(U) in thousandths: base_damage * (1000 + 60 U). The elemental base scales the
    // same way, so a weapon's element rises with its upgrades like its physical part.
    let level = 1000 + u * u128::from(rules.base_per_upgrade_permille);
    let base = u128::from(w.base_damage) * level;
    let elem_base = w.elem.map(|(_, b)| u128::from(b) * level);
    let unwieldy = !meets_requirements(rules, attr_max, w, attrs, two_handing);
    // Requirements: scaling is zero and base damage is multiplied by 0.6. Otherwise each
    // base gains base * (sum of k_a(U) * curve(attr_a)), in 1e-11: k is permille times
    // (100 + 3 U) percent (1e-5), and the curve is in 1e-6.
    let (phys, elem) = if unwieldy {
        let cut = u128::from(rules.unwieldy_base_percent);
        (base * cut / 100, elem_base.map(|e| e * cut / 100))
    } else {
        let halves = [
            might_halves(rules, attr_max, attrs.might, two_handing),
            u32::from(attrs.finesse) * 2,
            u32::from(attrs.insight) * 2,
            u32::from(attrs.devotion) * 2,
        ];
        let mut sum = 0u128;
        for (h, s) in halves.iter().zip(&w.scaling_permille) {
            let h = (*h).min(u32::from(attr_max) * 2);
            let k = u128::from(*s) * (100 + u * u128::from(rules.scaling_per_upgrade_percent));
            sum += k * u128::from(rules.scaling_curve_ppm(h));
        }
        (
            base + base * sum / 100_000_000_000,
            elem_base.map(|e| e + e * sum / 100_000_000_000),
        )
    };
    let mut elem_milli = [0u64; 4];
    if let (Some((element, _)), Some(e)) = (w.elem, elem) {
        elem_milli[element_slot(element)] = u64::try_from(e).unwrap_or(u64::MAX);
    }
    AttackRating {
        phys_milli: u64::try_from(phys).unwrap_or(u64::MAX),
        elem_milli,
        element: w.elem.map(|(e, _)| e),
        unwieldy,
    }
}

/// The index of `e` in the Aether, Fire, Storm, Radiance arrays of `AttackRating`.
fn element_slot(e: Element) -> usize {
    match e {
        Element::Aether => 0,
        Element::Fire => 1,
        Element::Storm => 2,
        Element::Radiance => 3,
    }
}

/// One hitbox connecting, as the damage formula sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub ar: AttackRating,
    /// The hitbox's physical subtype.
    pub phys: DamageType,
    /// The hitbox's element, if any.
    pub element: Option<DamageType>,
    /// The hitbox's motion value (after charge), in thousandths.
    pub motion_milli: u32,
    /// The target is in the startup or active frames of its own attack.
    pub counter: bool,
    /// The attacker's active damage buffs, in percent (100 for none); capped here.
    pub buff_percent: u16,
}

/// What a hit deals before any guard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Damage {
    /// `max(1, round(sum of final_type))`.
    pub total: u32,
    /// The physical part in thousandths, and the elemental part (0 with no element).
    pub phys_milli: u64,
    pub elem_milli: u64,
}

impl Damage {
    /// More of the damage is elemental than physical: the guard uses its elemental
    /// absorption (the guard takes one damage kind per hit).
    pub fn mostly_elemental(&self) -> bool {
        self.elem_milli > self.phys_milli
    }
}

/// `raw = AR * motion_value * counter_mult * buff_mult`, in thousandths.
fn raw_milli(rules: &DamageRules, ar_milli: u64, hit: &Hit) -> u128 {
    let counter = if hit.counter {
        u128::from(rules.counter_percent)
    } else {
        100
    };
    let buff = u128::from(hit.buff_percent.min(rules.buff_cap_percent));
    u128::from(ar_milli) * u128::from(hit.motion_milli) * counter * buff / 10_000_000
}

/// One type's `final_type`, in thousandths:
/// `raw * raw / (raw + defense) * (1 - absorption) * weakness`.
fn final_milli(rules: &DamageRules, raw: u128, t: DamageType, def: &Defense) -> u64 {
    let flat = u128::from(def.flat_of(t)) * 1000;
    // Never zero for a nonzero raw: it favors big hits. A zero raw adds nothing.
    let after_def = if raw == 0 {
        0
    } else {
        raw * raw / (raw + flat)
    };
    let absorb =
        u128::from(def.absorb_percent[t as usize]).min(u128::from(rules.absorption_cap_percent));
    let weak = u128::from(def.weakness_percent[t as usize]);
    (after_def * (100 - absorb) * weak / 10_000) as u64
}

/// The damage `hit` deals to a target with `def`.
pub fn damage(rules: &DamageRules, hit: &Hit, def: &Defense) -> Damage {
    let phys = final_milli(
        rules,
        raw_milli(rules, hit.ar.phys_milli, hit),
        hit.phys,
        def,
    );
    let elem = match hit.element {
        Some(e) => {
            // Aether through Radiance are the last four types; a physical type here has
            // no elemental AR.
            let ar = (e as usize)
                .checked_sub(DamageType::Aether as usize)
                .map_or(0, |i| hit.ar.elem_milli[i]);
            final_milli(rules, raw_milli(rules, ar, hit), e, def)
        }
        None => 0,
    };
    let sum = phys + elem;
    Damage {
        total: (((sum + 500) / 1000) as u32).max(1),
        phys_milli: phys,
        elem_milli: elem,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> (DamageRules, u16) {
        let c = Content::load_embedded().expect("embedded content");
        (c.attributes.damage.clone(), c.attributes.max)
    }

    /// The Wayfarer's Blade: base 105, Might 0.6 and Finesse 0.6, requires 10 and 10.
    fn blade() -> WeaponStats {
        WeaponStats {
            base_damage: 105,
            elem: None,
            scaling_permille: [600, 600, 0, 0],
            requirements: [10, 10, 0, 0],
            unique: false,
        }
    }

    fn attrs(might: u16, finesse: u16, insight: u16) -> Attrs {
        Attrs {
            might,
            finesse,
            insight,
            ..Attrs::default()
        }
    }

    /// Hand-computed attack ratings, in thousandths. Each expected value is the spec's
    /// formula worked out by hand (see the comment on each row), truncated to a
    /// thousandth.
    #[test]
    fn attack_rating_matches_hand_computed_values() {
        let (r, max) = rules();
        // (name, weapon, attrs, upgrade, two-handing, AR in thousandths, unwieldy)
        let cases: Vec<(&str, WeaponStats, Attrs, u8, bool, u64, bool)> = vec![
            // curve(12) = 0.40 * 12 / 20 = 0.24; AR = 105 * (1 + 0.6 * 0.24 * 2) = 135.24.
            (
                "base +0",
                blade(),
                attrs(12, 12, 0),
                0,
                false,
                135_240,
                false,
            ),
            // +5: base 105 * 1.3 = 136.5, k = 0.6 * 1.15 = 0.69;
            // AR = 136.5 * (1 + 0.69 * 0.24 * 2) = 181.7088.
            (
                "upgrade +5",
                blade(),
                attrs(12, 12, 0),
                5,
                false,
                181_708,
                false,
            ),
            // Might 8 is under the requirement of 10: scaling 0, base * 0.6 = 63.
            (
                "unmet requirement",
                blade(),
                attrs(8, 12, 0),
                0,
                false,
                63_000,
                true,
            ),
            // The same Might 8 two-handed counts as 12, meeting the requirement, so the
            // result is case 1 again.
            (
                "two-handing meets the requirement",
                blade(),
                attrs(8, 12, 0),
                0,
                true,
                135_240,
                false,
            ),
            // Might 20 two-handed is 30: curve(30) = 0.40 + 0.35 * 10 / 20 = 0.575;
            // Finesse 10: curve = 0.20. AR = 105 * (1 + 0.6 * 0.575 + 0.6 * 0.2) = 153.825.
            (
                "two-handing scales",
                blade(),
                attrs(20, 10, 0),
                0,
                true,
                153_825,
                false,
            ),
            // Might 50 two-handed is 75, capped at 60: curve = 1.0.
            // AR = 105 * (1 + 0.6 * 1.0 + 0.6 * 0.2) = 180.6.
            (
                "two-handed Might caps at 60",
                blade(),
                attrs(50, 10, 0),
                0,
                true,
                180_600,
                false,
            ),
            // Third segment: curve(50) = 0.75 + 0.25 * 10 / 20 = 0.875, curve(40) = 0.75.
            // AR = 105 * (1 + 0.6 * 0.875 + 0.6 * 0.75) = 207.375.
            (
                "past the second breakpoint",
                blade(),
                attrs(50, 40, 0),
                0,
                false,
                207_375,
                false,
            ),
            // Everything maxed, +10: base 105 * 1.6 = 168, k = 0.6 * 1.3 = 0.78;
            // AR = 168 * (1 + 0.78 + 0.78) = 430.08.
            (
                "maxed",
                blade(),
                attrs(60, 60, 0),
                10,
                false,
                430_080,
                false,
            ),
            // Insight weapon: base 80, scaling 1.0, requires Insight 15; Insight 25, +3:
            // base 94.4, k = 1.09, curve(25) = 0.4875; AR = 94.4 * 1.531375 = 144.5618.
            (
                "insight scaling",
                WeaponStats {
                    base_damage: 80,
                    elem: None,
                    scaling_permille: [0, 0, 1000, 0],
                    requirements: [0, 0, 15, 0],
                    unique: false,
                },
                attrs(0, 0, 25),
                3,
                false,
                144_561,
                false,
            ),
        ];
        assert!(cases.len() >= 9);
        for (name, w, a, upgrade, two, want, unwieldy) in cases {
            let got = attack_rating(&r, max, &w, &a, upgrade, two);
            assert_eq!(got.phys_milli, want, "{name}");
            assert_eq!(got.unwieldy, unwieldy, "{name}");
        }
    }

    fn hit(ar_phys: u64, mv: u32) -> Hit {
        Hit {
            ar: AttackRating {
                phys_milli: ar_phys * 1000,
                ..AttackRating::default()
            },
            phys: DamageType::Neutral,
            element: None,
            motion_milli: mv,
            counter: false,
            buff_percent: 100,
        }
    }

    fn flat(n: u32) -> Defense {
        Defense::flat(n)
    }

    /// Hand-computed damage. `raw * raw / (raw + defense)`, then absorption and
    /// weakness, rounded to a whole point.
    #[test]
    fn damage_matches_hand_computed_values() {
        let (r, _) = rules();
        let mut blunt_weak = flat(100);
        blunt_weak.weakness_percent[DamageType::Blunt as usize] = 150;
        let mut absorbing = flat(100);
        absorbing.absorb_percent[DamageType::Neutral as usize] = 30;
        let mut over_cap = flat(100);
        over_cap.absorb_percent[DamageType::Neutral as usize] = 90;
        let mut fire_weak = flat(100);
        fire_weak.weakness_percent[DamageType::Fire as usize] = 120;

        let mut counter = hit(100, 1000);
        counter.counter = true;
        let mut buffed = hit(100, 1000);
        buffed.buff_percent = 200;
        let mut blunt = hit(100, 1000);
        blunt.phys = DamageType::Blunt;
        let mut mixed = hit(100, 1000);
        mixed.element = Some(DamageType::Fire);
        mixed.ar.elem_milli[1] = 60_000; // Fire
        let unwieldy = Hit {
            ar: AttackRating {
                phys_milli: 63_000,
                unwieldy: true,
                ..AttackRating::default()
            },
            ..hit(0, 1000)
        };
        let two_handed = Hit {
            ar: AttackRating {
                phys_milli: 135_240,
                ..AttackRating::default()
            },
            ..hit(0, 1000)
        };

        // (name, hit, defense, damage)
        let cases: Vec<(&str, Hit, Defense, u32)> = vec![
            // 100^2 / (100 + 0) = 100.
            ("no defense", hit(100, 1000), flat(0), 100),
            // 100^2 / 200 = 50.
            ("flat defense", hit(100, 1000), flat(100), 50),
            // Heavy, motion 1.3: raw 130, 130^2 / 230 = 73.48.
            ("heavy motion value", hit(100, 1300), flat(100), 73),
            // Counter hit: raw 115, 115^2 / 215 = 61.51.
            ("counter hit", counter, flat(100), 62),
            // 50 * (1 - 0.30) = 35.
            ("absorption 30", hit(100, 1000), absorbing, 35),
            // 90 percent is capped at 70: 50 * 0.3 = 15.
            ("absorption cap", hit(100, 1000), over_cap, 15),
            // Blunt against a Blunt weakness of 1.5: 50 * 1.5 = 75.
            ("weakness", blunt, blunt_weak, 75),
            // Buff 2.0 is capped at 1.5: raw 150, 150^2 / 250 = 90.
            ("buff cap", buffed, flat(100), 90),
            // Raw 1 against 1000 defense is 0.001, which rounds to 0: the minimum is 1.
            ("minimum 1", hit(1, 1000), flat(1000), 1),
            // Physical 100^2 / 200 = 50, plus Fire 60^2 / 160 * 1.2 = 27: 77.
            ("physical and fire", mixed, fire_weak, 77),
            // Unmet requirement: AR 63, 63^2 / 163 = 24.35.
            ("unwieldy hit", unwieldy, flat(100), 24),
            // Two-handed AR 135.24 (see the AR table): 135.24^2 / 235.24 = 77.75.
            ("two-handed hit", two_handed, flat(100), 78),
        ];
        assert!(cases.len() >= 10);
        for (name, h, d, want) in cases {
            assert_eq!(damage(&r, &h, &d).total, want, "{name}");
        }
    }

    #[test]
    fn a_mostly_elemental_hit_is_reported_as_such() {
        let (r, _) = rules();
        let mut h = hit(10, 1000);
        h.element = Some(DamageType::Storm);
        h.ar.elem_milli[2] = 100_000;
        let d = damage(&r, &h, &flat(0));
        assert!(d.mostly_elemental(), "{d:?}");
        assert_eq!(d.total, 110);
    }

    #[test]
    fn the_scaling_curve_is_piecewise_linear_through_the_spec_points() {
        let (r, _) = rules();
        // 0.40 at 20, 0.75 at 40, 1.0 at 60, in millionths, from half points.
        assert_eq!(r.scaling_curve_ppm(40), 400_000);
        assert_eq!(r.scaling_curve_ppm(80), 750_000);
        assert_eq!(r.scaling_curve_ppm(120), 1_000_000);
        // Half a point: Might 22.5 is 0.40 + 0.35 * 2.5 / 20 = 0.44375.
        assert_eq!(r.scaling_curve_ppm(45), 443_750);
    }

    #[test]
    fn the_table_reads_the_blade_from_content() {
        let c = Content::load_embedded().expect("embedded content");
        let t = DamageTable::from_content(&c).unwrap();
        let num = c.registry.num("wpn_wayfarer_blade").unwrap();
        // WP-089 lowered the content's requirements to Might 7 and Finesse 8.
        let content_blade = WeaponStats {
            requirements: [7, 8, 0, 0],
            ..blade()
        };
        assert_eq!(t.weapon(num), Some(&content_blade));
        let ar = t.attack_rating(num, &attrs(12, 12, 0), 0, false).unwrap();
        assert_eq!(ar.phys_milli, 135_240);
    }
}
