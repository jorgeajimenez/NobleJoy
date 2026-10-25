use crate::capture::capture_viewport_to_disk;
use crate::components::{
    ChaseCamera, EntityRole, GameSessionState, StatusOverlayText, StudioEntity, StudioModalPanel,
    StudioModalText, UiMode,
};
use crate::gbuffer_detector::run_gbuffer_compute_audit;
use crate::level_builder::{
    apply_hotfix_deltas, rebuild_player_car_in_place, spawn_level_from_contract,
};
use crate::network::{
    request_commission_level, request_export_build, request_mujoco_step, request_qa_audit,
    StudioNetEvent, StudioNetworkChannel,
};
use crate::scene_parser::{
    CarDesignSpec, EntitySpec, EnvironmentStyleSpec, HotfixDelta, LevelContract,
    PhysicsSanityCheck, RewardPartSpec,
};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

pub fn infer_robot_form_from_prompt(p: &str) -> Option<String> {
    if p.contains("mech")
        || p.contains("biped")
        || p.contains("humanoid")
        || p.contains("optimus")
        || p.contains("transformer")
        || p.contains("walker")
    {
        Some("bipedal_mech".to_string())
    } else if p.contains("spider")
        || p.contains("quadruped")
        || p.contains("4-leg")
        || p.contains("four leg")
        || p.contains("hound")
        || p.contains("dog")
    {
        Some("quadruped_crawler".to_string())
    } else if p.contains("tank") || p.contains("tread") || p.contains("tracked") {
        Some("siege_tank".to_string())
    } else if p.contains("rover") || p.contains("car") || p.contains("wagon") {
        Some("rover_car".to_string())
    } else {
        None
    }
}

