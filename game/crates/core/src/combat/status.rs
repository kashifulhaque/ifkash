//! Status effects and buffs (docs/game/03-combat.md, "Status effects" and "Buffs").
//!
//! # Buildup
//!
//! Every actor has one meter per status ([`StatusState::meter_milli`]). A landed hit adds
//! its hitbox's buildup for each status it lists, plus the buildup its attacker's weapon
//! buff adds. The amount is `value * (1 + bonus)`: a backstab adds
//! `backstab_bonus_percent` (20), a riposte `crit_bonus_percent` (30, the spec's
//! `crit_or_stealth` 0.3). A blocked hit adds `guarded_percent` of that (0: the spec is
//! silent, so a block stops buildup); a guard-breaking hit adds all of it. Meters count
//! thousandths of a point so the 1.5 a tick decay stays exact.
//!
//! When a meter reaches the actor's resistance the status **procs**: the meter resets to
//! 0, the status can't build up for `lockout_ticks` (60), and the effect starts. A boss's
//! resistance is `resistance * (1 + 0.25 * procs)`, counting every proc this fight
//! ([`StatusState::threshold_milli`]). A resistance of 0 means the actor has no meter (it
//! can't build up), and an actor can be immune to a status outright.
//!
//! A meter that hasn't built up for `decay_delay_ticks` (60) loses `decay_milli_per_tick`
//! (1.5) a tick.
//!
//! # Effects
//!
//! Burst damage (Rupture, Rime, Delirium) is plain HP loss that ignores defense and the
//! guard. Poison and Blight take HP once a second for their duration. Rime's debuff
//! (+15 percent damage taken, half stamina regeneration) and a sleeper's 1.2x hit are
//! applied where damage is taken ([`incoming_damage`]) and stamina regenerates
//! (`systems::status_regen`). An asleep enemy is held locked through the same
//! `guard.stagger` lockout a stagger uses, refreshed every tick. [`StatusState::active`]
//! holds each status's remaining effect ticks.
//!
//! # Buffs
//!
//! An actor holds at most one weapon buff and one body buff; applying another of the same
//! kind replaces it ([`apply_buff`]). The product of their `damage_percent` is written to
//! `Stats::buff_percent`, which the damage pipeline caps at `damage.buff_cap_percent`.
//! A weapon buff's `status` list is added to every hit its holder lands.
//!
//! # Hooks for later work
//!
//! [`cure`], [`cure_all`], [`apply_buff`] and [`add_buildup`] are the entry points for
//! consumables, spells, and traps (WP-055 to WP-057); none of those is wired to an item yet.
//! Charms add to the resistances (they are plain edits of `StatusState::resist`).

use lf_content::moves::{StatusBuildup, StatusKind};
use lf_content::status::{BuffDef, BuffSlot, StatusRules};
use lf_content::{Attributes, Content, NumId};
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::actor::stats::Attrs;
use crate::arena::Id;
use crate::combat::projectile::PROJECTILE_HITBOX;
use crate::events::{Event, HitResult, StatusProc};
use crate::world::World;

/// Number of statuses.
pub const COUNT: usize = 6;

/// Every status, in the order of [`index`].
pub const KINDS: [StatusKind; COUNT] = [
    StatusKind::Poison,
    StatusKind::Blight,
    StatusKind::Drowse,
    StatusKind::Delirium,
    StatusKind::Rupture,
    StatusKind::Rime,
];

/// A status's index into the per-status arrays, and its bit in [`StatusState::aura_mask`].
pub fn index(kind: StatusKind) -> usize {
    match kind {
        StatusKind::Poison => 0,
        StatusKind::Blight => 1,
        StatusKind::Drowse => 2,
        StatusKind::Delirium => 3,
        StatusKind::Rupture => 4,
        StatusKind::Rime => 5,
    }
}

/// A buff that is running: its registry number and the ticks it has left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveBuff {
    pub num: NumId,
    pub ticks: u32,
}

/// Per-actor status state. Simulation state: part of the state hash.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusState {
    /// The meter each status needs to fill, by [`index`]. 0: no meter.
    pub resist: [u16; COUNT],
    /// Bit `1 << index` set: immune to that status.
    pub immune: u8,
    /// A boss: resistances grow with every proc.
    pub boss: bool,
    /// Delirium staggers this enemy (the player is always affected).
    pub can_go_mad: bool,
    /// Buildup so far, in thousandths of a point.
    pub meter_milli: [u32; COUNT],
    /// Ticks since each meter last built up.
    pub quiet: [u16; COUNT],
    /// Ticks left in which a status can't build up (after a proc).
    pub lockout: [u16; COUNT],
    /// Procs this fight. Only a boss's resistance reads it.
    pub procs: [u8; COUNT],
    /// Ticks left of each status's effect: Poison, Blight, Rime, and Drowse's sleep.
    pub active: [u16; COUNT],
    pub weapon_buff: Option<ActiveBuff>,
    pub body_buff: Option<ActiveBuff>,
}

impl StatusState {
    /// The player's state: resistances from the attributes that raise them (Vitality for
    /// Immunity, Endurance for Robustness, Focus for Composure; `attributes.ron`).
    pub fn for_player(attrs: &Attrs, rules: &Attributes) -> Self {
        let r = |attr| (rules.resistance_tenths(attr) / 10).clamp(0, i32::from(u16::MAX)) as u16;
        Self::with_resist(r(attrs.vitality), r(attrs.endurance), r(attrs.focus))
    }

    /// An enemy's state from its data.
    pub fn for_enemy(
        resist: &lf_content::scenario::ResistData,
        boss: bool,
        immune: &[StatusKind],
        can_go_mad: bool,
    ) -> Self {
        let mut s = Self::with_resist(resist.immunity, resist.robustness, resist.composure);
        s.boss = boss;
        s.can_go_mad = can_go_mad;
        for &k in immune {
            s.immune |= 1 << index(k);
        }
        s
    }

    fn with_resist(immunity: u16, robustness: u16, composure: u16) -> Self {
        let mut s = Self::default();
        s.resist[index(StatusKind::Poison)] = immunity;
        s.resist[index(StatusKind::Blight)] = immunity;
        s.resist[index(StatusKind::Rupture)] = robustness;
        s.resist[index(StatusKind::Rime)] = robustness;
        s.resist[index(StatusKind::Drowse)] = composure;
        s.resist[index(StatusKind::Delirium)] = composure;
        s
    }

