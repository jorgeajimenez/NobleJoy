use crate::scene_parser::{
    ExportBuildResponse, LevelContract, MujocoStepResponse, QaAuditReport,
};
use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::thread;

#[derive(Debug, Clone)]
pub enum StudioNetEvent {
    LevelCommissioned {
        contract: LevelContract,
        local_skin_asset: Option<String>,
    },
    CommissionFallback {
        prompt: String,
        level_index: u32,
        reason: String,
    },
    MujocoStepCompleted(MujocoStepResponse),
    QaReportReceived(QaAuditReport),
    BuildExported(ExportBuildResponse),
    NetworkError(String),
}

#[derive(Resource, Clone)]
pub struct StudioNetworkChannel {
    pub tx: Sender<StudioNetEvent>,
    pub rx: Receiver<StudioNetEvent>,
    pub backend_url: String,
}

impl Default for StudioNetworkChannel {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let backend_url = std::env::var("STUDIO_BACKEND_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        Self {
            tx,
            rx,
            backend_url,
        }
    }
}

pub fn request_commission_level(net: &StudioNetworkChannel, prompt: String, level_index: u32) {
    let tx = net.tx.clone();
    let base_url = net.backend_url.clone();
    let url = format!("{base_url}/api/commission-level");

    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let payload = json!({
            "prompt": prompt,
            "level_index": level_index,
            "seed_blockade": true
        });
        match client.post(&url).json(&payload).send() {
            Ok(resp) => match resp.json::<LevelContract>() {
                Ok(contract) => {
                    let mut local_skin_asset = None;
                    if !contract.skin_url.is_empty() {
                        let skin_full_url = format!("{base_url}{}", contract.skin_url);
                        if let Ok(img_resp) = client.get(&skin_full_url).send() {
                            if let Ok(bytes) = img_resp.bytes() {
                                let asset_rel =
                                    format!("generated/{}_skin.png", contract.project_id);
                                let disk_path = Path::new("assets").join(&asset_rel);
                                if let Some(parent) = disk_path.parent() {
                                    let _ = fs::create_dir_all(parent);
                                }
                                if fs::write(&disk_path, &bytes).is_ok() {
                                    local_skin_asset = Some(asset_rel);
                                }
                            }
                        }
                    }
                    let _ = tx.send(StudioNetEvent::LevelCommissioned {
                        contract,
                        local_skin_asset,
                    });
                }
                Err(err) => {
                    let _ = tx.send(StudioNetEvent::CommissionFallback {
                        prompt,
                        level_index,
                        reason: format!("Parse fallback ({err})"),
                    });
                }
            },
            Err(err) => {
                let _ = tx.send(StudioNetEvent::CommissionFallback {
                    prompt,
                    level_index,
                    reason: format!("Offline Studio Engine ({err})"),
                });
            }
        }
    });
}

pub fn request_mujoco_step(
    net: &StudioNetworkChannel,
    project_id: String,
    throttle: f32,
    steering: f32,
    equip_upgrade: Option<bool>,
) {
    let tx = net.tx.clone();
    let url = format!("{}/api/physics-step", net.backend_url);

    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let payload = json!({
            "project_id": project_id,
            "throttle": throttle,
            "steering": steering,
            "equip_upgrade": equip_upgrade,
        });
        if let Ok(resp) = client.post(&url).json(&payload).send() {
            if let Ok(step_data) = resp.json::<MujocoStepResponse>() {
                let _ = tx.send(StudioNetEvent::MujocoStepCompleted(step_data));
            }
        }
    });
}

pub fn request_qa_audit(
    net: &StudioNetworkChannel,
    project_id: String,
    screenshot_path: Option<String>,
) {
    let tx = net.tx.clone();
    let url = format!("{}/api/qa-audit", net.backend_url);

    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut form = reqwest::blocking::multipart::Form::new().text("project_id", project_id);

        if let Some(path_str) = screenshot_path {
            if Path::new(&path_str).exists() {
                if let Ok(bytes) = fs::read(&path_str) {
                    if let Ok(part) = reqwest::blocking::multipart::Part::bytes(bytes)
                        .file_name("viewport.png")
                        .mime_str("image/png")
                    {
                        form = form.part("viewport_png", part);
                    }
                }
            }
        }

        match client.post(&url).multipart(form).send() {
            Ok(resp) => match resp.json::<QaAuditReport>() {
                Ok(report) => {
                    let _ = tx.send(StudioNetEvent::QaReportReceived(report));
                }
                Err(err) => {
                    let _ = tx.send(StudioNetEvent::NetworkError(format!(
                        "Failed to parse QaAuditReport: {err}"
                    )));
                }
            },
            Err(err) => {
                let _ = tx.send(StudioNetEvent::NetworkError(format!(
                    "QA endpoint offline ({err})"
                )));
            }
        }
    });
}

pub fn request_export_build(net: &StudioNetworkChannel, project_id: String) {
    let tx = net.tx.clone();
    let url = format!("{}/api/export-build", net.backend_url);

    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let payload = json!({ "project_id": project_id });
        match client.post(&url).json(&payload).send() {
            Ok(resp) => match resp.json::<ExportBuildResponse>() {
                Ok(export_info) => {
                    let _ = tx.send(StudioNetEvent::BuildExported(export_info));
                }
                Err(err) => {
                    let _ = tx.send(StudioNetEvent::NetworkError(format!(
                        "Export parse error: {err}"
                    )));
                }
            },
            Err(err) => {
                let _ = tx.send(StudioNetEvent::NetworkError(format!(
                    "Export endpoint error ({err})"
                )));
            }
        }
    });
}
