"""Builds Waking Hollow's cave kit from the level layout, in headless Blender.

Reads the collision layout (`game/data/levels/waking_hollow/layout.ron`), the hollow wall
and the Lantern from `entities.ron`, and the art settings in `cave.json`, and writes:

- `game/assets-src/waking-hollow-cave/cave.glb`: the cave in world coordinates. One mesh
  node per section and material, so the renderer can cull and sort by section.
- `game/assets-src/waking-hollow-cave/hollow_wall.glb`: the cracked, bricked-up wall that
  plugs the closet, centered on the hollow wall's collider.
- `game/assets-src/art/waking_hollow.json`: the area art manifest (models, the props that
  replace the greybox, the lights, the fog, and the color grade).

The art follows the colliders. The script stops if the rooms in `cave.json` don't cover
exactly the open cells of the layout's floor blocks. Rock may stand at most 0.15 m in front
of a wall collider (the player's capsule is 0.35 m wide), floors stay within 2 cm of the
floor colliders, and dressing that pokes into walkable space is low or above head height.

Steps:
1. Carves the rooms as boxes (the arena with rounded corners and a dome), joins them with a
   voxel remesh, and turns the surface inward.
2. Pushes the rock outward with layered noise and horizontal strata. Built stone (the gate
   facade, the arena walls) and the hollow wall's doorway stay flat.
3. Adds floors, the jump ledge, the arena's pillars and floor ring, and the dressing:
   stalactites, stalagmites, rubble, a collapsed corridor, roots, bones, torches, candles.
4. Projects world-space UVs and assigns the scanned materials (`textures.json`).
5. Bakes light into vertex colors with Cycles: ambient occlusion times the torches,
   candles, and Lantern from `cave.json`. `--no-bake` skips this (flat white).
6. Exports the GLBs and writes the art manifest.

Run (or `mise run art:cave`):
    blender --background --factory-startup --python build_cave.py -- [--no-bake]
        [--textures DIR] [--repo DIR]
"""

import json
import math
import os
import random
import re
import sys

import bmesh
import bpy
from mathutils import Vector, noise
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))

# --------------------------------------------------------------------------------------
# Arguments and inputs


def parse_args():
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    opts = {
        "bake": True,
        "textures": os.path.expanduser("~/cinderheart-art/cave-textures"),
        "repo": os.path.abspath(os.path.join(HERE, "..", "..", "..")),
    }
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--no-bake":
            opts["bake"] = False
        elif a in ("--textures", "--repo"):
            opts[a[2:]] = os.path.abspath(os.path.expanduser(argv[i + 1]))
            i += 1
        else:
            raise SystemExit(f"build_cave: unknown argument {a}")
        i += 1
    return opts


OPTS = parse_args()
REPO = OPTS["repo"]
CFG = json.load(open(os.path.join(HERE, "cave.json")))
TEX = json.load(open(os.path.join(HERE, "textures.json")))
LEVEL = os.path.join(REPO, "game", "data", "levels", "waking_hollow")
OUT_DIR = os.path.join(REPO, "game", "assets-src", "waking-hollow-cave")
ART_JSON = os.path.join(REPO, "game", "assets-src", "art", "waking_hollow.json")

rng = random.Random(CFG["seed"])
noise.seed_set(CFG["seed"] % 100000)

NUM = r"(-?[\d.]+)"


def read_layout():
    text = open(os.path.join(LEVEL, "layout.ron")).read()
    pat = re.compile(
        r'\(name: "([^"]+)", kit: "[^"]+", pos: \(' + NUM + ", " + NUM + ", " + NUM + r"\), scale: \(" + NUM + ", " + NUM + ", " + NUM + r"\)"
    )
    blocks = {}
    for m in pat.finditer(text):
        v = [float(x) for x in m.groups()[1:]]
        blocks[m.group(1)] = {"pos": v[:3], "scale": v[3:]}
    if not blocks:
        raise SystemExit("build_cave: no blocks in layout.ron")
    return blocks


def read_entities():
    text = open(os.path.join(LEVEL, "entities.ron")).read()
    m = re.search(r"HollowWall\(id: \"[^\"]+\", center: \(" + NUM + ", " + NUM + ", " + NUM + r"\), half_extents: \(" + NUM + ", " + NUM + ", " + NUM + r"\)", text)
    lantern = re.search(r"Lantern\(id: \"[^\"]+\", pos: \(" + NUM + ", " + NUM + ", " + NUM + r"\), yaw: " + NUM, text)
    gate = re.search(r"gate: \(pos: \(" + NUM + ", " + NUM + ", " + NUM + r"\), yaw: " + NUM + r", half_width_m: " + NUM, text)
    if not (m and lantern and gate):
        raise SystemExit("build_cave: entities.ron has no hollow wall, Lantern, or boss gate")
    g = [float(x) for x in m.groups()]
    return {
        "hollow": {"center": g[:3], "half": g[3:]},
        "lantern": [float(x) for x in lantern.groups()[:3]],
        "gate": [float(x) for x in gate.groups()],
    }


BLOCKS = read_layout()
ENT = read_entities()
CELL = 2.0


def cells_of(x0, x1, z0, z1):
    out = set()
    for i in range(int(math.floor(x0 / CELL + 1e-6)), int(math.ceil(x1 / CELL - 1e-6))):
        for k in range(int(math.floor(z0 / CELL + 1e-6)), int(math.ceil(z1 / CELL - 1e-6))):
            out.add((i, k))
    return out


def block_box(b):
    (x, y, z), (sx, sy, sz) = b["pos"], b["scale"]
    return x - sx / 2, x + sx / 2, y - sy / 2, y + sy / 2, z - sz / 2, z + sz / 2


OPEN = set()
WALLS = set()
for name, b in BLOCKS.items():
    x0, x1, y0, y1, z0, z1 = block_box(b)
    if name.startswith("floor_"):
        if abs(y1) > 1e-6:
            raise SystemExit(f"build_cave: floor {name} does not top out at y = 0; the art assumes it does")
        OPEN |= cells_of(x0, x1, z0, z1)
    elif name.startswith("wall_"):
        WALLS |= cells_of(x0, x1, z0, z1)

ROOM_CELLS = set()
for r in CFG["rooms"]:
    ROOM_CELLS |= cells_of(r["x"][0], r["x"][1], r["z"][0], r["z"][1])
if ROOM_CELLS != OPEN:
    raise SystemExit(
        "build_cave: cave.json rooms don't match layout.ron floors. "
        f"Only in rooms: {sorted(ROOM_CELLS - OPEN)[:8]}; only in floors: {sorted(OPEN - ROOM_CELLS)[:8]}"
    )
for i, k in OPEN:
    for di, dk in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        n = (i + di, k + dk)
        if n not in OPEN and n not in WALLS:
            raise SystemExit(f"build_cave: open cell {(i, k)} has no wall collider at {n}")
print(f"build_cave: {len(OPEN)} open cells match the layout; {len(BLOCKS)} blocks read")


def is_open(x, z):
    return (math.floor(x / CELL), math.floor(z / CELL)) in OPEN


