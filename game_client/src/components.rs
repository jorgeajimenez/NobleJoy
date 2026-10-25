use crate::scene_parser::{PhysicsSanityCheck, RewardPartSpec};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum EntityRole {
    Player,
    Goal,
    Obstacle,
    Building,
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

#[derive(Resource)]
pub struct GameSessionState {
    pub active_project_id: String,
    pub theme: String,
    pub environmental_hazard: String,
    pub reward_part: RewardPartSpec,
    pub sanity_check: PhysicsSanityCheck,
    pub speed_mps: f32,
    pub rover_yaw: f32,
    pub rover_speed: f32,
    pub mujoco_engine: String,
    pub downloaded_skin_asset: Option<String>,
    pub status_banner: String,
    pub victory: bool,
    pub upgrade_equipped: bool,
    pub step_in_flight: bool,
}

impl Default for GameSessionState {
    fn default() -> Self {
        Self {
            active_project_id: String::new(),
            theme: String::new(),
            environmental_hazard: String::new(),
            reward_part: RewardPartSpec::default(),
            sanity_check: PhysicsSanityCheck::default(),
            speed_mps: 0.0,
            // Point straight down Main City Avenue (+X axis, yaw = 0.0)
            rover_yaw: 0.0,
            rover_speed: 0.0,
            mujoco_engine: String::new(),
            downloaded_skin_asset: None,
            status_banner: String::new(),
            victory: false,
            upgrade_equipped: false,
            step_in_flight: false,
        }
    }
}