    pub fn is_immune(&self, kind: StatusKind) -> bool {
        self.immune & (1 << index(kind)) != 0
    }

    /// The meter `kind` must reach to proc, in thousandths: the resistance, grown by the
    /// boss rule (`resistance * (1 + 0.25 * procs)`).
    pub fn threshold_milli(&self, kind: StatusKind, rules: &StatusRules) -> u64 {
        let i = index(kind);
        let growth = if self.boss {
            u64::from(rules.boss_growth_percent) * u64::from(self.procs[i])
        } else {
            0
        };
        u64::from(self.resist[i]) * 10 * (100 + growth)
    }

    /// Ticks left of `kind`'s effect (0 when it isn't running).
    pub fn active_ticks(&self, kind: StatusKind) -> u16 {
        self.active[index(kind)]
    }

    /// Asleep from Drowse.
    pub fn asleep(&self) -> bool {
        self.active_ticks(StatusKind::Drowse) > 0
    }

    /// Chilled by Rime.
    pub fn chilled(&self) -> bool {
        self.active_ticks(StatusKind::Rime) > 0
    }

    /// The statuses whose effect is running, as a bit mask (bit `index(kind)`): the
    /// snapshot's `status_mask`, which the renderer turns into auras. Only the lasting
    /// effects show: Poison, Blight, Drowse (asleep), and Rime. Rupture and Delirium are
    /// instant.
    pub fn aura_mask(&self) -> u32 {
        let mut mask = 0;
        for k in KINDS {
            if self.active_ticks(k) > 0 {
                mask |= 1 << index(k);
            }
        }
        mask
    }

    /// The buff in `slot`, if any.
    pub fn buff(&self, slot: BuffSlot) -> Option<ActiveBuff> {
        match slot {
            BuffSlot::Weapon => self.weapon_buff,
            BuffSlot::Body => self.body_buff,
        }
    }
}

/// The status and buff rules, from content. Not hashed: the content hash covers it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusTable {
    pub rules: StatusRules,
    /// Buff registry numbers with their index in `rules.buffs`, sorted by number.
    buffs: Vec<(NumId, usize)>,
}

impl StatusTable {
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let rules = content
            .status
            .clone()
            .ok_or_else(|| "the content has no status.ron".to_owned())?;
        let mut buffs = Vec::new();
        for (i, b) in rules.buffs.iter().enumerate() {
            let num = content
                .registry
                .num(&b.id)
                .ok_or_else(|| format!("buff \"{}\" is missing from registry.ron", b.id))?;
            buffs.push((num, i));
        }
        buffs.sort_by_key(|(n, _)| *n);
        Ok(Self { rules, buffs })
    }

    /// The buff with registry number `num`.
    pub fn buff(&self, num: NumId) -> Option<&BuffDef> {
        self.buffs
            .binary_search_by_key(&num, |(n, _)| *n)
            .ok()
            .map(|at| &self.rules.buffs[self.buffs[at].1])
    }

    /// The registry number of the buff with string ID `id`.
    pub fn buff_num(&self, id: &str) -> Option<NumId> {
        self.buffs
            .iter()
            .find(|&&(_, i)| self.rules.buffs[i].id == id)
            .map(|&(n, _)| n)
    }
}

/// Ends the actor's move and locks it for `ticks`, the way a stagger does.
fn lock(a: &mut Actor, ticks: u8) {
    a.moves.interrupt();
    a.moves.guard.drop_guard();
    a.moves.guard.counter = 0;
    a.moves.guard.stagger = a.moves.guard.stagger.max(ticks);
}

/// Takes `amount` HP (never below 0) and returns what was taken. It doesn't set `dead`:
/// the hit resolver does after its last landing, and [`tick`] does for ticking damage.
fn hurt(a: &mut Actor, amount: u32) -> u32 {
    let taken = amount.min(a.hp);
    a.hp -= taken;
    taken
}

/// `percent` of `max` (the actor's max FP or HP), rounded down.
fn share(max: f32, percent: u8) -> f32 {
    max * f32::from(percent) / 100.0
}

/// Starts `kind`'s effect on `a`. Returns the HP the burst took.
fn start_effect(table: &StatusTable, a: &mut Actor, is_player: bool, kind: StatusKind) -> u32 {
    let r = &table.rules;
    let i = index(kind);
    match kind {
        StatusKind::Rupture => hurt(a, r.rupture.damage(a.max_hp)),
        StatusKind::Rime => {
            a.status.active[i] = r.rime.duration_ticks;
            hurt(a, r.rime.burst.damage(a.max_hp))
        }
        StatusKind::Poison => {
            a.status.active[i] = r.poison.duration_ticks;
            0
        }
        StatusKind::Blight => {
            a.status.active[i] = r.blight.duration_ticks;
            0
        }
        StatusKind::Drowse if is_player => {
            lock(a, r.drowse.knockdown_ticks);
            a.fp = (a.fp - share(a.max_fp, r.drowse.fp_loss_percent)).max(0.0);
            0
        }
        StatusKind::Drowse => {
            a.status.active[i] = r.drowse.sleep_ticks;
            lock(a, 1);
            0
        }
        StatusKind::Delirium if is_player => {
            a.fp = (a.fp - share(a.max_fp, r.delirium.fp_loss_percent)).max(0.0);
            lock(a, r.delirium.player_stagger_ticks);
            let loss = (u64::from(a.max_hp) * u64::from(r.delirium.hp_loss_percent) / 100) as u32;
            hurt(a, loss)
        }
        StatusKind::Delirium => {
            if a.status.can_go_mad {
                lock(a, r.delirium.enemy_stagger_ticks);
            }
            0
        }
    }
}

