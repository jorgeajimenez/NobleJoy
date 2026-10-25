use crate::capture::capture_viewport_to_disk;
use crate::components::{EntityRole, GameSessionState, StatusOverlayText, StudioEntity};
use crate::level_builder::{apply_hotfix_deltas, spawn_level_from_contract};
use crate::network::{
    request_commission_level, request_export_build, request_mujoco_step, request_qa_audit,
    StudioNetEvent, StudioNetworkChannel,
};
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;

pub fn setup_base_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    net: Res<StudioNetworkChannel>,
    mut session: ResMut<GameSessionState>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(28.0, 28.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.13, 0.17),
            perceptual_roughness: 0.85,
            ..default()
        }),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(14.0, 22.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-8.0, 16.0, 18.0)
            .looking_at(Vec3::new(4.0, 0.5, 4.0), Vec3::Y),
        ..default()
    });

    commands.spawn((
        TextBundle::from_section(
            "JORGE RUST CLIENT + MUJOCO ROVER | Commissioning Gretchen Studio...",
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

    session.status_banner =
        "Commissioning MuJoCo driving course & skin from Gretchen Studio (Gemini 4 Argon)..."
            .to_string();
    request_commission_level(
        &net,
        "Design an industrial rover test track with a shielded reactor core and high-torque wheel upgrade.".to_string(),
    );
}

pub fn handle_studio_hotkeys(
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
        session.status_banner = format!(
            "Streaming viewport PNG + MuJoCo telemetry to Visual QA Agent for [{project_id}]..."
        );
        request_qa_audit(&net, project_id.clone(), snap_path);
    }

    if keys.just_pressed(KeyCode::KeyU) {
        session.upgrade_equipped = !session.upgrade_equipped;
        session.status_banner = format!(
            "Snapped M3 Modular Part [{}] onto Rover Chassis -> Verifying CoM & Torque in MuJoCo...",
            session.reward_part.part_id
        );
        request_mujoco_step(&net, project_id.clone(), 0.0, 0.0, Some(session.upgrade_equipped));
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
                for (ent, mut transform) in transform_query.iter_mut() {
                    if ent.role == EntityRole::Player {
                        transform.translation =
                            Vec3::new(step.chassis.x, step.chassis.y, step.chassis.z);
                        let [w, x, y, z] = step.chassis.quat_wxyz;
                        transform.rotation = Quat::from_xyzw(x, y, z, w);
                    }
                }
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
            StudioNetEvent::NetworkError(err) => {
                session.step_in_flight = false;
                session.status_banner = format!("Studio Network Alert: {err}");
            }
        }
    }
}

pub fn player_movement_and_collision(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    net: Res<StudioNetworkChannel>,
    mut query: Query<(&StudioEntity, &mut Transform)>,
    mut session: ResMut<GameSessionState>,
) {
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
        for (ent, mut transform) in query.iter_mut() {
            if ent.role == EntityRole::Player {
                transform.translation = Vec3::new(0.0, 0.35, 0.0);
                session.victory = false;
            }
        }
        return;
    }

    if throttle == 0.0 && steering == 0.0 {
        return;
    }

    // Stream control inputs to MuJoCo backend if not already in flight
    if !session.step_in_flight && !session.active_project_id.is_empty() {
        session.step_in_flight = true;
        request_mujoco_step(
            &net,
            session.active_project_id.clone(),
            throttle,
            steering,
            None,
        );
    }

    // Immediate client-side predictive visual responsiveness + collision check
    let mut dir = Vec3::ZERO;
    if throttle != 0.0 {
        dir += Vec3::new(throttle, 0.0, throttle);
    }
    if steering != 0.0 {
        dir += Vec3::new(-steering * 0.7, 0.0, steering * 0.7);
    }

    let speed_mult = if session.sanity_check.upgrade_equipped {
        8.5
    } else {
        6.0
    };
    let velocity = dir.normalize_or_zero() * speed_mult * time.delta_seconds();

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

    if !blocked {
        for (ent, mut transform) in query.iter_mut() {
            if ent.role == EntityRole::Player {
                transform.translation = proposed;
            }
        }
        if let Some(g_pos) = goal_pos {
            if proposed.distance(g_pos) < 1.45 && !session.victory {
                session.victory = true;
                session.status_banner = format!(
                    "OBJECTIVE COMPLETE! Unlocked [{}] ({}) | Press [U] Snap Part & Verify Torque | [E] Export STLs/BOM/Firmware",
                    session.reward_part.part_id, session.reward_part.mount_type
                );
            }
        }
    }
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
             Track: [{}] | Hazard: {} | Skin: {}\n\
             MuJoCo Telemetry -> Drive Torque: {:.1} N*m | Friction: {:.2} | CoM Footprint: {} | Servo Torque (tau=r*F): {:.1}/{:.1} kg*cm [{}]\n\
             {}",
            session.theme,
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
