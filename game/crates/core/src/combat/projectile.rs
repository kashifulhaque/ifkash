//! Projectiles, tick system 6 (docs/game/01-architecture.md, "Tick schedule" and
//! "Physics"; docs/game/03-combat.md, "Spells"). A move launches one from its
//! `projectiles` list. It is a sphere swept along its motion each tick, never a rigid
//! body: against the level with a Rapier shape cast, against actors with the same swept
//! overlap test as melee ([`hitbox::swept_overlap`]), so a fast projectile can't tunnel
//! through either.
//!
//! # Per-tick order
//!
//! [`run`] runs after the move state machines and locomotion, so moves and positions are
//! this tick's, and before hit resolution.
//!
//! 1. Advance every projectile in flight, in arena index order, reading the world without
//!    changing it. Each one:
//!    - speeds up under gravity, and turns toward its target if it homes;
//!    - moves by the average of its old and new velocity over one tick, which is exact
//!      under constant gravity;
//!    - finds the level geometry it touches on the way (a sphere cast) and the actors it
//!      touches (a swept overlap against each hurtbox), earliest first. Actors it has
//!      already struck, the shooter, the dead, and actors in i-frames don't count; an
//!      actor behind the level geometry doesn't either;
//!    - ends at the first actor past its `pierce` count, at the level, or when its lifetime
//!      runs out, whichever comes first.
//! 2. Apply the landings in that order through [`resolve::apply_landing`], the same
//!    guard, HP, and `Hit` event step melee uses, then emit the projectile events and
//!    update or remove each projectile. Actors left at 0 HP are marked `dead` after the
//!    last landing, so two shots in one tick both land.
//! 3. Spawn the projectiles that moves launch on the frame executed this tick. A new
//!    projectile sits at its spawn point until the next tick moves it.
//!
//! # Damage
//!
//! A projectile deals what its shooter's attack rating, buffs, and the projectile's motion
//! value make of it *when it spawns*, so changing weapons mid-flight doesn't change it.
//! The damage pipeline (`combat::damage`) is the melee one. A projectile isn't parried or
//! checked for teams yet; a later work package adds both.

use lf_content::moves::{HitTag, ProjectileSpawn};
use lf_content::{Content, NumId, ProjectileDef};
use serde::{Deserialize, Serialize};

use super::damage::{self, AttackRating};
use super::dodge;
use super::hitbox::{self, Capsule, Pose};
use super::moves::{MoveNum, flags};
use super::resolve::{self, Landing};
use crate::actor::Actor;
use crate::arena::Id;
use crate::events::{Event, ProjectileEvent, ProjectileEventKind};
use crate::items::gear;
use crate::math::dmath::{atan2, cos, lerp, sin, sqrt};
use crate::world::{DT, World};

/// The `hitbox` of a `Hit` event that a projectile caused: there is no hitbox index.
pub const PROJECTILE_HITBOX: u8 = u8::MAX;

/// Degrees to radians. Plain multiplication, no transcendental call.
const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;

/// The longest stretch of a projectile's move swept in one `swept_overlap` call, in
/// meters. A cost and coverage cap, not a gameplay number: with the 0.2 m substep spacing
/// of a thin projectile against a 0.35 m hurtbox, 6 m stays well inside the 64 substeps
/// `hitbox::MAX_SUBSTEPS` allows.
const SWEEP_CHUNK_M: f32 = 6.0;

/// Below this length a vector counts as zero. Float noise, not gameplay.
const EPS: f32 = 1.0e-6;

type V3 = [f32; 3];

/// A projectile in flight. Simulation state: part of the state hash (once any spawned).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    /// The definition's registry number.
    pub def: NumId,
    pub shooter: Id,
    /// The move that launched it, reported in the `Hit` event.
    pub move_num: MoveNum,
    pub pos: V3,
    pub vel: V3,
    /// Ticks flown.
    pub age: u16,
    /// Actors it has struck, sorted. It can't strike one twice.
    pub struck: Vec<Id>,
    /// The shooter's attack rating when it spawned, in thousandths: physical, then
    /// Aether, Fire, Storm, and Radiance.
    pub ar_phys_milli: u64,
    pub ar_elem_milli: [u64; 4],
    /// The shooter's damage buff in percent when it spawned.
    pub buff_percent: u16,
}

/// Every projectile definition by registry number. Built from content, so it isn't
/// hashed: the content hash covers it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectileTable {
    /// Sorted by registry number.
    pub defs: Vec<(NumId, ProjectileDef)>,
}

impl ProjectileTable {
    /// Resolves every definition's number. Fails on a move that launches a projectile the
    /// content doesn't define (the validator reports those too).
    pub fn from_content(content: &Content) -> Result<Self, String> {
        let mut defs = Vec::new();
        for p in &content.projectiles {
            let num = content
                .registry
                .num(&p.id)
                .ok_or_else(|| format!("projectile \"{}\" is missing from registry.ron", p.id))?;
            defs.push((num, p.clone()));
        }
        defs.sort_by_key(|d| d.0);
        for (_, m) in content.all_moves() {
            for s in &m.projectiles {
                if content.projectile_by_id(&s.projectile).is_none() {
                    return Err(format!(
                        "move \"{}\" launches projectile \"{}\", which is not defined",
                        m.id, s.projectile
                    ));
                }
            }
        }
        Ok(Self { defs })
    }

    pub fn get(&self, num: NumId) -> Option<&ProjectileDef> {
        self.defs
            .binary_search_by_key(&num, |d| d.0)
            .ok()
            .map(|i| &self.defs[i].1)
    }