def dist_to_open(x, z, reach=3):
    """Distance from (x, z) to the nearest open cell, 0 inside one."""
    ci, ck = math.floor(x / CELL), math.floor(z / CELL)
    if (ci, ck) in OPEN:
        return 0.0
    best = 99.0
    for di in range(-reach, reach + 1):
        for dk in range(-reach, reach + 1):
            c = (ci + di, ck + dk)
            if c not in OPEN:
                continue
            x0, z0 = c[0] * CELL, c[1] * CELL
            dx = max(x0 - x, 0.0, x - (x0 + CELL))
            dz = max(z0 - z, 0.0, z - (z0 + CELL))
            best = min(best, math.hypot(dx, dz))
    return best


def room_at(x, z):
    for r in CFG["rooms"]:
        if r["x"][0] <= x < r["x"][1] and r["z"][0] <= z < r["z"][1]:
            return r["name"]
    return None


def section_of(z):
    for s in CFG["sections"]:
        if s["z"][0] <= z < s["z"][1]:
            return s["name"]
    return CFG["sections"][-1]["name"]


# Game (x, y-up, z) to Blender (x, y, z-up). The glTF exporter's +Y-up turns it back.
def g2b(p):
    return Vector((p[0], -p[2], p[1]))


def b2g(v):
    return (v.x, v.z, -v.y)


# --------------------------------------------------------------------------------------
# Geometry store: (section, material) -> vertices (game coords) and faces.

MESHES = {}


def store(section, material):
    key = (section, material)
    if key not in MESHES:
        MESHES[key] = {"v": [], "f": []}
    return MESHES[key]


def add_mesh(section, material, verts, faces):
    s = store(section, material)
    base = len(s["v"])
    s["v"].extend(tuple(v) for v in verts)
    s["f"].extend(tuple(i + base for i in f) for f in faces)


def ring(center, radius, sides, phase=0.0, jitter=0.0, squash=(1.0, 1.0)):
    out = []
    for i in range(sides):
        a = phase + 2 * math.pi * i / sides
        r = radius * (1 + (rng.uniform(-jitter, jitter) if jitter else 0))
        out.append((center[0] + math.cos(a) * r * squash[0], center[1], center[2] + math.sin(a) * r * squash[1]))
    return out


def tube(points, radii, sides=6, cap_start=True, cap_end=True, jitter=0.0):
    """A tube along `points` (any direction), one radius per point. Returns verts, faces."""
    verts, faces = [], []
    pts = [Vector(p) for p in points]
    for idx, p in enumerate(pts):
        if idx == 0:
            d = pts[1] - pts[0]
        elif idx == len(pts) - 1:
            d = pts[-1] - pts[-2]
        else:
            d = pts[idx + 1] - pts[idx - 1]
        d.normalize()
        up = Vector((0, 1, 0)) if abs(d.y) < 0.9 else Vector((1, 0, 0))
        u = d.cross(up).normalized()
        w = d.cross(u).normalized()
        for s in range(sides):
            a = 2 * math.pi * s / sides
            r = radii[idx] * (1 + (rng.uniform(-jitter, jitter) if jitter else 0))
            q = p + (u * math.cos(a) + w * math.sin(a)) * r
            verts.append((q.x, q.y, q.z))
    for idx in range(len(pts) - 1):
        for s in range(sides):
            a = idx * sides + s
            b = idx * sides + (s + 1) % sides
            faces.append((a, b, b + sides, a + sides))
    if cap_start:
        faces.append(tuple(reversed(range(sides))))
    if cap_end:
        last = (len(pts) - 1) * sides
        faces.append(tuple(range(last, last + sides)))
    return verts, faces


def blob(center, size, subdiv=0, rough=0.25, flat_bottom=None):
    """An irregular rock: a jittered icosphere. `flat_bottom` clamps its underside to a height."""
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=1.0)
    verts = []
    off = Vector((rng.uniform(0, 50), rng.uniform(0, 50), rng.uniform(0, 50)))
    for v in bm.verts:
        n = noise.noise(v.co * 1.7 + off)
        k = 1 + rough * n
        # (x, z, -y): a rotation, so the faces keep looking outward.
        p = (center[0] + v.co.x * size[0] * k, center[1] + v.co.z * size[1] * k, center[2] - v.co.y * size[2] * k)
        if flat_bottom is not None:
            p = (p[0], max(p[1], flat_bottom), p[2])
        verts.append(p)
    faces = [tuple(v.index for v in f.verts) for f in bm.faces]
    bm.free()
    return verts, faces


# --------------------------------------------------------------------------------------
# 1. The cavity: rooms as boxes, joined by a voxel remesh.


def box_bmesh(bm, x0, x1, y0, y1, z0, z1, corner=0.0):
    if corner <= 0:
        r = bmesh.ops.create_cube(bm, size=1.0)
        for v in r["verts"]:
            gx = x0 if v.co.x < 0 else x1
            gy = y0 if v.co.z < 0 else y1
            gz = z0 if v.co.y > 0 else z1  # Blender +y is game -z.
            v.co = g2b((gx, gy, gz))
        return
    # A rounded rectangle prism: corners as quarter circles.
    pts = []
    cx = [(x1 - corner, z1 - corner, 0), (x0 + corner, z1 - corner, 90), (x0 + corner, z0 + corner, 180), (x1 - corner, z0 + corner, 270)]
    for x, z, a0 in cx:
        for s in range(7):
            a = math.radians(a0 + 90 * s / 6)
            pts.append((x + math.cos(a) * corner, z + math.sin(a) * corner))
    bot = [bm.verts.new(g2b((x, y0, z))) for x, z in pts]
    top = [bm.verts.new(g2b((x, y1, z))) for x, z in pts]
    n = len(pts)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((bot[i], bot[j], top[j], top[i]))
    bm.faces.new(top)
    bm.faces.new(list(reversed(bot)))


def dome_bmesh(bm, cx, cz, base_y, rx, rz, rise):
    r = bmesh.ops.create_uvsphere(bm, u_segments=32, v_segments=16, radius=1.0)
    for v in r["verts"]:
        x, y, z = v.co.x, v.co.y, v.co.z
        v.co = g2b((cx + x * rx, base_y + z * rise, cz - y * rz))


def build_cavity():
    bm = bmesh.new()
    for r in CFG["rooms"]:
        g = r.get("grow", [0, 0, 0, 0])
        x0, x1 = r["x"][0] - g[0], r["x"][1] + g[1]
        z0, z1 = r["z"][0] - g[2], r["z"][1] + g[3]
        box_bmesh(bm, x0, x1, -0.6, r["ceiling"], z0, z1, r.get("corner_m", 0.0))
        if "dome" in r:
            cx, cz = (r["x"][0] + r["x"][1]) / 2, (r["z"][0] + r["z"][1]) / 2
            half = (r["x"][1] - r["x"][0]) / 2
            dome_bmesh(bm, cx, cz, r["ceiling"] - 0.4, half + 0.3, (r["z"][1] - r["z"][0]) / 2 + 0.3, r["dome"]["rise_m"])
    me = bpy.data.meshes.new("cavity_src")
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new("cavity_src", me)
    bpy.context.scene.collection.objects.link(obj)
    mod = obj.modifiers.new("remesh", "REMESH")
    mod.mode = "VOXEL"
    mod.voxel_size = CFG["voxel_m"]
    mod.adaptivity = 0.0
    dg = bpy.context.evaluated_depsgraph_get()
    me2 = bpy.data.meshes.new_from_object(obj.evaluated_get(dg))
    bpy.data.objects.remove(obj)
    bm = bmesh.new()
    bm.from_mesh(me2)
    bpy.data.meshes.remove(me2)
    # Inward: the faces look into the cave.
    bmesh.ops.reverse_faces(bm, faces=bm.faces[:])
    # The underside lies below the floor meshes; drop it.
    drop = [f for f in bm.faces if all(b2g(v.co)[1] < -0.25 for v in f.verts)]
    bmesh.ops.delete(bm, geom=drop, context="FACES")
    bm.normal_update()
    bm.verts.index_update()
    return bm


