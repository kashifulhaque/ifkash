//! The validation framework: a [`Validate`] trait that collects every [`Issue`]
//! instead of stopping at the first.

use std::collections::BTreeSet;
use std::fmt;

use crate::Content;
use crate::attributes::{Attributes, Curve};
use crate::moves::{FrameWindow, Move};
use crate::weapons::Weapon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// One validation finding, naming where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub severity: Severity,
    /// The data file, relative to the data root (for example `moves/straight_sword.ron`).
    pub file: String,
    /// The item inside the file, such as a move or weapon ID.
    pub item: String,
    /// The field path inside the item, such as `hitboxes[0].frames`.
    pub field: String,
    pub message: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(
            f,
            "{sev}: {}: {}: {}: {}",
            self.file, self.item, self.field, self.message
        )
    }
}

/// Something that can check itself and report every problem it finds.
pub trait Validate {
    fn validate(&self) -> Vec<Issue>;
}

/// Collects issues for one item in one file.
struct Sink<'a> {
    out: &'a mut Vec<Issue>,
    file: &'a str,
    item: &'a str,
}

impl Sink<'_> {
    fn push(&mut self, severity: Severity, field: impl Into<String>, message: impl Into<String>) {
        self.out.push(Issue {
            severity,
            file: self.file.to_owned(),
            item: self.item.to_owned(),
            field: field.into(),
            message: message.into(),
        });
    }

    fn error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.push(Severity::Error, field, message);
    }

    fn warn(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.push(Severity::Warning, field, message);
    }

    /// Checks `frame` is within `0..=total`.
    fn frame(&mut self, field: &str, frame: u16, total: u16) {
        if frame > total {
            self.error(
                field,
                format!("frame {frame} is outside 0..={total} (total_frames)"),
            );
        }
    }

    fn window(&mut self, field: &str, from: u16, to: u16, total: u16) {
        if from > to {
            self.error(field, format!("window start {from} is after its end {to}"));
        }
        self.frame(&format!("{field}.from"), from, total);
        self.frame(&format!("{field}.to"), to, total);
    }

    fn range_f32(&mut self, field: &str, v: f32, lo: f32, hi: f32) {
        if !(lo..=hi).contains(&v) {
            self.error(field, format!("{v} is outside {lo}..={hi}"));
        }
    }
}

fn check_move(m: &Move, file: &str, content: &Content, out: &mut Vec<Issue>) {
    let mut s = Sink {
        out,
        file,
        item: &m.id,
    };
    let total = m.total_frames;
    if total == 0 {
        s.error("total_frames", "must be at least 1");
    }
    if m.stamina > 200 {
        s.error("stamina", format!("{} is outside 0..=200", m.stamina));
    }
    if m.fp > 200 {
        s.error("fp", format!("{} is outside 0..=200", m.fp));
    }
    s.range_f32("track_deg_per_tick", m.track_deg_per_tick, 0.0, 360.0);

    let mut last_key: Option<u16> = None;
    for (i, (frame, meters)) in m.root_motion.iter().enumerate() {
        s.frame(&format!("root_motion[{i}].frame"), *frame, total);
        s.range_f32(&format!("root_motion[{i}].meters"), *meters, -20.0, 20.0);
        if last_key.is_some_and(|prev| *frame < prev) {
            s.error(
                format!("root_motion[{i}].frame"),
                "keys must be in ascending frame order",
            );
        }
        last_key = Some(*frame);
    }
    for (name, w) in [("hyperarmor", &m.hyperarmor), ("iframes", &m.iframes)] {
        if let Some(FrameWindow { from, to }) = w {
            s.window(name, *from, *to, total);
        }
    }
    for (name, f) in [
        ("cancel_attack_from", m.cancel_attack_from),
        ("cancel_dodge_from", m.cancel_dodge_from),
        ("cancel_any_from", m.cancel_any_from),
    ] {
        if let Some(f) = f {
            s.frame(name, f, total);
        }
    }
    if let Some(next) = &m.next_chain
        && content.move_by_id(next).is_none()
    {
        s.error("next_chain", format!("unknown move \"{next}\""));
    }
    if let Some(c) = &m.charge {
        s.frame("charge.charge_from", c.charge_from, total);
        if c.max_frames == 0 {
            s.error("charge.max_frames", "must be at least 1");
        }
        s.range_f32("charge.motion_value_max", c.motion_value_max, 0.0, 10.0);
    }
    for (i, h) in m.hitboxes.iter().enumerate() {
        let (from, to) = h.frames;
        s.window(&format!("hitboxes[{i}].frames"), from, to, total);
        s.range_f32(
            &format!("hitboxes[{i}].motion_value"),
            h.motion_value,
            0.0,
            10.0,
        );
        if h.motion_value <= 0.0 {
            s.error(format!("hitboxes[{i}].motion_value"), "must be above 0");
        }
        if h.poise_damage > 1000 {
            s.error(
                format!("hitboxes[{i}].poise_damage"),
                format!("{} is outside 0..=1000", h.poise_damage),
            );
        }
        s.range_f32(&format!("hitboxes[{i}].knockback"), h.knockback, 0.0, 10.0);
        if h.hitstop > 30 {
            s.error(
                format!("hitboxes[{i}].hitstop"),
                format!("{} is outside 0..=30", h.hitstop),
            );
        }
        let radius = match h.shape {
            crate::moves::Shape::Capsule { radius, .. }
            | crate::moves::Shape::Sphere { radius, .. } => radius,
        };
        if !(0.0..=5.0).contains(&radius) || radius == 0.0 {
            s.error(
                format!("hitboxes[{i}].shape.radius"),
                format!("{radius} is outside (0, 5]"),
            );
        }
    }
    for (name, list) in [("sfx", &m.sfx), ("vfx", &m.vfx)] {
        for (i, (frame, _)) in list.iter().enumerate() {
            s.frame(&format!("{name}[{i}].frame"), *frame, total);
        }
    }
    if m.stub {
        s.warn("stub", "move is a stub placeholder");
    }
}