    /// Finds a definition's number by string ID. A linear scan, for spawning and tests.
    pub fn num_of(&self, id: &str) -> Option<NumId> {
        self.defs.iter().find(|d| d.1.id == id).map(|d| d.0)
    }
}

/// A hurtbox in flight this tick, as melee sweeps it.
struct Body {
    id: Id,
    hurt0: Capsule,
    hurt1: Capsule,
}

impl Body {
    /// The middle of the capsule: where a homing projectile aims.
    fn center(&self) -> V3 {
        [
            0.5 * (self.hurt1.a[0] + self.hurt1.b[0]),
            0.5 * (self.hurt1.a[1] + self.hurt1.b[1]),
            0.5 * (self.hurt1.a[2] + self.hurt1.b[2]),
        ]
    }
}

/// How a projectile's flight ended this tick.
#[derive(Clone, Copy, PartialEq, Eq)]
enum End {
    /// Struck its last actor (its pierce count was used up).
    Spent,
    /// Hit the level.
    Impact,
    /// Lifetime ran out.
    Expired,
}

/// One projectile's tick, decided from the unchanged world.
struct Step {
    id: Id,
    pos: V3,
    vel: V3,
    /// Actors it strikes this tick, in the order it reaches them, with the landing and the
    /// event's position.
    landings: Vec<Landing>,
    end: Option<(End, V3)>,
    /// A reflector turned the projectile back: it flies on from `pos` under this actor.
    reflected_by: Option<Id>,
}