MAS = CFG["masonry"]
HW = ENT["hollow"]
HW_X0 = HW["center"][0] - HW["half"][0]
HW_X1 = HW["center"][0] + HW["half"][0]
HW_Z0 = HW["center"][2] - HW["half"][2]
HW_Z1 = HW["center"][2] + HW["half"][2]
HW_TOP = HW["center"][1] + HW["half"][1]
ARENA = next(r for r in CFG["rooms"] if r["name"] == "arena")


def smoothstep(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def masonry_weight(x, y, z):
    """1 where the surface is built stone, 0 where it is rock, smooth between."""
    zf = smoothstep(MAS["facade_z"] - 0.5, MAS["facade_z"] + 0.1, z)
    yf = 1 - smoothstep(MAS["top_m"] - 0.3, MAS["top_m"] + 0.3, y)
    return zf * yf


def doorway_calm(x, y, z):
    """0 at the hollow wall's doorway frame, so the rock doesn't open gaps around the plug."""
    if y > HW_TOP + 1.2 or z < HW_Z0 - 1.5 or z > HW_Z1 + 1.5 or x > HW_X1 + 1.5:
        return 1.0
    dx = abs(x - (HW_X0 + HW_X1) / 2)
    return smoothstep(0.35, 1.2, dx) if x > HW_X0 - 6.5 else 1.0


def rock_push(p):
    """Outward push (into the rock) in meters, before weights: -0.15 to about 1.0."""
    v = Vector(p)
    n1 = noise.fractal(v * 0.16, 1.0, 2.0, 3)
    n2 = noise.noise(v * 0.85 + Vector((7.1, 3.3, 1.7)))
    n3 = noise.noise(v * 2.6 + Vector((1.9, 8.2, 4.4)))
    s = p[1] * 1.35 + n1 * 0.9
    strata = 0.16 * smoothstep(0.62, 0.92, s - math.floor(s))
    d = 0.38 + 0.42 * n1 + 0.14 * n2 + 0.05 * n3 + strata
    return max(-0.15, min(1.05, d))


def displace_cavity(bm):
    for v in bm.verts:
        x, y, z = b2g(v.co)
        out = -Vector(b2g(v.normal))  # Normals face the room; push the other way.
        w = (1 - masonry_weight(x, y, z)) * doorway_calm(x, y, z)
        if w <= 0:
            continue
        d = rock_push((x, y, z))
        if out[1] > 0.6:  # Ceiling: deeper, lumpier.
            d += 0.35 + 0.55 * noise.noise(Vector((x, y, z)) * 0.3)
        # Low on the walls the rock may lean in a little but never past -0.15 m.
        d = max(-0.15, d) * w
        q = Vector((x, y, z)) + out * d
        v.co = g2b(q)
    bm.normal_update()


# --------------------------------------------------------------------------------------
# Material helpers for faces

def face_material(center, normal):
    x, y, z = center
    if masonry_weight(x, y, z) > 0.5:
        return "masonry"
    return "rock"


def split_bmesh_into_store(bm, chooser):
    """Moves a bmesh's faces into MESHES by section and material."""
    groups = {}
    for f in bm.faces:
        c = b2g(f.calc_center_median())
        n = b2g(f.normal)
        key = (section_of(c[2]), chooser(c, n))
        groups.setdefault(key, []).append(f)
    for key, faces in groups.items():
        s = store(*key)
        index = {}
        for f in faces:
            ids = []
            for v in f.verts:
                if v.index not in index:
                    index[v.index] = len(s["v"])
                    s["v"].append(b2g(v.co))
                ids.append(index[v.index])
            s["f"].append(tuple(ids))


# --------------------------------------------------------------------------------------
# 2. Floors


def build_floors():
    step = 0.5
    xs = [x * step for x in range(int(-22 / step), int(18 / step) + 1)]
    zs = [z * step for z in range(int(-12 / step), int(120 / step) + 1)]
    height = {}

    def h(x, z):
        key = (x, z)
        if key not in height:
            d = dist_to_open(x, z)
            n = noise.noise(Vector((x * 0.7, z * 0.7, 3.1)))
            if d <= 0.0:
                # Walkable: within 2 cm of the collider's top.
                y = 0.012 * n - 0.004
            else:
                y = 0.32 * smoothstep(0.1, 1.2, d) + 0.05 * n
            height[key] = y
        return height[key]

    for x in xs[:-1]:
        for z in zs[:-1]:
            cx, cz = x + step / 2, z + step / 2
            if dist_to_open(cx, cz) > 1.2:
                continue
            mat = "flagstone" if (cz > MAS["facade_z"] + 0.4 and is_open(cx, cz)) or (cz > 98.0 and dist_to_open(cx, cz) < 1.3) else "dirt"
            verts = [(x, h(x, z), z), (x, h(x, z + step), z + step), (x + step, h(x + step, z + step), z + step), (x + step, h(x + step, z), z)]
            add_mesh(section_of(cz), mat, verts, [(0, 1, 2, 3)])


# --------------------------------------------------------------------------------------
# 3. Set pieces


def build_ledge():
    b = BLOCKS["jump_ledge"]
    x0, x1, y0, y1, z0, z1 = block_box(b)
    x0 -= 0.7
    x1 += 0.7
    nx, nz = int((x1 - x0) / 0.25), int((z1 - z0) / 0.25)
    verts, faces = [], []
    # Top: flat at the collider's height, rounded only at the very edge.
    grid = {}
    for i in range(nx + 1):
        for k in range(nz + 1):
            x = x0 + (x1 - x0) * i / nx
            z = z0 + (z1 - z0) * k / nz
            grid[(i, k)] = len(verts)
            verts.append((x, y1, z))
    for i in range(nx):
        for k in range(nz):
            faces.append((grid[(i, k)], grid[(i, k + 1)], grid[(i + 1, k + 1)], grid[(i + 1, k)]))
    add_mesh(section_of((z0 + z1) / 2), "rock", verts, faces)
    # Faces at z0 and z1: rough, within 6 cm of the collider face.
    for zf, sgn in ((z0, -1), (z1, 1)):
        verts, faces = [], []
        ny = 4
        idx = {}
        for i in range(nx + 1):
            for j in range(ny + 1):
                x = x0 + (x1 - x0) * i / nx
                y = -0.05 + (y1 - -0.05) * j / ny
                edge = 1 - smoothstep(y1 - 0.12, y1, y)
                push = (0.03 + 0.03 * noise.noise(Vector((x * 2.2, y * 2.2, zf)))) * edge
                idx[(i, j)] = len(verts)
                verts.append((x, y, zf + sgn * push if j < ny else zf))
        for i in range(nx):
            for j in range(ny):
                q = (idx[(i, j)], idx[(i + 1, j)], idx[(i + 1, j + 1)], idx[(i, j + 1)])
                faces.append(q if sgn > 0 else tuple(reversed(q)))
        add_mesh(section_of(zf), "rock", verts, faces)
    # Broken stones at the foot of the step, against the walls.
    for side in (-1, 1):
        for _ in range(3):
            x = side * rng.uniform(1.55, 1.95)
            z = rng.choice((z0 - rng.uniform(0.1, 0.4), z1 + rng.uniform(0.1, 0.4)))
            s = rng.uniform(0.12, 0.22)
            v, f = blob((x, s * 0.4, z), (s, s * 0.7, s), 0, 0.3, flat_bottom=-0.02)
            add_mesh(section_of(z), "rock", v, f)


def pillar_names():
    return sorted(n for n in BLOCKS if n.startswith("pillar_"))


def build_pillars():
    for name in pillar_names():
        b = BLOCKS[name]
        x0, x1, y0, y1, z0, z1 = block_box(b)
        cx, cz = (x0 + x1) / 2, (z0 + z1) / 2
        half = (x1 - x0) / 2
        sec = section_of(cz)
        # Plinth: the collider's full footprint.
        verts, faces = [], []
        bm = bmesh.new()
        box_bmesh(bm, x0, x1, -0.02, 0.35, z0, z1)
        bm.verts.index_update()
        add_mesh(sec, "masonry", [b2g(v.co) for v in bm.verts], [tuple(v.index for v in f.verts) for f in bm.faces])
        bm.free()
        # Shaft: an octagon, broken off unevenly near the collider's top.
        sides = 8
        r = half * 0.84
        levels = [0.35, 0.6, 1.4, 2.2, 3.0]
        top = [y1 - rng.uniform(0.1, 0.65) for _ in range(sides)]
        pts = []
        for lv in levels:
            pts.append([(cx + math.cos(2 * math.pi * s / sides + math.pi / 8) * r * (1.06 if lv < 0.7 else 1), lv, cz + math.sin(2 * math.pi * s / sides + math.pi / 8) * r * (1.06 if lv < 0.7 else 1)) for s in range(sides)])
        pts.append([(cx + math.cos(2 * math.pi * s / sides + math.pi / 8) * r * rng.uniform(0.9, 1.0), top[s], cz + math.sin(2 * math.pi * s / sides + math.pi / 8) * r * rng.uniform(0.9, 1.0)) for s in range(sides)])
        verts = [p for row in pts for p in row]
        faces = []
        for L in range(len(pts) - 1):
            for s in range(sides):
                a, bb = L * sides + s, L * sides + (s + 1) % sides
                faces.append((a, a + sides, bb + sides, bb))
        last = (len(pts) - 1) * sides
        cap = len(verts)
        verts.append((cx, min(top) - 0.05, cz))
        for s in range(sides):
            faces.append((last + s, cap, last + (s + 1) % sides))
        add_mesh(sec, "masonry", verts, faces)
        # A fallen piece of the shaft at its foot, inside the plinth's footprint.
        v, f = blob((cx + rng.uniform(-0.2, 0.2), 0.42, cz + rng.uniform(-0.2, 0.2)), (0.16, 0.08, 0.13), 0, 0.2)
        add_mesh(sec, "masonry", v, f)


def build_arena_ring():
    """A ring of laid stone in the arena floor: the fight's circle, flush with the floor."""
    cx = (ARENA["x"][0] + ARENA["x"][1]) / 2
    cz = (ARENA["z"][0] + ARENA["z"][1]) / 2
    r0, r1, n = 7.3, 7.75, 72
    verts, faces = [], []
    for i in range(n):
        a = 2 * math.pi * i / n
        verts.append((cx + math.cos(a) * r0, 0.012, cz + math.sin(a) * r0))
        verts.append((cx + math.cos(a) * r1, 0.012, cz + math.sin(a) * r1))
    for i in range(n):
        j = (i + 1) % n
        faces.append((2 * i, 2 * j, 2 * j + 1, 2 * i + 1))
    add_mesh(section_of(cz), "masonry", verts, faces)
    # A worn seal at the center.
    verts, faces = [(cx, 0.014, cz)], []
    for i in range(24):
        a = 2 * math.pi * i / 24
        verts.append((cx + math.cos(a) * 1.4, 0.014, cz + math.sin(a) * 1.4))
    for i in range(24):
        faces.append((0, 1 + (i + 1) % 24, 1 + i))
    add_mesh(section_of(cz), "masonry", verts, faces)


# --------------------------------------------------------------------------------------
# Dressing (needs the displaced cavity for ray casts)


def cast(bvh, origin, direction, dist=30.0):
    hit = bvh.ray_cast(g2b(origin), g2b(direction).normalized(), dist)
    if hit[0] is None:
        return None, None
    return b2g(hit[0]), b2g(hit[1])


def open_points(per_m2, rooms=None, margin=0.6):
    """Random points over open cells, at `per_m2` density."""
    out = []
    for i, k in sorted(OPEN):
        x0, z0 = i * CELL, k * CELL
        room = room_at(x0 + 1, z0 + 1)
        density = per_m2.get(room, 0.0) if isinstance(per_m2, dict) else per_m2
        if rooms is not None and room not in rooms:
            continue
        expect = density * CELL * CELL
        n = int(expect) + (1 if rng.random() < expect - int(expect) else 0)
        for _ in range(n):
            out.append((x0 + rng.uniform(0.1, CELL - 0.1), z0 + rng.uniform(0.1, CELL - 0.1)))
    return out


def wall_edges():
    """Boundary edges between open cells and wall cells: (x, z, nx, nz, length) midpoints."""
    edges = []
    for i, k in sorted(OPEN):
        for di, dk in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if (i + di, k + dk) in OPEN:
                continue
            cx, cz = i * CELL + 1 + di, k * CELL + 1 + dk
            edges.append((cx, cz, di, dk))
    return edges


def build_stalactites(bvh):
    n = 0
    for x, z in open_points(CFG["dressing"]["stalactites_per_m2"]):
        hit, nrm = cast(bvh, (x, 2.0, z), (0, 1, 0))
        if hit is None or nrm[1] > -0.3:
            continue
        room = room_at(x, z)
        floor_clear = 3.3 if room in ("first_fight", "hall", "twist", "arena") else 3.0
        room_l = hit[1] - floor_clear
        if room_l < 0.35:
            continue
        L = min(room_l, rng.uniform(0.35, 2.2) * (1.4 if rng.random() < 0.15 else 1))
        r = rng.uniform(0.07, 0.13) * (0.6 + L * 0.5)
        pts = [(hit[0], hit[1] + 0.35, hit[2])]
        rad = [r * 1.25]
        segs = 4
        bend = (rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08))
        for s in range(1, segs + 1):
            t = s / segs
            pts.append((hit[0] + bend[0] * t * t, hit[1] - L * t, hit[2] + bend[1] * t * t))
            rad.append(max(0.008, r * (1 - t) ** 1.3))
        v, f = tube(pts, rad, sides=6, cap_start=False, cap_end=True, jitter=0.12)
        add_mesh(section_of(z), "rock", v, f)
        n += 1
    print(f"build_cave: {n} stalactites")


