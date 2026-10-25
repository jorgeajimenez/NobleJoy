use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpec {
    pub id: String,
    pub asset_key: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default)]
    pub solid: bool,
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelContract {
    pub project_id: String,
    pub theme: String,
    pub world_bounds: HashMap<String, f32>,
    pub entities: Vec<EntitySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotfixDelta {
    pub id: String,
    pub action: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaAuditReport {
    pub project_id: String,
    pub qa_status: String,
    pub critique: String,
    pub hotfixes: Vec<HotfixDelta>,
}
