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
    #[serde(default = "default_solid")]
    pub solid: bool,
}

fn default_scale() -> f32 {
    1.0
}

fn default_solid() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStyleSpec {
    #[serde(default = "default_ground_color")]
    pub ground_color: [f32; 3],
    #[serde(default = "default_road_color")]
    pub road_color: [f32; 3],
    #[serde(default = "default_sidewalk_color")]
    pub sidewalk_color: [f32; 3],
    #[serde(default = "default_sky_color")]
    pub sky_color: [f32; 3],
    #[serde(default = "default_building_style")]
    pub building_style: String,
    #[serde(default = "default_prop_style")]
    pub prop_style: String,
    #[serde(default = "default_has_dashes")]
    pub has_center_dashes: bool,
}

fn default_ground_color() -> [f32; 3] {
    [0.07, 0.08, 0.11]
}
fn default_road_color() -> [f32; 3] {
    [0.14, 0.16, 0.20]
}
fn default_sidewalk_color() -> [f32; 3] {
    [0.26, 0.29, 0.34]
}
fn default_sky_color() -> [f32; 3] {
    [0.05, 0.07, 0.12]
}
fn default_building_style() -> String {
    "cyber_skyscraper".to_string()
}
fn default_prop_style() -> String {
    "streetlamp".to_string()
}
fn default_has_dashes() -> bool {
    true
}

impl Default for EnvironmentStyleSpec {
    fn default() -> Self {
        Self {
            ground_color: default_ground_color(),
            road_color: default_road_color(),
            sidewalk_color: default_sidewalk_color(),
            sky_color: default_sky_color(),
            building_style: default_building_style(),
            prop_style: default_prop_style(),
            has_center_dashes: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarDesignSpec {
    #[serde(default = "default_body_style")]
    pub body_style: String,
    #[serde(default = "default_primary_color")]
    pub primary_color: [f32; 3],
    #[serde(default = "default_accent_color")]
    pub accent_color: [f32; 3],
    #[serde(default = "default_scale")]
    pub wheel_scale: f32,
    #[serde(default)]
    pub ride_height: f32,
    #[serde(default = "default_roof_attachment")]
    pub roof_attachment: String,
}

fn default_body_style() -> String {
    "street_muscle".to_string()
}
fn default_primary_color() -> [f32; 3] {
    [0.18, 0.78, 0.98]
}
fn default_accent_color() -> [f32; 3] {
    [1.0, 0.50, 0.12]
}
fn default_roof_attachment() -> String {
    "m3_servo_arm".to_string()
}

impl Default for CarDesignSpec {
    fn default() -> Self {
        Self {
            body_style: default_body_style(),
            primary_color: default_primary_color(),
            accent_color: default_accent_color(),
            wheel_scale: 1.0,
            ride_height: 0.0,
            roof_attachment: default_roof_attachment(),
        }
    }
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
    #[serde(default = "default_level_index")]
    pub level_index: u32,
    #[serde(default = "default_game_title")]
    pub game_title: String,
    #[serde(default = "default_plot_summary")]
    pub plot_summary: String,
    #[serde(default)]
    pub objective: String,
    pub theme: String,
    #[serde(default)]
    pub world_bounds: HashMap<String, f32>,
    #[serde(default)]
    pub environment_style: EnvironmentStyleSpec,
    #[serde(default)]
    pub car_design: CarDesignSpec,
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

fn default_level_index() -> u32 {
    1
}
fn default_game_title() -> String {
    "Neon Overdrive 2099".to_string()
}
fn default_plot_summary() -> String {
    "Breach the street blockade and deliver the core to the Extraction Beacon.".to_string()
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
