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
    session.rover_yaw = -std::f32::consts::FRAC_PI_4;
    session.rover_speed = 0.0;
    session.status_banner = format!(
        "Commissioned [{}] | GTA Chase Cam Active | [W/S] Drive/Reverse | [A/D] Steer | [Q] Visual QA | [U] Equip {} | [E] Export",
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
    let cab_mesh = meshes.add(Cuboid::new(0.60, 0.24, 0.68));
    let cab_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.16, 0.24),
        metallic: 0.8,
        perceptual_roughness: 0.2,
        ..default()
    });
    let taillight_mesh = meshes.add(Cuboid::new(0.05, 0.08, 0.65));
    let taillight_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.15, 0.15),
        emissive: LinearRgba::rgb(2.5, 0.1, 0.1),
        ..default()
    });
    let headlight_mesh = meshes.add(Cuboid::new(0.05, 0.08, 0.65));
    let headlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.98, 1.0),
        emissive: LinearRgba::rgb(2.0, 2.4, 2.8),
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
                    base_color: Color::srgb(0.20, 0.78, 0.98),
                    base_color_texture: chassis_texture.clone(),
                    metallic: 0.45,
                    perceptual_roughness: 0.3,
                    ..default()
                });

                let initial_rot = Quat::from_rotation_y(session.rover_yaw);

                // Spawn articulated 4-wheel MuJoCo Rover (+X is forward)
                commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(
                                1.45 * spec.scale,
                                0.30 * spec.scale,
                                0.86 * spec.scale,
                            )),
                            material: chassis_mat,
                            transform: Transform::from_xyz(spec.x, spec.y, spec.z)
                                .with_rotation(initial_rot),
                            ..default()
                        },
                        StudioEntity {
                            id: spec.id.clone(),
                            role: EntityRole::Player,
                            solid: false,
                            half_extents: Vec3::new(0.72, 0.25, 0.45) * spec.scale,
                        },
                    ))
                    .with_children(|parent| {
                        // Cockpit / Cabin (slightly rear of center for classic off-road rover silhouette)
                        parent.spawn(PbrBundle {
                            mesh: cab_mesh.clone(),
                            material: cab_mat.clone(),
                            transform: Transform::from_xyz(-0.10, 0.24, 0.0),
                            ..default()
                        });
                        // Glowing red rear light bar (visible in GTA behind-the-car view!)
                        parent.spawn(PbrBundle {
                            mesh: taillight_mesh.clone(),
                            material: taillight_mat.clone(),
                            transform: Transform::from_xyz(-0.73, 0.06, 0.0),
                            ..default()
                        });
                        // Front headlights
                        parent.spawn(PbrBundle {
                            mesh: headlight_mesh.clone(),
                            material: headlight_mat.clone(),
                            transform: Transform::from_xyz(0.73, 0.06, 0.0),
                            ..default()
                        });
                        let wheel_offsets = [
                            Vec3::new(0.48, -0.10, 0.48),
                            Vec3::new(0.48, -0.10, -0.48),
                            Vec3::new(-0.48, -0.10, 0.48),
                            Vec3::new(-0.48, -0.10, -0.48),
                        ];
                        for offset in wheel_offsets {
                            parent.spawn(PbrBundle {
                                mesh: wheel_mesh.clone(),
                                material: wheel_mat.clone(),
                                transform: Transform::from_translation(offset).with_rotation(
                                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                                ),
                                ..default()
                            });
                        }
                        // 2-DOF Modular Actuator Arm Mount on front hood
                        parent.spawn(PbrBundle {
                            mesh: arm_mesh.clone(),
                            material: arm_mat.clone(),
                            transform: Transform::from_xyz(0.42, 0.24, 0.0)
                                .with_rotation(Quat::from_rotation_z(0.35)),
                            ..default()
                        });
                    });
            }
            "goal" => {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Torus::new(0.45 * spec.scale, 0.95 * spec.scale)),
                        material: materials.add(StandardMaterial {
                            base_color: Color::srgb(0.1, 1.0, 0.45),
                            emissive: LinearRgba::rgb(0.2, 2.0, 0.5),
                            ..default()
                        }),
                        transform: Transform::from_xyz(spec.x, spec.y + 0.6, spec.z)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                        ..default()
                    },
                    StudioEntity {
                        id: spec.id.clone(),
                        role: EntityRole::Goal,
                        solid: false,
                        half_extents: Vec3::splat(0.9 * spec.scale),
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
