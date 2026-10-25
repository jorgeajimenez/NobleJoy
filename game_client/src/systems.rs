use crate::capture::capture_viewport_to_disk;
use crate::components::{
    ChaseCamera, EntityRole, GameSessionState, StatusOverlayText, StudioEntity,
};
use crate::level_builder::{apply_hotfix_deltas, spawn_level_from_contract};
use crate::network::{
    request_commission_level, request_export_build, request_mujoco_step, request_qa_audit,
    StudioNetEvent, StudioNetworkChannel,
};
use crate::scene_parser::{
    EntitySpec, HotfixDelta, LevelContract, PhysicsSanityCheck, RewardPartSpec,
};
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

pub fn build_offline_preview_contract() -> LevelContract {
    let mut bounds = HashMap::new();
    bounds.insert("x".to_string(), 32.0);
    bounds.insert("z".to_string(), 32.0);

    LevelContract {
        project_id: "local_preview".to_string(),
        theme: "offline_rover_proving_ground".to_string(),
        world_bounds: bounds,
        environmental_hazard: "low_clearance_gap_40cm + blast_door".to_string(),
        required_capability: "high_torque_4wd + 2-DOF arm".to_string(),
        skin_url: String::new(),
        mjcf_url: String::new(),
        urdf_url: String::new(),
        stl_url: String::new(),
        reward_part: RewardPartSpec {
            part_id: "high_torque_wheel_hub_v2".to_string(),
            mount_type: "M3_faceplate_40mm".to_string(),
            compatible_motor: "MG996R_servo".to_string(),
            mass_grams: 180.0,
            torque_capacity_kg_cm: 11.0,
        },
        sanity_check: PhysicsSanityCheck {
            com_stable: true,
            required_torque_kg_cm: 8.75,
            rated_torque_kg_cm: 1.8,
            torque_valid: false,
            upgrade_equipped: false,
            motor_torque_nm: 3.5,
            wheel_friction: 1.2,
        },
        sandbox_files: vec![],
        entities: vec![
            EntitySpec {
                id: "spawn_pad".to_string(),
                asset_key: "player".to_string(),
                x: 0.0,
                y: 0.35,
                z: 0.0,
                scale: 1.0,
                solid: false,
            },
            EntitySpec {
                id: "data_core".to_string(),
                asset_key: "goal".to_string(),
                x: 11.0,
                y: 0.5,
                z: 11.0,
                scale: 1.0,
                solid: false,
            },
            EntitySpec {
                id: "track_barrier_west".to_string(),
                asset_key: "obstacle".to_string(),
                x: 2.5,
                y: 1.2,
                z: 6.5,
                scale: 2.0,
                solid: true,
            },
            EntitySpec {
                id: "track_barrier_east".to_string(),
                asset_key: "obstacle".to_string(),
                x: 6.5,
                y: 1.2,
                z: 2.5,
                scale: 2.0,
                solid: true,
            },
            EntitySpec {
                id: "blast_door".to_string(),
                asset_key: "obstacle".to_string(),
                x: 5.0,
                y: 1.5,
                z: 5.0,
                scale: 2.4,
                solid: true,
            },
        ],
    }
}

pub fn setup_base_environment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<StudioEntity>>,
    net: Res<StudioNetworkChannel>,
    mut session: ResMut<GameSessionState>,
) {
    // Large asphalt/industrial floor plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(80.0, 80.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.12, 0.16),
            perceptual_roughness: 0.88,
            ..default()
        }),
        ..default()
    });

    // Reference grid strips along the road so speed and steering feel palpable in GTA view
    let stripe_mesh = meshes.add(Cuboid::new(1.2, 0.02, 0.18));
    let stripe_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.35, 0.48),
        emissive: LinearRgba::rgb(0.05, 0.12, 0.22),
        ..default()
    });
    for i in -8..=16 {
        let p = i as f32 * 2.2;
        commands.spawn(PbrBundle {
            mesh: stripe_mesh.clone(),
            material: stripe_mat.clone(),
            transform: Transform::from_xyz(p, 0.01, p)
                .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4)),
            ..default()
        });
    }

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(12.0, 26.0, -10.0).looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
        ..default()
    });

    // GTA Third-Person Chase Camera: starts right behind and slightly above the rover at (0, 0.35, 0)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.2, 2.8, -4.2)
                .looking_at(Vec3::new(3.0, 0.8, 3.0), Vec3::Y),
            ..default()
        },
        ChaseCamera,
    ));

    commands.spawn((
        TextBundle::from_section(
            "JORGE RUST CLIENT + MUJOCO ROVER | Initializing GTA Chase Camera...",
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(14.0),
            ..default()
        }),
        StatusOverlayText,
    ));

    let preview = build_offline_preview_contract();
    spawn_level_from_contract(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &existing_entities,
        &mut session,
        &preview,
        None,
    );

    session.status_banner =
        "GTA Chase View Ready! [W/S] Accelerate/Reverse | [A/D] Steer | [Q] Visual QA Hotfix | [U] Snap Upgrade"
            .to_string();
    request_commission_level(
        &net,
        "Design an industrial rover test track with a shielded reactor core and high-torque wheel upgrade.".to_string(),
    );
}

