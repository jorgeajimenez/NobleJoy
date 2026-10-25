use crate::scene_parser::{LevelContract, QaAuditReport};
use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::thread;

#[derive(Debug, Clone)]
pub enum StudioNetEvent {
    LevelCommissioned(LevelContract),
    QaReportReceived(QaAuditReport),
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

pub fn request_commission_level(net: &StudioNetworkChannel, prompt: String) {
    let tx = net.tx.clone();
    let url = format!("{}/api/commission-level", net.backend_url);

    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let payload = json!({
            "prompt": prompt,
            "seed_blockade": true
        });
        match client.post(&url).json(&payload).send() {
            Ok(resp) => match resp.json::<LevelContract>() {
                Ok(contract) => {
                    let _ = tx.send(StudioNetEvent::LevelCommissioned(contract));
                }
                Err(err) => {
                    let _ = tx.send(StudioNetEvent::NetworkError(format!(
                        "Failed to parse LevelContract: {err}"
                    )));
                }
            },
            Err(err) => {
                let _ = tx.send(StudioNetEvent::NetworkError(format!(
                    "Studio backend unreachable ({err})"
                )));
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
                    "QA endpoint error ({err})"
                )));
            }
        }
    });
}