/// Adds `milli` thousandths of buildup to `kind` on `a`. Returns true if the meter
/// filled: it is reset, the lockout starts, and the proc counts toward a boss's tolerance
/// (the effect itself is [`start_effect`]'s). Nothing happens to an immune actor, one with
/// no meter, one still locked out, or a dead one.
fn build_up(table: &StatusTable, a: &mut Actor, kind: StatusKind, milli: u64) -> bool {
    let i = index(kind);
    let s = &mut a.status;
    if a.dead || a.hp == 0 || s.is_immune(kind) || s.resist[i] == 0 || s.lockout[i] > 0 {
        return false;
    }
    s.quiet[i] = 0;
    let meter = u64::from(s.meter_milli[i]) + milli;
    if meter >= s.threshold_milli(kind, &table.rules) {
        s.meter_milli[i] = 0;
        s.lockout[i] = table.rules.lockout_ticks;
        s.procs[i] = s.procs[i].saturating_add(1);
        true
    } else {
        s.meter_milli[i] = u32::try_from(meter).unwrap_or(u32::MAX);
        false
    }
}

/// Adds buildup to `target`, and starts the effect (with a `StatusProc` event) if the
/// meter fills. `milli` is in thousandths of a point. Returns whether it procced. The
/// entry point for anything that isn't a hit (traps, consumables, spells).
pub fn add_buildup(world: &mut World, target: Id, kind: StatusKind, milli: u64) -> bool {
    let is_player = target == world.player;
    let Some(a) = world.actors.get_mut(target) else {
        return false;
    };
    if !build_up(&world.status, a, kind, milli) {
        return false;
    }
    let damage = start_effect(&world.status, a, is_player, kind);
    world.events.push(Event::StatusProc(StatusProc {
        target,
        status: kind,
        damage,
    }));
    // Thornheart and the like react to a proc near the wearer (WP-057).
    crate::items::gear::on_status_proc(world, target, kind);
    true
}

/// Applies the buildup of a hit that landed on `target` (tick system 7, the apply step,
/// for each landing): the hitbox's own and its attacker's weapon buff's. `result` says
/// whether the target blocked it.
pub fn apply_landing(
    world: &mut World,
    attacker: Id,
    target: Id,
    move_num: crate::combat::moves::MoveNum,
    hitbox: u8,
    projectile_status: Option<StatusBuildup>,
    result: HitResult,
) {
    let r = &world.status.rules;
    // A projectile landing (`PROJECTILE_HITBOX`) has no hitbox of its own: it carries its
    // definition's buildup, and the attacker's weapon buff applies as well.
    let mut list: Vec<StatusBuildup> = if hitbox == PROJECTILE_HITBOX {
        projectile_status.into_iter().collect()
    } else {
        world
            .moves
            .get(move_num)
            .and_then(|d| d.data.hitboxes.get(usize::from(hitbox)))
            .map(|h| h.status.clone())
            .unwrap_or_default()
    };
    if let Some(b) = world
        .actors
        .get(attacker)
        .and_then(|a| a.status.weapon_buff)
        .and_then(|b| world.status.buff(b.num))
    {
        list.extend(b.status.iter().copied());
    }
    if list.is_empty() {
        return;
    }
    let bonus = match world.moves.crits {
        _ if hitbox == PROJECTILE_HITBOX => 0,
        Some(c) if c.backstab == move_num => u64::from(r.backstab_bonus_percent),
        Some(c) if c.riposte == move_num => u64::from(r.crit_bonus_percent),
        _ => 0,
    };
    let percent = match result {
        HitResult::Guarded => u64::from(r.guarded_percent),
        HitResult::Connected | HitResult::GuardBroken => 100,
    };
    for b in list {
        // amount * 1000 thousandths * (100 + bonus) / 100 * percent / 100.
        let milli = u64::from(b.amount) * 10 * (100 + bonus) * percent / 100;
        add_buildup(world, target, b.kind, milli);
    }
}

/// `damage` as `t` takes it, scaled by Rime's debuff and a sleeper's weakness. A hit
/// wakes a sleeper.
pub fn incoming_damage(t: &mut Actor, table: &StatusTable, damage: u32) -> u32 {
    if t.status.active == [0; COUNT] {
        return damage;
    }
    let mut d = u64::from(damage);
    if t.status.chilled() {
        d = d * u64::from(table.rules.rime.damage_taken_percent) / 100;
    }
    if t.status.asleep() {
        d = d * u64::from(table.rules.drowse.asleep_hit_percent) / 100;
        t.status.active[index(StatusKind::Drowse)] = 0;
    }
    u32::try_from(d).unwrap_or(u32::MAX)
}

/// Clears `kind`'s meter and ends its effect (a cure item).
pub fn cure(a: &mut Actor, kind: StatusKind) {
    let i = index(kind);
    a.status.meter_milli[i] = 0;
    a.status.quiet[i] = 0;
    a.status.active[i] = 0;
}

/// Clears every meter and ends every effect: resting at a Lantern and respawning do.
pub fn cure_all(a: &mut Actor) {
    for k in KINDS {
        cure(a, k);
    }
}

/// Ends both buffs (a respawn does).
pub fn clear_buffs(a: &mut Actor, table: &StatusTable) {
    a.status.weapon_buff = None;
    a.status.body_buff = None;
    recompute_buff_percent(a, table);
}

/// Puts the buff `num` in its slot, replacing the buff there. Returns false if no such
/// buff exists.
pub fn apply_buff(a: &mut Actor, table: &StatusTable, num: NumId) -> bool {
    let Some(def) = table.buff(num) else {
        return false;
    };
    let active = Some(ActiveBuff {
        num,
        ticks: def.duration_ticks,
    });
    match def.slot {
        BuffSlot::Weapon => a.status.weapon_buff = active,
        BuffSlot::Body => a.status.body_buff = active,
    }
    recompute_buff_percent(a, table);
    true
}

/// `Stats::buff_percent`: the product of the running buffs' `damage_percent` (the damage
/// pipeline applies the cap).
pub fn recompute_buff_percent(a: &mut Actor, table: &StatusTable) {
    let mut percent = 100u64;
    for slot in [BuffSlot::Weapon, BuffSlot::Body] {
        if let Some(def) = a.status.buff(slot).and_then(|b| table.buff(b.num)) {
            percent = percent * u64::from(def.damage_percent) / 100;
        }
    }
    a.stats.buff_percent = u16::try_from(percent).unwrap_or(u16::MAX);
}

