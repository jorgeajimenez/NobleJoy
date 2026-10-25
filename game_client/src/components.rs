use bevy::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum EntityRole {
    Player,
    Goal,
    Obstacle,
}

#[derive(Component, Debug, Clone)]
pub struct StudioEntity {
    pub id: String,
    pub role: EntityRole,
    pub solid: bool,
    pub half_extents: Vec3,
}

#[derive(Component)]
pub struct StatusOverlayText;

#[derive(Resource, Default)]
pub struct GameSessionState {
    pub active_project_id: String,
    pub theme: String,
    pub status_banner: String,
    pub victory: bool,
}