def build_wall_dressing(bvh):
    """Stalagmites and rubble along the wall bases, and bones near them."""
    d = CFG["dressing"]
    stal = rub = 0
    for cx, cz, di, dk in wall_edges():
        room = room_at(cx - di * 0.5, cz - dk * 0.5)
        # The collider face runs through (cx, cz), along the cell edge.
        tx, tz = -dk, di
        # Rubble along this 2 m edge.
        n = int(d["rubble_per_m"] * CELL * rng.uniform(0.5, 1.5))
        if room == "corr_3":
            n *= 2
        for _ in range(n):
            t = rng.uniform(-1, 1)
            into = rng.uniform(-0.25, 0.6)  # Positive: into the wall.
            s = rng.uniform(0.05, 0.16) if into < 0 else rng.uniform(0.1, 0.4)
            px, pz = cx + tx * t + di * into, cz + tz * t + dk * into
            if HW_X0 - 0.8 < px < HW_X1 + 0.8 and HW_Z0 - 0.4 < pz < HW_Z1 + 0.4:
                continue
            v, f = blob((px, s * 0.35, pz), (s, s * rng.uniform(0.5, 0.8), s * rng.uniform(0.8, 1.2)), 0, 0.3, flat_bottom=-0.03)
            add_mesh(section_of(pz), "rock", v, f)
            rub += 1
        # Stalagmites, set into the wall so they stand out of the rock face.
        if room not in ("arena", "gate_neck", "closet") and rng.random() < d["stalagmites_per_m"] * CELL:
            t = rng.uniform(-0.8, 0.8)
            into = rng.uniform(0.1, 0.45)
            px, pz = cx + tx * t + di * into, cz + tz * t + dk * into
            if masonry_weight(px, 0.5, pz) > 0.1:
                continue
            H = rng.uniform(0.5, 2.1)
            r = rng.uniform(0.16, 0.34)
            if r - into > 0.3:
                r = into + 0.3
            pts = [(px, -0.1, pz)]
            rad = [r]
            for s in range(1, 5):
                tt = s / 4
                pts.append((px + rng.uniform(-0.03, 0.03), H * tt, pz + rng.uniform(-0.03, 0.03)))
                rad.append(max(0.01, r * (1 - tt) ** 1.2))
            v, f = tube(pts, rad, sides=6, cap_start=True, cap_end=True, jitter=0.15)
            add_mesh(section_of(pz), "rock", v, f)
            stal += 1
    print(f"build_cave: {rub} rubble stones, {stal} stalagmites")


