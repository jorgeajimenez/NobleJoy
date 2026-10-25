use crate::components::{EntityRole, GameSessionState, StudioEntity};
use crate::scene_parser::{CarDesignSpec, HotfixDelta, LevelContract};
use bevy::prelude::*;

pub fn spawn_player_car_entity(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    yaw: f32,
    scale: f32,
    car_design: &CarDesignSpec,
    local_skin_asset: &Option<String>,
) {
    let chassis_texture: Option<Handle<Image>> = local_skin_asset
        .as_ref()
        .map(|rel_path| asset_server.load(rel_path.clone()));

    let [pr, pg, pb] = car_design.primary_color;
    let [ar, ag, ab] = car_design.accent_color;
    let ride_y = pos.y + car_design.ride_height;

    let is_medieval = car_design.body_style == "medieval_wagon";
    let is_crawler = car_design.body_style == "armored_crawler";
    let is_cyber = car_design.body_style == "cyber_interceptor";

    let (chassis_len, chassis_h, chassis_w) = if is_medieval {
        (1.55 * scale, 0.36 * scale, 0.92 * scale)
    } else if is_crawler {
        (1.48 * scale, 0.38 * scale, 0.96 * scale)
    } else if is_cyber {
        (1.60 * scale, 0.24 * scale, 0.94 * scale)
    } else {
        (1.45 * scale, 0.30 * scale, 0.86 * scale)
    };

    let chassis_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(pr, pg, pb),
        base_color_texture: chassis_texture,
        metallic: if is_medieval { 0.25 } else { 0.55 },
        perceptual_roughness: if is_medieval { 0.75 } else { 0.28 },
        ..default()
    });

    let accent_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(ar, ag, ab),
        emissive: if is_cyber {
            LinearRgba::rgb(ar * 1.4, ag * 1.4, ab * 1.4)
        } else {
            LinearRgba::BLACK
        },
        metallic: 0.75,
        perceptual_roughness: 0.25,
        ..default()
    });

    let wheel_r = 0.24 * car_design.wheel_scale;
    let wheel_w = 0.15 * car_design.wheel_scale;
    let wheel_mesh = meshes.add(Cylinder::new(wheel_r, wheel_w));
    let wheel_mat = materials.add(StandardMaterial {
        base_color: if is_medieval {
            Color::srgb(0.28, 0.20, 0.12)
        } else {
            Color::srgb(0.12, 0.13, 0.16)
        },
        perceptual_roughness: 0.75,
        ..default()
    });

    let cab_mesh = meshes.add(Cuboid::new(
        if is_cyber { 0.75 } else { 0.60 },
        if is_medieval { 0.34 } else { 0.24 },
        0.68,
    ));
    let cab_mat = materials.add(StandardMaterial {
        base_color: if is_medieval {
            Color::srgb(0.42, 0.28, 0.15)
        } else {
            Color::srgb(0.08, 0.13, 0.20)
        },
        metallic: if is_medieval { 0.2 } else { 0.85 },
        perceptual_roughness: if is_medieval { 0.8 } else { 0.18 },
        ..default()
    });

    let taillight_mesh = meshes.add(Cuboid::new(0.05, 0.08, 0.65));
    let taillight_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.15, 0.15),
        emissive: LinearRgba::rgb(2.8, 0.1, 0.1),
        ..default()
    });
    let headlight_mesh = meshes.add(Cuboid::new(0.06, 0.09, 0.65));
    let headlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.80),
        emissive: if is_medieval {
            LinearRgba::rgb(3.0, 1.8, 0.4)
        } else {
            LinearRgba::rgb(2.2, 2.5, 3.0)
        },
        ..default()
    });

    let initial_rot = Quat::from_rotation_y(yaw);

    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(chassis_len, chassis_h, chassis_w)),
                material: chassis_mat,
                transform: Transform::from_xyz(pos.x, ride_y, pos.z).with_rotation(initial_rot),
                ..default()
            },
            StudioEntity {
                id: "spawn_pad".to_string(),
                role: EntityRole::Player,
                solid: false,
                half_extents: Vec3::new(0.82, 0.35, 0.58) * scale,
            },
        ))
        .with_children(|parent| {
            // Cabin / canopy
            parent.spawn(PbrBundle {
                mesh: cab_mesh,
                material: cab_mat,
                transform: Transform::from_xyz(-0.08, 0.24, 0.0),
                ..default()
            });
            // Rear spoiler or iron trim
            parent.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.18, 0.14, chassis_w * 0.95)),
                material: accent_mat.clone(),
                transform: Transform::from_xyz(-chassis_len * 0.44, 0.22, 0.0),
                ..default()
            });
            parent.spawn(PbrBundle {
                mesh: taillight_mesh,
                material: taillight_mat,
                transform: Transform::from_xyz(-chassis_len * 0.50, 0.05, 0.0),
                ..default()
            });
            parent.spawn(PbrBundle {
                mesh: headlight_mesh,
                material: headlight_mat,
                transform: Transform::from_xyz(chassis_len * 0.50, 0.05, 0.0),
                ..default()
            });

            // 4 Wheels
            let wx = chassis_len * 0.34;
            let wz = chassis_w * 0.55;
            let wheel_offsets = [
                Vec3::new(wx, -0.10, wz),
                Vec3::new(wx, -0.10, -wz),
                Vec3::new(-wx, -0.10, wz),
                Vec3::new(-wx, -0.10, -wz),
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

            // Custom hardware attachment (Battering Ram / Twin Turbo / M3 Servo Arm)
            match car_design.roof_attachment.as_str() {
                "battering_ram" => {
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Cylinder::new(0.11, 0.95)),
                        material: accent_mat.clone(),
                        transform: Transform::from_xyz(chassis_len * 0.45, 0.08, 0.0)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                        ..default()
                    });
                }
                "twin_turbo" => {
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Cuboid::new(0.38, 0.18, 0.44)),
                        material: accent_mat.clone(),
                        transform: Transform::from_xyz(0.32, 0.22, 0.0),
                        ..default()
                    });
                }
                _ => {
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Cuboid::new(0.48, 0.09, 0.09)),
                        material: accent_mat,
                        transform: Transform::from_xyz(0.40, 0.25, 0.0)
                            .with_rotation(Quat::from_rotation_z(0.35)),
                        ..default()
                    });
                }
            }
        });
}

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
    session.current_level = contract.level_index.max(1);
    session.game_title = contract.game_title.clone();
    session.plot_summary = contract.plot_summary.clone();
    session.objective = contract.objective.clone();
    session.theme = contract.theme.clone();
    session.environment_style = contract.environment_style.clone();
    session.car_design = contract.car_design.clone();
    session.current_contract = Some(contract.clone());
    session.environmental_hazard = contract.environmental_hazard.clone();
    session.reward_part = contract.reward_part.clone();
    session.sanity_check = contract.sanity_check.clone();
    if local_skin_asset.is_some() {
        session.downloaded_skin_asset = local_skin_asset.clone();
    }
    session.victory = false;
    session.upgrade_equipped = false;
    session.qa_cleaned = false;
    session.rover_yaw = 0.0;
    session.rover_speed = 0.0;
    session.last_gbuffer_summary =
        "Pending QA (Press [Q] to run WGPU Compute G-Buffer Validator + Roadblock Hotfix)"
            .to_string();
    session.status_banner = format!(
        "{} (Lvl {}) | [Tab] AI Studio Menu | [V] Car Designer | [W/A/S/D] Drive | [Q] WGPU QA Fix | [N] Next Level",
        session.game_title, session.current_level
    );

    let env = &contract.environment_style;
    let is_medieval = env.building_style == "medieval_keep" || contract.theme.contains("medieval");
    let is_desert = env.building_style == "desert_mesa" || contract.theme.contains("desert");
    let is_arctic = env.building_style == "arctic_dome" || contract.theme.contains("arctic");

    // 1. Dynamic Theme Ground Terrain
    let [gr, gg, gb] = env.ground_color;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(180.0, 110.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(gr, gg, gb),
                perceptual_roughness: 0.95,
                ..default()
            }),
            transform: Transform::from_xyz(28.0, -0.02, 0.0),
            ..default()
        },
        StudioEntity {
            id: "env_ground".to_string(),
            role: EntityRole::EnvironmentProp,
            solid: false,
            half_extents: Vec3::ZERO,
        },
    ));

    // 2. Dynamic Street / Cobblestone / Dirt / Ice Roadway
    let [rr, rg, rb] = env.road_color;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(120.0, 0.04, 9.6)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(rr, rg, rb),
                perceptual_roughness: if is_arctic { 0.25 } else { 0.88 },
                metallic: if is_arctic { 0.3 } else { 0.05 },
                ..default()
            }),
            transform: Transform::from_xyz(28.0, 0.0, 0.0),
            ..default()
        },
        StudioEntity {
            id: "env_road".to_string(),
            role: EntityRole::EnvironmentProp,
            solid: false,
            half_extents: Vec3::ZERO,
        },
    ));

    // 3. Raised Curbs / Castle Borders & SOLID Corridor Boundary Walls (prevents leaving the world!)
    let [sr, sg, sb] = env.sidewalk_color;
    let sidewalk_mesh = meshes.add(Cuboid::new(120.0, 0.22, 2.2));
    let sidewalk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(sr, sg, sb),
        perceptual_roughness: 0.85,
        ..default()
    });
    for side_z in [-5.8_f32, 5.8_f32] {
        commands.spawn((
            PbrBundle {
                mesh: sidewalk_mesh.clone(),
                material: sidewalk_mat.clone(),
                transform: Transform::from_xyz(28.0, 0.11, side_z),
                ..default()
            },
            StudioEntity {
                id: format!("env_sidewalk_{side_z}"),
                role: EntityRole::EnvironmentProp,
                solid: false,
                half_extents: Vec3::ZERO,
            },
        ));
    }

    // Solid outer corridor walls along Z = -6.8 and Z = +6.8 so the car stays on the street & never clips through buildings!
    let wall_mesh = meshes.add(Cuboid::new(120.0, 1.2, 0.6));
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(sr * 0.85, sg * 0.85, sb * 0.85),
        perceptual_roughness: 0.8,
        ..default()
    });
    for side_z in [-6.8_f32, 6.8_f32] {
        commands.spawn((
            PbrBundle {
                mesh: wall_mesh.clone(),
                material: wall_mat.clone(),
                transform: Transform::from_xyz(28.0, 0.5, side_z),
                ..default()
            },
            StudioEntity {
                id: format!("corridor_wall_{side_z}"),
                role: EntityRole::Obstacle,
                solid: true,
                half_extents: Vec3::new(60.0, 1.0, 0.35),
            },
        ));
    }

    // Solid rear boundary behind spawn pad (X = -8.0)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.8, 1.5, 14.0)),
            material: wall_mat.clone(),
            transform: Transform::from_xyz(-8.0, 0.75, 0.0),
            ..default()
        },
        StudioEntity {
            id: "rear_boundary_wall".to_string(),
            role: EntityRole::Obstacle,
            solid: true,
            half_extents: Vec3::new(0.5, 1.0, 7.0),
        },
    ));

    // 4. Road Markings OR Medieval Cobblestones + Roadside Torches / Streetlamps
    let dash_mesh = meshes.add(Cuboid::new(1.8, 0.05, 0.18));
    let cobble_mesh = meshes.add(Cuboid::new(1.2, 0.05, 1.6));
    let dash_mat = materials.add(StandardMaterial {
        base_color: if is_medieval {
            Color::srgb(0.44, 0.39, 0.33)
        } else {
            Color::srgb(1.0, 0.82, 0.12)
        },
        emissive: if env.has_center_dashes {
            LinearRgba::rgb(0.8, 0.6, 0.05)
        } else {
            LinearRgba::BLACK
        },
        ..default()
    });

    let pole_mesh = meshes.add(Cylinder::new(0.09, 3.8));
    let pole_mat = materials.add(StandardMaterial {
        base_color: if is_medieval {
            Color::srgb(0.25, 0.18, 0.12)
        } else {
            Color::srgb(0.3, 0.35, 0.42)
        },
        metallic: if is_medieval { 0.3 } else { 0.8 },
        ..default()
    });

    let bulb_mesh = if env.prop_style == "torch_post" || is_medieval || is_desert {
        meshes.add(Sphere::new(0.24))
    } else {
        meshes.add(Cuboid::new(0.35, 0.14, 0.65))
    };
    let bulb_mat = materials.add(StandardMaterial {
        base_color: if is_medieval || is_desert {
            Color::srgb(1.0, 0.55, 0.12)
        } else if is_arctic {
            Color::srgb(0.2, 0.9, 1.0)
        } else {
            Color::srgb(1.0, 0.92, 0.75)
        },
        emissive: if is_medieval || is_desert {
            LinearRgba::rgb(3.5, 1.4, 0.15)
        } else if is_arctic {
            LinearRgba::rgb(0.3, 2.4, 3.2)
        } else {
            LinearRgba::rgb(2.5, 2.0, 1.2)
        },
        ..default()
    });

    for i in -2..=20 {
        let x_pos = i as f32 * 4.0;
        if env.has_center_dashes {
            commands.spawn((
                PbrBundle {
                    mesh: dash_mesh.clone(),
                    material: dash_mat.clone(),
                    transform: Transform::from_xyz(x_pos, 0.01, 0.0),
                    ..default()
                },
                StudioEntity {
                    id: format!("env_dash_{i}"),
                    role: EntityRole::EnvironmentProp,
                    solid: false,
                    half_extents: Vec3::ZERO,
                },
            ));
        } else if is_medieval {
            for cz in [-2.2_f32, 0.0, 2.2] {
                commands.spawn((
                    PbrBundle {
                        mesh: cobble_mesh.clone(),
                        material: dash_mat.clone(),
                        transform: Transform::from_xyz(
                            x_pos + (cz * 0.3),
                            0.008,
                            cz,
                        ),
                        ..default()
                    },
                    StudioEntity {
                        id: format!("env_cobble_{i}_{cz}"),
                        role: EntityRole::EnvironmentProp,
                        solid: false,
                        half_extents: Vec3::ZERO,
                    },
                ));
            }
        }

        if i % 3 == 0 {
            for side_z in [-5.1_f32, 5.1_f32] {
                commands.spawn((
                    PbrBundle {
                        mesh: pole_mesh.clone(),
                        material: pole_mat.clone(),
                        transform: Transform::from_xyz(x_pos, 1.9, side_z),
                        ..default()
                    },
                    StudioEntity {
                        id: format!("env_pole_{i}_{side_z}"),
                        role: EntityRole::EnvironmentProp,
                        solid: false,
                        half_extents: Vec3::ZERO,
                    },
                ));
                commands.spawn((
                    PbrBundle {
                        mesh: bulb_mesh.clone(),
                        material: bulb_mat.clone(),
                        transform: Transform::from_xyz(
                            x_pos,
                            3.85,
                            side_z - side_z.signum() * 0.25,
                        ),
                        ..default()
                    },
                    StudioEntity {
                        id: format!("env_bulb_{i}_{side_z}"),
                        role: EntityRole::EnvironmentProp,
                        solid: false,
                        half_extents: Vec3::ZERO,
                    },
                ));
            }
        }
    }

    // 5. Spawn Uncleaned Coplanar Z-Fighting Artifact Patch (detected & cleared by WGPU Compute Shader on [Q])
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(3.2, 0.04, 3.2)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.15, 0.85),
                emissive: LinearRgba::rgb(1.4, 0.1, 1.2),
                ..default()
            }),
            transform: Transform::from_xyz(16.2, 0.002, 0.0),
            ..default()
        },
        StudioEntity {
            id: "z_fighting_hazard_patch".to_string(),
            role: EntityRole::ZFightArtifact,
            solid: false,
            half_extents: Vec3::ZERO,
        },
    ));

    // 6. Spawn Level Entities (Player Car, Goal, Theme Buildings, Solid Chicanes & Blockade)
    let window_mat = materials.add(StandardMaterial {
        base_color: if is_medieval || is_desert {
            Color::srgb(1.0, 0.72, 0.25)
        } else {
            Color::srgb(0.1, 0.85, 1.0)
        },
        emissive: if is_medieval || is_desert {
            LinearRgba::rgb(2.0, 1.1, 0.2)
        } else {
            LinearRgba::rgb(0.2, 1.1, 1.6)
        },
        ..default()
    });

    for spec in &contract.entities {
        match spec.asset_key.as_str() {
            "player" => {
                spawn_player_car_entity(
                    commands,
                    asset_server,
                    meshes,
                    materials,
                    Vec3::new(spec.x, spec.y, spec.z),
                    session.rover_yaw,
                    spec.scale,
                    &session.car_design,
                    &session.downloaded_skin_asset,
                );
            }
            "goal" => {
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Torus::new(0.48 * spec.scale, 1.25 * spec.scale)),
                        material: materials.add(StandardMaterial {
                            base_color: Color::srgb(0.15, 1.0, 0.45),
                            emissive: LinearRgba::rgb(0.3, 2.5, 0.65),
                            ..default()
                        }),
                        transform: Transform::from_xyz(spec.x, spec.y + 0.9, spec.z)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                        ..default()
                    },
                    StudioEntity {
                        id: spec.id.clone(),
                        role: EntityRole::Goal,
                        solid: false,
                        half_extents: Vec3::splat(1.3 * spec.scale),
                    },
                ));
            }
            "building" => {
                let width = 6.2;
                let height = 3.0 * spec.scale;
                let depth = 4.8;
                let bldg_color = if is_medieval {
                    Color::srgb(0.42, 0.38, 0.33)
                } else if is_desert {
                    Color::srgb(0.68, 0.36, 0.20)
                } else if is_arctic {
                    Color::srgb(0.75, 0.86, 0.95)
                } else {
                    Color::srgb(0.13, 0.16, 0.23)
                };
                let bldg_mat = materials.add(StandardMaterial {
                    base_color: bldg_color,
                    metallic: if is_medieval || is_desert { 0.1 } else { 0.65 },
                    perceptual_roughness: if is_medieval || is_desert { 0.88 } else { 0.35 },
                    ..default()
                });
                let accent_strip_mesh = meshes.add(Cuboid::new(width + 0.08, 0.18, depth + 0.08));
                let battlement_mesh = meshes.add(Cuboid::new(width + 0.35, 0.55, depth + 0.35));

                commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(width, height, depth)),
                            material: bldg_mat.clone(),
                            transform: Transform::from_xyz(spec.x, height * 0.5, spec.z),
                            ..default()
                        },
                        StudioEntity {
                            id: spec.id.clone(),
                            role: EntityRole::Building,
                            solid: true,
                            half_extents: Vec3::new(width * 0.5, height * 0.5, depth * 0.5),
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(PbrBundle {
                            mesh: accent_strip_mesh.clone(),
                            material: window_mat.clone(),
                            transform: Transform::from_xyz(0.0, height * 0.22, 0.0),
                            ..default()
                        });
                        parent.spawn(PbrBundle {
                            mesh: accent_strip_mesh,
                            material: window_mat.clone(),
                            transform: Transform::from_xyz(0.0, -height * 0.15, 0.0),
                            ..default()
                        });
                        if is_medieval {
                            // Castle stone battlements crown at the top of medieval towers/keeps
                            parent.spawn(PbrBundle {
                                mesh: battlement_mesh,
                                material: bldg_mat,
                                transform: Transform::from_xyz(0.0, height * 0.5 + 0.2, 0.0),
                                ..default()
                            });
                        }
                    });
            }
            _ => {
                // Solid Street Obstacle / Medieval Cart / Portcullis Gate / Blast Door
                let is_blast_door = spec.id == "blast_door";
                let width = if is_blast_door {
                    1.2 * spec.scale
                } else {
                    1.3 * spec.scale
                };
                let height = 1.1 * spec.scale;
                let depth = if is_blast_door {
                    2.6 * spec.scale
                } else {
                    1.3 * spec.scale
                };

                let obs_color = if is_medieval {
                    if is_blast_door {
                        Color::srgb(0.28, 0.30, 0.34) // Iron Portcullis
                    } else {
                        Color::srgb(0.52, 0.34, 0.18) // Oak Merchant Cart / Barricade
                    }
                } else if is_desert {
                    Color::srgb(0.72, 0.28, 0.14)
                } else {
                    Color::srgb(0.95, 0.22, 0.20)
                };

                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cuboid::new(width, height, depth)),
                        material: materials.add(StandardMaterial {
                            base_color: obs_color,
                            emissive: if is_blast_door && !is_medieval {
                                LinearRgba::rgb(0.55, 0.05, 0.05)
                            } else {
                                LinearRgba::BLACK
                            },
                            metallic: if is_blast_door { 0.75 } else { 0.3 },
                            perceptual_roughness: 0.45,
                            ..default()
                        }),
                        transform: Transform::from_xyz(spec.x, height * 0.5, spec.z),
                        ..default()
                    },
                    StudioEntity {
                        id: spec.id.clone(),
                        role: EntityRole::Obstacle,
                        solid: true,
                        half_extents: Vec3::new(width * 0.5, height * 0.5, depth * 0.5),
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