/// Advances every living actor's statuses by a tick (tick system 8): lockouts, decay,
/// ticking damage, effect timers, sleep, and buffs.
pub fn tick(world: &mut World) {
    let rules = &world.status.rules;
    let ids: Vec<Id> = world.actors.iter().map(|(id, _)| id).collect();
    for id in ids {
        let Some(a) = world.actors.get_mut(id) else {
            continue;
        };
        if a.dead {
            continue;
        }
        for i in 0..COUNT {
            let s = &mut a.status;
            s.lockout[i] = s.lockout[i].saturating_sub(1);
            if s.meter_milli[i] > 0 && s.quiet[i] >= rules.decay_delay_ticks {
                s.meter_milli[i] = s.meter_milli[i].saturating_sub(rules.decay_milli_per_tick);
            }
            s.quiet[i] = s.quiet[i].saturating_add(1);
        }
        // Poison and Blight: damage every `dot_interval_ticks` until the effect ends.
        for (kind, dot) in [
            (StatusKind::Poison, rules.poison),
            (StatusKind::Blight, rules.blight),
        ] {
            let i = index(kind);
            if a.status.active[i] > 0 {
                a.status.active[i] -= 1;
                if a.status.active[i] % rules.dot_interval_ticks == 0 {
                    hurt(a, dot.per_second.damage(a.max_hp));
                }
            }
        }
        let rime = index(StatusKind::Rime);
        a.status.active[rime] = a.status.active[rime].saturating_sub(1);
        // A sleeper stays locked: its stagger is held at 1 each tick, so it never gets
        // the free tick the next AI think would use.
        let drowse = index(StatusKind::Drowse);
        if a.status.active[drowse] > 0 {
            a.status.active[drowse] -= 1;
            if a.status.active[drowse] > 0 {
                a.moves.guard.stagger = a.moves.guard.stagger.max(1);
            }
        }
        let mut expired = false;
        for slot in [&mut a.status.weapon_buff, &mut a.status.body_buff] {
            if let Some(b) = slot {
                b.ticks = b.ticks.saturating_sub(1);
                if b.ticks == 0 {
                    *slot = None;
                    expired = true;
                }
            }
        }
        if expired {
            recompute_buff_percent(a, &world.status);
        }
        if a.hp == 0 {
            a.dead = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::tick::tick as world_tick;
    use lf_content::status::BuffDef;

    fn content() -> Content {
        Content::load_embedded().expect("embedded content")
    }

    /// The `duel_status` world: the player at the origin, `sparring` (enemy 0) 1.2 m
    /// ahead, and the boss `warden` (enemy 1). The sparring dummy's script is removed
    /// unless a test wants its Poison swings.
    fn duel(script: bool) -> World {
        let mut w = World::new(3, &content(), "duel_status").expect("duel_status exists");
        if !script {
            let id = w.state.enemies[0];
            w.actors.get_mut(id).unwrap().script = None;
        }
        w
    }

    fn sparring(w: &World) -> Id {
        w.state.enemies[0]
    }

    fn warden(w: &World) -> Id {
        w.state.enemies[1]
    }

    fn actor(w: &World, id: Id) -> &Actor {
        w.actors.get(id).unwrap()
    }

    fn actor_mut(w: &mut World, id: Id) -> &mut Actor {
        w.actors.get_mut(id).unwrap()
    }

    fn input(buttons: u16) -> TickInput {
        TickInput {
            frame: ActionFrame {
                move_x: 0,
                move_z: 0,
                buttons,
            },
            commands: Vec::new(),
        }
    }

    fn idle(w: &mut World, ticks: u32) {
        for _ in 0..ticks {
            world_tick(w, &input(0));
        }
    }

    fn hits(w: &mut World) -> Vec<crate::events::Hit> {
        w.events
            .drain()
            .into_iter()
            .filter_map(|e| match e {
                Event::Hit(h) => Some(h),
                _ => None,
            })
            .collect()
    }

    fn procs(w: &mut World) -> Vec<StatusProc> {
        w.events
            .drain()
            .into_iter()
            .filter_map(|e| match e {
                Event::StatusProc(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    fn buff(w: &mut World, id: Id, name: &str) {
        let num = w.status.buff_num(name).expect("buff exists");
        let table = w.status.clone();
        assert!(apply_buff(actor_mut(w, id), &table, num));
    }

    /// One swing of LIGHT from the origin: the player is put back first (a swing has
    /// root motion), and the tick loop runs until the move ends. Returns the hits.
    fn swing(w: &mut World) -> Vec<crate::events::Hit> {
        let p = w.player;
        let a = actor_mut(w, p);
        a.pos = [0.0, 0.0, 0.0];
        a.yaw = 0.0;
        a.stamina = a.max_stamina;
        world_tick(w, &input(buttons::LIGHT));
        let mut out = hits(w);
        // Long enough for the move to end, short of the 60 quiet ticks before decay.
        for _ in 0..40 {
            world_tick(w, &input(0));
            out.extend(hits(w));
        }
        out
    }

    fn meter(w: &World, id: Id, kind: StatusKind) -> u32 {
        actor(w, id).status.meter_milli[index(kind)]
    }

    /// A bare actor with resistance 110 to everything and 1000 HP, for the per-status
    /// timing tests. `buildup` calls go straight to `build_up`.
    fn bare() -> Actor {
        Actor {
            hp: 1000,
            max_hp: 1000,
            fp: 100.0,
            max_fp: 100.0,
            status: StatusState::with_resist(110, 110, 110),
            ..Actor::default()
        }
    }

    // --- Resistance ---

    #[test]
    fn the_players_resistances_come_from_their_attributes() {
        let w = duel(false);
        // Stub Vitality, Focus, and Endurance are 10: 83.0 + 9 points * 3.0 = 110.
        let s = &actor(&w, w.player).status;
        assert_eq!(s.resist, [110; COUNT]);
        assert!(!s.boss);
    }

    #[test]
    fn enemy_resistances_and_immunities_come_from_the_scenario() {
        let w = duel(false);
        let s = &actor(&w, sparring(&w)).status;
        assert_eq!(s.resist, [100; COUNT]);
        assert!(!s.boss && s.immune == 0);
        let b = &actor(&w, warden(&w)).status;
        assert!(b.boss && b.is_immune(StatusKind::Poison) && !b.is_immune(StatusKind::Blight));
    }

    // --- Proc timing, per status ---

    #[test]
    fn every_status_procs_on_the_hit_that_fills_its_meter() {
        let w = duel(false);
        let t = &w.status;
        // 55 a hit against 110: the second hit fills it exactly.
        for kind in KINDS {
            let mut a = bare();
            assert!(!build_up(t, &mut a, kind, 55_000), "{kind:?} first hit");
            assert_eq!(a.status.meter_milli[index(kind)], 55_000);
            assert!(build_up(t, &mut a, kind, 55_000), "{kind:?} second hit");
            assert_eq!(a.status.meter_milli[index(kind)], 0, "{kind:?} resets");
            assert_eq!(a.status.lockout[index(kind)], 60);
        }
        // 40 a hit against 110 takes three hits.
        let mut a = bare();
        let fired: Vec<bool> = (0..3)
            .map(|_| build_up(t, &mut a, StatusKind::Rupture, 40_000))
            .collect();
        assert_eq!(fired, [false, false, true]);
        // One hit short stays short: 109.999 doesn't fill 110.
        let mut a = bare();
        assert!(!build_up(t, &mut a, StatusKind::Rime, 109_999));
        assert!(build_up(t, &mut a, StatusKind::Rime, 1));
    }

    #[test]
    fn a_proc_locks_the_status_out_for_60_ticks() {
        let mut w = duel(false);
        let p = w.player;
        assert!(add_buildup(&mut w, p, StatusKind::Poison, 110_000));
        // Locked: buildup does nothing and the meter stays at 0.
        for _ in 0..59 {
            tick(&mut w);
            assert!(!add_buildup(&mut w, p, StatusKind::Poison, 110_000));
            assert_eq!(meter(&w, p, StatusKind::Poison), 0);
        }
        // 59 ticks have passed, plus this one: the lockout counts down to 0.
        tick(&mut w);
        assert_eq!(actor(&w, p).status.lockout[index(StatusKind::Poison)], 0);
        assert!(add_buildup(&mut w, p, StatusKind::Poison, 110_000));
        // Another status was never locked.
        assert!(add_buildup(&mut w, p, StatusKind::Blight, 110_000));
    }

    #[test]
    fn an_immune_actor_or_one_without_a_meter_builds_nothing() {
        let w = duel(false);
        let mut a = bare();
        a.status.immune = 1 << index(StatusKind::Rime);
        assert!(!build_up(&w.status, &mut a, StatusKind::Rime, 1_000_000));
        assert_eq!(a.status.meter_milli[index(StatusKind::Rime)], 0);
        let mut none = bare();
        none.status.resist = [0; COUNT];
        assert!(!build_up(
            &w.status,
            &mut none,
            StatusKind::Rupture,
            1_000_000
        ));
        let mut dead = bare();
        dead.dead = true;
        assert!(!build_up(
            &w.status,
            &mut dead,
            StatusKind::Rupture,
            1_000_000
        ));
    }

    // --- Decay ---

    #[test]
    fn a_meter_decays_1_5_a_tick_starting_60_ticks_after_the_last_buildup() {
        let mut w = duel(false);
        let p = w.player;
        // Built up on a tick whose status step follows (hit resolution is system 7).
        add_buildup(&mut w, p, StatusKind::Rupture, 50_000);
        for _ in 0..60 {
            tick(&mut w);
            assert_eq!(meter(&w, p, StatusKind::Rupture), 50_000);
        }
        tick(&mut w);
        assert_eq!(meter(&w, p, StatusKind::Rupture), 48_500);
        tick(&mut w);
        assert_eq!(meter(&w, p, StatusKind::Rupture), 47_000);
        // A new hit restarts the wait.
        add_buildup(&mut w, p, StatusKind::Rupture, 1_000);
        assert_eq!(meter(&w, p, StatusKind::Rupture), 48_000);
        for _ in 0..60 {
            tick(&mut w);
        }
        assert_eq!(meter(&w, p, StatusKind::Rupture), 48_000);
        // And it never goes below 0.
        for _ in 0..40 {
            tick(&mut w);
        }
        assert_eq!(meter(&w, p, StatusKind::Rupture), 0);
    }

    // --- Boss tolerance ---

    #[test]
    fn a_boss_needs_25_percent_more_after_each_proc() {
        let mut w = duel(false);
        let b = warden(&w);
        let rules = w.status.rules.clone();
        let need = |w: &World| {
            actor(w, b)
                .status
                .threshold_milli(StatusKind::Rupture, &rules)
        };
        assert_eq!(need(&w), 100_000);
        assert!(add_buildup(&mut w, b, StatusKind::Rupture, 100_000));
        assert_eq!(need(&w), 125_000);
        // The lockout (60 ticks) has to run out before the second.
        for _ in 0..60 {
            tick(&mut w);
        }
        assert!(!add_buildup(&mut w, b, StatusKind::Rupture, 124_999));
        assert!(add_buildup(&mut w, b, StatusKind::Rupture, 1));
        // After two procs the resistance is 100 * (1 + 0.25 * 2) = 150.
        assert_eq!(need(&w), 150_000);
        // Other statuses keep their own count.
        assert_eq!(
            actor(&w, b)
                .status
                .threshold_milli(StatusKind::Rime, &rules),
            100_000
        );
        // A non-boss never grows.
        let s = sparring(&w);
        assert!(add_buildup(&mut w, s, StatusKind::Rupture, 100_000));
        assert_eq!(
            actor(&w, s)
                .status
                .threshold_milli(StatusKind::Rupture, &rules),
            100_000
        );
    }

    #[test]
    fn a_boss_is_immune_to_what_its_data_lists() {
        let mut w = duel(false);
        let b = warden(&w);
        assert!(!add_buildup(&mut w, b, StatusKind::Poison, 1_000_000));
        assert_eq!(meter(&w, b, StatusKind::Poison), 0);
        assert!(add_buildup(&mut w, b, StatusKind::Blight, 100_000));
    }

    // --- Effects ---

    #[test]
    fn rupture_bursts_for_12_percent_of_max_hp_plus_80() {
        let mut w = duel(false);
        let p = w.player;
        let max = actor(&w, p).max_hp;
        add_buildup(&mut w, p, StatusKind::Rupture, 110_000);
        let ev = procs(&mut w);
        let want = max * 120 / 1000 + 80;
        assert_eq!(actor(&w, p).hp, max - want);
        assert_eq!(ev.len(), 1);
        assert_eq!(
            (ev[0].status, ev[0].damage, ev[0].code()),
            (StatusKind::Rupture, want, 4)
        );
        // Instant: no lasting effect.
        assert_eq!(actor(&w, p).status.aura_mask(), 0);
    }

    #[test]
    fn a_burst_that_empties_hp_leaves_the_death_to_the_caller() {
        let mut w = duel(false);
        let p = w.player;
        actor_mut(&mut w, p).hp = 5;
        add_buildup(&mut w, p, StatusKind::Rupture, 110_000);
        assert_eq!(actor(&w, p).hp, 0);
        // Ticking damage sets `dead`; a burst inside hit resolution doesn't.
        assert!(!actor(&w, p).dead);
        tick(&mut w);
        assert!(actor(&w, p).dead);
    }

    #[test]
    fn rime_bursts_then_chills_for_600_ticks() {
        let mut w = duel(false);
        let p = w.player;
        let max = actor(&w, p).max_hp;
        add_buildup(&mut w, p, StatusKind::Rime, 110_000);
        assert_eq!(actor(&w, p).hp, max - (max * 80 / 1000 + 50));
        assert!(actor(&w, p).status.chilled());
        assert_eq!(actor(&w, p).status.aura_mask(), 1 << 5);
        // +15 percent damage taken.
        let table = w.status.clone();
        assert_eq!(incoming_damage(actor_mut(&mut w, p), &table, 100), 115);
        // The chill lasts 600 ticks.
        for _ in 0..599 {
            tick(&mut w);
        }
        assert!(actor(&w, p).status.chilled());
        tick(&mut w);
        assert!(!actor(&w, p).status.chilled());
        assert_eq!(incoming_damage(actor_mut(&mut w, p), &table, 100), 100);
    }

    #[test]
    fn rime_halves_stamina_regeneration() {
        let regen = |chilled: bool| {
            let mut w = duel(false);
            let p = w.player;
            if chilled {
                add_buildup(&mut w, p, StatusKind::Rime, 110_000);
            }
            let a = actor_mut(&mut w, p);
            a.stamina = 20.0;
            a.motor.stamina_idle = 15;
            world_tick(&mut w, &input(0));
            actor(&w, p).stamina - 20.0
        };
        assert_eq!(regen(false), 1.5);
        assert_eq!(regen(true), 0.75);
    }

    #[test]
    fn poison_takes_hp_every_second_for_1800_ticks() {
        let mut w = duel(false);
        let s = sparring(&w);
        let max = actor(&w, s).max_hp; // 9000: 0.2% is 18, plus 7
        let per = max * 2 / 1000 + 7;
        add_buildup(&mut w, s, StatusKind::Poison, 100_000);
        assert_eq!(actor(&w, s).status.aura_mask(), 1);
        for _ in 0..29 {
            tick(&mut w);
        }
        assert_eq!(actor(&w, s).hp, max);
        tick(&mut w);
        assert_eq!(actor(&w, s).hp, max - per);
        for _ in 30..1800 {
            tick(&mut w);
        }
        // 60 applications, then it ends.
        assert_eq!(actor(&w, s).hp, max - 60 * per);
        assert_eq!(actor(&w, s).status.aura_mask(), 0);
        tick(&mut w);
        assert_eq!(actor(&w, s).hp, max - 60 * per);
    }

    #[test]
    fn blight_takes_hp_every_second_for_900_ticks() {
        let mut w = duel(false);
        let s = sparring(&w);
        let max = actor(&w, s).max_hp;
        let per = max * 6 / 1000 + 14;
        add_buildup(&mut w, s, StatusKind::Blight, 100_000);
        assert_eq!(actor(&w, s).status.aura_mask(), 1 << 1);
        for _ in 0..900 {
            tick(&mut w);
        }
        assert_eq!(actor(&w, s).hp, max - 30 * per);
        assert_eq!(actor(&w, s).status.aura_mask(), 0);
    }

    #[test]
    fn poison_can_kill() {
        let mut w = duel(false);
        let s = sparring(&w);
        actor_mut(&mut w, s).hp = 10;
        add_buildup(&mut w, s, StatusKind::Poison, 100_000);
        for _ in 0..30 {
            tick(&mut w);
        }
        assert_eq!(actor(&w, s).hp, 0);
        assert!(actor(&w, s).dead);
    }

    #[test]
    fn drowse_puts_an_enemy_to_sleep_for_300_ticks() {
        let mut w = duel(true);
        let s = sparring(&w);
        add_buildup(&mut w, s, StatusKind::Drowse, 100_000);
        assert!(actor(&w, s).status.asleep());
        assert_eq!(actor(&w, s).status.aura_mask(), 1 << 2);
        // Its scripted swing (due on tick 40, 130, 220) never starts while it sleeps.
        for _ in 0..299 {
            world_tick(&mut w, &input(0));
            assert!(hits(&mut w).is_empty());
            assert!(actor(&w, s).moves.current.is_none());
            assert!(actor(&w, s).status.asleep());
        }
        world_tick(&mut w, &input(0));
        assert!(!actor(&w, s).status.asleep());
        // Awake, the next due tick (310) starts it again.
        idle(&mut w, 30);
        assert!(w.tick > 310);
        assert_eq!(
            hits(&mut w).len(),
            1,
            "awake, it swings and hits the player"
        );
    }

    #[test]
    fn a_hit_wakes_a_sleeper_and_deals_1_2x() {
        let mut w = duel(false);
        let s = sparring(&w);
        let p = w.player;
        buff(&mut w, p, "buf_bloodbloom_oil");
        // The baseline swing, then the same swing on a sleeper (a fresh world each).
        let awake = swing(&mut w);
        assert_eq!(awake.len(), 1);
        let mut w2 = duel(false);
        let s2 = sparring(&w2);
        add_buildup(&mut w2, s2, StatusKind::Drowse, 100_000);
        let woken = swing(&mut w2);
        assert_eq!(woken.len(), 1);
        assert!(!actor(&w2, s).status.asleep(), "the hit woke it");
        // 1.2x on the damage before defense; with 500 flat defense the dealt damage is
        // strictly more but not exactly 1.2x, so compare against the pipeline.
        assert!(woken[0].damage > awake[0].damage);
    }

    #[test]
    fn drowse_knocks_the_player_down_and_costs_10_percent_fp() {
        let mut w = duel(false);
        let p = w.player;
        let max_fp = actor(&w, p).max_fp;
        add_buildup(&mut w, p, StatusKind::Drowse, 110_000);
        let a = actor(&w, p);
        assert_eq!(a.moves.guard.stagger, 60);
        assert!(!a.status.asleep());
        assert_eq!(a.fp, max_fp - max_fp * 10.0 / 100.0);
    }

    #[test]
    fn delirium_costs_the_player_fp_and_hp_and_staggers_them() {
        let mut w = duel(false);
        let p = w.player;
        let (max_fp, max_hp) = (actor(&w, p).max_fp, actor(&w, p).max_hp);
        add_buildup(&mut w, p, StatusKind::Delirium, 110_000);
        let a = actor(&w, p);
        assert_eq!(a.fp, max_fp - max_fp * 30.0 / 100.0);
        assert_eq!(a.hp, max_hp - max_hp * 10 / 100);
        assert_eq!(a.moves.guard.stagger, 40);
    }

    #[test]
    fn delirium_staggers_only_enemies_that_can_go_mad() {
        let mut w = duel(false);
        let s = sparring(&w);
        add_buildup(&mut w, s, StatusKind::Delirium, 100_000);
        assert_eq!(actor(&w, s).moves.guard.stagger, 0);
        let b = warden(&w);
        actor_mut(&mut w, b).status.can_go_mad = true;
        add_buildup(&mut w, b, StatusKind::Delirium, 100_000);
        assert_eq!(actor(&w, b).moves.guard.stagger, 60);
        // No HP loss for an enemy.
        assert_eq!(actor(&w, b).hp, actor(&w, b).max_hp);
    }

    // --- Cures ---

    #[test]
    fn a_cure_clears_one_status_and_cure_all_clears_every_one() {
        let mut w = duel(false);
        let p = w.player;
        add_buildup(&mut w, p, StatusKind::Poison, 110_000);
        add_buildup(&mut w, p, StatusKind::Blight, 110_000);
        add_buildup(&mut w, p, StatusKind::Rupture, 30_000);
        cure(actor_mut(&mut w, p), StatusKind::Poison);
        let a = actor(&w, p);
        assert_eq!(a.status.active_ticks(StatusKind::Poison), 0);
        assert!(a.status.active_ticks(StatusKind::Blight) > 0);
        cure_all(actor_mut(&mut w, p));
        let a = actor(&w, p);
        assert_eq!(a.status.aura_mask(), 0);
        assert_eq!(a.status.meter_milli, [0; COUNT]);
    }

    #[test]
    fn resting_cures_statuses_but_leaves_buffs() {
        let mut w = World::new(9, &content(), "test_field").unwrap();
        let p = w.player;
        add_buildup(&mut w, p, StatusKind::Poison, 110_000);
        add_buildup(&mut w, p, StatusKind::Rupture, 30_000);
        buff(&mut w, p, "buf_rallying_roar");
        crate::world_state::lantern::rest(&mut w, 0);
        let a = actor(&w, p);
        assert_eq!(a.status.aura_mask(), 0);
        assert_eq!(a.status.meter_milli, [0; COUNT]);
        assert!(a.status.body_buff.is_some());
    }

    #[test]
    fn respawning_cures_statuses_and_ends_buffs() {
        let mut w = World::new(9, &content(), "test_field").unwrap();
        let p = w.player;
        add_buildup(&mut w, p, StatusKind::Blight, 110_000);
        buff(&mut w, p, "buf_rallying_roar");
        assert_eq!(actor(&w, p).stats.buff_percent, 115);
        crate::world_state::death::respawn(&mut w);
        let a = actor(&w, p);
        assert_eq!(a.status.aura_mask(), 0);
        assert_eq!(a.status.body_buff, None);
        assert_eq!(a.stats.buff_percent, 100);
    }

    // --- Buffs ---

    /// Adds a test buff to the table: a registry number the content doesn't use.
    fn add_def(w: &mut World, num: NumId, slot: BuffSlot, percent: u16) {
        let i = w.status.rules.buffs.len();
        w.status.rules.buffs.push(BuffDef {
            id: format!("test_buff_{num}"),
            slot,
            damage_percent: percent,
            status: Vec::new(),
            duration_ticks: 100,
            vfx: "test".to_owned(),
            element: None,
        });
        w.status.buffs.push((num, i));
        w.status.buffs.sort_by_key(|(n, _)| *n);
    }

    fn apply(w: &mut World, num: NumId) {
        let p = w.player;
        let table = w.status.clone();
        assert!(apply_buff(actor_mut(w, p), &table, num));
    }

    #[test]
    fn buff_slots_hold_one_each_and_a_second_replaces_the_first() {
        let mut w = duel(false);
        add_def(&mut w, 9001, BuffSlot::Weapon, 120);
        add_def(&mut w, 9002, BuffSlot::Weapon, 130);
        add_def(&mut w, 9003, BuffSlot::Body, 110);
        let p = w.player;
        assert_eq!(actor(&w, p).stats.buff_percent, 100);
        apply(&mut w, 9001);
        assert_eq!(actor(&w, p).stats.buff_percent, 120);
        apply(&mut w, 9003);
        assert_eq!(actor(&w, p).stats.buff_percent, 132, "1.2 * 1.1");
        apply(&mut w, 9002);
        assert_eq!(actor(&w, p).status.weapon_buff.map(|b| b.num), Some(9002));
        assert_eq!(actor(&w, p).stats.buff_percent, 143, "1.3 * 1.1");
        assert_eq!(actor(&w, p).status.body_buff.map(|b| b.num), Some(9003));
        let table = w.status.clone();
        assert!(!apply_buff(actor_mut(&mut w, p), &table, 424_242));
    }

    #[test]
    fn a_buff_expires_after_its_duration() {
        let mut w = duel(false);
        add_def(&mut w, 9001, BuffSlot::Weapon, 120);
        apply(&mut w, 9001);
        let p = w.player;
        for _ in 0..99 {
            tick(&mut w);
        }
        assert_eq!(actor(&w, p).stats.buff_percent, 120);
        tick(&mut w);
        assert_eq!(actor(&w, p).stats.buff_percent, 100);
        assert_eq!(actor(&w, p).status.weapon_buff, None);
    }

    #[test]
    fn a_weapon_buff_raises_damage_and_the_pipeline_caps_the_product() {
        let dealt = |weapon: u16, body: u16| {
            let mut w = duel(false);
            add_def(&mut w, 9001, BuffSlot::Weapon, weapon);
            add_def(&mut w, 9002, BuffSlot::Body, body);
            apply(&mut w, 9001);
            apply(&mut w, 9002);
            // Padding is for the other tests; remove it so the damage is large.
            let s = sparring(&w);
            actor_mut(&mut w, s).stats.defense = crate::actor::stats::Defense::default();
            let hits = swing(&mut w);
            assert_eq!(hits.len(), 1);
            hits[0].damage
        };
        let plain = dealt(100, 100);
        let buffed = dealt(130, 100);
        assert!(buffed > plain, "{buffed} vs {plain}");
        // 200% x 115% = 230% is capped at 150% by `damage.buff_cap_percent`.
        assert_eq!(content().attributes.damage.buff_cap_percent, 150);
        assert_eq!(dealt(200, 115), dealt(150, 100));
    }

    // --- Through the hit resolver ---

    #[test]
    fn a_weapon_buff_adds_buildup_to_each_hit_and_procs_on_the_third() {
        let mut w = duel(false);
        let (p, s) = (w.player, sparring(&w));
        buff(&mut w, p, "buf_bloodbloom_oil");
        let max = actor(&w, s).max_hp;
        // Bloodbloom Oil: Rupture 40 a hit against 100.
        assert_eq!(swing(&mut w).len(), 1);
        assert_eq!(meter(&w, s, StatusKind::Rupture), 40_000);
        assert_eq!(swing(&mut w).len(), 1);
        assert_eq!(meter(&w, s, StatusKind::Rupture), 80_000);
        let before = actor(&w, s).hp;
        let third = swing(&mut w);
        assert_eq!(third.len(), 1);
        assert_eq!(meter(&w, s, StatusKind::Rupture), 0);
        let burst = max * 120 / 1000 + 80;
        assert_eq!(before - actor(&w, s).hp, third[0].damage + burst);
    }

    #[test]
    fn the_proc_event_comes_after_the_hit_that_caused_it() {
        let mut w = duel(false);
        let (p, s) = (w.player, sparring(&w));
        buff(&mut w, p, "buf_bloodbloom_oil");
        actor_mut(&mut w, s).status.meter_milli[index(StatusKind::Rupture)] = 70_000;
        let p_id = p;
        let a = actor_mut(&mut w, p_id);
        a.pos = [0.0; 3];
        world_tick(&mut w, &input(buttons::LIGHT));
        let mut kinds = Vec::new();
        for _ in 0..40 {
            world_tick(&mut w, &input(0));
            kinds.extend(w.events.drain().into_iter().filter_map(|e| match e {
                Event::Hit(_) => Some("hit"),
                Event::StatusProc(_) => Some("proc"),
                _ => None,
            }));
        }
        assert_eq!(kinds, ["hit", "proc"]);
    }

    #[test]
    fn two_venom_hits_proc_poison_on_the_player() {
        let mut w = duel(true);
        let p = w.player;
        // Swing every 50 ticks so the second hit comes before the meter starts to decay.
        let sp = sparring(&w);
        actor_mut(&mut w, sp).script.as_mut().unwrap().every_ticks = 50;
        let mut landed = 0;
        let mut proc_at = None;
        for _ in 0..200 {
            world_tick(&mut w, &input(0));
            landed += hits(&mut w).len();
            if proc_at.is_none() && actor(&w, p).status.active_ticks(StatusKind::Poison) > 0 {
                proc_at = Some(landed);
                break;
            }
            if landed == 1 {
                assert_eq!(meter(&w, p, StatusKind::Poison), 55_000);
            }
        }
        assert_eq!(proc_at, Some(2));
    }

    #[test]
    fn a_blocked_hit_builds_nothing_and_a_connected_one_does() {
        // Guarding from the start: the first swing (tick 40, hits near tick 53) is blocked.
        let mut w = duel(true);
        let p = w.player;
        let mut results = Vec::new();
        for _ in 0..70 {
            world_tick(&mut w, &input(buttons::GUARD));
            results.extend(hits(&mut w).into_iter().map(|h| h.result));
        }
        assert_eq!(results, [HitResult::Guarded]);
        assert_eq!(meter(&w, p, StatusKind::Poison), 0);
        // Not guarding: it connects and builds up.
        let mut w = duel(true);
        let p = w.player;
        let mut results = Vec::new();
        for _ in 0..70 {
            world_tick(&mut w, &input(0));
            results.extend(hits(&mut w).into_iter().map(|h| h.result));
        }
        assert_eq!(results, [HitResult::Connected]);
        assert_eq!(meter(&w, p, StatusKind::Poison), 55_000);
    }

    #[test]
    fn a_backstab_adds_20_percent_buildup() {
        let mut w = World::new(5, &content(), "duel_backstab").unwrap();
        let (p, s) = (w.player, w.state.enemies[0]);
        buff(&mut w, p, "buf_bloodbloom_oil");
        // Keep the sentry alive through the crit.
        let sentry = actor_mut(&mut w, s);
        (sentry.hp, sentry.max_hp) = (100_000, 100_000);
        let mut landed = 0;
        for t in 0..120 {
            world_tick(&mut w, &input(if t == 0 { buttons::LIGHT } else { 0 }));
            if hits(&mut w).len() == 1 {
                landed += 1;
                // 40 * 1.2 = 48 (an ordinary hit would be 40).
                assert_eq!(meter(&w, s, StatusKind::Rupture), 48_000);
            }
        }
        assert_eq!(landed, 1);
    }
}
