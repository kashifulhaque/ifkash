"""Bakes a Mixamo character and its clips into one game-ready GLB and a JSON sidecar.

Reads a per-character config (`game/art/characters/NAME.json`), a rigged character FBX,
and one FBX per clip from the source folder, and writes
`game/assets-src/characters/NAME/NAME.glb` and `NAME.json`. `game/scripts/art-character.sh`
runs this and then compresses the textures with `lf-tools assets`.

Steps:
1. Mesh: joins the meshes into one, merges duplicate vertices, recalculates normals,
   decimates to the triangle budget, and keeps at most four bone weights per vertex.
2. Textures: scales any image larger than the config's `textureSize` down to it.
3. Bones: strips the `mixamorig:` prefix (any `mixamorigN:` variant too).
4. Clips, each at 30 fps from frame 0:
   - Removes horizontal root travel: the hips' drift from the first to the last frame,
     so the clip stays in place and a loop's ends meet (`"root": "lock"` pins the hips
     instead).
   - Finds the contact frame of an attack: the frame where the hand moves fastest.
   - Smooths every channel with a small Gaussian filter to remove capture jitter, but
     leaves the frames around the contact untouched so the strike stays sharp.
5. Exports one GLB with the skinned mesh and every clip, named by its game clip id.

Run:
    blender --background --factory-startup --python bake_character.py -- CONFIG [--source DIR]
"""

import json
import math
import os
import re
import sys
import tempfile

import bmesh
import bpy
from mathutils import Matrix, Quaternion, Vector

FPS = 30
PREFIX = re.compile(r"^mixamorig\d*:")
REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
REQUIRED_CLIPS = ("idle", "walk", "run")
DEFAULTS = {
    "triangles": 20000,
    "textureSize": 1024,
    "contactBone": "RightHand",
    "smooth": 1.0,
}


def log(msg):
    print(f"art: {msg}", flush=True)


def fail(msg):
    print(f"art: error: {msg}", file=sys.stderr, flush=True)
    sys.exit(1)


# -- config and files ---------------------------------------------------------------------

def read_config(path):
    with open(path) as f:
        cfg = json.load(f)
    for k, v in DEFAULTS.items():
        cfg.setdefault(k, v)
    if "name" not in cfg or not re.fullmatch(r"[a-z0-9_]+", cfg["name"]):
        fail(f"{path}: \"name\" must be lowercase letters, digits, and underscores")
    if "clips" not in cfg or not isinstance(cfg["clips"], dict):
        fail(f"{path}: missing a \"clips\" object")
    return cfg


def norm(name):
    """File names compare without case, extension, or repeated spaces."""
    base = os.path.splitext(os.path.basename(name))[0]
    return re.sub(r"\s+", " ", base).strip().lower()


def find_file(source, wanted, files):
    """The FBX in `source` that one of the `wanted` names matches, or None."""
    names = [wanted] if isinstance(wanted, str) else list(wanted)
    by_norm = {norm(f): f for f in files}
    for w in names:
        hit = by_norm.get(norm(w))
        if hit:
            return os.path.join(source, hit)
    return None


# -- scene helpers ------------------------------------------------------------------------

def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.fps = FPS
    scene.render.fps_base = 1