fn midpoint(a: V3, b: V3) -> V3 {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn scale(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn len(a: V3) -> f32 {
    sqrt(dot(a, a))
}

/// `a` as a unit vector, or zero if it has no length.
fn unit(a: V3) -> V3 {
    let l = len(a);
    if l > EPS { scale(a, 1.0 / l) } else { [0.0; 3] }
}

/// Turns the direction `cur` toward `want` by at most `max_rad` radians, along the great
/// circle between them. Returns a unit vector. `want` of zero length leaves `cur`.
pub fn turn_dir(cur: V3, want: V3, max_rad: f32) -> V3 {
    let c = unit(cur);
    let d = unit(want);
    if d == [0.0; 3] || c == [0.0; 3] {
        return if c == [0.0; 3] { d } else { c };
    }
    let axis_raw = cross(c, d);
    let s = len(axis_raw);
    let angle = atan2(s, dot(c, d));
    if angle <= max_rad {
        return d;
    }
    let axis = if s > EPS {
        scale(axis_raw, 1.0 / s)
    } else {
        // Facing straight away from the target: any turn is as good as another. Turn about
        // the vertical, or the X axis if the heading is vertical.
        let up = cross(c, [0.0, 1.0, 0.0]);
        unit(if len(up) > EPS {
            up
        } else {
            cross(c, [1.0, 0.0, 0.0])
        })
    };
    // Rodrigues' rotation with the axis perpendicular to `c`: c cos m + (axis x c) sin m.
    let (sn, cs) = (sin(max_rad), cos(max_rad));
    let side = cross(axis, c);
    unit([
        c[0] * cs + side[0] * sn,
        c[1] * cs + side[1] * sn,
        c[2] * cs + side[2] * sn,
    ])
}

/// Where a homing projectile aims: the shooter's lock target at any range if it is still
/// a valid target, else the nearest valid actor within `range_m` (the lowest arena index
/// on a tie). Valid means alive, not the shooter, and not struck already.
fn homing_aim(world: &World, p: &Projectile, range_m: f32, bodies: &[Body]) -> Option<V3> {
    let valid = |b: &&Body| b.id != p.shooter && p.struck.binary_search(&b.id).is_err();
    let lock = world.actors.get(p.shooter).and_then(|a| a.lock_target);
    if let Some(b) = lock.and_then(|l| bodies.iter().filter(valid).find(|b| b.id == l)) {
        return Some(b.center());
    }
    let mut best: Option<(f32, V3)> = None;
    for b in bodies.iter().filter(valid) {
        let c = b.center();
        let d = sub(c, p.pos);
        let d2 = dot(d, d);
        if d2 <= range_m * range_m && best.is_none_or(|(b2, _)| d2 < b2) {
            best = Some((d2, c));
        }
    }
    best.map(|(_, c)| c)
}

/// The shooter's damage inputs for `def`, decided now.
fn landing_for(
    world: &World,
    p: &Projectile,
    def: &ProjectileDef,
    target: &Actor,
    target_id: Id,
    point: V3,
    heading: V3,
) -> Landing {
    let counter = resolve::is_counter(target, &world.moves);
    let hit = damage::Hit {
        ar: AttackRating {
            phys_milli: p.ar_phys_milli,
            elem_milli: p.ar_elem_milli,
            unwieldy: false,
        },
        phys: damage::phys_type(def.phys),
        element: def.element.map(damage::element_type),
        // Thousandths. Rounding the f32 is exact for the listed values (1.2 -> 1200).
        motion_milli: (def.motion_value * 1000.0).round().max(0.0) as u32,
        counter,
        buff_percent: p.buff_percent,
    };
    // The shooter's gear scales the damage (WP-057): Ember Locket, Last Stand, and the
    // Formula bonus.
    let dealt = damage::damage(&world.damage.rules, &hit, &target.stats.defense);
    let dealt = world.actors.get(p.shooter).map_or(dealt, |s| {
        gear::outgoing(
            s,
            &world.items.gear,
            &hit,
            dealt,
            crate::ai::special::reflect::is_formula(def),
        )
    });
    // The knockback direction is the way the projectile was flying, flattened.
    let flat = [heading[0], 0.0, heading[2]];
    let dir = if len(flat) > EPS {
        unit(flat)
    } else {
        world.actors.get(p.shooter).map_or([0.0, 0.0, 1.0], |a| {
            Pose {
                pos: a.pos,
                yaw: a.yaw,
            }
            .forward()
        })
    };
    Landing {
        attacker: p.shooter,
        target: target_id,
        move_num: p.move_num,
        hitbox: PROJECTILE_HITBOX,
        motion_value: def.motion_value,
        charge_frac: 0.0,
        point,
        dir,
        damage: dealt,
        counter,
        poise_damage: u32::from(def.poise_damage),
        unblockable: def.tags.contains(&HitTag::Unblockable),
        status: def.status,
    }
}

/// The earliest touch between a projectile sphere moving from `from` to `to` and a
/// hurtbox, as a fraction of the tick and a contact point. `swept_overlap` caps its
/// substeps, which would leave gaps on a very fast shot, so a long move is cut into
/// chunks of at most [`SWEEP_CHUNK_M`] and each is swept in turn.
fn sweep_body(from: &Capsule, to: &Capsule, b: &Body) -> Option<(f32, V3)> {
    let travel = len(sub(to.a, from.a));
    // `ceil` is exact in IEEE 754, so the count is the same on every target.
    let n = ((travel / SWEEP_CHUNK_M).ceil() as u32).max(1);
    (0..n).find_map(|k| {
        let (t0, t1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
        let (c0, c1) = (from.lerp(to, t0), from.lerp(to, t1));
        let (h0, h1) = (b.hurt0.lerp(&b.hurt1, t0), b.hurt0.lerp(&b.hurt1, t1));
        hitbox::swept_overlap(&c0, &c1, &h0, &h1).map(|(t, p)| (lerp(t0, t1, t), p))
    })
}

/// Decides one projectile's tick. Reads the world; changes nothing.
fn plan(world: &World, id: Id, p: &Projectile, def: &ProjectileDef, bodies: &[Body]) -> Step {
    // Gravity pulls the velocity down; a homing projectile then turns toward its target
    // and keeps its speed.
    let mut v1 = [p.vel[0], p.vel[1] - def.gravity * DT, p.vel[2]];
    if let Some(h) = &def.homing {
        if let Some(aim) = homing_aim(world, p, h.range_m, bodies) {
            v1 = turn_dir(v1, sub(aim, p.pos), h.turn_deg_per_tick * DEG_TO_RAD);
        } else {
            v1 = unit(v1);
        }
        v1 = scale(v1, def.speed);
    }
    // The average of the old and new velocity is exact under constant gravity (the
    // character controller does the same for falls).
    let delta = scale(
        [p.vel[0] + v1[0], p.vel[1] + v1[1], p.vel[2] + v1[2]],
        0.5 * DT,
    );
    let end_pos = [
        p.pos[0] + delta[0],
        p.pos[1] + delta[1],
        p.pos[2] + delta[2],
    ];

    // The level, as the fraction of the move at which the sphere first touches it.
    let wall = world.physics.sphere_cast(p.pos, delta, def.radius);

    // Actors, earliest first.
    let from = Capsule {
        a: p.pos,
        b: p.pos,
        radius: def.radius,
    };
    let to = Capsule {
        a: end_pos,
        b: end_pos,
        radius: def.radius,
    };
    let mut touched: Vec<(f32, Id, V3)> = Vec::new();
    for b in bodies {
        if b.id == p.shooter || p.struck.binary_search(&b.id).is_ok() {
            continue;
        }
        let Some(t) = world.actors.get(b.id) else {
            continue;
        };
        if world
            .actors
            .get(p.shooter)
            .is_some_and(|s| crate::ai::attack::allied(s, t))
        {
            continue;
        }
        // Passing an actor in i-frames (a roll) doesn't count: it can still hit it later.
        if t.dead || dodge::is_invulnerable(&t.moves) {
            continue;
        }
        if let Some((frac, point)) = sweep_body(&from, &to, b) {
            touched.push((frac, b.id, point));
        }
    }
    touched.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));

    let mut landings = Vec::new();
    let mut end = None;
    let mut reflect = None;
    let mut struck = p.struck.len();
    for (frac, target, point) in touched {
        if wall.is_some_and(|w| frac > w) {
            break;
        }
        let Some(t) = world.actors.get(target) else {
            continue;
        };
        // A Formula that meets a reflector flies back at its shooter (WP-036).
        if crate::ai::special::reflect::reflects(world, target, def) {
            reflect = Some((target, point));
            break;
        }
        landings.push(landing_for(world, p, def, t, target, point, v1));
        struck += 1;
        if struck > usize::from(def.pierce) {
            end = Some((End::Spent, point));
            break;
        }
    }
    if let Some((by, point)) = reflect {
        // Back toward where the shooter stands (its chest), at the speed it had; or straight
        // back if the shooter is gone.
        let speed = len(v1);
        let back = world
            .actors
            .get(p.shooter)
            .map(|s| {
                let c = s.hurtbox.capsule(s.pos);
                sub(midpoint(c.a, c.b), point)
            })
            .filter(|d| len(*d) > EPS)
            .unwrap_or_else(|| scale(v1, -1.0));
        return Step {
            id,
            pos: point,
            vel: scale(unit(back), speed),
            landings,
            end: None,
            reflected_by: Some(by),
        };
    }
    if end.is_none() {
        if let Some(w) = wall {
            let at = [
                p.pos[0] + delta[0] * w,
                p.pos[1] + delta[1] * w,
                p.pos[2] + delta[2] * w,
            ];
            end = Some((End::Impact, at));
        } else if p.age + 1 >= def.lifetime_ticks {
            end = Some((End::Expired, end_pos));
        }
    }
    Step {
        id,
        pos: end_pos,
        vel: v1,
        landings,
        end,
        reflected_by: None,
    }
}

/// Emits one projectile event.
fn emit(
    world: &mut World,
    kind: ProjectileEventKind,
    id: Id,
    p: &Projectile,
    target: Option<Id>,
    pos: V3,
    ended: bool,
) {
    world.events.push(Event::Projectile(ProjectileEvent {
        kind,
        projectile: id,
        def: p.def,
        shooter: p.shooter,
        target,
        pos,
        ended,
    }));
}

/// Launches `spawn` from `shooter`, which is performing `move_num`. Returns the new
/// projectile's handle, or `None` if the shooter or the definition doesn't exist.
pub fn spawn(
    world: &mut World,
    shooter: Id,
    move_num: MoveNum,
    spawn: &ProjectileSpawn,
) -> Option<Id> {
    // A bow's shot takes the arrow slot's ammunition, if any (WP-057).
    let ammo = crate::items::usage::take_ammo(world, shooter, &spawn.projectile);
    let (name, elem_percent) = ammo
        .as_ref()
        .map_or((spawn.projectile.as_str(), 0), |(n, e)| (n.as_str(), *e));
    let num = world.projectile_defs.num_of(name)?;
    let def = world.projectile_defs.get(num)?;
    let a = world.actors.get(shooter)?;
    let origin = Pose {
        pos: a.pos,
        yaw: a.yaw,
    }
    .to_world(spawn.offset);
    // Facing turned by the spawn's yaw and pitch: yaw grows from +Z toward +X.
    let yaw = a.yaw + spawn.yaw_deg * DEG_TO_RAD;
    let pitch = spawn.pitch_deg * DEG_TO_RAD;
    let heading = [cos(pitch) * sin(yaw), sin(pitch), cos(pitch) * cos(yaw)];
    // A spell's projectile deals the catalyst's spell power (WP-055), not the weapon's AR.
    let ar = super::spells::power(world, a, move_num)
        .or_else(|| {
            a.inventory.equipment.right_weapon().and_then(|w| {
                world.damage.attack_rating(
                    w,
                    &a.stats.attrs,
                    a.stats.weapon_upgrade,
                    a.stats.two_handing,
                )
            })
        })
        // An enemy has no weapon: its definition's attack rating (WP-032).
        .or_else(|| crate::ai::attack::rating(world, shooter, a))
        .unwrap_or_default();
    // Special ammunition adds its element at a share of the physical attack rating.
    let mut ar = ar;
    if let Some(e) = def.element
        && elem_percent > 0
    {
        ar.elem_milli[gear::element_index(e)] = ar.phys_milli * u64::from(elem_percent) / 100;
    }
    let p = Projectile {
        def: num,
        shooter,
        move_num,
        pos: origin,
        vel: scale(heading, def.speed),
        age: 0,
        struck: Vec::new(),
        ar_phys_milli: ar.phys_milli,
        ar_elem_milli: ar.elem_milli,
        buff_percent: a.stats.buff_percent,
    };
    let id = world.projectiles.insert(p.clone());
    emit(
        world,
        ProjectileEventKind::Spawned,
        id,
        &p,
        None,
        origin,
        false,
    );
    Some(id)
}

/// Tick system 6. See the module docs for the order.
pub fn run(world: &mut World) {
    advance(world);
    launch(world);
}

/// Steps 1 and 2: move, sweep, and resolve every projectile in flight.
fn advance(world: &mut World) {
    if world.projectiles.iter().next().is_none() {
        return;
    }
    // The hurtboxes, swept from where each actor stood last tick (`HitState::prev`).
    let bodies: Vec<Body> = world
        .actors
        .iter()
        .filter(|(_, a)| !a.dead)
        .map(|(id, a)| {
            let prev = a.hit.prev.map_or(a.pos, |p| p.pos);
            Body {
                id,
                hurt0: a.hurtbox.capsule(prev),
                hurt1: a.hurtbox.capsule(a.pos),
            }
        })
        .collect();
    let steps: Vec<Step> = world
        .projectiles
        .iter()
        .filter_map(|(id, p)| {
            let def = world.projectile_defs.get(p.def)?;
            Some(plan(world, id, p, def, &bodies))
        })
        .collect();

    let mut landed: Vec<Id> = Vec::new();
    for step in steps {
        let Some(mut p) = world.projectiles.get(step.id).cloned() else {
            continue;
        };
        let last = step.landings.len();
        for (i, l) in step.landings.iter().enumerate() {
            if !resolve::apply_landing(world, l) {
                continue;
            }
            landed.push(l.target);
            if let Err(at) = p.struck.binary_search(&l.target) {
                p.struck.insert(at, l.target);
            }
            // Only the last landing of a spent projectile ends it.
            let ended = i + 1 == last && step.end.is_some_and(|(e, _)| e == End::Spent);
            emit(
                world,
                ProjectileEventKind::Struck,
                step.id,
                &p,
                Some(l.target),
                l.point,
                ended,
            );
        }
        // A bursting projectile (the Pitch Bomb) also hits what stands near where it ended.
        if let Some((_, at)) = step.end {
            burst(world, step.id, &mut p, at, &mut landed);
        }
        match step.end {
            Some((End::Impact, at)) => {
                emit(
                    world,
                    ProjectileEventKind::Impact,
                    step.id,
                    &p,
                    None,
                    at,
                    true,
                );
            }
            Some((End::Expired, at)) => {
                emit(
                    world,
                    ProjectileEventKind::Expired,
                    step.id,
                    &p,
                    None,
                    at,
                    true,
                );
            }
            Some((End::Spent, _)) => {}
            None => {
                p.pos = step.pos;
                p.vel = step.vel;
                p.age += 1;
                // A reflected Formula belongs to the reflector now, and can strike its old
                // shooter.
                if let Some(by) = step.reflected_by {
                    p.shooter = by;
                    p.struck.clear();
                }
                if let Some(slot) = world.projectiles.get_mut(step.id) {
                    *slot = p;
                }
                continue;
            }
        }
        world.projectiles.remove(step.id);
    }
    // An actor at 0 HP is dead. Set after the last landing, as in `resolve::run`.
    for t in landed {
        if let Some(a) = world.actors.get_mut(t)
            && a.hp == 0
        {
            a.dead = true;
        }
    }
}

/// The burst of a projectile whose definition has an `area_radius_m` (WP-057): every actor
/// within that many meters of `at` that it hasn't struck takes the hit, in arena order.
/// A burst ignores cover; the shooter and its allies aren't hurt.
fn burst(world: &mut World, pid: Id, p: &mut Projectile, at: V3, landed: &mut Vec<Id>) {
    let Some(def) = world.projectile_defs.get(p.def) else {
        return;
    };
    let radius = def.area_radius_m;
    if radius <= 0.0 {
        return;
    }
    let shooter = world.actors.get(p.shooter);
    let mut hits: Vec<(Id, Landing)> = Vec::new();
    for (id, t) in world.actors.iter() {
        if t.dead
            || id == p.shooter
            || p.struck.binary_search(&id).is_ok()
            || shooter.is_some_and(|s| crate::ai::attack::allied(s, t))
        {
            continue;
        }
        let c = t.hurtbox.capsule(t.pos);
        let d = sub(midpoint(c.a, c.b), at);
        if dot(d, d) <= radius * radius {
            hits.push((id, landing_for(world, p, def, t, id, at, [0.0; 3])));
        }
    }
    for (id, l) in hits {
        if !resolve::apply_landing(world, &l) {
            continue;
        }
        landed.push(id);
        if let Err(i) = p.struck.binary_search(&id) {
            p.struck.insert(i, id);
        }
        // `ended` is false: the projectile's own end event follows.
        let snapshot = p.clone();
        emit(
            world,
            ProjectileEventKind::Struck,
            pid,
            &snapshot,
            Some(id),
            l.point,
            false,
        );
    }
}

/// Step 3: spawns what moves launch on the frame executed this tick.
fn launch(world: &mut World) {
    let mut pending: Vec<(Id, MoveNum, ProjectileSpawn)> = Vec::new();
    for (id, a) in world.actors.iter() {
        let Some(c) = a.moves.current.filter(|_| !a.dead) else {
            continue;
        };
        // A charging move holds its frame; a spawn on it would repeat every tick.
        if a.moves.has(flags::CHARGING) {
            continue;
        }
        let Some(def) = world.moves.get(c.num) else {
            continue;
        };
        for s in def.data.projectiles.iter().filter(|s| s.frame == c.frame) {
            pending.push((id, c.num, s.clone()));
        }
    }
    for (shooter, num, s) in pending {
        spawn(world, shooter, num, &s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::moves::flags;
    use crate::events::ProjectileEventKind::{Expired, Impact, Spawned, Struck};
    use crate::input::{ActionFrame, TickInput, buttons};
    use crate::tick::tick;

    const ARROW: &str = "proj_arrow";
    const BOLT: &str = "proj_seeker_bolt";
    const TO_DEG: f32 = 180.0 / core::f32::consts::PI;

    fn range() -> World {
        let content = Content::load_embedded().expect("embedded content");
        World::new(7, &content, "m2_range").expect("m2_range exists")
    }

    /// Changes the definition `id` in this world only.
    fn tweak(w: &mut World, id: &str, f: impl FnOnce(&mut ProjectileDef)) {
        let d = w
            .projectile_defs
            .defs
            .iter_mut()
            .find(|d| d.1.id == id)
            .expect("definition exists");
        f(&mut d.1);
    }

    /// Launches `id` from the player, standing with its feet at `pos` and facing +Z, along
    /// the facing turned by `yaw_deg` and `pitch_deg`.
    fn fire(w: &mut World, id: &str, pos: V3, yaw_deg: f32, pitch_deg: f32) -> Id {
        let me = w.player;
        let a = w.actors.get_mut(me).unwrap();
        a.pos = pos;
        a.yaw = 0.0;
        a.hit.prev = None;
        let s = ProjectileSpawn {
            frame: 0,
            projectile: id.to_owned(),
            offset: [0.0; 3],
            yaw_deg,
            pitch_deg,
        };
        spawn(w, me, 0, &s).expect("spawns")
    }

    /// A copy of the first dummy with its feet at `pos`.
    fn dummy_at(w: &mut World, pos: V3, hp: u32) -> Id {
        let proto = w
            .actors
            .iter()
            .find(|(id, _)| *id != w.player)
            .map(|(_, a)| a.clone())
            .unwrap();
        w.actors.insert(Actor {
            pos,
            hp,
            max_hp: hp,
            hit: Default::default(),
            ..proto
        })
    }

    /// Runs the projectile system `n` times and returns the events.
    fn fly(w: &mut World, n: u32) -> Vec<ProjectileEvent> {
        let mut out = Vec::new();
        for _ in 0..n {
            run(w);
            for e in w.events.drain() {
                if let Event::Projectile(p) = e {
                    out.push(p);
                }
            }
        }
        out
    }

    fn only(w: &World) -> &Projectile {
        let mut it = w.projectiles.iter();
        let p = it.next().expect("one projectile in flight").1;
        assert!(it.next().is_none());
        p
    }

    fn angle_between(a: V3, b: V3) -> f32 {
        atan2(len(cross(a, b)), dot(a, b)) * TO_DEG
    }

    #[test]
    fn gravity_bends_the_flight_into_a_parabola() {
        let mut w = range();
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 10.0);
        w.events.drain();
        let pitch = 10.0 * DEG_TO_RAD;
        let (speed, g) = (40.0, 12.0);
        let (vz, vy) = (speed * cos(pitch), speed * sin(pitch));
        assert!((len(only(&w).vel) - speed).abs() < 1e-3);
        for k in 1..=10 {
            fly(&mut w, 1);
            let t = k as f32 * DT;
            let p = only(&w);
            // z = v t and y = y0 + v t - g t^2 / 2: the average-velocity step is exact.
            assert!((p.pos[2] - vz * t).abs() < 2e-3, "z at {k}: {:?}", p.pos);
            assert!(
                (p.pos[1] - (1.4 + vy * t - 0.5 * g * t * t)).abs() < 2e-3,
                "y at {k}: {:?}",
                p.pos
            );
            assert_eq!(p.age, k);
        }
        // It rose, peaked, and is falling.
        assert!(only(&w).vel[1] < vy);
    }

    #[test]
    fn a_projectile_expires_after_its_lifetime() {
        let mut w = range();
        tweak(&mut w, BOLT, |d| {
            d.lifetime_ticks = 10;
            d.homing = None;
            d.speed = 5.0;
        });
        let id = fire(&mut w, BOLT, [0.0, 1.4, 0.0], 0.0, 90.0);
        w.events.drain();
        assert!(fly(&mut w, 9).is_empty(), "still flying after 9 ticks");
        let before = only(&w).pos;
        let events = fly(&mut w, 1);
        assert_eq!(events.len(), 1, "{events:?}");
        let e = events[0];
        assert_eq!((e.kind, e.ended, e.projectile), (Expired, true, id));
        // It moved on all 10 ticks: 5 m/s straight up for 10 ticks.
        assert!(
            (e.pos[1] - (1.4 + 5.0 * 10.0 * DT)).abs() < 1e-3,
            "{:?}",
            e.pos
        );
        assert!(e.pos[1] > before[1]);
        assert!(w.projectiles.iter().next().is_none());
    }

    #[test]
    fn pierce_counts_the_actors_it_passes_through() {
        for (pierce, expect) in [(0u8, 1usize), (1, 2), (2, 3)] {
            let mut w = range();
            tweak(&mut w, ARROW, |d| {
                d.gravity = 0.0;
                d.pierce = pierce;
            });
            let line: Vec<Id> = [5.0, 8.0, 12.0, 16.0]
                .map(|z| dummy_at(&mut w, [0.0, 0.0, z], 1000))
                .to_vec();
            fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
            w.events.drain();
            let events = fly(&mut w, 40);
            let struck: Vec<_> = events.iter().filter(|e| e.kind == Struck).collect();
            assert_eq!(struck.len(), expect, "pierce {pierce}: {events:?}");
            // In order along the line, and only the last strike ends the flight.
            for (i, e) in struck.iter().enumerate() {
                assert_eq!(e.target, Some(line[i]));
                assert_eq!(e.ended, i + 1 == expect);
            }
            for (i, &id) in line.iter().enumerate() {
                let hit = w.actors.get(id).unwrap().hp < 1000;
                assert_eq!(hit, i < expect, "pierce {pierce}, dummy {i}");
            }
            assert!(w.projectiles.iter().next().is_none());
        }
    }

    #[test]
    fn a_slow_projectile_strikes_an_actor_once() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| {
            d.gravity = 0.0;
            d.speed = 3.0;
            d.pierce = 5;
            d.lifetime_ticks = 120;
        });
        let target = dummy_at(&mut w, [0.0, 0.0, 3.0], 1000);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 120);
        let n = events.iter().filter(|e| e.kind == Struck).count();
        assert_eq!(n, 1, "{events:?}");
        assert!(w.actors.get(target).unwrap().hp < 1000);
        // It carried on past the dummy and ran out its lifetime.
        assert_eq!(events.last().map(|e| e.kind), Some(Expired));
    }

    #[test]
    fn level_geometry_stops_a_projectile() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        // Straight into the back wall (its near face is at z = 29.5).
        fire(&mut w, ARROW, [0.0, 1.4, 25.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 10);
        let e = events.last().copied().unwrap();
        assert_eq!((e.kind, e.ended), (Impact, true), "{events:?}");
        assert!((e.pos[2] - (29.5 - 0.05)).abs() < 0.01, "{:?}", e.pos);
        assert!((e.pos[1] - 1.4).abs() < 1e-3);
        assert!(w.projectiles.iter().next().is_none());

        // Down into the floor (its top is at y = 0): it stops a radius above it.
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        fire(&mut w, ARROW, [0.0, 3.0, 10.0], 0.0, -45.0);
        w.events.drain();
        let e = fly(&mut w, 10).last().copied().unwrap();
        assert_eq!(e.kind, Impact);
        assert!((e.pos[1] - 0.05).abs() < 0.01, "{:?}", e.pos);
        assert!((e.pos[2] - 13.0).abs() < 0.1, "{:?}", e.pos);
    }

    #[test]
    fn a_wall_hides_the_actors_behind_it() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        let behind = dummy_at(&mut w, [0.0, 0.0, 33.0], 1000);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 40);
        assert_eq!(events.last().map(|e| e.kind), Some(Impact), "{events:?}");
        assert!(events.iter().all(|e| e.kind != Struck));
        assert_eq!(w.actors.get(behind).unwrap().hp, 1000);
    }

    #[test]
    fn fast_projectiles_do_not_tunnel() {
        // 400 m/s is the validator's cap (13 m a tick); 1200 m/s is three times past it.
        for speed in [400.0, 1200.0] {
            // Through a 1 m wall: a shape cast is continuous.
            let mut w = range();
            tweak(&mut w, ARROW, |d| {
                d.gravity = 0.0;
                d.speed = speed;
            });
            fire(&mut w, ARROW, [0.0, 1.4, 20.0], 0.0, 0.0);
            w.events.drain();
            let e = fly(&mut w, 3).last().copied().unwrap();
            assert_eq!(e.kind, Impact, "{speed} m/s");
            assert!((e.pos[2] - 29.45).abs() < 0.01, "{speed} m/s: {:?}", e.pos);

            // Through a dummy's 0.35 m capsule, in one tick.
            let mut w = range();
            tweak(&mut w, ARROW, |d| {
                d.gravity = 0.0;
                d.speed = speed;
            });
            let target = dummy_at(&mut w, [0.0, 0.0, 9.0], 1000);
            fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
            w.events.drain();
            let events = fly(&mut w, 3);
            let hit = events.iter().find(|e| e.kind == Struck);
            assert_eq!(hit.and_then(|e| e.target), Some(target), "{speed} m/s");
            assert_eq!(w.projectiles.iter().count(), 0);
        }
    }

    #[test]
    fn a_fast_projectile_catches_a_target_that_it_passes_near() {
        // 0.3 m off the axis of a dummy (radius 0.35) and 0.05 m of its own radius: a
        // graze at 1200 m/s, 40 m a tick, must still count.
        let mut w = range();
        tweak(&mut w, ARROW, |d| {
            d.gravity = 0.0;
            d.speed = 1200.0;
        });
        let target = dummy_at(&mut w, [0.37, 0.0, 9.0], 1000);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 2);
        let hit = events.iter().find(|e| e.kind == Struck);
        assert_eq!(hit.and_then(|e| e.target), Some(target), "{events:?}");
    }

    #[test]
    fn homing_turns_by_no_more_than_its_limit() {
        let mut w = range();
        let mark = w
            .actors
            .iter()
            .find(|(_, a)| a.pos == [3.0, 0.0, 10.0])
            .map(|(id, _)| id)
            .unwrap();
        w.actors.get_mut(w.player).unwrap().lock_target = Some(mark);
        // Launched along +X, 73 degrees away from the lock target.
        fire(&mut w, BOLT, [0.0, 1.4, 0.0], 90.0, 0.0);
        w.events.drain();
        let limit = 8.0;
        let mut turns = Vec::new();
        let mut struck = None;
        for _ in 0..90 {
            let before = only(&w).vel;
            let events = fly(&mut w, 1);
            if let Some(e) = events.iter().find(|e| e.kind == Struck) {
                struck = Some(e.target);
                break;
            }
            let p = only(&w);
            assert!((len(p.vel) - 12.0).abs() < 1e-3, "speed stays 12");
            turns.push(angle_between(before, p.vel));
        }
        assert!(turns.iter().all(|&t| t <= limit + 0.05), "{turns:?}");
        // It is turn-limited at first: 8 degrees every one of the first several ticks.
        assert!(
            turns[..5].iter().all(|&t| (t - limit).abs() < 0.05),
            "{turns:?}"
        );
        // And it turned toward the target rather than away, and hit it.
        assert_eq!(struck, Some(Some(mark)));
    }

    #[test]
    fn homing_falls_back_to_the_nearest_actor_within_range() {
        // No lock: the nearest actor is dummy_aside (9.4 m), not dummy_mark (10.4 m).
        let mut w = range();
        let aside = w
            .actors
            .iter()
            .find(|(_, a)| a.pos == [-5.0, 0.0, 8.0])
            .map(|(id, _)| id)
            .unwrap();
        fire(&mut w, BOLT, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 90);
        let hit = events.iter().find(|e| e.kind == Struck);
        assert_eq!(hit.and_then(|e| e.target), Some(aside), "{events:?}");

        // With nothing within range it flies straight.
        let mut w = range();
        tweak(&mut w, BOLT, |d| d.homing.as_mut().unwrap().range_m = 5.0);
        fire(&mut w, BOLT, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        fly(&mut w, 5);
        assert_eq!(only(&w).vel, [0.0, 0.0, 12.0]);
    }

    #[test]
    fn homing_ignores_the_shooter_and_actors_it_already_struck() {
        // Pierce 1: after the first dummy the bolt must go for the second, not turn back.
        let mut w = range();
        tweak(&mut w, BOLT, |d| d.pierce = 1);
        let first = dummy_at(&mut w, [0.0, 0.0, 4.0], 1000);
        fire(&mut w, BOLT, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 90);
        let targets: Vec<_> = events
            .iter()
            .filter(|e| e.kind == Struck)
            .map(|e| e.target.unwrap())
            .collect();
        assert_eq!(targets.len(), 2, "{events:?}");
        assert_eq!(targets[0], first);
        assert_ne!(targets[1], first);
        assert_ne!(targets[1], w.player);
    }

    #[test]
    fn turn_dir_turns_by_exactly_the_limit_the_short_way() {
        let limit = 30.0 * DEG_TO_RAD;
        let r = turn_dir([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], limit);
        assert!((angle_between(r, [1.0, 0.0, 0.0]) - 30.0).abs() < 1e-3);
        assert!(r[1] > 0.0 && r[2].abs() < 1e-6, "turns in the plane: {r:?}");
        // Inside the limit it lands on the target.
        let r = turn_dir([1.0, 0.0, 0.0], [1.0, 0.1, 0.0], limit);
        assert!(angle_between(r, [1.0, 0.1, 0.0]) < 1e-3);
        // Facing straight away still turns, by the limit.
        let r = turn_dir([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], limit);
        assert!((angle_between(r, [1.0, 0.0, 0.0]) - 30.0).abs() < 1e-3);
        assert!((len(r) - 1.0).abs() < 1e-5);
        // No target direction leaves the heading alone.
        assert_eq!(turn_dir([0.0, 0.0, 2.0], [0.0; 3], limit), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn a_hit_goes_through_the_melee_apply_path() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        let target = dummy_at(&mut w, [0.0, 0.0, 1.5], 500);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        run(&mut w);
        let hits: Vec<_> = w
            .events
            .drain()
            .into_iter()
            .filter_map(|e| match e {
                Event::Hit(h) => Some(h),
                _ => None,
            })
            .collect();
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert_eq!((h.attacker, h.target), (w.player, target));
        assert_eq!(h.hitbox, PROJECTILE_HITBOX);
        assert!(h.damage > 0 && !h.killed && !h.counter, "{h:?}");
        let t = w.actors.get(target).unwrap();
        assert_eq!((h.hp, t.hp), (500 - h.damage, 500 - h.damage));
        // Flying along +Z: the knockback direction is +Z.
        assert!((h.dir[2] - 1.0).abs() < 1e-5 && h.dir[0].abs() < 1e-5);
    }

    #[test]
    fn two_shots_in_one_tick_both_land_even_if_the_first_kills() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        let target = dummy_at(&mut w, [0.0, 0.0, 1.5], 1);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        run(&mut w);
        let hits = w
            .events
            .drain()
            .into_iter()
            .filter(|e| matches!(e, Event::Hit(_)))
            .count();
        assert_eq!(hits, 2);
        let t = w.actors.get(target).unwrap();
        assert!(t.dead && t.hp == 0);
        assert!(w.projectiles.iter().next().is_none());
    }

    #[test]
    fn a_projectile_passes_an_actor_in_iframes() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        let target = dummy_at(&mut w, [0.0, 0.0, 1.5], 500);
        w.actors.get_mut(target).unwrap().moves.flags = flags::IFRAMES;
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 8);
        assert!(events.iter().all(|e| e.kind != Struck), "{events:?}");
        assert_eq!(w.actors.get(target).unwrap().hp, 500);
    }

    #[test]
    fn a_projectile_cannot_hit_its_shooter_or_the_dead() {
        let mut w = range();
        tweak(&mut w, ARROW, |d| d.gravity = 0.0);
        let corpse = dummy_at(&mut w, [0.0, 0.0, 6.0], 500);
        w.actors.get_mut(corpse).unwrap().dead = true;
        // It starts inside the shooter's own capsule and flies on through the corpse.
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 0.0);
        w.events.drain();
        let events = fly(&mut w, 8);
        assert!(events.iter().all(|e| e.kind != Struck), "{events:?}");
        let me = w.actors.get(w.player).unwrap();
        assert_eq!(me.hp, me.max_hp);
        assert_eq!(w.actors.get(corpse).unwrap().hp, 500);
    }

    #[test]
    fn a_move_launches_its_projectile_on_the_listed_frame() {
        let mut w = range();
        let mut spawned_on = Vec::new();
        for t in 0..40u32 {
            let buttons = if t == 0 { buttons::LIGHT } else { 0 };
            tick(
                &mut w,
                &TickInput {
                    frame: ActionFrame {
                        move_x: 0,
                        move_z: 0,
                        buttons,
                    },
                    commands: Vec::new(),
                },
            );
            for e in w.events.drain() {
                if let Event::Projectile(p) = e
                    && p.kind == Spawned
                {
                    assert_eq!(p.def, w.projectile_defs.num_of(ARROW).unwrap());
                    assert_eq!(p.shooter, w.player);
                    spawned_on.push(t);
                }
            }
        }
        // `stub_bow_shot` spawns on frame 8, which executes 8 ticks after the press.
        assert_eq!(spawned_on, [8]);
    }

    #[test]
    fn projectiles_are_in_the_state_hash() {
        let mut w = range();
        let fresh = w.state_hash();
        tweak(&mut w, ARROW, |d| d.lifetime_ticks = 3);
        fire(&mut w, ARROW, [0.0, 1.4, 0.0], 0.0, 90.0);
        let in_flight = w.state_hash();
        assert_ne!(fresh, in_flight);
        let mut twin = w.clone();
        fly(&mut w, 1);
        fly(&mut twin, 1);
        assert_eq!(w.state_hash(), twin.state_hash());
        assert_ne!(w.state_hash(), in_flight);
        // Even after it is gone, the world remembers a projectile was ever spawned.
        fly(&mut w, 5);
        assert!(w.projectiles.iter().next().is_none());
        assert_ne!(w.state_hash(), fresh);
    }

    #[test]
    fn the_snapshot_lists_projectiles_in_flight() {
        let mut w = range();
        let id = fire(&mut w, ARROW, [1.0, 1.4, 2.0], 0.0, 0.0);
        let mut out = Vec::new();
        crate::snapshot::write_projectile_snapshot(&w, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, crate::snapshot::pack_id(id));
        assert_eq!(out[0].def, 5000);
        assert_eq!(out[0].pos, [1.0, 1.4, 2.0]);
        assert_eq!(out[0].vel, [0.0, 0.0, 40.0]);
    }
}