def build_collapse(bvh):
    c = CFG["collapse"]
    room = next(r for r in CFG["rooms"] if r["name"] == c["room"])
    x0, x1 = room["x"]
    z0, z1 = room["z"]
    # Boulders heaped against both walls, poking at most 0.3 m into the corridor.
    for i in range(c["boulders"]):
        side = -1 if i % 2 == 0 else 1
        r = rng.uniform(0.45, 0.95)
        face = x0 if side < 0 else x1
        px = face + side * (r - 0.3)
        pz = rng.uniform(z0 - 0.3, z1 + 0.3)
        v, f = blob((px, r * 0.55, pz), (r, r * 0.8, r * 1.1), 1, 0.25, flat_bottom=-0.05)
        add_mesh(section_of(pz), "rock", v, f)
    # A slab fallen from the roof, wedged across the corridor above head height.
    a, b = Vector(c["slab"]["from"]), Vector(c["slab"]["to"])
    axis = (b - a).normalized()
    side = axis.cross(Vector((0, 1, 0))).normalized()
    w = c["slab"]["width_m"] / 2
    th = 0.35
    bm = bmesh.new()
    r = bmesh.ops.create_cube(bm, size=1.0)
    for v in r["verts"]:
        t, s, h = v.co.x + 0.5, (1 if v.co.y > 0 else -1), v.co.z + 0.5
        v.co = g2b(a.lerp(b, t) + side * s * w + Vector((0, 1, 0)) * h * th)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.verts.index_update()
    add_mesh(section_of((a.z + b.z) / 2), "rock", [b2g(v.co) for v in bm.verts], [tuple(v.index for v in f.verts) for f in bm.faces])
    bm.free()


def build_roots(bvh):
    n = 0
    for x, z in open_points(CFG["dressing"]["roots_per_m2"]):
        hit, nrm = cast(bvh, (x, 2.0, z), (0, 1, 0))
        if hit is None:
            continue
        clear = 3.0
        L = min(hit[1] - clear, rng.uniform(0.8, 3.2))
        if L < 0.5:
            continue
        strands = rng.choice((1, 1, 2, 3))
        for _ in range(strands):
            ox, oz = rng.uniform(-0.25, 0.25), rng.uniform(-0.25, 0.25)
            segs = 6
            sway = (rng.uniform(-0.35, 0.35), rng.uniform(-0.35, 0.35))
            Ls = L * rng.uniform(0.55, 1.0)
            pts, rad = [], []
            r0 = rng.uniform(0.03, 0.07)
            for s in range(segs + 1):
                t = s / segs
                pts.append((hit[0] + ox + sway[0] * math.sin(t * 2.4), hit[1] + 0.3 - (Ls + 0.3) * t, hit[2] + oz + sway[1] * math.sin(t * 2.4)))
                rad.append(max(0.006, r0 * (1 - 0.85 * t)))
            v, f = tube(pts, rad, sides=5, cap_start=False, cap_end=True)
            add_mesh(section_of(z), "bark", v, f)
            n += 1
    print(f"build_cave: {n} hanging roots")


def bone_pile(x, z):
    sec = section_of(z)
    # Long bones.
    for _ in range(rng.randint(2, 5)):
        a = rng.uniform(0, math.pi)
        L = rng.uniform(0.25, 0.48)
        cx, cz = x + rng.uniform(-0.35, 0.35), z + rng.uniform(-0.35, 0.35)
        dx, dz = math.cos(a) * L / 2, math.sin(a) * L / 2
        v, f = tube([(cx - dx, 0.025, cz - dz), (cx, 0.022, cz), (cx + dx, 0.025, cz + dz)], [0.022, 0.016, 0.022], sides=5)
        add_mesh(sec, "bone", v, f)
        for e in (-1, 1):
            v, f = blob((cx + e * dx, 0.03, cz + e * dz), (0.035, 0.03, 0.035), 0, 0.15, flat_bottom=0.0)
            add_mesh(sec, "bone", v, f)
    # A skull, half sunk.
    sx, sz = x + rng.uniform(-0.2, 0.2), z + rng.uniform(-0.2, 0.2)
    v, f = blob((sx, 0.07, sz), (0.1, 0.085, 0.12), 1, 0.06, flat_bottom=0.0)
    add_mesh(sec, "bone", v, f)
    # Ribs: arcs.
    if rng.random() < 0.7:
        a0 = rng.uniform(0, math.pi)
        for i in range(4):
            off = (i - 1.5) * 0.06
            pts = []
            for s in range(5):
                t = s / 4
                ang = a0 + math.pi * 0.5 * (t - 0.5)
                pts.append((x + 0.3 + math.cos(a0) * off + math.cos(ang) * 0.18, 0.02 + 0.06 * math.sin(t * math.pi), z + math.sin(a0) * off + math.sin(ang) * 0.18))
            v, f = tube(pts, [0.008] * 5, sides=4)
            add_mesh(sec, "bone", v, f)


def build_bones():
    for x, z in CFG["dressing"]["bones"]:
        bone_pile(x, z)


LIGHTS = []


def light_entry(name, pos, spec, override=None):
    s = dict(spec)
    if override:
        s.update(override)
    LIGHTS.append(
        {
            "name": name,
            "pos": [round(p, 3) for p in pos],
            "color": s["color"],
            "intensity": s["intensity"],
            "distance": s["distance"],
            "flicker": s.get("flicker", 0.0),
        }
    )