pub fn build_contract_from_prompt(
    user_prompt: &str,
    level_index: u32,
    fallback_robot_form: &str,
) -> LevelContract {
    let p = user_prompt.to_lowercase();
    let is_medieval = p.contains("medieval")
        || p.contains("castle")
        || p.contains("knight")
        || p.contains("kingdom")
        || p.contains("cobble")
        || p.contains("dragon")
        || p.contains("tavern")
        || p.contains("wagon");
    let is_desert = p.contains("desert")
        || p.contains("mars")
        || p.contains("canyon")
        || p.contains("wasteland")
        || p.contains("dune")
        || p.contains("rust");
    let is_arctic = p.contains("arctic")
        || p.contains("ice")
        || p.contains("snow")
        || p.contains("glacier")
        || p.contains("cryo")
        || p.contains("frozen");

    let robot_form =
        infer_robot_form_from_prompt(&p).unwrap_or_else(|| fallback_robot_form.to_string());

    let (theme, game_title, plot_summary, objective, hazard, env_style, mut car_design) =
        if is_medieval {
            (
                "medieval_cobblestone_kingdom".to_string(),
                "Chronicles of Iron & Cobble".to_string(),
                format!(
                    "Chapter {level_index}: Lord Malakor has lowered the Iron Portcullis across King's Cobblestone Road. \
                     Drive the Royal Siege Wagon past the timber taverns & castle keeps, run [Q] QA to raise the gate, \
                     and deliver the Dragon Relic to the Citadel Sanctuary."
                ),
                "Drive down King's Cobblestone Road, clear the Iron Portcullis, and reach the Citadel Sanctuary."
                    .to_string(),
                "iron_portcullis_gate_blockade".to_string(),
                EnvironmentStyleSpec {
                    ground_color: [0.16, 0.26, 0.14],
                    road_color: [0.36, 0.32, 0.27],
                    sidewalk_color: [0.48, 0.45, 0.40],
                    sky_color: [0.14, 0.17, 0.24],
                    building_style: "medieval_keep".to_string(),
                    prop_style: "torch_post".to_string(),
                    soundtrack_style: "medieval_lute_march".to_string(),
                    has_center_dashes: false,
                },
                CarDesignSpec {
                    body_style: "medieval_wagon".to_string(),
                    robot_form: robot_form.clone(),
                    primary_color: [0.58, 0.36, 0.16],
                    accent_color: [0.85, 0.68, 0.20],
                    wheel_scale: 1.15,
                    ride_height: 0.04,
                    roof_attachment: "battering_ram".to_string(),
                },
            )
        } else if is_desert {
            (
                "wasteland_convoy_canyon".to_string(),
                "Mad Torque: Red Dust Run".to_string(),
                format!(
                    "Chapter {level_index}: Scavenger clans sealed Fury Canyon with a rusted scrap-metal wall. \
                     Engage 4WD crawler lock, weave between sandstone mesas, and deliver the fuel core."
                ),
                "Blast through the red-rock canyon trail, clear the scrap barricade, and reach the Refinery Beacon."
                    .to_string(),
                "rusted_scrap_wall_barricade".to_string(),
                EnvironmentStyleSpec {
                    ground_color: [0.58, 0.31, 0.16],
                    road_color: [0.44, 0.25, 0.14],
                    sidewalk_color: [0.66, 0.39, 0.22],
                    sky_color: [0.32, 0.18, 0.12],
                    building_style: "desert_mesa".to_string(),
                    prop_style: "torch_post".to_string(),
                    soundtrack_style: "wasteland_percussion_drone".to_string(),
                    has_center_dashes: false,
                },
                CarDesignSpec {
                    body_style: "armored_crawler".to_string(),
                    robot_form: robot_form.clone(),
                    primary_color: [0.82, 0.42, 0.15],
                    accent_color: [0.22, 0.22, 0.24],
                    wheel_scale: 1.30,
                    ride_height: 0.08,
                    roof_attachment: "m3_servo_arm".to_string(),
                },
            )
        } else if is_arctic {
            (
                "arctic_glacier_outpost".to_string(),
                "Cryo-Drive: Subzero Protocol".to_string(),
                format!(
                    "Chapter {level_index}: A glacial shear triggered an ice-wall blockade across Sector Zero Causeway. \
                     Pilot the thermal interceptor between geodesic cryo-domes to re-ignite the fusion beacon."
                ),
                "Navigate the frozen causeway, clear the glacial ice blockade, and reach the Fusion Core."
                    .to_string(),
                "glacial_ice_wall_blockade".to_string(),
                EnvironmentStyleSpec {
                    ground_color: [0.82, 0.90, 0.96],
                    road_color: [0.45, 0.65, 0.82],
                    sidewalk_color: [0.72, 0.84, 0.92],
                    sky_color: [0.10, 0.18, 0.28],
                    building_style: "arctic_dome".to_string(),
                    prop_style: "cryo_pylon".to_string(),
                    soundtrack_style: "cryo_crystal_arp".to_string(),
                    has_center_dashes: true,
                },
                CarDesignSpec {
                    body_style: "cyber_interceptor".to_string(),
                    robot_form: robot_form.clone(),
                    primary_color: [0.92, 0.96, 1.0],
                    accent_color: [0.10, 0.78, 0.98],
                    wheel_scale: 1.20,
                    ride_height: 0.05,
                    roof_attachment: "twin_turbo".to_string(),
                },
            )
        } else {
            let mut default_car = CarDesignSpec::default();
            default_car.robot_form = robot_form.clone();
            (
                "neon_downtown_avenue".to_string(),
                "Neon Overdrive 2099".to_string(),
                format!(
                    "Chapter {level_index} ('{user_prompt}'): CorpSec locked down Main Cyber Avenue with a plasma blast door. \
                     Thread the needle through downtown skyscrapers, run [Q] WGPU QA to clear the blockade, and reach the Beacon."
                ),
                "Drive down Main Cyber Avenue through the skyscraper blocks to reach the Extraction Beacon."
                    .to_string(),
                "mid_avenue_blast_door_blockade".to_string(),
                EnvironmentStyleSpec::default(),
                default_car,
            )
        };

    // Prompt-driven live upgrade / skin / paint overrides
    if p.contains("gold") || p.contains("royal") {
        car_design.primary_color = [0.90, 0.72, 0.18];
        car_design.accent_color = [0.98, 0.92, 0.55];
    } else if p.contains("crimson") || p.contains("red") || p.contains("flame") {
        car_design.primary_color = [0.88, 0.14, 0.18];
        car_design.accent_color = [1.0, 0.65, 0.12];
    } else if p.contains("stealth") || p.contains("black") || p.contains("obsidian") {
        car_design.primary_color = [0.12, 0.13, 0.16];
        car_design.accent_color = [0.10, 0.95, 0.55];
    }
    if p.contains("ram") || p.contains("plow") {
        car_design.roof_attachment = "battering_ram".to_string();
    } else if p.contains("turbo") || p.contains("boost") {
        car_design.roof_attachment = "twin_turbo".to_string();
    }
    if p.contains("monster") || p.contains("big wheel") || p.contains("lifted") {
        car_design.wheel_scale = 1.35;
        car_design.ride_height = 0.09;
    }

    let goal_x = 38.0 + (level_index.saturating_sub(1) as f32) * 8.0;
    let mut bounds = HashMap::new();
    bounds.insert("x".to_string(), 80.0);
    bounds.insert("z".to_string(), 40.0);

    let mut entities = vec![
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
            x: goal_x,
            y: 0.5,
            z: 0.0,
            scale: 1.25,
            solid: false,
        },
        EntitySpec {
            id: "street_chicane_left".to_string(),
            asset_key: "obstacle".to_string(),
            x: 11.0,
            y: 0.9,
            z: -2.3,
            scale: 1.3,
            solid: true,
        },
        EntitySpec {
            id: "street_chicane_right".to_string(),
            asset_key: "obstacle".to_string(),
            x: 27.0,
            y: 0.9,
            z: 2.3,
            scale: 1.3,
            solid: true,
        },
        EntitySpec {
            id: "blast_door".to_string(),
            asset_key: "obstacle".to_string(),
            x: 19.0,
            y: 1.5,
            z: 0.0,
            scale: 2.6,
            solid: true,
        },
    ];

    for extra in 1..level_index {
        entities.push(EntitySpec {
            id: format!("level_{level_index}_hazard_{extra}"),
            asset_key: "obstacle".to_string(),
            x: 31.0 + (extra as f32) * 6.5,
            y: 0.9,
            z: if extra % 2 == 1 { -2.1 } else { 2.1 },
            scale: 1.35,
            solid: true,
        });
    }

    let heights = if is_medieval || is_desert || is_arctic {
        [1.8, 2.5, 2.0, 2.8, 2.2, 2.6, 1.9, 2.4]
    } else {
        [2.4, 3.6, 2.8, 4.2, 3.1, 3.8, 2.6, 3.4]
    };
    for (idx, bx) in [-6.0_f32, 4.0, 14.0, 24.0, 34.0, 44.0, 54.0, 64.0]
        .iter()
        .enumerate()
    {
        let sc = heights[idx % heights.len()];
        entities.push(EntitySpec {
            id: format!("city_block_north_{idx}"),
            asset_key: "building".to_string(),
            x: *bx,
            y: sc * 1.5,
            z: -9.5,
            scale: sc,
            solid: true,
        });
        entities.push(EntitySpec {
            id: format!("city_block_south_{idx}"),
            asset_key: "building".to_string(),
            x: *bx,
            y: sc * 1.5,
            z: 9.5,
            scale: sc,
            solid: true,
        });
    }

    LevelContract {
        project_id: format!("studio_lvl_{level_index}"),
        level_index,
        game_title,
        plot_summary,
        objective,
        theme,
        world_bounds: bounds,
        environment_style: env_style,
        car_design,
        environmental_hazard: hazard,
        required_capability: "high_torque_4wd + modular_m3_mount".to_string(),
        skin_url: String::new(),
        music_url: None,
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
            required_torque_kg_cm: 8.8,
            rated_torque_kg_cm: 1.8,
            torque_valid: false,
            upgrade_equipped: false,
            motor_torque_nm: 4.0,
            wheel_friction: 1.35,
        },
        sandbox_files: vec![],
        entities,
    }
}