pub fn handle_studio_hotkeys(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<StudioEntity>>,
    mut transform_query: Query<(&StudioEntity, &mut Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
    net: Res<StudioNetworkChannel>,
    mut session: ResMut<GameSessionState>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    let project_id = if session.active_project_id.is_empty() {
        "job_4091".to_string()
    } else {
        session.active_project_id.clone()
    };

    if keys.just_pressed(KeyCode::KeyL) {
        let preview = build_offline_preview_contract();
        spawn_level_from_contract(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            &existing_entities,
            &mut session,
            &preview,
            None,
        );
        session.status_banner =
            "Loaded Local Offline Preview Track | [W/S] Drive | [A/D] Steer | [Q] Simulate QA Hotfix | [U] Toggle M3 Upgrade"
                .to_string();
    }

    if keys.just_pressed(KeyCode::KeyC) {
        session.status_banner =
            "Studio Director & Architect generating new MuJoCo level + vehicle skin...".to_string();
        request_commission_level(
            &net,
            "Design an abandoned hangar rover track with low clearance gap and M3 actuator mount."
                .to_string(),
        );
    }

    if keys.just_pressed(KeyCode::KeyQ) {
        let snap_path = capture_viewport_to_disk(&main_window, &mut screenshot_manager);
        if session.active_project_id == "local_preview" {
            let offline_delta = vec![HotfixDelta {
                id: "blast_door".to_string(),
                action: "reposition".to_string(),
                x: 9.5,
                y: 1.5,
                z: 4.0,
            }];
            apply_hotfix_deltas(&mut transform_query, &offline_delta);
            session.sanity_check.motor_torque_nm = 5.5;
            session.sanity_check.wheel_friction = 1.55;
            session.sanity_check.rated_torque_kg_cm = 11.0;
            session.sanity_check.torque_valid = true;
            session.status_banner =
                "QA Status [HOTFIX_APPLIED]: Repositioned 'blast_door' out of lane & boosted rover torque to 5.5 N*m!"
                    .to_string();
        } else {
            session.status_banner = format!(
                "Streaming viewport PNG + MuJoCo telemetry to Visual QA Agent for [{project_id}]..."
            );
            request_qa_audit(&net, project_id.clone(), snap_path);
        }
    }

    if keys.just_pressed(KeyCode::KeyU) {
        session.upgrade_equipped = !session.upgrade_equipped;
        session.sanity_check.upgrade_equipped = session.upgrade_equipped;
        if session.upgrade_equipped {
            session.sanity_check.motor_torque_nm = 6.5;
            session.sanity_check.wheel_friction = 1.65;
            session.sanity_check.rated_torque_kg_cm = 11.0;
            session.sanity_check.torque_valid = true;
        } else {
            session.sanity_check.motor_torque_nm = 3.5;
            session.sanity_check.wheel_friction = 1.2;
            session.sanity_check.rated_torque_kg_cm = 1.8;
            session.sanity_check.torque_valid = false;
        }
        session.status_banner = format!(
            "Snapped M3 Modular Part [{}] onto Rover Chassis -> CoM STABLE | Servo Torque: {:.1} kg*cm",
            session.reward_part.part_id, session.sanity_check.rated_torque_kg_cm
        );
        if session.active_project_id != "local_preview" {
            request_mujoco_step(
                &net,
                project_id.clone(),
                0.0,
                0.0,
                Some(session.upgrade_equipped),
            );
        }
    }

    if keys.just_pressed(KeyCode::KeyE) {
        session.status_banner = format!(
            "Exporting Physical Build Bundle (M3 STLs + URDF + BOM.csv + MicroPython) for [{project_id}]..."
        );
        request_export_build(&net, project_id);
    }
}

pub fn poll_studio_network_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    net: Res<StudioNetworkChannel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<StudioEntity>>,
    mut transform_query: Query<(&StudioEntity, &mut Transform)>,
    mut session: ResMut<GameSessionState>,
) {
    while let Ok(event) = net.rx.try_recv() {
        match event {
            StudioNetEvent::LevelCommissioned {
                contract,
                local_skin_asset,
            } => {
                spawn_level_from_contract(
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &existing_entities,
                    &mut session,
                    &contract,
                    local_skin_asset,
                );
            }
            StudioNetEvent::MujocoStepCompleted(step) => {
                session.step_in_flight = false;
                session.mujoco_engine = step.engine;
                session.speed_mps = step.chassis.speed_mps;
                session.sanity_check = step.sanity_check;
            }
            StudioNetEvent::QaReportReceived(report) => {
                apply_hotfix_deltas(&mut transform_query, &report.hotfixes);
                if let Some(sc) = report.sanity_check {
                    session.sanity_check = sc;
                }
                session.status_banner = format!(
                    "QA Status [{}]: {}",
                    report.qa_status.to_uppercase(),
                    report.critique
                );
            }
            StudioNetEvent::BuildExported(export) => {
                session.status_banner = format!(
                    "EXPORTED BUILD PACKAGE -> {} (STLs: {}, URDF, BOM.csv, MicroPython main.py)",
                    export.export_dir,
                    export.stl_files.len()
                );
            }
            StudioNetEvent::NetworkError(_err) => {
                session.step_in_flight = false;
                session.status_banner =
                    "GTA Chase Cam Active (Offline Preview) | [W/S] Drive | [A/D] Steer | [Q] QA Hotfix | [U] Snap Upgrade | [C] Retry Studio"
                        .to_string();
            }
        }
    }
}

