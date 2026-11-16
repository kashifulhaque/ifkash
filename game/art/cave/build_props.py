"""Builds Waking Hollow's entity props in headless Blender: the Lantern, the chest, ground
pickups, and message marks.

Writes one GLB per prop to `game/assets-src/waking-hollow-cave/`, each in its own frame with
the feet at the origin and the front toward +Z (game coordinates: Y up):

- `lantern.glb`: a stone grave-lantern, about 2 m tall, with a lit iron chamber. Its flame
  and panes are emissive, so the game flickers them; the game adds the point light.
- `chest.glb`: an iron-bound wooden chest. The node `lid` has its origin on the hinge (the
  top of the back edge), so the game can swing it open; `body` is everything else.
- `pickup.glb`: a small stoppered clay jar with a few pale crystals spilled beside it. The
  game adds the soft glint.
- `message.glb`: a glowing inscription scrawled on the floor, about 1 m across.

Textures are the CC0 scans in `textures.json`, fetched by `fetch_textures.sh`.

Run (or `mise run art:cave`, which runs this after the cave):
    blender --background --factory-startup --python build_props.py -- [--textures DIR]
"""

import json
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
OUT_DIR = os.path.join(REPO, "game", "assets-src", "waking-hollow-cave")
TEX = json.load(open(os.path.join(HERE, "textures.json")))

argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
TEXTURES = os.path.expanduser("~/cinderheart-art/cave-textures")
if "--textures" in argv:
    TEXTURES = os.path.abspath(os.path.expanduser(argv[argv.index("--textures") + 1]))

rng = random.Random(20261116)


def g2b(p):
    """Game (x, y-up, z) to Blender (x, y, z-up); the glTF exporter turns it back."""
    return Vector((p[0], -p[2], p[1]))


def b2g(v):
    return (v.x, v.z, -v.y)


# --------------------------------------------------------------------------------------
# Materials

FLAT = {
    # name: base color, roughness, metallic, emission color, emission strength
    "flame": ((0, 0, 0), 1.0, 0.0, (1.0, 0.55, 0.18), 8.0),
    "pane": ((0.05, 0.03, 0.01), 0.4, 0.0, (1.0, 0.62, 0.26), 2.2),
    "glyph": ((0, 0, 0), 1.0, 0.0, (1.0, 0.45, 0.14), 1.1),
    "crystal": ((0.75, 0.78, 0.8), 0.3, 0.0, (0.55, 0.62, 0.7), 0.6),
    "clay": ((0.32, 0.2, 0.13), 0.85, 0.0, None, 0.0),
    "cork": ((0.38, 0.28, 0.17), 0.95, 0.0, None, 0.0),
}
MATERIALS = {}


def image(name, colorspace, size):
    path = os.path.join(TEXTURES, name)
    if not os.path.exists(path):
        raise SystemExit(f"build_props: {path} is missing; run game/art/cave/fetch_textures.sh")
    img = bpy.data.images.load(path, check_existing=True)
    img.colorspace_settings.name = colorspace
    if size and img.size[0] > size:
        img.scale(size, size)
    return img


def material(name):
    if name in MATERIALS:
        return MATERIALS[name]
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes.get("Principled BSDF")
    if name in FLAT:
        color, rough, metal, emit, strength = FLAT[name]
        bsdf.inputs["Base Color"].default_value = (*color, 1)
        bsdf.inputs["Roughness"].default_value = rough
        bsdf.inputs["Metallic"].default_value = metal
        if emit:
            bsdf.inputs["Emission Color"].default_value = (*emit, 1)
            bsdf.inputs["Emission Strength"].default_value = strength
    else:
        spec = TEX["materials"][name]
        asset = spec["asset"]
        size = spec.get("size_px", 512)
        diff = nt.nodes.new("ShaderNodeTexImage")
        diff.image = image(f"{asset}_diff_1k.jpg", "sRGB", size)
        nt.links.new(diff.outputs["Color"], bsdf.inputs["Base Color"])
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
    MATERIALS[name] = mat
    return mat


# --------------------------------------------------------------------------------------
# Parts: each part is a bmesh in game coordinates with one material.

PARTS = []  # (node, material, tile_m, bmesh)


def part(node, mat, tile=1.0):
    bm = bmesh.new()
    PARTS.append((node, mat, tile, bm))
    return bm