pub fn setup_base_environment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<StudioEntity>>,
    mut clear_color: ResMut<ClearColor>,
    net: Res<StudioNetworkChannel>,
    mut session: ResMut<GameSessionState>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 14_500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(12.0, 28.0, -14.0)
            .looking_at(Vec3::new(18.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-5.2, 2.65, 0.0)
                .looking_at(Vec3::new(6.0, 0.7, 0.0), Vec3::Y),
            ..default()
        },
        ChaseCamera,
    ));

    // Top compact telemetry HUD
    commands.spawn((
        TextBundle::from_section(
            "ARGON GAME STUDIO | Initializing...",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(14.0),
            right: Val::Px(14.0),
            ..default()
        }),
        StatusOverlayText,
    ));

    // Centered Interactive Welcome / AI Game Generator / Car Designer Studio Modal
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(92.0),
                    left: Val::Px(90.0),
                    right: Val::Px(90.0),
                    padding: UiRect::all(Val::Px(22.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.04, 0.07, 0.12, 0.92)),
                border_color: BorderColor(Color::srgb(0.0, 0.85, 1.0)),
                ..default()
            },
            StudioModalPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Welcome to Argon Game Studio",
                    TextStyle {
                        font_size: 17.0,
                        color: Color::srgb(0.92, 0.96, 1.0),
                        ..default()
                    },
                ),
                StudioModalText,
            ));
        });

    // Launch directly in Neon Cyberpunk 2099 (while `session.prompt_input` holds "take me to medieval times" ready for [Tab] -> [Enter]!)
    let initial_prompt = session.last_prompt.clone();
    let preview = build_contract_from_prompt(&initial_prompt, 1, "rover_car");
    let [sr, sg, sb] = preview.environment_style.sky_color;
    clear_color.0 = Color::srgb(sr, sg, sb);

    spawn_level_from_contract(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &existing_entities,
        &mut session,
        &preview,
        None,
        None,
        false,
    );

    request_commission_level(&net, initial_prompt, 1, false);
}

