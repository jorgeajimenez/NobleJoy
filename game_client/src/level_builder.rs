use crate::components::{EntityRole, GameSessionState, StudioEntity};
use crate::scene_parser::{HotfixDelta, LevelContract};
use bevy::prelude::*;

pub fn spawn_level_from_contract(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    existing_entities: &Query<Entity, With<StudioEntity>>,
    session: &mut ResMut<GameSessionState>,
    contract: &LevelContract,
    local_skin_asset: Option<String>,
) {
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    session.active_project_id = contract.project_id.clone();
    session.theme = contract.theme.clone();
    session.environmental_hazard = contract.environmental_hazard.clone();
    session.reward_part = contract.reward_part.clone();
    session.sanity_check = contract.sanity_check.clone();
    session.downloaded_skin_asset = local_skin_asset.clone();
    session.victory = false;
    session.upgrade_equipped = false;
    session.status_banner = format!(
        "Commissioned [{}] | Skin & MJCF pulled from Managed Agent | [WASD] Drive | [Q] Visual QA | [U] Equip {} | [E] Export STLs/URDF",
        contract.project_id, contract.reward_part.part_id
    );

    let chassis_texture: Option<Handle<Image>> =
        local_skin_asset.map(|rel_path| asset_server.load(rel_path));

    let wheel_mesh = meshes.add(Cylinder::new(0.24, 0.14));
    let wheel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.15, 0.18),
        perceptual_roughness: 0.7,
        ..default()
    });
    let arm_mesh = meshes.add(Cuboid::new(0.45, 0.08, 0.08));
    let arm_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.1),
        metallic: 0.7,
        ..default()
    });

    for spec in &contract.entities {
        match spec.asset_key.as_str() {
            "player" => {
                let chassis_mat = materials.add(StandardMaterial {
                    base_color: Color::srgb(0.85, 0.95, 1.0),
                    base_color_texture: chassis_texture.clone(),
                    metallic: 0.35,
                    perceptual_roughness: 0.35,
                    ..default()
                });

                // Spawn articulated 4-wheel MuJoCo Rover + 2-DOF modular arm
                commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(
                                1.3 * spec.scale,
                                0.32 * spec.scale,
                                0.85 * spec.scale,
                            )),
                            material: chassis_mat,
                            transform: Transform::from_xyz(spec.x, spec.y, spec.z),
                            ..default()
                        },
                        StudioEntity {
                            id: spec.id.clone(),
                            role: EntityRole::Player,
                            solid: false,
                            half_extents: Vec3::new(0.65, 0.25, 0.45) * spec.scale,
                        },
                    ))
                    .with_children(|parent| {
                        let wheel_offsets = [
                            Vec3::new(0.45, -0.10, 0.48),
                            Vec3::new(0.45, -0.10, -0.48),
                            Vec3::new(-0.45, -0.10, 0.48),
                            Vec3::new(-0.45, -0.10, -0.48),
                        ];
                        for offset in wheel_offsets {
                            parent.spawn(PbrBundle {
                                mesh: wheel_mesh.clone(),
                                material: wheel_mat.clone(),
                                transform: Transform::from_translation(offset)
                                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                                ..default()
                            });
                        }
                        // 2-DOF Modular Actuator Arm Mount on top of chassis
                        parent.spawn(PbrBundle {
                            mesh: arm_mesh.clone(),
                            material: arm_mat.clone(),
                            transform: Transform::from_xyz(0.35, 0.24, 0.0)
                                .with_rotation(Quat::from_rotation_z(0.35)),
                            ..default()
                        });
                    });
            }
            "goal" => {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Torus::new(0.35 * spec.scale, 0.75 * spec.scale)),
                        material: materials.add(StandardMaterial {
                            base_color: Color::srgb(0.1, 1.0, 0.45),
                            emissive: LinearRgba::rgb(0.1, 1.2, 0.35),
                            ..default()
                        }),
                        transform: Transform::from_xyz(spec.x, spec.y, spec.z),
                        ..default()
                    },
                    StudioEntity {
                        id: spec.id.clone(),
                        role: EntityRole::Goal,
                        solid: false,
                        half_extents: Vec3::splat(0.8 * spec.scale),
                    },
                ));
            }
            _ => {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(
                            1.2 * spec.scale,
                            1.2 * spec.scale,
                            1.2 * spec.scale,
                        )),
                        material: materials.add(StandardMaterial {
                            base_color: Color::srgb(0.92, 0.24, 0.24),
                            metallic: 0.5,
                            perceptual_roughness: 0.4,
                            ..default()
                        }),
                        transform: Transform::from_xyz(spec.x, spec.y, spec.z),
                        ..default()
                    },
                    StudioEntity {
                        id: spec.id.clone(),
                        role: EntityRole::Obstacle,
                        solid: spec.solid,
                        half_extents: Vec3::splat(0.6 * spec.scale),
                    },
                ));
            }
        }
    }
}

pub fn apply_hotfix_deltas(
    query: &mut Query<(&StudioEntity, &mut Transform)>,
    hotfixes: &[HotfixDelta],
) {
    for delta in hotfixes {
        if delta.action == "reposition" {
            for (studio_ent, mut transform) in query.iter_mut() {
                if studio_ent.id == delta.id {
                    transform.translation = Vec3::new(delta.x, delta.y, delta.z);
                }
            }
        }
    }
}