def flame(sec, x, y, z, h, r):
    verts = [(x, y + h, z), (x, y - h * 0.25, z)] + [(x + math.cos(a) * r, y + h * 0.15, z + math.sin(a) * r) for a in (0, 2.1, 4.2)]
    faces = [(0, 3, 2), (0, 4, 3), (0, 2, 4), (1, 2, 3), (1, 3, 4), (1, 4, 2)]
    add_mesh(sec, "flame", verts, faces)


def build_torches(bvh):
    spec = CFG["torch_light"]
    for t in CFG["torches"]:
        fx, fz = t["from"]
        dx, dz = t["dir"]
        hit, nrm = cast(bvh, (fx, 2.3, fz), (dx, 0, dz))
        if hit is None:
            raise SystemExit(f"build_cave: torch {t['name']} finds no wall")
        sec = section_of(hit[2])
        out = Vector((-dx, 0, -dz))
        base = Vector(hit) - Vector((dx, 0, dz)) * 0.04
        # Iron plate and arm.
        side = out.cross(Vector((0, 1, 0))).normalized()
        plate = [base + side * s * 0.07 + Vector((0, 1, 0)) * h * 0.22 - Vector((0, 0.11, 0)) for s in (-1, 1) for h in (0, 1)]
        plate_back = [p + Vector((dx, 0, dz)) * 0.25 for p in plate]
        verts = [tuple(p) for p in plate + plate_back]
        faces = [(0, 2, 3, 1), (4, 5, 7, 6), (0, 1, 5, 4), (2, 6, 7, 3), (0, 4, 6, 2), (1, 3, 7, 5)]
        add_mesh(sec, "iron", verts, faces)
        cup = base + out * 0.24 + Vector((0, 0.12, 0))
        v, f = tube([tuple(base + Vector((0, -0.04, 0))), tuple(base + out * 0.12 + Vector((0, -0.02, 0))), tuple(cup - Vector((0, 0.06, 0)))], [0.018, 0.018, 0.018], sides=5)
        add_mesh(sec, "iron", v, f)
        v, f = tube([tuple(cup - Vector((0, 0.08, 0))), tuple(cup + Vector((0, 0.04, 0)))], [0.035, 0.06], sides=6, cap_end=False)
        add_mesh(sec, "iron", v, f)
        # The torch: a wrapped stick leaning out, and its flame.
        tip = cup + out * 0.1 + Vector((0, 0.32, 0))
        v, f = tube([tuple(cup - Vector((0, 0.18, 0)) - out * 0.04), tuple(tip)], [0.022, 0.034], sides=6)
        add_mesh(sec, "bark", v, f)
        flame(sec, tip.x, tip.y + 0.06, tip.z, 0.22, 0.06)
        flame(sec, tip.x + 0.02, tip.y + 0.04, tip.z - 0.02, 0.15, 0.045)
        lp = tip + out * 0.3 + Vector((0, 0.15, 0))
        override = {k: t[k] for k in ("intensity", "flicker", "distance") if k in t}
        light_entry(t["name"], tuple(lp), spec, override)


def build_candles():
    spec = CFG["candle_light"]
    for c in CFG["candles"]:
        if "pillar" in c:
            b = BLOCKS[c["pillar"]]
            x0, x1, y0, y1, z0, z1 = block_box(b)
            cx, cz, base_y = (x0 + x1) / 2, (z0 + z1) / 2, 0.35
            spots = [(cx + sx * 0.38, cz + sz * 0.38) for sx in (-1, 1) for sz in (-1, 1)]
            rng.shuffle(spots)
            spots = spots[: c["count"]]
        else:
            cx, _, cz = c["pos"]
            base_y = 0.0
            spots = [(cx + rng.uniform(-0.35, 0.35), cz + rng.uniform(-0.35, 0.35)) for _ in range(c["count"])]
        sec = section_of(cz)
        top = 0.0
        for x, z in spots:
            h = rng.uniform(0.07, 0.3)
            r = rng.uniform(0.022, 0.045)
            v, f = tube([(x, base_y - 0.01, z), (x, base_y + h, z)], [r * 1.15, r], sides=7)
            add_mesh(sec, "wax", v, f)
            # A puddle of old wax.
            v, f = blob((x, base_y + 0.004, z), (r * 2.4, 0.012, r * 2.4), 0, 0.25, flat_bottom=base_y - 0.005)
            add_mesh(sec, "wax", v, f)
            flame(sec, x, base_y + h + 0.035, z, 0.045, 0.014)
            top = max(top, base_y + h)
        light = c.get("light", True)
        if light is not False:
            override = light if isinstance(light, dict) else None
            light_entry(c["name"], (cx, top + 0.45, cz), spec, override)
    for e in CFG["extra_lights"]:
        light_entry(e["name"], e["pos"], e)


def build_hollow_wall():
    """The bricked-up closet wall, in its own frame (centered on the collider)."""
    hx, hy, hz = HW["half"]
    face = hx  # Local +X faces the hall.
    width = hz * 2 + 0.3
    height = hy * 2 + 0.2
    verts, faces, cols = [], [], []
    y = -hy - 0.1
    row = 0
    crack = [(0.2, -hy), (-0.3, -0.6), (0.25, 0.1), (-0.15, 0.8), (0.35, hy)]

    def crack_dist(zc, yc):
        best = 9.0
        for (z0, y0), (z1, y1) in zip(crack, crack[1:]):
            t = max(0.0, min(1.0, ((zc - z0) * (z1 - z0) + (yc - y0) * (y1 - y0)) / ((z1 - z0) ** 2 + (y1 - y0) ** 2)))
            best = min(best, math.hypot(zc - (z0 + t * (z1 - z0)), yc - (y0 + t * (y1 - y0))))
        return best

    while y < hy + 0.1 - 0.05:
        bh = min(rng.uniform(0.34, 0.46), hy + 0.1 - y)
        z = -width / 2 + (rng.uniform(0, 0.3) if row % 2 else 0)
        while z < width / 2 - 0.05:
            bw = min(rng.uniform(0.45, 0.8), width / 2 - z)
            zc, yc = z + bw / 2, y + bh / 2
            cd = crack_dist(zc, yc)
            sink = rng.uniform(-0.02, 0.03) + (0.05 if cd < 0.3 else 0)
            tilt = rng.uniform(-0.02, 0.02) + (rng.uniform(-0.05, 0.05) if cd < 0.3 else 0)
            g = 0.018
            x1 = face + 0.04 - sink
            x0 = -face
            b = [
                (x0, y + g, z + g), (x0, y + bh - g, z + g), (x0, y + bh - g, z + bw - g), (x0, y + g, z + bw - g),
                (x1, y + g, z + g + tilt), (x1 + tilt, y + bh - g, z + g), (x1, y + bh - g, z + bw - g - tilt), (x1 - tilt, y + g, z + bw - g),
            ]
            base = len(verts)
            verts.extend(b)
            dark = 0.35 + 0.65 * smoothstep(0.05, 0.4, cd)
            col = rng.uniform(0.78, 1.0) * dark
            cols.extend([col] * 8)
            for q in ((4, 5, 6, 7), (0, 3, 2, 1), (0, 1, 5, 4), (3, 7, 6, 2), (1, 2, 6, 5), (0, 4, 7, 3)):
                faces.append(tuple(base + i for i in q))
            z += bw
        y += bh
        row += 1
    # Mortar behind the bricks, dark, so the gaps read as depth.
    base = len(verts)
    m = face - 0.06
    verts.extend([(m, -hy - 0.1, -width / 2), (m, hy + 0.1, -width / 2), (m, hy + 0.1, width / 2), (m, -hy - 0.1, width / 2)])
    cols.extend([0.12] * 4)
    faces.append((base, base + 1, base + 2, base + 3))
    return verts, faces, cols