def box(bm, x0, x1, y0, y1, z0, z1):
    r = bmesh.ops.create_cube(bm, size=1.0)
    for v in r["verts"]:
        v.co = g2b((x0 if v.co.x < 0 else x1, y0 if v.co.z < 0 else y1, z0 if v.co.y > 0 else z1))
    return r["verts"]


def lathe(bm, profile, sides=12, cx=0.0, cz=0.0, phase=0.0):
    """A solid of revolution about the Y axis. `profile` is (radius, y) from bottom to top."""
    rings = []
    for r, y in profile:
        ring = []
        for s in range(sides):
            a = phase + 2 * math.pi * s / sides
            ring.append(bm.verts.new(g2b((cx + math.cos(a) * r, y, cz + math.sin(a) * r))))
        rings.append(ring)
    for i in range(len(rings) - 1):
        for s in range(sides):
            t = (s + 1) % sides
            bm.faces.new((rings[i][s], rings[i][t], rings[i + 1][t], rings[i + 1][s]))
    if profile[0][0] > 0:
        bm.faces.new(rings[0])
    if profile[-1][0] > 0:
        bm.faces.new(list(reversed(rings[-1])))


def bar(bm, a, b, w):
    """A square bar from a to b, `w` thick."""
    a, b = Vector(a), Vector(b)
    d = (b - a).normalized()
    up = Vector((0, 1, 0)) if abs(d.y) < 0.9 else Vector((1, 0, 0))
    u = d.cross(up).normalized() * (w / 2)
    v = d.cross(u).normalized() * (w / 2)
    corners = [a + u + v, a - u + v, a - u - v, a + u - v, b + u + v, b - u + v, b - u - v, b + u - v]
    vs = [bm.verts.new(g2b(c)) for c in corners]
    for f in ((0, 1, 2, 3), (4, 7, 6, 5), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)):
        bm.faces.new([vs[i] for i in f])


def flame(bm, x, y, z, h, r):
    lathe(bm, [(0.0, y - h * 0.2), (r, y + h * 0.15), (r * 0.6, y + h * 0.5), (0.0, y + h)], sides=6, cx=x, cz=z)


def uv_box(bm, tile):
    uv = bm.loops.layers.uv.verify()
    for f in bm.faces:
        n = b2g(f.normal)
        ax = max(range(3), key=lambda i: abs(n[i]))
        for loop in f.loops:
            x, y, z = b2g(loop.vert.co)
            u, v = ((x, z) if ax == 1 else (z, y) if ax == 0 else (x, y))
            loop[uv].uv = (u / tile, v / tile)


def build_objects(prefix):
    """Turns PARTS into one object per node, each part keeping its material."""
    nodes = {}
    for node, mat, tile, bm in PARTS:
        bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=1e-5)
        if mat != "glyph":  # The glyphs are flat strokes, already facing up.
            bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        uv_box(bm, tile)
        me = bpy.data.meshes.new(f"{prefix}_{node}_{mat}")
        bm.to_mesh(me)
        bm.free()
        me.materials.append(material(mat))
        for p in me.polygons:
            p.use_smooth = mat in ("clay", "crystal", "flame")
        o = bpy.data.objects.new(me.name, me)
        bpy.context.scene.collection.objects.link(o)
        nodes.setdefault(node, []).append(o)
    PARTS.clear()
    out = []
    for node, objs in nodes.items():
        bpy.ops.object.select_all(action="DESELECT")
        for o in objs:
            o.select_set(True)
        bpy.context.view_layer.objects.active = objs[0]
        if len(objs) > 1:
            bpy.ops.object.join()
        o = bpy.context.view_layer.objects.active
        o.name = node
        o.data.name = node
        out.append(o)
    return out


def set_origin(obj, game_point):
    """Moves an object's origin to `game_point` without moving its geometry."""
    p = g2b(game_point)
    obj.data.transform(Matrix.Translation(-p))
    obj.location = p


def export(objs, name):
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    path = os.path.join(OUT_DIR, f"{name}.glb")
    bpy.ops.export_scene.gltf(
        filepath=path,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_yup=True,
        export_texcoords=True,
        export_normals=True,
        export_materials="EXPORT",
        export_image_format="AUTO",
        export_cameras=False,
        export_lights=False,
        export_animations=False,
    )
    for o in objs:
        bpy.data.objects.remove(o)
    print(f"build_props: wrote {path} ({os.path.getsize(path)} bytes)")


