"""Builds a stand-in for a Mixamo download, to test the character pipeline without one.

Writes the same kind of files the owner downloads from Mixamo into a folder: a rigged
character FBX ("With Skin") and one FBX per clip ("Without Skin", 30 fps). The skeleton
uses Mixamo's bone names with the `mixamorig:` prefix. Every part is made here by script,
so the files carry no third-party license.

The stand-in also carries the faults the pipeline must fix: a mesh over the triangle
budget with duplicate vertices, a 2048-pixel texture, clips that travel forward instead
of staying in place, and frame-to-frame jitter like raw motion capture.

Run:
    blender --background --factory-startup --python make_standin.py -- OUT_DIR
"""

import math
import os
import random
import sys
import tempfile

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

FPS = 30
PREFIX = "mixamorig:"

# Bone name: (parent, head, tail, radius). Blender axes: Z up, the character faces -Y,
# and its left side is +X. Meters.
BONES = {
    "Hips": (None, (0, 0, 0.98), (0, 0, 1.08), 0.15),
    "Spine": ("Hips", (0, 0, 1.08), (0, 0, 1.2), 0.14),
    "Spine1": ("Spine", (0, 0, 1.2), (0, 0, 1.32), 0.15),
    "Spine2": ("Spine1", (0, 0, 1.32), (0, 0, 1.46), 0.17),
    "Neck": ("Spine2", (0, 0, 1.46), (0, 0, 1.56), 0.05),
    "Head": ("Neck", (0, 0, 1.56), (0, 0, 1.74), 0.1),
    "HeadTop_End": ("Head", (0, 0, 1.74), (0, 0, 1.8), 0.0),
    "LeftShoulder": ("Spine2", (0.04, 0, 1.43), (0.17, 0, 1.43), 0.06),
    "LeftArm": ("LeftShoulder", (0.17, 0, 1.43), (0.44, 0, 1.43), 0.055),
    "LeftForeArm": ("LeftArm", (0.44, 0, 1.43), (0.69, 0, 1.43), 0.045),
    "LeftHand": ("LeftForeArm", (0.69, 0, 1.43), (0.79, 0, 1.43), 0.04),
    "LeftUpLeg": ("Hips", (0.1, 0, 0.95), (0.1, 0, 0.53), 0.08),
    "LeftLeg": ("LeftUpLeg", (0.1, 0, 0.53), (0.1, 0, 0.1), 0.06),
    "LeftFoot": ("LeftLeg", (0.1, 0, 0.1), (0.1, -0.12, 0.03), 0.045),
    "LeftToeBase": ("LeftFoot", (0.1, -0.12, 0.03), (0.1, -0.2, 0.03), 0.04),
    "LeftToe_End": ("LeftToeBase", (0.1, -0.2, 0.03), (0.1, -0.24, 0.03), 0.0),
}
for name in [n for n in BONES if n.startswith("Left")]:
    parent, head, tail, r = BONES[name]
    mirror = lambda p: (-p[0], p[1], p[2])
    rparent = parent.replace("Left", "Right") if parent.startswith("Left") else parent
    BONES[name.replace("Left", "Right")] = (rparent, mirror(head), mirror(tail), r)


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.fps = FPS
    scene.render.fps_base = 1


def build_armature():
    data = bpy.data.armatures.new("Armature")
    arm = bpy.data.objects.new("Armature", data)
    bpy.context.scene.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    for name, (parent, head, tail, _) in BONES.items():
        b = data.edit_bones.new(PREFIX + name)
        b.head, b.tail = Vector(head), Vector(tail)
        if parent:
            b.parent = data.edit_bones[PREFIX + parent]
            b.use_connect = Vector(head) == Vector(BONES[parent][2])
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm


