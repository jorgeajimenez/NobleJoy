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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RewardPartSpec {
    #[serde(default)]
    pub part_id: String,
    #[serde(default)]
    pub mount_type: String,
    #[serde(default)]
    pub compatible_motor: String,
    #[serde(default)]
    pub mass_grams: f32,
    #[serde(default)]
    pub torque_capacity_kg_cm: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhysicsSanityCheck {
    #[serde(default)]
    pub com_stable: bool,
    #[serde(default)]
    pub required_torque_kg_cm: f32,
    #[serde(default)]
    pub rated_torque_kg_cm: f32,
    #[serde(default)]
    pub torque_valid: bool,
    #[serde(default)]
    pub upgrade_equipped: bool,
    #[serde(default)]
    pub motor_torque_nm: f32,
    #[serde(default)]
    pub wheel_friction: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelContract {
    pub project_id: String,
    pub theme: String,
    pub world_bounds: HashMap<String, f32>,
    #[serde(default)]
    pub environmental_hazard: String,
    #[serde(default)]
    pub required_capability: String,
    #[serde(default)]
    pub skin_url: String,
    #[serde(default)]
    pub mjcf_url: String,
    #[serde(default)]
    pub urdf_url: String,
    #[serde(default)]
    pub stl_url: String,
    #[serde(default)]
    pub reward_part: RewardPartSpec,
    #[serde(default)]
    pub sanity_check: PhysicsSanityCheck,
    #[serde(default)]
    pub sandbox_files: Vec<String>,
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
    #[serde(default)]
    pub sanity_check: Option<PhysicsSanityCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChassisPose {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub quat_wxyz: [f32; 4],
    pub speed_mps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MujocoStepResponse {
    pub engine: String,
    pub chassis: ChassisPose,
    pub sanity_check: PhysicsSanityCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportBuildResponse {
    pub project_id: String,
    pub export_dir: String,
    pub stl_files: Vec<String>,
    pub urdf_file: String,
    pub bom_file: String,
    pub firmware_file: String,
}