# --------------------------------------------------------------------------------------
# The props


def build_lantern():
    """A stone grave-lantern with a lit iron chamber: plinth, octagonal shaft, chamber, cap."""
    st = part("lantern", "masonry", 1.2)
    box(st, -0.38, 0.38, 0.0, 0.16, -0.38, 0.38)
    box(st, -0.3, 0.3, 0.16, 0.34, -0.3, 0.3)
    lathe(st, [(0.17, 0.34), (0.15, 0.5), (0.13, 1.25), (0.16, 1.3), (0.2, 1.36)], sides=8, phase=math.pi / 8)
    box(st, -0.25, 0.25, 1.36, 1.42, -0.25, 0.25)
    # The cap: a stepped pyramid with a finial.
    box(st, -0.3, 0.3, 1.86, 1.92, -0.3, 0.3)
    lathe(st, [(0.33, 1.92), (0.08, 2.12), (0.0, 2.14)], sides=4, phase=math.pi / 4)
    lathe(st, [(0.0, 2.12), (0.05, 2.17), (0.035, 2.24), (0.0, 2.27)], sides=6)
    # The chamber: iron corner posts, rails, and amber panes behind them.
    fe = part("lantern", "rusted_iron", 0.5)
    h = 0.2
    for sx in (-1, 1):
        for sz in (-1, 1):
            bar(fe, (sx * h, 1.42, sz * h), (sx * h, 1.86, sz * h), 0.035)
    for y in (1.44, 1.84):
        for a, b in (((-h, -h), (h, -h)), ((h, -h), (h, h)), ((h, h), (-h, h)), ((-h, h), (-h, -h))):
            bar(fe, (a[0], y, a[1]), (b[0], y, b[1]), 0.03)
    # A cross of bars on each face, like a grille.
    for a, b in (((-h, 0.0), (h, 0.0)), ((0.0, -h), (0.0, h))):
        for y in (1.64,):
            bar(fe, (a[0], y, -h), (b[0], y, -h), 0.018)
            bar(fe, (a[0], y, h), (b[0], y, h), 0.018)
            bar(fe, (-h, y, a[1]), (-h, y, b[1]), 0.018)
            bar(fe, (h, y, a[1]), (h, y, b[1]), 0.018)
    pane = part("lantern", "pane", 1.0)
    box(pane, -h + 0.02, h - 0.02, 1.45, 1.83, -h + 0.02, h - 0.02)
    fl = part("lantern", "flame", 1.0)
    flame(fl, 0.0, 1.5, 0.0, 0.26, 0.05)
    # Old wax on the plinth.
    wax = part("lantern", "clay", 1.0)
    for _ in range(4):
        x, z = rng.uniform(-0.3, 0.3), rng.choice((-1, 1)) * rng.uniform(0.31, 0.35)
        lathe(wax, [(0.03, 0.34), (0.028, 0.34 + rng.uniform(0.04, 0.12)), (0.0, 0.34 + rng.uniform(0.12, 0.13))], sides=6, cx=x, cz=z)
    export(build_objects("lantern"), "lantern")


