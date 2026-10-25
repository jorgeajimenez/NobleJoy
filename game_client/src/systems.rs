use crate::capture::capture_viewport_to_disk;
use crate::components::{EntityRole, GameSessionState, StatusOverlayText, StudioEntity};
use crate::level_builder::{apply_hotfix_deltas, spawn_level_from_contract};
use crate::network::{
    request_commission_level, request_qa_audit, StudioNetEvent, StudioNetworkChannel,
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
    // Ground floor plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(28.0, 28.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.13, 0.17),
            perceptual_roughness: 0.85,
            ..default()
        }),
        ..default()
    });

    // Studio lighting
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 12_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(14.0, 22.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Isometric / tactical 3D camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-8.0, 16.0, 18.0)
            .looking_at(Vec3::new(4.0, 0.5, 4.0), Vec3::Y),
        ..default()
    });

    // HUD Status Overlay
    commands.spawn((
        TextBundle::from_section(
            "JORGE RUST CLIENT | Press [C] Commission Studio Level | [Q] Visual QA Audit | [WASD] Move",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Px(16.0),
            ..default()
        }),
        StatusOverlayText,
    ));

    session.status_banner =
        "Commissioning initial level from Gretchen Studio (Gemini 4 Argon)...".to_string();
    request_commission_level(
        &net,
        "Construct a sci-fi reactor bay with defensive blockades.".to_string(),
    );
}

pub fn handle_studio_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    net: Res<StudioNetworkChannel>,
    mut session: ResMut<GameSessionState>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        session.status_banner =
            "Studio Director & Architect generating new level contract...".to_string();
        request_commission_level(
            &net,
            "Design an abandoned hangar where the reactor core is shielded.".to_string(),
        );
    }

    if keys.just_pressed(KeyCode::KeyQ) {
        let snap_path = capture_viewport_to_disk(&main_window, &mut screenshot_manager);
        let project_id = if session.active_project_id.is_empty() {
            "job_4091".to_string()
        } else {
            session.active_project_id.clone()
        };
        session.status_banner = format!(
            "Streaming viewport PNG to Visual QA Tester Agent for [{}]...",
            project_id
        );
        request_qa_audit(&net, project_id, snap_path);
    }
}

pub fn poll_studio_network_events(
    mut commands: Commands,
    net: Res<StudioNetworkChannel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<StudioEntity>>,
    mut transform_query: Query<(&StudioEntity, &mut Transform)>,
    mut session: ResMut<GameSessionState>,
) {
    while let Ok(event) = net.rx.try_recv() {
        match event {
            StudioNetEvent::LevelCommissioned(contract) => {
                spawn_level_from_contract(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &existing_entities,
                    &mut session,
                    &contract,
                );
            }
            StudioNetEvent::QaReportReceived(report) => {
                apply_hotfix_deltas(&mut transform_query, &report.hotfixes);
                session.status_banner = format!(
                    "QA Status [{}]: {}",
                    report.qa_status.to_uppercase(),
                    report.critique
                );
            }
            StudioNetEvent::NetworkError(err) => {
                session.status_banner = format!("Studio Network Alert: {err}");
            }
        }
    }
}

pub fn player_movement_and_collision(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&StudioEntity, &mut Transform)>,
    mut session: ResMut<GameSessionState>,
) {
    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir += Vec3::new(1.0, 0.0, 1.0);
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir -= Vec3::new(1.0, 0.0, 1.0);
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir += Vec3::new(-1.0, 0.0, 1.0);
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir += Vec3::new(1.0, 0.0, -1.0);
    }

    if keys.just_pressed(KeyCode::KeyR) {
        for (ent, mut transform) in query.iter_mut() {
            if ent.role == EntityRole::Player {
                transform.translation = Vec3::new(0.0, 0.5, 0.0);
                session.victory = false;
            }
        }
        return;
    }

    if dir.length_squared() == 0.0 {
        return;
    }
    let velocity = dir.normalize() * 6.5 * time.delta_seconds();

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
            if proposed.distance(g_pos) < 1.35 && !session.victory {
                session.victory = true;
                session.status_banner =
                    "OBJECTIVE COMPLETE: Reactor Data Core Reached! Press [C] for next commission."
                        .to_string();
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
    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!(
            "JORGE ENGINE <-> GRETCHEN STUDIO (Gemini 4 Argon)\n{}",
            session.status_banner
        );
    }
}