# --------------------------------------------------------------------------------------
# 4. Materials and UVs

TILE = {k: v["tile_m"] for k, v in TEX["materials"].items()}
FLAT = {
    "bone": ((0.62, 0.57, 0.47), 0.82, 0.0),
    "wax": ((0.72, 0.66, 0.52), 0.55, 0.0),
    "iron": ((0.09, 0.085, 0.08), 0.55, 0.75),
}
# Materials whose pieces are all closed solids.
CLOSED = {"iron", "wax", "bone", "flame", "bark"}
TEXTURED = {"rock": "rock", "dirt": "dirt", "masonry": "masonry", "flagstone": "flagstone", "bark": "bark"}


def image(name, colorspace, size=None):
    path = os.path.join(OPTS["textures"], name)
    if not os.path.exists(path):
        raise SystemExit(f"build_cave: {path} is missing; run game/art/cave/fetch_textures.sh")
    img = bpy.data.images.load(path, check_existing=True)
    img.colorspace_settings.name = colorspace
    if size and img.size[0] > size:
        # The exporter writes the scaled pixels (as PNG); the source JPEG is untouched.
        img.scale(size, size)
    return img


def make_material(name):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes.get("Principled BSDF")
    vc = nt.nodes.new("ShaderNodeVertexColor")
    vc.layer_name = "Col"
    if name == "flame":
        bsdf.inputs["Base Color"].default_value = (0, 0, 0, 1)
        bsdf.inputs["Emission Color"].default_value = (1.0, 0.55, 0.18, 1)
        bsdf.inputs["Emission Strength"].default_value = 6.0
        return mat
    if name in FLAT:
        color, rough, metal = FLAT[name]
        # The bake folds this color into the vertex colors, so the material reads them plain.
        nt.links.new(vc.outputs["Color"], bsdf.inputs["Base Color"])
        bsdf.inputs["Roughness"].default_value = rough
        bsdf.inputs["Metallic"].default_value = metal
        return mat
    spec = TEX["materials"][TEXTURED[name]]
    asset = spec["asset"]
    size = spec.get("size_px")
    diff = nt.nodes.new("ShaderNodeTexImage")
    diff.image = image(f"{asset}_diff_1k.jpg", "sRGB", size)
    mix = nt.nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    mix.blend_type = "MULTIPLY"
    mix.inputs[0].default_value = 1.0
    nt.links.new(diff.outputs["Color"], mix.inputs[6])
    nt.links.new(vc.outputs["Color"], mix.inputs[7])
    nt.links.new(mix.outputs[2], bsdf.inputs["Base Color"])
    arm = nt.nodes.new("ShaderNodeTexImage")
    arm.image = image(f"{asset}_arm_1k.jpg", "Non-Color", size)
    sep = nt.nodes.new("ShaderNodeSeparateColor")
    nt.links.new(arm.outputs["Color"], sep.inputs["Color"])
    nt.links.new(sep.outputs["Green"], bsdf.inputs["Roughness"])
    nt.links.new(sep.outputs["Blue"], bsdf.inputs["Metallic"])
    nor = nt.nodes.new("ShaderNodeTexImage")
    nor.image = image(f"{asset}_nor_gl_1k.jpg", "Non-Color", size)
    nmap = nt.nodes.new("ShaderNodeNormalMap")
    nt.links.new(nor.outputs["Color"], nmap.inputs["Color"])
    nt.links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])
    return mat


MATERIALS = {}


def material(name):
    if name not in MATERIALS:
        MATERIALS[name] = make_material(name)
    return MATERIALS[name]


def project_uvs(bm, name, origin=(0.0, 0.0, 0.0)):
    """World-space box projection: each face takes the plane its normal faces most."""
    tile = TILE.get(TEXTURED.get(name, ""), 2.0)
    uv = bm.loops.layers.uv.verify()
    for f in bm.faces:
        n = b2g(f.normal)
        ax = max(range(3), key=lambda i: abs(n[i]))
        for loop in f.loops:
            x, y, z = b2g(loop.vert.co)
            x, y, z = x + origin[0], y + origin[1], z + origin[2]
            if ax == 1:
                u, v = x, z
            elif ax == 0:
                u, v = z * (1 if n[0] > 0 else -1), y
            else:
                u, v = x * (-1 if n[2] > 0 else 1), y
            loop[uv].uv = (u / tile, v / tile)


def make_object(name, verts, faces, mat_name, cols=None, origin=(0.0, 0.0, 0.0)):
    bm = bmesh.new()
    vs = [bm.verts.new(g2b(v)) for v in verts]
    for f in faces:
        try:
            bm.faces.new([vs[i] for i in f])
        except ValueError:
            pass  # A duplicate face from a seam; skip it.
    if cols is None:
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=0.0005)
    if mat_name in CLOSED:
        # Every piece of these is a closed solid: let Blender point the faces outward.
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.normal_update()
    project_uvs(bm, mat_name, origin)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = mat_name not in ("iron",)
    obj = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(obj)
    me.materials.append(material(mat_name))
    attr = me.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
    data = [1.0] * (len(me.vertices) * 4)
    if cols is not None:
        data = []
        for c in cols[: len(me.vertices)]:
            data.extend((c, c, c, 1.0))
        data.extend([1.0] * (len(me.vertices) * 4 - len(data)))
    attr.data.foreach_set("color", data)
    me.color_attributes.active_color = attr
    obj["lf_material"] = mat_name
    return obj


# --------------------------------------------------------------------------------------
# 5. Bake


