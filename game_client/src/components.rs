use crate::scene_parser::{
    CarDesignSpec, EnvironmentStyleSpec, LevelContract, PhysicsSanityCheck, RewardPartSpec,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum EntityRole {
    Player,
    Goal,
    Obstacle,
    Building,
    EnvironmentProp,
    ZFightArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    WelcomeStudio,
    CarDesigner,
    Driving,
}

#[derive(Component, Debug, Clone)]
pub struct StudioEntity {
    pub id: String,
    pub role: EntityRole,
    pub solid: bool,
    pub half_extents: Vec3,
}

#[derive(Component)]
pub struct ChaseCamera;

#[derive(Component)]
pub struct StatusOverlayText;

#[derive(Component)]
pub struct StudioModalPanel;

#[derive(Component)]
pub struct StudioModalText;

#[derive(Resource)]
pub struct GameSessionState {
    pub ui_mode: UiMode,
    pub prompt_input: String,
    pub last_prompt: String,
    pub current_level: u32,
    pub game_title: String,
    pub plot_summary: String,
    pub objective: String,
    pub active_project_id: String,
    pub theme: String,
    pub environment_style: EnvironmentStyleSpec,
    pub car_design: CarDesignSpec,
    pub current_contract: Option<LevelContract>,
    pub environmental_hazard: String,
    pub reward_part: RewardPartSpec,
    pub sanity_check: PhysicsSanityCheck,
    pub speed_mps: f32,
    pub rover_yaw: f32,
    pub rover_speed: f32,
    pub showroom_orbit_angle: f32,
    pub mujoco_engine: String,
    pub downloaded_skin_asset: Option<String>,
    pub status_banner: String,
    pub last_gbuffer_summary: String,
    pub qa_cleaned: bool,
    pub victory: bool,
    pub upgrade_equipped: bool,
    pub step_in_flight: bool,
}

impl Default for GameSessionState {
    fn default() -> Self {
        Self {
            ui_mode: UiMode::WelcomeStudio,
            prompt_input: "Medieval cobblestone castle streets with dragon gate and royal war wagon"
                .to_string(),
            last_prompt: "Medieval cobblestone castle streets with dragon gate and royal war wagon"
                .to_string(),
            current_level: 1,
            game_title: "Argon Game Studio".to_string(),
            plot_summary: String::new(),
            objective: String::new(),
            active_project_id: String::new(),
            theme: String::new(),
            environment_style: EnvironmentStyleSpec::default(),
            car_design: CarDesignSpec::default(),
            current_contract: None,
            environmental_hazard: String::new(),
            reward_part: RewardPartSpec::default(),
            sanity_check: PhysicsSanityCheck::default(),
            speed_mps: 0.0,
            rover_yaw: 0.0,
            rover_speed: 0.0,
            showroom_orbit_angle: 0.5,
            mujoco_engine: String::new(),
            downloaded_skin_asset: None,
            status_banner: String::new(),
            last_gbuffer_summary: "Unverified (Press [Q] to run WGPU Compute G-Buffer Validator + Visual QA)"
                .to_string(),
            qa_cleaned: false,
            victory: false,
            upgrade_equipped: false,
            step_in_flight: false,
        }
    }
}