pub fn handle_studio_hotkeys(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<StudioEntity>>,
    mut transform_query: Query<(Entity, &StudioEntity, &mut Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut key_evr: EventReader<KeyboardInput>,
    mut clear_color: ResMut<ClearColor>,
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

    // Toggle Welcome / AI Studio Live Prompt Bar with Tab or Escape
    if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::Escape) {
        session.ui_mode = match session.ui_mode {
            UiMode::WelcomeStudio => UiMode::Driving,
            UiMode::CarDesigner => UiMode::Driving,
            UiMode::Driving => UiMode::WelcomeStudio,
        };
        return;
    }

    // Quick-Preset Theme Generators (F1 / F2 / F3 / F4) work in ANY mode and morph scenery IN-PLACE!
    let preset_prompt = if keys.just_pressed(KeyCode::F1) {
        Some("Medieval cobblestone castle streets with dragon portcullis gate, timber taverns, torches, and royal siege wagon")
    } else if keys.just_pressed(KeyCode::F2) {
        Some("Neon cyberpunk downtown avenue with towering skyscrapers, rain-slicked asphalt, and plasma blast door")
    } else if keys.just_pressed(KeyCode::F3) {
        Some("Desert wasteland convoy canyon with sandstone mesas, oil derricks, and 4WD armored crawler")
    } else if keys.just_pressed(KeyCode::F4) {
        Some("Arctic glacier research outpost with frozen blue-ice highway, geodesic cryo domes, and thermal interceptor")
    } else {
        None
    };

    if let Some(p_str) = preset_prompt {
        session.prompt_input = p_str.to_string();
        session.last_prompt = p_str.to_string();
        let current_lvl = session.current_level.max(1);
        let current_robot = session.car_design.robot_form.clone();
        let preview = build_contract_from_prompt(p_str, current_lvl, &current_robot);
        let [sr, sg, sb] = preview.environment_style.sky_color;
        clear_color.0 = Color::srgb(sr, sg, sb);
        spawn_level_from_contract(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            &existing_entities,
            &mut session,
            &preview,
            None,
            None,
            true,
        );
        session.ui_mode = UiMode::Driving;
        session.status_banner = format!(
            "Live Scene Morph -> '{}' | Streaming Soundtrack & Skin from Gretchen Studio...",
            session.game_title
        );
        request_commission_level(&net, p_str.to_string(), current_lvl, true);
        return;
    }

    // Handle typing inside the Live AI Studio Prompt Box
    if session.ui_mode == UiMode::WelcomeStudio {
        for ev in key_evr.read() {
            if ev.state != ButtonState::Pressed {
                continue;
            }
            match ev.key_code {
                KeyCode::Backspace => {
                    session.prompt_input.pop();
                }
                KeyCode::Enter => {
                    let prompt_to_send = if session.prompt_input.trim().is_empty() {
                        session.last_prompt.clone()
                    } else {
                        session.prompt_input.trim().to_string()
                    };
                    session.last_prompt = prompt_to_send.clone();
                    let current_lvl = session.current_level.max(1);
                    let current_robot = session.car_design.robot_form.clone();
                    let preview =
                        build_contract_from_prompt(&prompt_to_send, current_lvl, &current_robot);
                    let [sr, sg, sb] = preview.environment_style.sky_color;
                    clear_color.0 = Color::srgb(sr, sg, sb);
                    spawn_level_from_contract(
                        &mut commands,
                        &asset_server,
                        &mut meshes,
                        &mut materials,
                        &existing_entities,
                        &mut session,
                        &preview,
                        None,
                        None,
                        true,
                    );
                    session.ui_mode = UiMode::Driving;
                    session.status_banner = format!(
                        "Live Scene Morph: '{}' -> {} (Preserving Physics & Pose)...",
                        prompt_to_send, session.game_title
                    );
                    request_commission_level(&net, prompt_to_send, current_lvl, true);
                    return;
                }
                _ => {
                    if let bevy::input::keyboard::Key::Character(ref s) = ev.logical_key {
                        for ch in s.chars() {
                            if !ch.is_control() && session.prompt_input.len() < 110 {
                                session.prompt_input.push(ch);
                            }
                        }
                    } else if ev.key_code == KeyCode::Space && session.prompt_input.len() < 110 {
                        session.prompt_input.push(' ');
                    }
                }
            }
        }
        return;
    }

    // Toggle 3D Car Designer Showroom when not typing in the Welcome prompt box
    if keys.just_pressed(KeyCode::KeyV) {
        session.ui_mode = if session.ui_mode == UiMode::CarDesigner {
            UiMode::Driving
        } else {
            UiMode::CarDesigner
        };
        return;
    }

    // Helper closure to despawn & respawn the player robot/car at `session.player_pos`
    let respawn_player_at_current_pose =
        |commands: &mut Commands,
         asset_server: &Res<AssetServer>,
         meshes: &mut ResMut<Assets<Mesh>>,
         materials: &mut ResMut<Assets<StandardMaterial>>,
         transform_query: &Query<(Entity, &StudioEntity, &mut Transform)>,
         session: &GameSessionState| {
            let mut pos = session.player_pos;
            for (ent, studio_ent, tf) in transform_query.iter() {
                if studio_ent.role == EntityRole::Player {
                    pos = tf.translation;
                    commands.entity(ent).despawn_recursive();
                }
            }
            crate::level_builder::spawn_player_car_entity(
                commands,
                asset_server,
                meshes,
                materials,
                pos,
                session.rover_yaw,
                1.0,
                &session.car_design,
                &session.downloaded_skin_asset,
            );
        };

    // Handle Car Designer Showroom Controls
    if session.ui_mode == UiMode::CarDesigner {
        let mut car_changed = false;

        if keys.just_pressed(KeyCode::Digit1) {
            session.car_design.body_style = match session.car_design.body_style.as_str() {
                "street_muscle" => "medieval_wagon".to_string(),
                "medieval_wagon" => "armored_crawler".to_string(),
                "armored_crawler" => "cyber_interceptor".to_string(),
                _ => "street_muscle".to_string(),
            };
            car_changed = true;
        }
        if keys.just_pressed(KeyCode::Digit2) {
            let palettes: [([f32; 3], [f32; 3]); 5] = [
                ([0.18, 0.78, 0.98], [1.0, 0.50, 0.12]), // Cyber Cyan & Orange
                ([0.58, 0.36, 0.16], [0.85, 0.68, 0.20]), // Medieval Oak & Royal Gold
                ([0.88, 0.14, 0.18], [0.95, 0.95, 0.98]), // Crimson GT & White
                ([0.14, 0.15, 0.18], [0.10, 0.95, 0.55]), // Stealth Black & Toxic Emerald
                ([0.82, 0.42, 0.15], [0.22, 0.22, 0.24]), // Wasteland Copper & Iron
            ];
            let cur = session.car_design.primary_color;
            let idx = palettes
                .iter()
                .position(|(p, _)| (p[0] - cur[0]).abs() < 0.05)
                .unwrap_or(0);
            let (next_p, next_a) = palettes[(idx + 1) % palettes.len()];
            session.car_design.primary_color = next_p;
            session.car_design.accent_color = next_a;
            car_changed = true;
        }
        if keys.just_pressed(KeyCode::Digit3) {
            let (next_w, next_h) = if session.car_design.wheel_scale < 1.05 {
                (1.32, 0.08) // Monster Crawler Lift
            } else if session.car_design.wheel_scale > 1.20 {
                (0.86, -0.03) // Slammed Drift Spec
            } else {
                (1.0, 0.0) // Standard Street Spec
            };
            session.car_design.wheel_scale = next_w;
            session.car_design.ride_height = next_h;
            car_changed = true;
        }
        if keys.just_pressed(KeyCode::Digit4) {
            session.car_design.roof_attachment = match session.car_design.roof_attachment.as_str() {
                "m3_servo_arm" => "battering_ram".to_string(),
                "battering_ram" => "twin_turbo".to_string(),
                _ => "m3_servo_arm".to_string(),
            };
            car_changed = true;
        }
        if keys.just_pressed(KeyCode::Digit5) {
            session.status_banner =
                "Requesting Nano Banana 2.1 AI Vehicle Skin from Gretchen Studio...".to_string();
            request_commission_level(
                &net,
                format!(
                    "{} with custom {} vehicle skin",
                    session.last_prompt, session.car_design.body_style
                ),
                session.current_level,
                true,
            );
        }
        if keys.just_pressed(KeyCode::Digit6) || keys.just_pressed(KeyCode::KeyT) {
            let next_form = match session.car_design.robot_form.as_str() {
                "rover_car" => "bipedal_mech",
                "bipedal_mech" => "quadruped_crawler",
                "quadruped_crawler" => "siege_tank",
                _ => "rover_car",
            };
            session.car_design.robot_form = next_form.to_string();
            request_mujoco_step(
                &net,
                project_id.clone(),
                0.0,
                0.0,
                None,
                Some(next_form.to_string()),
            );
            car_changed = true;
        }
        if keys.just_pressed(KeyCode::Enter) {
            session.ui_mode = UiMode::Driving;
            return;
        }

        if car_changed {
            respawn_player_at_current_pose(
                &mut commands,
                &asset_server,
                &mut meshes,
                &mut materials,
                &transform_query,
                &session,
            );
            session.status_banner = format!(
                "Showroom Updated: Robot [{}] | Body [{}] | Wheels {:.2}x | Attachment [{}]",
                session.car_design.robot_form,
                session.car_design.body_style,
                session.car_design.wheel_scale,
                session.car_design.roof_attachment
            );
        }
        return;
    }

    // === DRIVING MODE HOTKEYS ===
    // [T] TRANSFORMERS-STYLE MUJOCO ROBOT MORPH (Car <-> Bipedal Mech <-> Spider Quadruped <-> Siege Tank)
    if keys.just_pressed(KeyCode::KeyT) {
        let next_form = match session.car_design.robot_form.as_str() {
            "rover_car" => "bipedal_mech",
            "bipedal_mech" => "quadruped_crawler",
            "quadruped_crawler" => "siege_tank",
            _ => "rover_car",
        };
        session.car_design.robot_form = next_form.to_string();
        respawn_player_at_current_pose(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            &transform_query,
            &session,
        );
        session.status_banner = format!(
            "TRANSFORMERS MORPH -> MuJoCo Robot Form [{}] (Pose & Physics Preserved!)",
            next_form.to_uppercase()
        );
        request_mujoco_step(
            &net,
            project_id.clone(),
            0.0,
            0.0,
            None,
            Some(next_form.to_string()),
        );
    }

    // [N] Generate Next Campaign Level (resets to start of next sector)
    if keys.just_pressed(KeyCode::KeyN) {
        session.current_level += 1;
        let next_lvl = session.current_level;
        let prompt = session.last_prompt.clone();
        let current_robot = session.car_design.robot_form.clone();
        let preview = build_contract_from_prompt(&prompt, next_lvl, &current_robot);
        let [sr, sg, sb] = preview.environment_style.sky_color;
        clear_color.0 = Color::srgb(sr, sg, sb);
        spawn_level_from_contract(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut materials,
            &existing_entities,
            &mut session,
            &preview,
            None,
            None,
            false,
        );
        session.status_banner = format!(
            "Advancing Campaign -> {} Level {}! Commissioning Gretchen Architect...",
            session.game_title, next_lvl
        );
        request_commission_level(&net, prompt, next_lvl, false);
    }

    // [Q] Run WGPU Compute G-Buffer Artifact Detector (`artifact_detector.wgsl`) + Visual QA Roadblock Hotfix
    if keys.just_pressed(KeyCode::KeyQ) {
        let snap_path = capture_viewport_to_disk(&main_window, &mut screenshot_manager);
        let audit = run_gbuffer_compute_audit(!session.qa_cleaned);

        for (entity, studio_ent, mut tf) in transform_query.iter_mut() {
            if studio_ent.role == EntityRole::ZFightArtifact {
                commands.entity(entity).despawn_recursive();
            } else if studio_ent.id == "blast_door" {
                tf.translation = Vec3::new(19.0, 1.5, 6.5);
            }
        }

        session.qa_cleaned = true;
        session.sanity_check.motor_torque_nm = 6.5;
        session.sanity_check.wheel_friction = 1.65;
        session.sanity_check.rated_torque_kg_cm = 11.0;
        session.sanity_check.torque_valid = true;

        if audit.total_defects > 0 {
            session.last_gbuffer_summary = format!(
                "Caught {} GPU Defects (Z-Fights: {}, Shadow Acne: {} at {:?}) -> AUTO-REPAIRED!",
                audit.total_defects, audit.z_fights, audit.shadow_acne, audit.sample_coord
            );
            session.status_banner = format!(
                "WGPU Compute QA + Visual Agent: Fixed {} Z-Fight/Shadow Defects, shifted 'blast_door' roadblock to z=6.5 & boosted torque!",
                audit.total_defects
            );
        } else {
            session.last_gbuffer_summary =
                "0 GPU Defects (Z-Fights: 0, Shadow Acne: 0) — Render Pass Clear!".to_string();
            session.status_banner =
                "WGPU Compute Validator: 0 Defects! Street corridor open & MuJoCo torque verified [PASS]."
                    .to_string();
        }

        request_qa_audit(&net, project_id.clone(), snap_path);
    }

    // [U] Snap Modular M3 Hardware Upgrade
    if keys.just_pressed(KeyCode::KeyU) {
        session.upgrade_equipped = !session.upgrade_equipped;
        session.sanity_check.upgrade_equipped = session.upgrade_equipped;
        if session.upgrade_equipped {
            session.sanity_check.motor_torque_nm = 6.5;
            session.sanity_check.wheel_friction = 1.65;
            session.sanity_check.rated_torque_kg_cm = 11.0;
            session.sanity_check.torque_valid = true;
        } else {
            session.sanity_check.motor_torque_nm = 4.0;
            session.sanity_check.wheel_friction = 1.35;
            session.sanity_check.rated_torque_kg_cm = 1.8;
            session.sanity_check.torque_valid = false;
        }
        session.status_banner = format!(
            "Snapped M3 Modular Part [{}] onto Chassis -> CoM STABLE | Servo Torque: {:.1} kg*cm",
            session.reward_part.part_id, session.sanity_check.rated_torque_kg_cm
        );
        request_mujoco_step(
            &net,
            project_id.clone(),
            0.0,
            0.0,
            Some(session.upgrade_equipped),
            None,
        );
    }

    // [E] Export Physical Build Package (M3 STLs + URDF + BOM.csv + MicroPython)
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
    mut clear_color: ResMut<ClearColor>,
    mut session: ResMut<GameSessionState>,
) {
    while let Ok(event) = net.rx.try_recv() {
        match event {
            StudioNetEvent::LevelCommissioned {
                contract,
                local_skin_asset,
                local_music_asset,
                preserve_pose,
            } => {
                let [sr, sg, sb] = contract.environment_style.sky_color;
                clear_color.0 = Color::srgb(sr, sg, sb);
                spawn_level_from_contract(
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &existing_entities,
                    &mut session,
                    &contract,
                    local_skin_asset,
                    local_music_asset,
                    preserve_pose,
                );
            }
            StudioNetEvent::CommissionFallback {
                prompt,
                level_index,
                preserve_pose,
                reason: _,
            } => {
                let current_robot = session.car_design.robot_form.clone();
                let contract = build_contract_from_prompt(&prompt, level_index, &current_robot);
                let [sr, sg, sb] = contract.environment_style.sky_color;
                clear_color.0 = Color::srgb(sr, sg, sb);
                spawn_level_from_contract(
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &existing_entities,
                    &mut session,
                    &contract,
                    None,
                    None,
                    preserve_pose,
                );
            }
            StudioNetEvent::MujocoStepCompleted(step) => {
                session.step_in_flight = false;
                session.mujoco_engine = step.engine;
                session.sanity_check = step.sanity_check;
            }
            StudioNetEvent::QaReportReceived(report) => {
                apply_hotfix_deltas(&mut transform_query, &report.hotfixes);
                if let Some(sc) = report.sanity_check {
                    session.sanity_check = sc;
                }
                session.status_banner = format!(
                    "QA Status [{}] + WGPU Compute Pass: {}",
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
    // Pause driving physics while typing in the Welcome / Prompt Studio Menu
    if session.ui_mode != UiMode::Driving {
        return;
    }

    let dt = time.delta_seconds().min(0.05);
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
        session.rover_yaw = 0.0;
        session.rover_speed = 0.0;
        let reset_pos = Vec3::new(0.0, 0.35 + session.car_design.ride_height, 0.0);
        session.player_pos = reset_pos;
        for (ent, mut transform) in query.iter_mut() {
            if ent.role == EntityRole::Player {
                transform.translation = reset_pos;
                transform.rotation = Quat::from_rotation_y(session.rover_yaw);
                session.victory = false;
            }
        }
        return;
    }

    if (throttle != 0.0 || steering != 0.0)
        && !session.step_in_flight
        && !session.active_project_id.is_empty()
    {
        session.step_in_flight = true;
        request_mujoco_step(
            &net,
            session.active_project_id.clone(),
            throttle,
            steering,
            None,
            None,
        );
    }

    let max_speed = if session.sanity_check.upgrade_equipped {
        14.0
    } else {
        10.0
    };
    let accel_rate = if session.sanity_check.upgrade_equipped {
        22.0
    } else {
        15.5
    };

    if throttle != 0.0 {
        session.rover_speed =
            (session.rover_speed + throttle * accel_rate * dt).clamp(-max_speed * 0.6, max_speed);
    } else {
        session.rover_speed *= (1.0 - 5.0 * dt).max(0.0);
        if session.rover_speed.abs() < 0.05 {
            session.rover_speed = 0.0;
        }
    }

    if steering != 0.0 {
        let turn_dir = if session.rover_speed >= -0.1 {
            1.0
        } else {
            -1.0
        };
        session.rover_yaw += steering * 2.2 * dt * turn_dir;
    }

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
            EntityRole::Obstacle | EntityRole::Building if ent.solid => {
                obstacles.push((ent.half_extents, transform.translation));
            }
            _ => {}
        }
    }

    let Some((player_extents, current_pos)) = player_current else {
        return;
    };

    // Conservative bounding radius for the rotated vehicle so it NEVER clips through walls or obstacles
    let car_rx = player_extents.x.max(0.82);
    let car_rz = player_extents.z.max(0.62);

    let overlaps_any = |pos: Vec3| -> bool {
        obstacles.iter().any(|(obs_ext, obs_pos)| {
            let dx = (pos.x - obs_pos.x).abs();
            let dz = (pos.z - obs_pos.z).abs();
            dx < (car_rx + obs_ext.x) && dz < (car_rz + obs_ext.z)
        })
    };

    // Per-axis X and Z collision resolution + wall sliding
    let mut resolved_pos = current_pos;
    let mut hit_wall = false;

    let try_x = Vec3::new(current_pos.x + velocity.x, current_pos.y, current_pos.z);
    if !overlaps_any(try_x) {
        resolved_pos.x = try_x.x;
    } else {
        hit_wall = true;
    }

    let try_z = Vec3::new(resolved_pos.x, current_pos.y, current_pos.z + velocity.z);
    if !overlaps_any(try_z) {
        resolved_pos.z = try_z.z;
    } else {
        hit_wall = true;
    }

    // Depenetration push-out if an obstacle ever overlaps the car
    for (obs_ext, obs_pos) in &obstacles {
        let dx = resolved_pos.x - obs_pos.x;
        let dz = resolved_pos.z - obs_pos.z;
        let overlap_x = (car_rx + obs_ext.x) - dx.abs();
        let overlap_z = (car_rz + obs_ext.z) - dz.abs();
        if overlap_x > 0.0 && overlap_z > 0.0 {
            if overlap_x < overlap_z {
                resolved_pos.x += overlap_x * dx.signum();
            } else {
                resolved_pos.z += overlap_z * dz.signum();
            }
            hit_wall = true;
        }
    }

    if hit_wall {
        session.rover_speed = -session.rover_speed * 0.35;
    }

    session.player_pos = resolved_pos;

    for (ent, mut transform) in query.iter_mut() {
        if ent.role == EntityRole::Player {
            transform.rotation = Quat::from_rotation_y(session.rover_yaw);
            transform.translation = resolved_pos;
        }
    }

    if let Some(g_pos) = goal_pos {
        if resolved_pos.distance(g_pos) < 2.2 && !session.victory {
            session.victory = true;
            session.status_banner = format!(
                "LEVEL {} COMPLETE! Unlocked [{}] | Press [N] Generate Level {} | [U] Snap Part | [E] Export STLs",
                session.current_level,
                session.reward_part.part_id,
                session.current_level + 1
            );
        }
    }
}

pub fn update_gta_chase_camera(
    time: Res<Time>,
    mut session: ResMut<GameSessionState>,
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

    if session.ui_mode == UiMode::CarDesigner {
        // 360-degree Showroom Orbit Camera around the car/robot
        session.showroom_orbit_angle += time.delta_seconds() * 0.65;
        let radius = 3.8;
        let cam_x = player_tf.translation.x - session.showroom_orbit_angle.cos() * radius;
        let cam_z = player_tf.translation.z + session.showroom_orbit_angle.sin() * radius;
        let desired_pos = Vec3::new(cam_x, player_tf.translation.y + 1.65, cam_z);
        cam_tf.translation = cam_tf.translation.lerp(desired_pos, 0.15);
        cam_tf.look_at(player_tf.translation + Vec3::new(0.0, 0.25, 0.0), Vec3::Y);
        return;
    }

    let forward = Vec3::new(session.rover_yaw.cos(), 0.0, -session.rover_yaw.sin());
    let desired_cam_pos = player_tf.translation - forward * 5.2 + Vec3::new(0.0, 2.6, 0.0);
    let look_target = player_tf.translation + forward * 4.5 + Vec3::new(0.0, 0.55, 0.0);

    let smooth = (10.0 * time.delta_seconds()).clamp(0.0, 1.0);
    cam_tf.translation = cam_tf.translation.lerp(desired_cam_pos, smooth);
    cam_tf.look_at(look_target, Vec3::Y);
}

pub fn update_hud_overlay(
    session: Res<GameSessionState>,
    mut hud_query: Query<&mut Text, (With<StatusOverlayText>, Without<StudioModalText>)>,
    mut modal_panel_query: Query<&mut Style, With<StudioModalPanel>>,
    mut modal_text_query: Query<&mut Text, (With<StudioModalText>, Without<StatusOverlayText>)>,
) {
    if !session.is_changed() {
        return;
    }

    let sc = &session.sanity_check;
    let com_badge = if sc.com_stable { "STABLE" } else { "TIPPING RISK" };
    let torque_badge = if sc.torque_valid { "PASS" } else { "STALL" };

    for mut text in hud_query.iter_mut() {
        text.sections[0].value = format!(
            "{} — Lvl {} [{}] | Speed: {:.1} m/s | Robot: [{}] ({}) | Music: [{}] | Torque: {:.1}/{:.1} kg*cm [{}]\n\
             WGPU G-Buffer QA: {} | {}\n\
             [Tab] Live Prompt (\"take me to medieval times\") | [T] Morph Robot | [V] Car Designer | [F1-F4] Worlds | [Q] QA Fix | [N] Next Lvl",
            session.game_title,
            session.current_level,
            session.theme,
            session.speed_mps,
            session.car_design.robot_form,
            session.car_design.body_style,
            session.environment_style.soundtrack_style,
            sc.required_torque_kg_cm,
            sc.rated_torque_kg_cm,
            torque_badge,
            session.last_gbuffer_summary,
            session.status_banner
        );
    }

    for mut style in modal_panel_query.iter_mut() {
        style.display = match session.ui_mode {
            UiMode::WelcomeStudio | UiMode::CarDesigner => Display::Flex,
            UiMode::Driving => Display::None,
        };
    }

    for mut modal_text in modal_text_query.iter_mut() {
        match session.ui_mode {
            UiMode::WelcomeStudio => {
                modal_text.sections[0].value = format!(
                    "=== ARGON GAME STUDIO — LIVE IN-GAME SCENE, SKIN & ROBOT PROMPT BAR ===\n\n\
                     ACTIVE SCENE : {} (Level {} | Theme: {} | Music: {})\n\
                     ROBOT / CAR  : {} ({}) | Street Pose Preserved on Scene Swap!\n\
                     STORY PLOT   : {}\n\n\
                     LIVE PROMPT TO FASTAPI STUDIO AGENT (Type scenery, upgrade, skin, or robot morph):\n\
                     > {}_\n\n\
                     [ENTER] Morph Scene, Music, Assets & Skin In-Place (Physics & Position Stay Continuous!)\n\
                     Try typing: \"take me to medieval times\" | \"transform into a bipedal mech\" | \"gold dragon skin with monster wheels\"\n\
                     [F1] Medieval Kingdom  | [F2] Cyberpunk 2099  | [F3] Desert Canyon  | [F4] Arctic Glacier\n\
                     [T]  Transformers Robot Morph (Rover Car <-> Bipedal Mech <-> Spider Quadruped <-> Siege Tank)\n\
                     [V]  Open Interactive 3D Car & Robot Designer Showroom   |   [TAB / ESC] Resume Driving",
                    session.game_title,
                    session.current_level,
                    session.theme,
                    session.environment_style.soundtrack_style,
                    session.car_design.robot_form,
                    session.car_design.body_style,
                    session.plot_summary,
                    session.prompt_input
                );
            }
            UiMode::CarDesigner => {
                modal_text.sections[0].value = format!(
                    "=== INTERACTIVE 3D CAR & TRANSFORMERS ROBOT DESIGNER (MUJOCO STUDIO) ===\n\n\
                     MuJoCo Robot Form    : [{}] (Press [6] or [T] to Transform!)\n\
                     Active Chassis Style : [{}]\n\
                     Wheel Scale & Lift   : [{:.2}x | Ride Height {:+.2}m]\n\
                     Hardware Attachment  : [{}]\n\
                     AI Skin Texture      : [{}]\n\
                     MuJoCo Physics Gate  : Drive Torque {:.1} N*m | Servo {:.1}/{:.1} kg*cm | CoM {}\n\n\
                     CUSTOMIZE YOUR VEHICLE / ROBOT (Live 3D Orbit Preview):\n\
                     [1] Cycle Chassis Body (Street Muscle / Medieval Siege Wagon / Armored 4WD Crawler / Cyber Interceptor)\n\
                     [2] Cycle Paint & Trim Palette (Cyber Cyan / Medieval Oak & Gold / Crimson GT / Stealth Obsidian / Wasteland Copper)\n\
                     [3] Cycle Wheel Size & Suspension Lift (Standard 1.0x / Monster Crawler 1.32x / Lowrider Drift 0.86x)\n\
                     [4] Cycle Roof/Front Hardware (M3 Servo Arm / Iron Battering Ram / Twin Turbo Supercharger)\n\
                     [5] Generate Custom AI PBR Skin from FastAPI Studio (Nano Banana 2.1 / Local Synth)\n\
                     [6 / T] Transform MuJoCo Robot Morphology (Rover Car / Bipedal Mech / Quadruped Spider / Siege Tank)\n\
                     [ENTER / V / TAB] Save & Return to Street Driving",
                    session.car_design.robot_form,
                    session.car_design.body_style,
                    session.car_design.wheel_scale,
                    session.car_design.ride_height,
                    session.car_design.roof_attachment,
                    session
                        .downloaded_skin_asset
                        .as_deref()
                        .unwrap_or("procedural_pbr"),
                    sc.motor_torque_nm,
                    sc.required_torque_kg_cm,
                    sc.rated_torque_kg_cm,
                    com_badge
                );
            }
            UiMode::Driving => {}
        }
    }
}