fn check_weapon(w: &Weapon, file: &str, content: &Content, out: &mut Vec<Issue>) {
    let mut s = Sink {
        out,
        file,
        item: &w.id,
    };
    if w.base_damage == 0 || w.base_damage > 1000 {
        s.error(
            "base_damage",
            format!("{} is outside 1..=1000", w.base_damage),
        );
    }
    let sc = &w.scaling;
    for (name, v) in [
        ("might", sc.might),
        ("finesse", sc.finesse),
        ("insight", sc.insight),
        ("devotion", sc.devotion),
    ] {
        s.range_f32(&format!("scaling.{name}"), v, 0.0, 2.5);
    }
    let rq = &w.requirements;
    for (name, v) in [
        ("might", rq.might),
        ("finesse", rq.finesse),
        ("insight", rq.insight),
        ("devotion", rq.devotion),
    ] {
        if v > content.attributes.max {
            s.error(
                format!("requirements.{name}"),
                format!("{v} is above the attribute cap {}", content.attributes.max),
            );
        }
    }
    for (field, id) in w.moveset.references() {
        if content.move_by_id(id).is_none() {
            s.error(field, format!("unknown move \"{id}\""));
        }
    }
    if w.stub {
        s.warn("stub", "weapon is a stub placeholder");
    }
}

fn check_curve(name: &str, c: &Curve, a: &Attributes, out: &mut Vec<Issue>) {
    let mut s = Sink {
        out,
        file: "attributes.ron",
        item: "attributes",
    };
    let mut prev = a.min;
    for (i, seg) in c.segments.iter().enumerate() {
        if seg.until <= prev || seg.until > a.max {
            s.error(
                format!("{name}.segments[{i}].until"),
                format!("{} must be above {prev} and at most {}", seg.until, a.max),
            );
        }
        prev = seg.until;
    }
    if c.segments.is_empty() {
        s.error(format!("{name}.segments"), "needs at least one segment");
    }
}

fn check_attributes(a: &Attributes, out: &mut Vec<Issue>) {
    {
        let mut s = Sink {
            out,
            file: "attributes.ron",
            item: "attributes",
        };
        if a.min == 0 || a.min >= a.max {
            s.error("min", format!("min {} must be in 1..max {}", a.min, a.max));
        }
        if a.start_min < a.min || a.start_max > a.max || a.start_min > a.start_max {
            s.error("start_min", "starting range must sit inside min..=max");
        }
        if a.attributes.len() != 7 {
            s.error(
                "attributes",
                format!("expected 7 attributes, found {}", a.attributes.len()),
            );
        }
        if a.spell_slots.every == 0 {
            s.error("spell_slots.every", "must be at least 1");
        }
    }
    for (name, c) in [
        ("max_hp", &a.max_hp),
        ("max_fp", &a.max_fp),
        ("max_stamina", &a.max_stamina),
        ("max_equip_load", &a.max_equip_load),
        ("resistance", &a.resistance),
    ] {
        check_curve(name, c, a, out);
    }
}

impl Validate for Content {
    fn validate(&self) -> Vec<Issue> {
        let mut out = Vec::new();
        check_attributes(&self.attributes, &mut out);
        let mut seen = BTreeSet::new();
        for (file, m) in self.all_moves() {
            if !seen.insert(m.id.as_str()) {
                Sink {
                    out: &mut out,
                    file,
                    item: &m.id,
                }
                .error("id", "duplicate move ID");
            }
            check_move(m, file, self, &mut out);
        }
        let mut seen = BTreeSet::new();
        for (file, w) in self.all_weapons() {
            if !seen.insert(w.id.as_str()) {
                Sink {
                    out: &mut out,
                    file,
                    item: &w.id,
                }
                .error("id", "duplicate weapon ID");
            }
            check_weapon(w, file, self, &mut out);
        }
        out
    }
}

/// Returns true when any issue is an error.
pub fn has_errors(issues: &[Issue]) -> bool {
    issues.iter().any(|i| i.severity == Severity::Error)
}