def build_chest():
    """An iron-bound chest, 0.9 x 0.45 x 0.6 m plus a rounded lid; the lid hinges at the back."""
    w, d, hb = 0.45, 0.3, 0.45
    wood = part("body", "wood", 0.9)
    box(wood, -w, w, 0.0, hb, -d, d)
    fe = part("body", "rusted_iron", 0.5)
    for x in (-w + 0.1, w - 0.1):
        # A band around the body.
        box(fe, x - 0.035, x + 0.035, 0.0, hb, -d - 0.012, d + 0.012)
    box(fe, -w - 0.012, w + 0.012, 0.0, 0.05, -d - 0.012, d + 0.012)
    # The lock plate and hasp on the front (+Z).
    box(fe, -0.07, 0.07, hb - 0.16, hb - 0.01, d, d + 0.02)
    box(fe, -0.025, 0.025, hb - 0.1, hb - 0.06, d + 0.02, d + 0.035)
    # Feet.
    for x in (-w + 0.04, w - 0.04):
        for z in (-d + 0.04, d - 0.04):
            box(fe, x - 0.04, x + 0.04, -0.01, 0.02, z - 0.04, z + 0.04)
    # The lid: a half cylinder along X, flattened, from the body's top.
    lw = part("lid", "wood", 0.9)
    lf = part("lid", "rusted_iron", 0.5)
    seg = 8
    for bm, x0, x1, grow in ((lw, -w, w, 0.0), (lf, -w + 0.065, -w + 0.135, 0.012), (lf, w - 0.135, w - 0.065, 0.012)):
        prof = []
        for i in range(seg + 1):
            a = math.pi * i / seg
            prof.append((-(d + grow) * math.cos(a), hb + (0.2 + grow) * math.sin(a)))
        vs0 = [bm.verts.new(g2b((x0, y, z))) for z, y in prof]
        vs1 = [bm.verts.new(g2b((x1, y, z))) for z, y in prof]
        for i in range(seg):
            bm.faces.new((vs0[i], vs1[i], vs1[i + 1], vs0[i + 1]))
        bm.faces.new(vs0)
        bm.faces.new(list(reversed(vs1)))
        bm.faces.new((vs0[0], vs0[-1], vs1[-1], vs1[0]))
    objs = build_objects("chest")
    lid = next(o for o in objs if o.name == "lid")
    set_origin(lid, (0.0, hb, -d))
    export(objs, "chest")


def build_pickup():
    """A small stoppered clay jar on its side among a few spilled crystals."""
    jar = part("pickup", "clay", 1.0)
    lathe(jar, [(0.0, 0.0), (0.045, 0.004), (0.062, 0.04), (0.058, 0.09), (0.03, 0.12), (0.026, 0.14), (0.032, 0.15), (0.0, 0.152)], sides=12)
    cork = part("pickup", "cork", 1.0)
    lathe(cork, [(0.0, 0.148), (0.024, 0.148), (0.022, 0.18), (0.0, 0.182)], sides=8)
    cr = part("pickup", "crystal", 1.0)
    for _ in range(5):
        a = rng.uniform(0, 2 * math.pi)
        r = rng.uniform(0.09, 0.17)
        x, z = math.cos(a) * r, math.sin(a) * r
        s = rng.uniform(0.012, 0.024)
        lathe(cr, [(0.0, 0.0), (s, s * 0.6), (s * 0.8, s * 1.8), (0.0, s * 2.6)], sides=5, cx=x, cz=z, phase=rng.uniform(0, 1))
    export(build_objects("pickup"), "pickup")


def build_message():
    """A scrawled, glowing inscription on the floor: a row of glyphs inside a broken ring."""
    gl = part("message", "glyph", 1.0)
    y = 0.012
    wdt = 0.022

    def stroke(a, b):
        a, b = Vector((a[0], y, a[1])), Vector((b[0], y, b[1]))
        d = (b - a).normalized()
        side = Vector((-d.z, 0.0, d.x)) * (wdt / 2)
        vs = [bm_v(a + side), bm_v(b + side), bm_v(b - side), bm_v(a - side)]
        gl.faces.new(vs)

    def bm_v(p):
        return gl.verts.new(g2b(p))

    # Glyphs: each a few strokes in a 0.12 x 0.18 cell, along two rows.
    for row, z0 in ((0, -0.12), (1, 0.12)):
        n = 5 if row == 0 else 4
        for i in range(n):
            cx = (i - (n - 1) / 2) * 0.16
            pts = [(cx + rng.uniform(-0.05, 0.05), z0 + rng.uniform(-0.08, 0.08)) for _ in range(rng.randint(3, 4))]
            for a, b in zip(pts, pts[1:]):
                stroke(a, b)
            if rng.random() < 0.5:
                stroke((cx - 0.05, z0 + 0.09), (cx + 0.05, z0 + 0.09))
    # A broken ring around the words.
    segs = 40
    for i in range(segs):
        if i % 9 == 0:
            continue
        a0, a1 = 2 * math.pi * i / segs, 2 * math.pi * (i + 1) / segs
        r = 0.42 + rng.uniform(-0.01, 0.01)
        stroke((math.cos(a0) * r, math.sin(a0) * r), (math.cos(a1) * r, math.sin(a1) * r))
    export(build_objects("message"), "message")


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    os.makedirs(OUT_DIR, exist_ok=True)
    build_lantern()
    build_chest()
    build_pickup()
    build_message()


main()
