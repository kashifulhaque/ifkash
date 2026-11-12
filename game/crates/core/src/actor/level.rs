//! Leveling: the Glimmer cost of each level, the attribute raises, and the respec rules.
//! See docs/game/06-progression-and-systems.md, "Leveling", and
//! docs/game/04-arsenal-and-builds.md, "Respec".
//!
//! The numbers are data (`level_cost` in `data/attributes.ron`). The Lantern menu
//! (`world_state::lantern_menu`) spends the Glimmer, applies these rules to the player, and
//! consumes the Rewrite Quill a respec costs.

use lf_content::Attributes;

use super::stats::Attrs;

/// Number of attributes the level-up and respec commands address, in `Attrs` order.
pub const ATTRIBUTE_COUNT: usize = 7;

/// Glimmer to go from character level `level` to the next:
/// `floor(base + per_level * L + quad_tenths * L^2 / 10)`, with the numbers from
/// `attributes.ron`. The division by 10 is the tenths of `quad_tenths`, so the integer
/// division is the exact floor of the quadratic term.
pub fn level_cost(rules: &Attributes, level: u32) -> u32 {
    let c = rules.level_cost;
    let l = u64::from(level);
    let cost =
        u64::from(c.base) + u64::from(c.per_level) * l + u64::from(c.quad_tenths) * l * l / 10;
    u32::try_from(cost).unwrap_or(u32::MAX)
}

/// The seven attribute levels as an array, in `Attrs` order.
pub fn values(a: &Attrs) -> [u16; ATTRIBUTE_COUNT] {
    [
        a.vitality,
        a.focus,
        a.endurance,
        a.might,
        a.finesse,
        a.insight,
        a.devotion,
    ]
}

/// The attribute levels from an array in `Attrs` order.
pub fn from_values(v: [u16; ATTRIBUTE_COUNT]) -> Attrs {
    Attrs {
        vitality: v[0],
        focus: v[1],
        endurance: v[2],
        might: v[3],
        finesse: v[4],
        insight: v[5],
        devotion: v[6],
    }
}

/// `attrs` with attribute `index` (in `Attrs` order) raised by one. `None` if the attribute
/// doesn't exist or is already at the maximum in `attributes.ron`.
pub fn raised(attrs: &Attrs, rules: &Attributes, index: u8) -> Option<Attrs> {
    let mut out = *attrs;
    let v = match index {
        0 => &mut out.vitality,
        1 => &mut out.focus,
        2 => &mut out.endurance,
        3 => &mut out.might,
        4 => &mut out.finesse,
        5 => &mut out.insight,
        6 => &mut out.devotion,
        _ => return None,
    };
    if *v >= rules.max {
        return None;
    }
    *v += 1;
    Some(out)
}

/// Whether `target` is a respec of `current` for a character whose starting class values
/// are `base` (docs/game/04-arsenal-and-builds.md, "Respec"). The points above the base
/// are redistributed, so `target` must:
/// - keep the same total (no points are gained or lost, so the level doesn't change);
/// - have no attribute below its class base or above the maximum;
/// - differ from `current` (a respec that changes nothing is refused).
pub fn respec_allowed(rules: &Attributes, base: &Attrs, current: &Attrs, target: &Attrs) -> bool {
    if target == current || target.total() != current.total() {
        return false;
    }
    values(target)
        .iter()
        .zip(values(base))
        .all(|(&t, b)| t >= b && t <= rules.max)
}

#[cfg(test)]
mod tests {
    use lf_content::Content;

    use super::*;

    fn rules() -> Attributes {
        Content::load_embedded()
            .expect("embedded content")
            .attributes
    }

    /// The Wayfarer's spread, the starting class in the spec's table (level 10).
    fn wayfarer() -> Attrs {
        from_values([12, 10, 12, 12, 12, 6, 6])
    }

    #[test]
    fn the_cost_curve_matches_the_leveling_table() {
        // The spec's table: Glimmer to the next level.
        let r = rules();
        for (level, cost) in [(10, 420), (20, 980), (30, 1780), (45, 3430), (60, 5620)] {
            assert_eq!(level_cost(&r, level), cost, "level {level}");
        }
    }