def import_fbx(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.fbx(
        filepath=path,
        use_anim=True,
        ignore_leaf_bones=False,
        automatic_bone_orientation=False,
        use_image_search=True,
    )
    new = [o for o in bpy.data.objects if o not in before]
    arms = [o for o in new if o.type == "ARMATURE"]
    if len(arms) != 1:
        fail(f"{os.path.basename(path)}: expected one armature, found {len(arms)}")
    return arms[0], [o for o in new if o.type == "MESH"], new


def override(active, selected):
    return bpy.context.temp_override(
        active_object=active,
        object=active,
        selected_objects=selected,
        selected_editable_objects=selected,
    )


def fcurves(action):
    """Every F-curve of an action (Blender 4.4+ keeps them in layers, strips, and slots)."""
    out = []
    for layer in action.layers:
        for strip in layer.strips:
            for bag in strip.channelbags:
                out.extend(bag.fcurves)
    return out


def strip_prefix(name):
    return PREFIX.sub("", name)


# -- character ----------------------------------------------------------------------------

def rename_bones(arm, meshes):
    renamed = 0
    for bone in arm.data.bones:
        short = strip_prefix(bone.name)
        if short != bone.name:
            bone.name = short
            renamed += 1
    # Renaming a bone renames the vertex groups of meshes bound to it; this catches the rest.
    for m in meshes:
        for g in m.vertex_groups:
            short = strip_prefix(g.name)
            if short != g.name:
                g.name = short
    return renamed


def triangles(obj):
    return sum(len(p.vertices) - 2 for p in obj.data.polygons)


def clean_mesh(arm, meshes, cfg):
    if not meshes:
        fail("the character file has no mesh; download it from Mixamo \"With Skin\"")
    if len(meshes) > 1:
        with override(meshes[0], meshes):
            bpy.ops.object.join()
    body = meshes[0]
    body.name = cfg["name"]
    before = triangles(body)

    # Merge duplicate vertices (0.1 mm in world units) and point every face outward.
    scale = max(body.matrix_world.to_scale())
    bm = bmesh.new()
    bm.from_mesh(body.data)
    count = len(bm.verts)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4 / max(scale, 1e-9))
    merged = count - len(bm.verts)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(body.data)
    bm.free()
    if body.data.has_custom_normals:
        with override(body, [body]):
            bpy.ops.mesh.customdata_custom_splitnormals_clear()

    # Decimate to the budget. The modifier goes before the armature so it applies at rest.
    target = int(cfg["triangles"])
    for _ in range(3):
        tris = triangles(body)
        if tris <= target * 1.02:
            break
        mod = body.modifiers.new("Decimate", "DECIMATE")
        mod.decimate_type = "COLLAPSE"
        mod.ratio = target / tris
        with override(body, [body]):
            bpy.ops.object.modifier_move_to_index(modifier=mod.name, index=0)
            bpy.ops.object.modifier_apply(modifier=mod.name)

    with override(body, [body]):
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(40))
        # glTF carries four weights per vertex; keep the four largest and renormalize.
        bpy.ops.object.vertex_group_limit_total(group_select_mode="BONE_DEFORM", limit=4)
        bpy.ops.object.vertex_group_normalize_all(group_select_mode="BONE_DEFORM", lock_active=False)
    after = triangles(body)
    log(f"mesh: {before} -> {after} triangles, {merged} duplicate vertices merged")
    return body


def single_sided(body):
    """Back faces are culled in the game; generated meshes are closed, so nothing shows through."""
    for mat in body.data.materials:
        if mat:
            mat.use_backface_culling = True


def shrink_textures(cfg):
    size = int(cfg["textureSize"])
    out = tempfile.mkdtemp(prefix="lf-art-")
    report = []
    for img in bpy.data.images:
        if img.type != "IMAGE" or img.size[0] == 0:
            continue
        w, h = img.size
        if max(w, h) > size:
            k = size / max(w, h)
            img.scale(max(1, round(w * k)), max(1, round(h * k)))
            # The glTF exporter reads an image's file or packed data, not the scaled pixels,
            # so save the scaled copy, drop the packed original, and point the image at it.
            path = os.path.join(out, re.sub(r"[^\w.-]", "_", img.name) + ".png")
            img.file_format = "PNG"
            img.save(filepath=path)
            if img.packed_file:
                img.unpack(method="REMOVE")
            img.filepath = path
            img.reload()
        report.append(f"{img.name} {w}x{h} -> {img.size[0]}x{img.size[1]}")
    for line in report:
        log(f"texture: {line}")
    return [r.split(" ")[0] for r in report]


# -- clips --------------------------------------------------------------------------------

def depth_order(arm):
    """The character's pose bones, parents before children."""
    def depth(b):
        return len(b.bone.parent_recursive)
    return sorted(arm.pose.bones, key=depth)