def build_mesh(arm):
    """One part per bone, each weighted fully to its bone, joined into one mesh.

    The parts are dense (about 60K triangles), and the torso's faces are split apart, so
    the pipeline has duplicate vertices to merge and a mesh to decimate.
    """
    bm = bmesh.new()
    deform = bm.verts.layers.deform.verify()
    groups = []
    for name, (_, head, tail, r) in BONES.items():
        if r <= 0:
            continue
        head, tail = Vector(head), Vector(tail)
        axis = tail - head
        length = axis.length
        part = bmesh.new()
        part.loops.layers.uv.new("UVMap")
        if name == "Head":
            bmesh.ops.create_uvsphere(part, u_segments=48, v_segments=32, radius=r, calc_uvs=True)
            bmesh.ops.scale(part, vec=(1.0, 1.1, length / (2 * r)), verts=part.verts)
        else:
            bmesh.ops.create_cone(
                part, cap_ends=True, segments=64, radius1=r, radius2=r * 0.85,
                depth=length, calc_uvs=True,
            )
            # Rings along the length, so there is something for the decimator to remove.
            edges = [e for e in part.edges if abs(e.verts[0].co.z - e.verts[1].co.z) > 1e-6]
            bmesh.ops.subdivide_edges(part, edges=edges, cuts=20, use_grid_fill=True)
        if name == "Spine2":
            # Every face on its own: duplicate vertices along every edge.
            bmesh.ops.split_edges(part, edges=part.edges[:])
        rot = axis.normalized().to_track_quat("Z", "Y").to_matrix().to_4x4()
        mat = Matrix.Translation((head + tail) / 2) @ rot
        bmesh.ops.transform(part, matrix=mat, verts=part.verts)
        tmp = bpy.data.meshes.new("tmp")
        part.to_mesh(tmp)
        part.free()
        offset = len(bm.verts)
        bm.from_mesh(tmp)
        bpy.data.meshes.remove(tmp)
        bm.verts.ensure_lookup_table()
        gi = len(groups)
        groups.append(PREFIX + name)
        deform = bm.verts.layers.deform.verify()
        for v in bm.verts[offset:]:
            v[deform][gi] = 1.0
    me = bpy.data.meshes.new("Body")
    bm.to_mesh(me)
    bm.free()
    obj = bpy.data.objects.new("Body", me)
    bpy.context.scene.collection.objects.link(obj)
    for g in groups:
        obj.vertex_groups.new(name=g)
    obj.parent = arm
    mod = obj.modifiers.new("Armature", "ARMATURE")
    mod.object = arm
    obj.data.materials.append(build_material())
    return obj