pub fn player_movement_and_collision(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    net: Res<StudioNetworkChannel>,
    mut query: Query<(&StudioEntity, &mut Transform), Without<ChaseCamera>>,
    mut session: ResMut<GameSessionState>,
) {
    let dt = time.delta_seconds();
    let mut throttle: f32 = 0.0;
    let mut steering: f32 = 0.0;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        throttle += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        throttle -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        steering += 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        steering -= 1.0;
    }

    if keys.just_pressed(KeyCode::KeyR) {
        session.rover_yaw = -std::f32::consts::FRAC_PI_4;
        session.rover_speed = 0.0;
        for (ent, mut transform) in query.iter_mut() {
            if ent.role == EntityRole::Player {
                transform.translation = Vec3::new(0.0, 0.35, 0.0);
                transform.rotation = Quat::from_rotation_y(session.rover_yaw);
                session.victory = false;
            }
        }
        return;
    }

    if (throttle != 0.0 || steering != 0.0)
        && !session.step_in_flight
        && !session.active_project_id.is_empty()
        && session.active_project_id != "local_preview"
    {
        session.step_in_flight = true;
        request_mujoco_step(
            &net,
            session.active_project_id.clone(),
            throttle,
            steering,
            None,
        );
    }

    // Car-relative Ackermann / GTA driving physics (+X local is car forward)
    let max_speed = if session.sanity_check.upgrade_equipped {
        11.5
    } else {
        8.0
    };
    let accel_rate = if session.sanity_check.upgrade_equipped {
        18.0
    } else {
        13.0
    };

    if throttle != 0.0 {
        session.rover_speed =
            (session.rover_speed + throttle * accel_rate * dt).clamp(-max_speed * 0.6, max_speed);
    } else {
        // Natural rolling drag
        session.rover_speed *= (1.0 - 4.5 * dt).max(0.0);
        if session.rover_speed.abs() < 0.05 {
            session.rover_speed = 0.0;
        }
    }

    // Steer heading angle (`A` turns left, `D` turns right)
    if steering != 0.0 {
        let turn_dir = if session.rover_speed >= -0.1 {
            1.0
        } else {
            -1.0
        };
        session.rover_yaw += steering * 2.1 * dt * turn_dir;
    }

    // In Bevy's right-handed Y-up coordinate system, Quat::from_rotation_y(yaw) rotates +X to (cos(yaw), 0, -sin(yaw))
    let forward = Vec3::new(session.rover_yaw.cos(), 0.0, -session.rover_yaw.sin());
    let velocity = forward * session.rover_speed * dt;
    session.speed_mps = session.rover_speed.abs();

    let mut obstacles = Vec::new();
    let mut goal_pos = None;
    let mut player_current = None;

    for (ent, transform) in query.iter() {
        match ent.role {
            EntityRole::Player => player_current = Some((ent.half_extents, transform.translation)),
            EntityRole::Goal => goal_pos = Some(transform.translation),
            EntityRole::Obstacle if ent.solid => {
                obstacles.push((ent.half_extents, transform.translation));
            }
            _ => {}
        }
    }

    let Some((player_extents, current_pos)) = player_current else {
        return;
    };
    let proposed = current_pos + velocity;

    let blocked = obstacles.iter().any(|(obs_extents, obs_pos)| {
        let dx = (proposed.x - obs_pos.x).abs();
        let dz = (proposed.z - obs_pos.z).abs();
        dx < (player_extents.x + obs_extents.x) && dz < (player_extents.z + obs_extents.z)
    });

    for (ent, mut transform) in query.iter_mut() {
        if ent.role == EntityRole::Player {
            transform.rotation = Quat::from_rotation_y(session.rover_yaw);
            if !blocked {
                transform.translation = proposed;
            } else {
                session.rover_speed = -session.rover_speed * 0.35;
            }
        }
    }

    if !blocked {
        if let Some(g_pos) = goal_pos {
            if proposed.distance(g_pos) < 1.65 && !session.victory {
                session.victory = true;
                session.status_banner = format!(
                    "OBJECTIVE COMPLETE! Unlocked [{}] ({}) | Press [U] Snap Part & Verify Torque | [E] Export STLs/BOM/Firmware",
                    session.reward_part.part_id, session.reward_part.mount_type
                );
            }
        }
    }
}