def take_action(path, arm, clip_id):
    """Imports a clip FBX and moves its motion onto `arm` as a new action.

    The pose moves over in armature space, bone by bone, by Mixamo name. A clip file
    without a mesh can have another rest pose than the character (Blender's FBX importer
    builds the rest from the file's bone nodes), so copying the action by bone name would
    play it against the wrong rest. Returns the action and its samples, frame 0 first.
    """
    src, _, new = import_fbx(path)
    src_act = src.animation_data.action if src.animation_data else None
    if src_act is None:
        fail(f"{os.path.basename(path)}: no animation in the file")
    start, end = src_act.frame_range
    frames = list(range(math.floor(start + 1e-4), math.ceil(end - 1e-4) + 1))
    src_bones = {strip_prefix(pb.name): pb for pb in src.pose.bones}
    order = depth_order(arm)
    missing = [pb.name for pb in order if pb.name not in src_bones]
    if "Hips" in missing:
        fail(f"{os.path.basename(path)}: the clip's skeleton has no Hips bone")
    if missing:
        log(f"warning: {clip_id}: bones not in the clip, held at rest: {', '.join(missing)}")

    to_char = arm.matrix_world.inverted() @ src.matrix_world
    rest = {pb.name: pb.bone.matrix_local for pb in order}
    data = {}
    for pb in order:
        pb.rotation_mode = "QUATERNION"
        for prop, size in (("location", 3), ("rotation_quaternion", 4), ("scale", 3)):
            data[(pb.name, prop)] = {i: [] for i in range(size)}
    scene = bpy.context.scene
    for f in frames:
        scene.frame_set(f)
        posed = {}
        for pb in order:
            sp = src_bones.get(pb.name)
            parent = pb.parent
            if sp is None:
                # Held at rest under its (moving) parent.
                local = Matrix.Identity(4)
            else:
                target = to_char @ sp.matrix
                if parent is None:
                    local = rest[pb.name].inverted() @ target
                else:
                    rel = rest[parent.name].inverted() @ rest[pb.name]
                    local = rel.inverted() @ posed[parent.name].inverted() @ target
            if parent is None:
                posed[pb.name] = rest[pb.name] @ local
            else:
                posed[pb.name] = posed[parent.name] @ rest[parent.name].inverted() @ rest[pb.name] @ local
            loc, quat, scale = local.decompose()
            for prop, vec in (("location", loc), ("rotation_quaternion", quat), ("scale", scale)):
                for i, v in enumerate(vec):
                    data[(pb.name, prop)][i].append(v)

    act = bpy.data.actions.new(clip_id)
    act.use_fake_user = True
    assign(arm, act)
    for (bone, prop), comps in data.items():
        for i in comps:
            act.fcurve_ensure_for_datablock(arm, f'pose.bones["{bone}"].{prop}', index=i, group_name=bone)
    for o in new:
        bpy.data.objects.remove(o, do_unlink=True)
    bpy.data.actions.remove(src_act)
    return act, data, len(frames)


def bone_channels(action):
    """{(bone, prop): [fcurve per index]} for pose-bone channels."""
    out = {}
    for fc in fcurves(action):
        m = re.match(r'pose\.bones\["([^"]+)"\]\.(\w+)$', fc.data_path)
        if not m:
            continue
        out.setdefault((m.group(1), m.group(2)), {})[fc.array_index] = fc
    return out


def write(action, data, n):
    """Replaces each channel's keys with the sampled values, linear, from frame 0."""
    chans = bone_channels(action)
    for key, comps in data.items():
        for i, values in comps.items():
            fc = chans[key][i]
            fc.keyframe_points.clear()
            fc.keyframe_points.add(n)
            co = []
            for f, v in enumerate(values):
                co += [float(f), float(v)]
            fc.keyframe_points.foreach_set("co", co)
            for k in fc.keyframe_points:
                k.interpolation = "LINEAR"
            fc.update()


def fix_quaternion_signs(data):
    """Keeps each bone's quaternions on one hemisphere, so filtering never crosses a flip."""
    for (bone, prop), comps in data.items():
        if prop != "rotation_quaternion" or len(comps) != 4:
            continue
        n = len(comps[0])
        for f in range(1, n):
            dot = sum(comps[i][f] * comps[i][f - 1] for i in range(4))
            if dot < 0:
                for i in range(4):
                    comps[i][f] = -comps[i][f]


def strip_root(arm, data, n, mode):
    """Takes the hips' horizontal travel out of the clip, measured in world axes."""
    comps = data.get(("Hips", "location"))
    if not comps or len(comps) != 3 or n < 2:
        return 0.0
    bone = arm.data.bones["Hips"]
    # Bone-local location to world offset (Blender's world is Z up).
    to_world = arm.matrix_world.to_3x3() @ bone.matrix_local.to_3x3()
    to_local = to_world.inverted()
    world = [to_world @ Vector((comps[0][f], comps[1][f], comps[2][f])) for f in range(n)]
    travel = math.hypot(world[-1].x - world[0].x, world[-1].y - world[0].y)
    rest = to_world @ Vector((0, 0, 0))
    for f in range(n):
        w = world[f]
        if mode == "lock":
            x, y = rest.x, rest.y
        else:
            u = f / (n - 1)
            x = w.x - (world[0].x + (world[-1].x - world[0].x) * u) + rest.x
            y = w.y - (world[0].y + (world[-1].y - world[0].y) * u) + rest.y
        local = to_local @ Vector((x, y, w.z))
        for i in range(3):
            comps[i][f] = local[i]
    return travel


