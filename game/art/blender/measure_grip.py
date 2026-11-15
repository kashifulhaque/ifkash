"""Measures how a Mixamo "W/Prop" character holds its weapon, for a character config's `grips`.

Mixamo sells no props, but its "W/Prop" characters carry one: a sword bound to a
`Sword_joint` bone under the hand. Mixamo's sword clips were captured with that prop, so
its rest pose relative to the hand is the grip the motion expects. This script finds the
mesh bound to the prop bone, fits the blade's axis, and prints the grip frame relative to
the hand, in the layout the game's weapon models use: the grip at the origin, the blade
along +Y, and the flat of the blade facing +Z.

Paste the printed list into the character config as `"grips": {"R": [...]}`. The bake
places it on the character's own hand and writes it to the sidecar.

Run:
    blender --background --factory-startup --python measure_grip.py -- REF.fbx [PROP_BONE] [HAND_BONE]

PROP_BONE defaults to `Sword_joint` and HAND_BONE to `RightHand`. The `mixamorig:` prefix
is optional.
"""

import json
import sys

import bpy
from mathutils import Matrix, Vector

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
if not argv:
    print("usage: blender --background --python measure_grip.py -- REF.fbx [PROP_BONE] [HAND_BONE]")
    sys.exit(2)
path = argv[0]
prop_name = argv[1] if len(argv) > 1 else "Sword_joint"
hand_name = argv[2] if len(argv) > 2 else "RightHand"

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.fbx(filepath=path)
arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")


def bone(name):
    for b in arm.data.bones:
        if b.name == name or b.name.split(":")[-1] == name:
            return b
    raise SystemExit(f"grip: no bone {name} in {path}")


prop = bone(prop_name)
hand = bone(hand_name)
mesh = next(
    (o for o in bpy.data.objects if o.type == "MESH" and prop.name in o.vertex_groups),
    None,
)
if mesh is None:
    raise SystemExit(f"grip: no mesh is bound to {prop.name}")
group = mesh.vertex_groups[prop.name].index
points = [
    mesh.matrix_world @ v.co
    for v in mesh.data.vertices
    if any(g.group == group and g.weight > 0.5 for g in v.groups)
]

# Principal axes of the prop's vertices: the longest is the blade, the next its width.
center = sum(points, Vector()) / len(points)
cov = Matrix(((0.0,) * 3,) * 3)
for p in points:
    d = p - center
    for i in range(3):
        for j in range(3):
            cov[i][j] += d[i] * d[j]


def principal(m, skip=()):
    v = Vector((1.0, 0.3, 0.1))
    for _ in range(200):
        for s in skip:
            v -= s * v.dot(s)
        v = (m @ v).normalized()
    return v


blade = principal(cov)
width = principal(cov, (blade,))

# The blade runs from the pommel toward the tip: away from the hand.
hand_at = arm.matrix_world @ hand.head_local
ts = [(p - center).dot(blade) for p in points]
lo, hi = min(ts), max(ts)
if abs((center + blade * hi - hand_at).length) < abs((center + blade * lo - hand_at).length):
    blade.negate()
    lo, hi = -hi, -lo

# The grip: the point on the blade's axis nearest the prop bone, which sits in the fist.
prop_at = arm.matrix_world @ prop.head_local
grip = center + blade * (prop_at - center).dot(blade)

x = (width - blade * width.dot(blade)).normalized()
z = x.cross(blade).normalized()
frame = Matrix((
    (x.x, blade.x, z.x, grip.x),
    (x.y, blade.y, z.y, grip.y),
    (x.z, blade.z, z.z, grip.z),
    (0.0, 0.0, 0.0, 1.0),
))
hand_world = (arm.matrix_world @ hand.matrix_local).normalized()
rel = hand_world.inverted() @ frame
flat = [round(rel[i][j], 5) for i in range(4) for j in range(4)]
print(f"grip: blade {hi - lo:.2f} m, grip {(grip - hand_at).length * 100:.1f} cm from the {hand_name} bone")
print("grip: " + json.dumps(flat))