def build_material():
    """A 2048-pixel texture: worn leather bands, so the pipeline has something to shrink."""
    size = 2048
    img = bpy.data.images.new("standin_albedo", size, size, alpha=False)
    step = size // 64
    rows = []
    for base in ((0.24, 0.2, 0.17), (0.36, 0.27, 0.19)):
        row = []
        for x in range(size):
            n = ((x * 7919) % 97) / 97.0 * 0.06
            row += [base[0] + n, base[1] + n, base[2] + n, 1.0]
        rows.append(row)
    px = []
    for y in range(size):
        px += rows[(y // step) % 2]
    img.pixels.foreach_set(px)
    img.file_format = "PNG"
    mat = bpy.data.materials.new("Leather")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    tex = nodes.new("ShaderNodeTexImage")
    tex.image = img
    bsdf = nodes["Principled BSDF"]
    mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
    bsdf.inputs["Roughness"].default_value = 0.8
    return mat


# -- motion -----------------------------------------------------------------------------

def rot(x=0.0, y=0.0, z=0.0):
    """A rotation in world axes, degrees."""
    return Euler((math.radians(x), math.radians(y), math.radians(z)), "XYZ").to_quaternion()


def ease(t):
    t = min(max(t, 0.0), 1.0)
    return t * t * (3 - 2 * t)


def keys(frame, table):
    """Interpolates a list of (frame, value) keys with smoothstep easing. Values are dicts."""
    if frame <= table[0][0]:
        return table[0][1]
    for (f0, a), (f1, b) in zip(table, table[1:]):
        if frame <= f1:
            u = ease((frame - f0) / max(f1 - f0, 1e-6))
            out = {}
            for k in set(a) | set(b):
                zero = Vector((0, 0, 0)) if k.endswith("_loc") else Quaternion()
                qa, qb = a.get(k, zero), b.get(k, zero)
                out[k] = qa.slerp(qb, u) if isinstance(qa, Quaternion) else qa.lerp(qb, u)
            return out
    return table[-1][1]


ARMS_DOWN = {"LeftArm": rot(y=72), "RightArm": rot(y=-72), "LeftForeArm": rot(z=-15), "RightForeArm": rot(z=15)}
# Sword up at the right side, shield arm forward: the ready stance of the attacks.
READY = {
    "LeftArm": rot(y=60, z=-30), "LeftForeArm": rot(z=-70),
    "RightArm": rot(y=-60, z=20), "RightForeArm": rot(z=60, x=-20),
    "Spine1": rot(z=8), "LeftUpLeg": rot(x=-12), "LeftLeg": rot(x=14), "RightUpLeg": rot(x=10),
}
HIPS0 = Vector((0, 0, 0))


def pose_idle(f, n):
    t = f / (n - 1)
    breathe = math.sin(t * 2 * math.pi)
    p = dict(ARMS_DOWN)
    p["Spine1"] = rot(x=-2 * breathe)
    p["LeftArm"] = rot(y=72 + 2 * breathe)
    p["RightArm"] = rot(y=-72 - 2 * breathe)
    p["Hips_loc"] = Vector((0, 0, -0.01 * breathe))
    return p


def gait(f, n, swing, knee, bob, arm, travel, lean):
    t = f / (n - 1)
    s = math.sin(t * 2 * math.pi)
    c = math.cos(t * 2 * math.pi)
    p = dict(ARMS_DOWN)
    p["LeftUpLeg"] = rot(x=-swing * s)
    p["RightUpLeg"] = rot(x=swing * s)
    p["LeftLeg"] = rot(x=knee * max(0.0, c))
    p["RightLeg"] = rot(x=knee * max(0.0, -c))
    p["LeftArm"] = rot(y=72, x=arm * s)
    p["RightArm"] = rot(y=-72, x=-arm * s)
    p["Spine"] = rot(x=-lean)
    p["Spine1"] = rot(z=4 * s)
    # Not in place: the hips travel forward (-Y), as a clip downloaded without "In Place" does.
    p["Hips_loc"] = Vector((0.02 * s, -travel * t, -bob * abs(c)))
    return p


def attack(table, travel=0.0):
    def pose(f, n):
        p = dict(keys(f, table))
        hips = p.get("Hips_loc", HIPS0).copy()
        hips.y -= travel * ease(f / (n - 1))
        p["Hips_loc"] = hips
        return p
    return pose


def merge(*poses):
    out = {}
    for p in poses:
        out.update(p)
    return out


SLASH_A = [
    (0, READY),
    (10, merge(READY, {"RightArm": rot(y=-30, z=70, x=-30), "RightForeArm": rot(z=80), "Spine1": rot(z=35), "Spine": rot(z=10)})),
    (16, merge(READY, {"RightArm": rot(y=-75, z=-60), "RightForeArm": rot(z=10), "Spine1": rot(z=-30), "Spine": rot(z=-10)})),
    (22, merge(READY, {"RightArm": rot(y=-80, z=-80), "RightForeArm": rot(z=20), "Spine1": rot(z=-38)})),
    (34, READY),
]
SLASH_B = [
    (0, READY),
    (9, merge(READY, {"RightArm": rot(y=-70, z=-75), "RightForeArm": rot(z=30), "Spine1": rot(z=-35)})),
    (15, merge(READY, {"RightArm": rot(y=-40, z=60, x=10), "RightForeArm": rot(z=5), "Spine1": rot(z=30)})),
    (21, merge(READY, {"RightArm": rot(y=-30, z=75), "Spine1": rot(z=36)})),
    (32, READY),
]
SLASH_C = [
    (0, READY),
    (11, merge(READY, {"RightArm": rot(y=-70, z=40), "RightForeArm": rot(z=110), "Spine1": rot(z=25), "Spine": rot(x=-5)})),
    (17, merge(READY, {"RightArm": rot(y=-5, z=-80), "RightForeArm": rot(z=0), "Spine1": rot(z=-20), "Spine": rot(x=12)})),
    (24, merge(READY, {"RightArm": rot(y=-10, z=-80), "Spine": rot(x=10)})),
    (36, READY),
]
HEAVY = [
    (0, READY),
    (20, merge(READY, {"RightArm": rot(y=-160, x=-30), "RightForeArm": rot(z=60), "LeftArm": rot(y=100, x=-20), "Spine": rot(x=-14), "Spine1": rot(x=-10)})),
    (26, merge(READY, {"RightArm": rot(y=-40, x=-70), "RightForeArm": rot(z=5), "Spine": rot(x=28), "Spine1": rot(x=12), "LeftUpLeg": rot(x=-30), "LeftLeg": rot(x=40), "Hips_loc": Vector((0, 0, -0.12))})),
    (32, merge(READY, {"RightArm": rot(y=-30, x=-80), "Spine": rot(x=30), "Spine1": rot(x=12), "LeftUpLeg": rot(x=-30), "LeftLeg": rot(x=40), "Hips_loc": Vector((0, 0, -0.14))})),
    (50, READY),
]
GUARD = merge(READY, {"LeftArm": rot(y=20, z=-60, x=-10), "LeftForeArm": rot(z=-95), "Spine1": rot(z=-10), "Hips_loc": Vector((0, 0, -0.04))})
BLOCK = [(0, READY), (6, GUARD), (14, GUARD), (24, READY)]
HIT = [
    (0, READY),
    (4, merge(READY, {"Spine": rot(x=-14), "Spine1": rot(x=-10, z=8), "Head": rot(x=-18), "Hips_loc": Vector((0, 0.06, -0.03))})),
    (20, READY),
]
DEATH = [
    (0, READY),
    (8, merge(READY, {"Spine": rot(x=-20), "Head": rot(x=-25), "LeftLeg": rot(x=20), "RightLeg": rot(x=20), "Hips_loc": Vector((0, 0.1, -0.1))})),
    (30, merge(ARMS_DOWN, {"Hips": rot(x=-82), "Head": rot(x=-10), "LeftArm": rot(y=95), "RightArm": rot(y=-95), "Hips_loc": Vector((0, 0.5, -0.8))})),
    (60, merge(ARMS_DOWN, {"Hips": rot(x=-88), "LeftArm": rot(y=100), "RightArm": rot(y=-100), "Hips_loc": Vector((0, 0.55, -0.84))})),
]
DRINK = [
    (0, ARMS_DOWN),
    (14, merge(ARMS_DOWN, {"RightArm": rot(y=-20, x=-60), "RightForeArm": rot(z=140), "Head": rot(x=-12)})),
    (40, merge(ARMS_DOWN, {"RightArm": rot(y=-25, x=-65), "RightForeArm": rot(z=145), "Head": rot(x=-30), "Spine1": rot(x=-6)})),
    (56, ARMS_DOWN),
]


def pose_roll(f, n):
    t = f / (n - 1)
    tuck = math.sin(min(t / 0.85, 1.0) * math.pi)
    p = merge(ARMS_DOWN, {
        "Hips": rot(x=360 * ease(min(t / 0.8, 1.0))),
        "Spine": rot(x=40 * tuck), "Spine1": rot(x=25 * tuck), "Head": rot(x=30 * tuck),
        "LeftUpLeg": rot(x=-100 * tuck), "RightUpLeg": rot(x=-100 * tuck),
        "LeftLeg": rot(x=130 * tuck), "RightLeg": rot(x=130 * tuck),
        "LeftArm": rot(y=40, x=-40 * tuck), "RightArm": rot(y=-40, x=-40 * tuck),
    })
    # A roll travels about 2.5 m: the pipeline's root strip must take that out.
    p["Hips_loc"] = Vector((0, -2.5 * ease(t), -0.55 * tuck))
    return p


CLIPS = {
    # file name (as Mixamo names its downloads): (frames, pose function)
    "Sword And Shield Idle": (61, pose_idle),
    "Sword And Shield Walk": (31, lambda f, n: gait(f, n, 26, 30, 0.03, 18, 1.4, 3)),
    "Sword And Shield Run": (21, lambda f, n: gait(f, n, 42, 70, 0.06, 35, 2.0, 10)),
    "Sword And Shield Slash": (35, attack(SLASH_A)),
    "Sword And Shield Slash (2)": (33, attack(SLASH_B)),
    "Sword And Shield Attack": (37, attack(SLASH_C, travel=0.3)),
    "Sword And Shield Power Up": (51, attack(HEAVY, travel=0.4)),
    "Sword And Shield Block": (25, attack(BLOCK)),
    "Sword And Shield Block Idle": (41, lambda f, n: merge(GUARD, {"Spine1": rot(z=-10, x=-1.5 * math.sin(f / (n - 1) * 2 * math.pi))})),
    "Stand To Roll": (25, pose_roll),
    "Sword And Shield Impact": (21, attack(HIT)),
    "Sword And Shield Death": (61, attack(DEATH)),
    "Drinking": (57, attack(DRINK)),
}


def bake_clip(arm, name, frames, pose_fn, rng):
    """Keys every frame, with jitter of about a degree on every rotation, like raw capture."""
    act = bpy.data.actions.new(name)
    arm.animation_data_create()
    arm.animation_data.action = act
    pbones = arm.pose.bones
    for pb in pbones:
        pb.rotation_mode = "QUATERNION"
    for f in range(frames):
        pose = pose_fn(f, frames)
        for pb in pbones:
            short = pb.name[len(PREFIX):]
            q_world = pose.get(short, Quaternion())
            jitter = rot(rng.gauss(0, 0.9), rng.gauss(0, 0.9), rng.gauss(0, 0.9))
            if short.endswith("_End"):
                jitter = Quaternion()
            rest = pb.bone.matrix_local.to_quaternion()
            pb.rotation_quaternion = rest.inverted() @ (jitter @ q_world) @ rest
            pb.keyframe_insert("rotation_quaternion", frame=f)
            if short == "Hips":
                loc = pose.get("Hips_loc", HIPS0) + Vector((rng.gauss(0, 0.002), rng.gauss(0, 0.002), rng.gauss(0, 0.002)))
                pb.location = pb.bone.matrix_local.to_3x3().inverted() @ loc
                pb.keyframe_insert("location", frame=f)
    return act


def export_fbx(path, objects, with_anim):
    bpy.ops.object.select_all(action="DESELECT")
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.export_scene.fbx(
        filepath=path,
        use_selection=True,
        object_types={"ARMATURE", "MESH"},
        add_leaf_bones=False,
        bake_anim=with_anim,
        bake_anim_use_all_actions=False,
        bake_anim_use_nla_strips=False,
        bake_anim_simplify_factor=0.0,
        path_mode="COPY",
        embed_textures=True,
    )


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not argv:
        sys.exit("usage: blender --background --python make_standin.py -- OUT_DIR")
    out = os.path.expanduser(argv[0])
    os.makedirs(out, exist_ok=True)
    reset()
    arm = build_armature()
    body = build_mesh(arm)
    tris = sum(len(p.vertices) - 2 for p in body.data.polygons)
    # The FBX exporter embeds images from disk, so the generated texture is saved first.
    img = bpy.data.images["standin_albedo"]
    img.filepath_raw = os.path.join(tempfile.mkdtemp(), "standin_albedo.png")
    img.save()
    export_fbx(os.path.join(out, "Standin.fbx"), [arm, body], with_anim=False)
    rng = random.Random(7)
    for name, (frames, fn) in CLIPS.items():
        bpy.context.scene.frame_start, bpy.context.scene.frame_end = 0, frames - 1
        bake_clip(arm, name, frames, fn, rng)
        export_fbx(os.path.join(out, name + ".fbx"), [arm], with_anim=True)
    print(f"standin: {tris} triangles, {len(CLIPS)} clips -> {out}")


main()
