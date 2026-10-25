# Local 3D Asset Repository

Place optional `.glb` models here:
- `player.glb`
- `obstacle.glb`
- `goal.glb`

When custom `.glb` meshes are not present, `level_builder.rs` automatically renders procedural PBR 3D primitives (`Capsule3d`, `Torus`, `Cuboid`) with emissive materials.