def bake(objects):
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 48
    scene.render.bake.target = "VERTEX_COLORS"
    world = bpy.data.worlds.new("bake_world")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.0
    world.light_settings.distance = 1.6
    scene.world = world
    grey = bpy.data.materials.new("bake_grey")
    grey.use_nodes = True
    grey.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.32, 0.3, 0.28, 1)
    grey.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
    saved = {}
    for o in objects:
        saved[o.name] = list(o.data.materials)
        o.data.materials.clear()
        o.data.materials.append(grey)
    lamps = []
    for L in LIGHTS:
        ld = bpy.data.lights.new(L["name"], "POINT")
        ld.energy = L["intensity"] * 45.0
        ld.shadow_soft_size = 0.15
        c = L["color"].lstrip("#")
        ld.color = tuple(int(c[i : i + 2], 16) / 255 for i in (0, 2, 4))
        lo = bpy.data.objects.new(L["name"], ld)
        lo.location = g2b(L["pos"])
        scene.collection.objects.link(lo)
        lamps.append(lo)
    # The Lantern: its light in the bake, the model drawn by the game.
    lan = ENT["lantern"]
    ll = CFG["lantern_light"]
    ld = bpy.data.lights.new("lantern", "POINT")
    ld.energy = ll["intensity"] * 45.0
    c = ll["color"].lstrip("#")
    ld.color = tuple(int(c[i : i + 2], 16) / 255 for i in (0, 2, 4))
    lo = bpy.data.objects.new("lantern", ld)
    lo.location = g2b((lan[0] + ll["at"][0], lan[1] + ll["at"][1], lan[2] + ll["at"][2]))
    scene.collection.objects.link(lo)
    lamps.append(lo)

    targets = [o for o in objects if o["lf_material"] != "flame"]
    for o in targets:
        me = o.data
        for nm in ("ao", "light"):
            a = me.color_attributes.new(nm, "FLOAT_COLOR", "POINT")
    bpy.ops.object.select_all(action="DESELECT")
    for o in targets:
        o.select_set(True)
    bpy.context.view_layer.objects.active = targets[0]

    def run(kind, attr, **kw):
        for o in targets:
            o.data.color_attributes.active_color = o.data.color_attributes[attr]
        print(f"build_cave: baking {kind} into {len(targets)} objects")
        bpy.ops.object.bake(type=kind, target="VERTEX_COLORS", **kw)

    run("AO", "ao")
    run("DIFFUSE", "light", pass_filter={"DIRECT", "INDIRECT"})

    # Map the light to a 0..1 range: the 92nd percentile of lit vertices reaches 0.9.
    lum = []
    reads = {}
    for o in targets:
        me = o.data
        n = len(me.vertices)
        ao = [0.0] * (n * 4)
        li = [0.0] * (n * 4)
        me.color_attributes["ao"].data.foreach_get("color", ao)
        me.color_attributes["light"].data.foreach_get("color", li)
        reads[o.name] = (ao, li)
        for i in range(n):
            L = 0.2126 * li[4 * i] + 0.7152 * li[4 * i + 1] + 0.0722 * li[4 * i + 2]
            if L > 1e-4:
                lum.append(L)
    lum.sort()
    ref = lum[int(len(lum) * 0.92)] if lum else 1.0
    k = -math.log(0.1) / max(ref, 1e-6)
    floor_light = CFG["bake"]["dark_floor"]
    for o in targets:
        me = o.data
        ao, li = reads[o.name]
        n = len(me.vertices)
        flat = FLAT.get(o["lf_material"])
        out = [1.0] * (n * 4)
        for i in range(n):
            r, g, b = li[4 * i], li[4 * i + 1], li[4 * i + 2]
            L = 0.2126 * r + 0.7152 * g + 0.0722 * b
            v = 1 - math.exp(-k * L)
            chroma = (r / L, g / L, b / L) if L > 1e-5 else (1.0, 1.0, 1.0)
            # Small props (bones, wax) lie in their own occlusion; soften it for them.
            a = max(0.0, min(1.0, ao[4 * i])) ** (0.5 if flat else 1.3)
            base = floor_light + (1 - floor_light) * v
            for c in range(3):
                tint = 1 + (min(chroma[c], 1.6) - 1) * 0.45 * v
                val = a * base * tint
                if flat:
                    val *= flat[0][c]
                out[4 * i + c] = min(1.0, val)
        me.color_attributes["Col"].data.foreach_set("color", out)
        me.color_attributes.active_color = me.color_attributes["Col"]
        for nm in ("ao", "light"):
            me.color_attributes.remove(me.color_attributes[nm])
    for o in objects:
        o.data.materials.clear()
        for m in saved[o.name]:
            o.data.materials.append(m)
    for lo in lamps:
        bpy.data.objects.remove(lo)


def flat_colors(objects):
    """Without a bake: plain white, or the flat material's color."""
    for o in objects:
        flat = FLAT.get(o["lf_material"])
        if not flat:
            continue
        me = o.data
        out = []
        for _ in range(len(me.vertices)):
            out.extend((*flat[0], 1.0))
        me.color_attributes["Col"].data.foreach_set("color", out)


# --------------------------------------------------------------------------------------
# 6. Export


def export(objects, path):
    bpy.ops.object.select_all(action="DESELECT")
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    os.makedirs(os.path.dirname(path), exist_ok=True)
    kw = dict(
        filepath=path,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_yup=True,
        export_texcoords=True,
        export_normals=True,
        export_tangents=False,
        export_materials="EXPORT",
        export_image_format="AUTO",
        export_cameras=False,
        export_lights=False,
        export_animations=False,
        export_extras=False,
    )
    try:
        bpy.ops.export_scene.gltf(export_vertex_color="ACTIVE", **kw)
    except TypeError:
        bpy.ops.export_scene.gltf(**kw)
    print(f"build_cave: wrote {path} ({os.path.getsize(path)} bytes)")


def tri_count(objects):
    return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objects)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    print("build_cave: carving the rooms")
    bm = build_cavity()
    displace_cavity(bm)
    # A BVH of the rock for the dressing's ray casts.
    bvh = BVHTree.FromBMesh(bm)
    split_bmesh_into_store(bm, face_material)
    bm.free()
    build_floors()
    build_ledge()
    build_pillars()
    build_arena_ring()
    build_stalactites(bvh)
    build_wall_dressing(bvh)
    build_collapse(bvh)
    build_roots(bvh)
    build_bones()
    build_torches(bvh)
    build_candles()

    objects = []
    for (sec, mat), m in sorted(MESHES.items()):
        objects.append(make_object(f"{sec}_{mat}", m["v"], m["f"], mat))
    print(f"build_cave: {len(objects)} mesh nodes, {tri_count(objects)} triangles")
    if OPTS["bake"]:
        bake(objects)
    else:
        flat_colors(objects)
    export(objects, os.path.join(OUT_DIR, "cave.glb"))

    for o in objects:
        bpy.data.objects.remove(o)
    v, f, cols = build_hollow_wall()
    hw = make_object("hollow_wall", v, f, "masonry", cols=cols, origin=tuple(HW["center"]))
    export([hw], os.path.join(OUT_DIR, "hollow_wall.glb"))

    write_manifest()


def write_manifest():
    covers = sorted(n for n in BLOCKS)
    lan = CFG["lantern_light"]
    art = {
        "format": 1,
        "comment": "Written by game/art/cave/build_cave.py from cave.json; edit those, not this file.",
        "models": {
            "cave": {"file": "kits/waking-hollow/cave.glb", "shadow": False, "nodes": True},
            "hollow_wall": {"file": "kits/waking-hollow/hollow_wall.glb", "shadow": False},
            "lantern": {"file": "kits/waking-hollow/lantern.glb"},
            "chest": {"file": "kits/waking-hollow/chest.glb"},
            "pickup": {"file": "kits/waking-hollow/pickup.glb", "shadow": False},
            "message": {"file": "kits/waking-hollow/message.glb", "shadow": False},
        },
        "props": [{"model": "cave", "pos": [0, 0, 0], "covers": covers}],
        "entities": {
            "lantern": {"model": "lantern", "light": {"color": lan["color"], "intensity": lan["intensity"], "distance": lan["distance"], "at": lan["at"]}},
            "hollow_wall": {"model": "hollow_wall"},
            "chest": {"model": "chest"},
            "pickup": {"model": "pickup"},
            "message": {"model": "message"},
        },
        "lights": LIGHTS,
        "look": {k: v for k, v in CFG["look"].items() if k != "comment"},
        "grade": CFG["grade"],
    }
    os.makedirs(os.path.dirname(ART_JSON), exist_ok=True)
    with open(ART_JSON, "w") as fh:
        json.dump(art, fh, indent=2)
        fh.write("\n")
    print(f"build_cave: wrote {ART_JSON} with {len(LIGHTS)} lights")


main()