def contact_frame(arm, action, n, cfg, clip):
    """The frame where the contact bone moves fastest, relative to the hips."""
    bone = clip.get("contactBone", cfg["contactBone"])
    if bone not in arm.pose.bones:
        fail(f"contact bone {bone} is not in the skeleton")
    scene = bpy.context.scene
    assign(arm, action)
    pos = []
    for f in range(n):
        scene.frame_set(f)
        hand = arm.matrix_world @ arm.pose.bones[bone].head
        hips = arm.matrix_world @ arm.pose.bones["Hips"].head
        pos.append(hand - Vector((hips.x, hips.y, 0)))
    lo, hi = clip.get("contactWindow", [0.1, 0.85])
    best, best_speed = 0, -1.0
    for f in range(max(1, int(n * lo)), min(n - 1, int(math.ceil(n * hi)))):
        speed = (pos[f + 1] - pos[f - 1]).length * FPS / 2
        if speed > best_speed:
            best, best_speed = f, speed
    return best, best_speed


def smooth(data, n, sigma, loop, keep):
    """Gaussian filter over every channel; frames near `keep` (the contact) are kept as is."""
    if sigma <= 0 or n < 5:
        return
    radius = max(1, int(math.ceil(sigma * 3)))
    kernel = [math.exp(-(k * k) / (2 * sigma * sigma)) for k in range(-radius, radius + 1)]
    total = sum(kernel)
    kernel = [k / total for k in kernel]
    period = n - 1 if loop else n

    def weight(f):
        w = 1.0
        if keep is not None:
            # Untouched within a frame of the contact, fully filtered four frames out.
            w = min(w, max(0.0, (abs(f - keep) - 1) / 3))
        if not loop:
            w = min(w, f / 2, (n - 1 - f) / 2)
        return max(0.0, min(1.0, w))

    weights = [weight(f) for f in range(n)]
    for comps in data.values():
        for i, values in comps.items():
            src = values[:]
            for f in range(n):
                if weights[f] <= 0:
                    continue
                acc = 0.0
                for j, k in enumerate(kernel):
                    g = f + j - radius
                    g = g % period if loop else min(max(g, 0), n - 1)
                    acc += src[g] * k
                values[f] = src[f] + (acc - src[f]) * weights[f]
            if loop:
                values[n - 1] = values[0]


def assign(arm, action):
    ad = arm.animation_data or arm.animation_data_create()
    ad.action = action
    if action.slots and ad.action_slot is None:
        ad.action_slot = action.slots[0]


def process_clip(arm, clip_id, path, clip, cfg):
    act, data, n = take_action(path, arm, clip_id)
    if n < 2:
        fail(f"{clip_id}: the clip has fewer than two frames")
    trim = clip.get("trim")
    if trim:
        a, b = int(trim[0]), min(int(trim[1]), n - 1)
        for comps in data.values():
            for i in comps:
                comps[i] = comps[i][a:b + 1]
        n = b - a + 1
    fix_quaternion_signs(data)
    travel = strip_root(arm, data, n, clip.get("root", "detrend"))
    write(act, data, n)

    contact = clip.get("contact")
    speed = None
    if contact == "auto":
        contact, speed = contact_frame(arm, act, n, cfg, clip)
    elif contact is not None:
        contact = int(contact)
    smooth(data, n, float(clip.get("smooth", cfg["smooth"])), bool(clip.get("loop")), contact)
    write(act, data, n)

    note = f", contact frame {contact}" if contact is not None else ""
    if speed is not None:
        note += f" (hand {speed:.1f} m/s)"
    log(f"clip {clip_id}: {os.path.basename(path)}, {n} frames, {travel:.2f} m travel removed{note}")
    return act, {
        "frames": n,
        "duration": round((n - 1) / FPS, 4),
        "loop": bool(clip.get("loop")),
        "contact": contact,
        "source": os.path.basename(path),
    }


# -- export -------------------------------------------------------------------------------

def export(arm, body, actions, path):
    ad = arm.animation_data or arm.animation_data_create()
    ad.action = None
    for track in list(ad.nla_tracks):
        ad.nla_tracks.remove(track)
    for clip_id, act in actions.items():
        track = ad.nla_tracks.new()
        track.name = clip_id
        strip = track.strips.new(clip_id, 0, act)
        if hasattr(strip, "action_slot") and strip.action_slot is None and act.slots:
            strip.action_slot = act.slots[0]
        track.mute = True
    for pb in arm.pose.bones:
        pb.matrix_basis = Matrix.Identity(4)
    bpy.context.scene.frame_set(0)
    bpy.ops.object.select_all(action="DESELECT")
    arm.select_set(True)
    body.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.export_scene.gltf(
        filepath=path,
        export_format="GLB",
        use_selection=True,
        export_yup=True,
        export_apply=False,
        export_skins=True,
        export_all_influences=False,
        export_def_bones=False,
        export_animations=True,
        export_animation_mode="NLA_TRACKS",
        export_force_sampling=True,
        export_anim_single_armature=True,
        export_image_format="AUTO",
        export_materials="EXPORT",
        export_extras=False,
        export_cameras=False,
        export_lights=False,
    )