pub fn update_gta_chase_camera(
    time: Res<Time>,
    session: Res<GameSessionState>,
    player_query: Query<(&StudioEntity, &Transform), Without<ChaseCamera>>,
    mut cam_query: Query<&mut Transform, With<ChaseCamera>>,
) {
    let Some((_, player_tf)) = player_query
        .iter()
        .find(|(ent, _)| ent.role == EntityRole::Player)
    else {
        return;
    };

    let Ok(mut cam_tf) = cam_query.get_single_mut() else {
        return;
    };

    // Forward vector along car's heading (+X rotated by rover_yaw)
    let forward = Vec3::new(session.rover_yaw.cos(), 0.0, -session.rover_yaw.sin());

    // Classic GTA third-person camera offset: 5.2 units behind the bumper and 2.65 units above the roof
    let desired_cam_pos = player_tf.translation - forward * 5.2 + Vec3::new(0.0, 2.65, 0.0);
    // Look slightly ahead of the car's hood so the horizon, obstacles, and reactor core stay framed
    let look_target = player_tf.translation + forward * 3.8 + Vec3::new(0.0, 0.55, 0.0);

    let smooth = (10.0 * time.delta_seconds()).clamp(0.0, 1.0);
    cam_tf.translation = cam_tf.translation.lerp(desired_cam_pos, smooth);
    cam_tf.look_at(look_target, Vec3::Y);
}

pub fn update_hud_overlay(
    session: Res<GameSessionState>,
    mut text_query: Query<&mut Text, With<StatusOverlayText>>,
) {
    if !session.is_changed() {
        return;
    }
    let sc = &session.sanity_check;
    let com_badge = if sc.com_stable { "STABLE" } else { "TIPPING RISK" };
    let torque_badge = if sc.torque_valid { "PASS" } else { "STALL" };

    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!(
            "JORGE RUST CLIENT <-> GRETCHEN STUDIO (Gemini 4 Argon + MuJoCo)\n\
             Track: [{}] | Speed: {:.1} m/s | Hazard: {} | Skin: {}\n\
             MuJoCo Telemetry -> Drive Torque: {:.1} N*m | Friction: {:.2} | CoM: {} | Servo Torque (tau=r*F): {:.1}/{:.1} kg*cm [{}]\n\
             {}",
            session.theme,
            session.speed_mps,
            session.environmental_hazard,
            session
                .downloaded_skin_asset
                .as_deref()
                .unwrap_or("procedural_pbr"),
            sc.motor_torque_nm,
            sc.wheel_friction,
            com_badge,
            sc.required_torque_kg_cm,
            sc.rated_torque_kg_cm,
            torque_badge,
            session.status_banner
        );
    }
}