    #[test]
    fn the_cost_curve_matches_the_formula_at_the_levels_between() {
        // floor(100 + 20 L + 1.2 L^2), worked out by hand.
        let r = rules();
        assert_eq!(level_cost(&r, 11), 465); // 100 + 220 + 145.2
        assert_eq!(level_cost(&r, 12), 512); // 100 + 240 + 172.8
        assert_eq!(level_cost(&r, 13), 562); // 100 + 260 + 202.8
        assert_eq!(level_cost(&r, 1), 121); // 100 + 20 + 1.2
    }

    #[test]
    fn the_cumulative_cost_matches_the_leveling_table() {
        // The table's cumulative column: Glimmer from level 10 up to each level.
        let r = rules();
        let from_10 = |to: u32| (10..to).map(|l| level_cost(&r, l)).sum::<u32>();
        assert_eq!(from_10(10), 0);
        assert_eq!(from_10(20), 6_518);
        assert_eq!(from_10(30), 19_716);
        assert_eq!(from_10(45), 57_288);
        assert_eq!(from_10(60), 123_390);
    }

    #[test]
    fn the_cost_reads_the_data_not_a_constant() {
        let mut r = rules();
        r.level_cost.base = 0;
        assert_eq!(level_cost(&r, 10), 320);
    }

    #[test]
    fn derived_stats_match_the_spec_at_10_and_40() {
        // docs/game/03-combat.md, "Derived stats": the table's values at 10 and 40.
        let r = rules();
        let at = |vit: u16, foc: u16, end: u16| Attrs {
            vitality: vit,
            focus: foc,
            endurance: end,
            ..Attrs::default()
        };
        let ten = at(10, 10, 10).derived(&r);
        let forty = at(40, 40, 40).derived(&r);
        assert_eq!(ten.max_hp, 570);
        assert_eq!(forty.max_hp, 1370);
        assert_eq!(ten.max_fp, 85.0);
        assert_eq!(forty.max_fp, 195.0);
        assert_eq!(ten.max_stamina, 98.0);
        assert_eq!(forty.max_stamina, 138.0);
        assert_eq!(ten.max_equip_load_tenths, 508); // 50.8
        assert_eq!(forty.max_equip_load_tenths, 868); // 86.8
        assert_eq!(ten.spell_slots, 2);
        assert_eq!(forty.spell_slots, 5);
        assert_eq!(r.resistance_tenths(10), 1100); // 110
        assert_eq!(r.resistance_tenths(40), 2000); // 200
    }

    #[test]
    fn a_raise_adds_one_point_and_stops_at_the_maximum() {
        let r = rules();
        let a = wayfarer();
        let up = raised(&a, &r, 3).expect("might can rise");
        assert_eq!(up.might, 13);
        assert_eq!(up.total(), a.total() + 1);
        assert!(raised(&a, &r, 7).is_none(), "no eighth attribute");
        let mut at_max = a;
        at_max.might = r.max;
        assert!(raised(&at_max, &r, 3).is_none());
    }

    #[test]
    fn respec_redistributes_points_above_the_base_and_nothing_else() {
        let r = rules();
        let base = wayfarer();
        // Three points above the base, all in Might.
        let current = from_values([12, 10, 12, 15, 12, 6, 6]);
        // Moving them to Insight and Devotion keeps the total and the base floor.
        assert!(respec_allowed(
            &r,
            &base,
            &current,
            &from_values([12, 10, 12, 12, 12, 8, 7])
        ));
        // Back to the base plus the free points spent in one place.
        assert!(respec_allowed(
            &r,
            &base,
            &current,
            &from_values([12, 10, 12, 12, 12, 6, 9])
        ));
    }

    #[test]
    fn respec_refuses_a_changed_total_a_value_below_base_or_no_change() {
        let r = rules();
        let base = wayfarer();
        let current = from_values([12, 10, 12, 15, 12, 6, 6]);
        // The total went up by one.
        assert!(!respec_allowed(
            &r,
            &base,
            &current,
            &from_values([12, 10, 12, 16, 12, 6, 6])
        ));
        // Vitality below the class base.
        assert!(!respec_allowed(
            &r,
            &base,
            &current,
            &from_values([11, 10, 12, 15, 12, 7, 6])
        ));
        // Nothing changed.
        assert!(!respec_allowed(&r, &base, &current, &current));
    }
}