def register_assets(name):
    """Adds the character's jobs to `game/assets-src/assets.json` once."""
    path = os.path.join(REPO, "game", "assets-src", "assets.json")
    with open(path) as f:
        text = f.read()
    src = f"characters/{name}/{name}"
    if f'"{src}.glb"' in text:
        return False
    lines = (
        f'    {{"src": "{src}.glb", "out": "characters/{name}.glb", "maxTextureSize": 1024}},\n'
        f'    {{"src": "{src}.json", "out": "characters/{name}.json", "copy": true}}'
    )
    end = text.rindex("\n  ]")
    text = text[:end] + ",\n" + lines + text[end:]
    json.loads(text)
    with open(path, "w") as f:
        f.write(text)
    return True


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if not argv:
        fail("usage: blender --background --python bake_character.py -- CONFIG [--source DIR]")
    cfg_path = argv[0]
    cfg = read_config(cfg_path)
    source = cfg.get("source", f"~/cinderheart-art/{cfg['name']}")
    if "--source" in argv:
        source = argv[argv.index("--source") + 1]
    source = os.path.expanduser(source)
    if not os.path.isdir(source):
        fail(f"source folder {source} does not exist")
    files = sorted(f for f in os.listdir(source) if f.lower().endswith(".fbx"))

    # Match clip files first; the character is the named file or the one left over.
    clip_paths, missing = {}, []
    for clip_id, clip in cfg["clips"].items():
        p = find_file(source, clip["file"], files)
        if p:
            clip_paths[clip_id] = p
        else:
            missing.append(clip_id)
    used = {os.path.basename(p) for p in clip_paths.values()}
    if cfg.get("character"):
        char_path = find_file(source, cfg["character"], files)
    else:
        rest = [f for f in files if f not in used]
        char_path = os.path.join(source, rest[0]) if len(rest) == 1 else None
    if not char_path:
        fail(f"no character FBX in {source}; set \"character\" in {cfg_path}")
    used.add(os.path.basename(char_path))
    for clip_id in missing:
        want = cfg["clips"][clip_id]["file"]
        log(f"warning: no file for clip {clip_id} (looked for {want})")
    unused = [f for f in files if f not in used]
    if unused:
        log(f"warning: files not in the config, skipped: {', '.join(unused)}")
    lacking = [c for c in REQUIRED_CLIPS if c not in clip_paths]
    if lacking:
        fail(f"the clips {', '.join(lacking)} are required")

    reset()
    arm, meshes, _ = import_fbx(char_path)
    arm.name = "Armature"
    if arm.animation_data:
        arm.animation_data.action = None
    renamed = rename_bones(arm, meshes)
    log(f"character: {os.path.basename(char_path)}, {len(arm.data.bones)} bones ({renamed} renamed)")
    body = clean_mesh(arm, meshes, cfg)
    single_sided(body)
    textures = shrink_textures(cfg)

    actions, sidecar_clips = {}, {}
    for clip_id, path in clip_paths.items():
        act, info = process_clip(arm, clip_id, path, cfg["clips"][clip_id], cfg)
        actions[clip_id] = act
        sidecar_clips[clip_id] = info

    out_dir = os.path.join(REPO, "game", "assets-src", "characters", cfg["name"])
    os.makedirs(out_dir, exist_ok=True)
    glb = os.path.join(out_dir, f"{cfg['name']}.glb")
    export(arm, body, actions, glb)
    sidecar = {
        "name": cfg["name"],
        "fps": FPS,
        "triangles": triangles(body),
        "bones": [b.name for b in arm.data.bones],
        "textures": textures,
        "contactBone": cfg["contactBone"],
        "clips": sidecar_clips,
    }
    if "credits" in cfg:
        sidecar["credits"] = cfg["credits"]
    with open(os.path.join(out_dir, f"{cfg['name']}.json"), "w") as f:
        json.dump(sidecar, f, indent=2)
        f.write("\n")
    if register_assets(cfg["name"]):
        log("added the character to game/assets-src/assets.json")
    log(f"wrote {os.path.relpath(glb, REPO)} ({os.path.getsize(glb) // 1024} KB) and its sidecar")


main()
