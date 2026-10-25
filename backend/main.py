"""GRETCHEN: Autonomous Studio Service — FastAPI Gateway, Web Dashboard, MuJoCo Physics Server & Artifact Dispatcher."""

import json
import os
from pathlib import Path
from typing import Any, Dict, Optional
import uuid

from dotenv import load_dotenv
from fastapi import FastAPI, File, Form, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from agents import LevelArchitectAgent, StudioDirectorAgent, VisualQATesterAgent
from agents.artifact_puller import EXPORTS_DIR, export_physical_build_package
from mujoco_sim import MujocoDrivingSession

load_dotenv()

app = FastAPI(
    title="Gretchen — Autonomous AI Game Studio & MuJoCo Physics Gateway",
    description="Gemini 4 Argon Managed Agents Studio Backend + Real-Time MuJoCo Driving Physics for the Rust Bevy Client",
    version="0.3.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

ASSETS_DIR = Path(__file__).parent / "assets"
ASSETS_DIR.mkdir(parents=True, exist_ok=True)
EXPORTS_DIR.mkdir(parents=True, exist_ok=True)
app.mount("/assets", StaticFiles(directory=str(ASSETS_DIR)), name="assets")
app.mount("/exports", StaticFiles(directory=str(EXPORTS_DIR)), name="exports")

MANIFEST_PATH = ASSETS_DIR / "asset_manifest.json"
with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    ASSET_MANIFEST: Dict[str, Any] = json.load(f)

director_agent = StudioDirectorAgent()
architect_agent = LevelArchitectAgent()
qa_tester_agent = VisualQATesterAgent()

ACTIVE_PROJECTS: Dict[str, Dict[str, Any]] = {}
MUJOCO_SESSIONS: Dict[str, MujocoDrivingSession] = {}


class CommissionRequest(BaseModel):
    prompt: str = "Design an industrial rover test track with a shielded reactor core and high-torque wheel upgrade."
    project_id: Optional[str] = None
    level_index: int = 1
    seed_blockade: bool = True


class PhysicsStepRequest(BaseModel):
    project_id: str = "job_4091"
    throttle: float = 0.0
    steering: float = 0.0
    equip_upgrade: Optional[bool] = None


class ExportBuildRequest(BaseModel):
    project_id: str = "job_4091"


@app.get("/", response_class=HTMLResponse)
async def studio_dashboard() -> str:
    """Built-in interactive Web Frontend for Gretchen's Cloud Run Studio Gateway."""
    return """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>Gretchen — Autonomous AI Game Studio (Gemini 4 Argon + MuJoCo)</title>
  <style>
    :root { --bg: #0b0f17; --card: #151c28; --accent: #00d8ff; --ok: #10b981; --warn: #f59e0b; --text: #e2e8f0; }
    * { box-sizing: border-box; }
    body { margin: 0; font-family: system-ui, -apple-system, sans-serif; background: var(--bg); color: var(--text); padding: 24px; }
    header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #263248; padding-bottom: 16px; margin-bottom: 24px; }
    .badge { background: #1e293b; border: 1px solid var(--accent); color: var(--accent); padding: 4px 10px; border-radius: 999px; font-size: 12px; font-weight: 600; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(360px, 1fr)); gap: 20px; }
    .card { background: var(--card); border: 1px solid #263248; border-radius: 12px; padding: 18px; }
    h2 { margin-top: 0; font-size: 16px; color: var(--accent); text-transform: uppercase; letter-spacing: 0.05em; }
    textarea, input[type="text"] { width: 100%; background: #0f1520; border: 1px solid #334155; color: #fff; padding: 10px; border-radius: 8px; margin-bottom: 10px; }
    button { background: var(--accent); color: #061018; border: none; padding: 10px 16px; border-radius: 8px; font-weight: 700; cursor: pointer; margin-right: 8px; margin-bottom: 8px; }
    button.secondary { background: #334155; color: #fff; }
    pre { background: #090d14; padding: 12px; border-radius: 8px; overflow-x: auto; font-size: 12px; max-height: 260px; border: 1px solid #1e293b; }
    .skin-preview { width: 140px; height: 140px; border-radius: 8px; border: 2px solid var(--accent); object-fit: cover; background: #090d14; }
    .links a { color: var(--accent); margin-right: 12px; font-size: 13px; }
  </style>
</head>
<body>
  <header>
    <div>
      <h1 style="margin:0;font-size:22px;">GRETCHEN // Autonomous AI Game Studio & MuJoCo Gateway</h1>
      <p style="margin:4px 0 0;color:#94a3b8;font-size:14px;">Gemini 4 Argon Managed Agents (<code>antigravity-preview-09-2026</code>) + Nano Banana 2.1 + MuJoCo Physics</p>
    </div>
    <div>
      <span class="badge">Cloud Run Ready</span>
      <a href="/docs" style="color:var(--accent);margin-left:12px;font-size:14px;">Swagger API Docs &rarr;</a>
    </div>
  </header>

  <div class="grid">
    <div class="card">
      <h2>1. Commission Level & Pull Sandbox Artifacts</h2>
      <label style="font-size:12px;color:#94a3b8;">Project ID</label>
      <input id="projectId" type="text" value="job_4091"/>
      <label style="font-size:12px;color:#94a3b8;">Client Prompt</label>
      <textarea id="promptInput" rows="3">Design an industrial rover test track with a shielded reactor core and high-torque wheel upgrade.</textarea>
      <button onclick="commissionLevel()">Commission Managed Agents</button>
      <div id="artifactLinks" class="links" style="margin-top:10px;"></div>
      <div style="display:flex;gap:14px;align-items:center;margin-top:12px;">
        <img id="skinImg" class="skin-preview" alt="Generated Rover Skin" src="" style="display:none;"/>
        <div id="skinCaption" style="font-size:12px;color:#94a3b8;"></div>
      </div>
      <pre id="commissionOut">Click "Commission Managed Agents" to run Studio Director -> Level Architect -> Sandbox Pull.</pre>
    </div>

    <div class="card">
      <h2>2. MuJoCo Rover Physics & Torque Gate (&tau; = r &times; F)</h2>
      <p style="font-size:13px;color:#94a3b8;margin-top:0;">Steps the 4-wheel rover MJCF model and verifies Center-of-Mass footprint & servo torque limits.</p>
      <button onclick="stepPhysics(1.0, 0.0, false)">Drive Forward (Stock SG90 Servo)</button>
      <button class="secondary" onclick="stepPhysics(1.0, 0.25, true)">Snap M3 Upgrade (MG996R 11 kg&middot;cm) & Step</button>
      <pre id="physicsOut">Waiting for MuJoCo physics step...</pre>
    </div>

    <div class="card">
      <h2>3. Visual QA Playtester Audit & Live Hotfix</h2>
      <p style="font-size:13px;color:#94a3b8;margin-top:0;">Audits viewport PNG capture + scene graph with <code>gemini-4-argon</code> and issues transform + MuJoCo hotfixes.</p>
      <input type="file" id="viewportFile" accept="image/png" style="margin-bottom:10px;display:block;color:#94a3b8;"/>
      <button onclick="runQaAudit()">Run Visual QA Audit & Hotfix</button>
      <pre id="qaOut">Waiting for Visual QA audit...</pre>
    </div>

    <div class="card">
      <h2>4. Endgame Manufacturing Bundle Export</h2>
      <p style="font-size:13px;color:#94a3b8;margin-top:0;">Generates M3-calibrated watertight STLs, ROS/Gazebo <code>rover.urdf</code>, <code>BOM.csv</code>, and MicroPython firmware.</p>
      <button onclick="exportBuild()">Export Physical Build Package</button>
      <pre id="exportOut">Waiting for export...</pre>
    </div>
  </div>

  <script>
    async function commissionLevel() {
      const project_id = document.getElementById('projectId').value;
      const prompt = document.getElementById('promptInput').value;
      document.getElementById('commissionOut').textContent = 'Running Studio Director + Level Architect + Managed Agent Pull...';
      const res = await fetch('/api/commission-level', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({project_id, prompt, seed_blockade: true})
      });
      const data = await res.json();
      document.getElementById('commissionOut').textContent = JSON.stringify(data, null, 2);
      if (data.skin_url) {
        const img = document.getElementById('skinImg');
        img.src = data.skin_url + '?t=' + Date.now();
        img.style.display = 'block';
        document.getElementById('skinCaption').innerHTML =
          '<b>Generated Rover Albedo Skin</b><br/>Served to Rust Bevy Client';
      }
      document.getElementById('artifactLinks').innerHTML = `
        <a href="${data.mjcf_url}" target="_blank">MuJoCo XML (car_level.xml)</a>
        <a href="${data.urdf_url}" target="_blank">ROS URDF (rover.urdf)</a>
        <a href="${data.stl_url}" target="_blank">M3 Upgrade Mesh (.stl)</a>
      `;
    }

    async function stepPhysics(throttle, steering, equip_upgrade) {
      const project_id = document.getElementById('projectId').value;
      const res = await fetch('/api/physics-step', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({project_id, throttle, steering, equip_upgrade})
      });
      const data = await res.json();
      document.getElementById('physicsOut').textContent = JSON.stringify(data, null, 2);
    }

    async function runQaAudit() {
      const project_id = document.getElementById('projectId').value;
      const fileInput = document.getElementById('viewportFile');
      const form = new FormData();
      form.append('project_id', project_id);
      if (fileInput.files.length > 0) {
        form.append('viewport_png', fileInput.files[0]);
      }
      document.getElementById('qaOut').textContent = 'Running Gemini 4 Argon Visual QA Playtester...';
      const res = await fetch('/api/qa-audit', { method: 'POST', body: form });
      const data = await res.json();
      document.getElementById('qaOut').textContent = JSON.stringify(data, null, 2);
    }

    async function exportBuild() {
      const project_id = document.getElementById('projectId').value;
      const res = await fetch('/api/export-build', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({project_id})
      });
      const data = await res.json();
      document.getElementById('exportOut').textContent = JSON.stringify(data, null, 2);
    }
  </script>
</body>
</html>"""


@app.get("/health")
async def health() -> Dict[str, str]:
    return {
        "status": "ok",
        "studio": "Gretchen Autonomous Game Studio",
        "model": "gemini-4-argon",
        "managed_agent": "antigravity-preview-09-2026",
        "physics": "mujoco",
        "environment": os.getenv("K_SERVICE", "local"),
    }


@app.post("/api/commission-level")
async def commission_level(req: CommissionRequest) -> Dict[str, Any]:
    """Run Studio Director -> Level Architect -> Managed Agent Artifact Pull -> MuJoCo Session Init."""
    project_id = req.project_id or f"job_{uuid.uuid4().hex[:4]}"
    print(f"[Studio Gateway] Commissioning driving level {project_id}: '{req.prompt}'")

    directive = director_agent.scope_brief(
        req.prompt,
        project_id=project_id,
        level_index=req.level_index,
    )
    level_contract = architect_agent.build_level_contract(
        directive=directive,
        manifest=ASSET_MANIFEST,
        seed_blockade=req.seed_blockade,
    )

    sim = MujocoDrivingSession(
        project_id=project_id,
        car_spec=level_contract.get("car_physics", {}),
        level_entities=level_contract.get("entities", []),
    )
    MUJOCO_SESSIONS[project_id] = sim
    level_contract["sanity_check"] = sim.compute_physics_sanity_check()
    ACTIVE_PROJECTS[project_id] = level_contract
    return level_contract


@app.post("/api/physics-step")
async def physics_step(req: PhysicsStepRequest) -> Dict[str, Any]:
    """Step the MuJoCo car/rover simulation and return live 3D body transforms + CoM/torque sanity checks."""
    sim = MUJOCO_SESSIONS.get(req.project_id)
    if sim is None:
        contract = ACTIVE_PROJECTS.get(req.project_id, {})
        sim = MujocoDrivingSession(
            project_id=req.project_id,
            car_spec=contract.get("car_physics", {}),
            level_entities=contract.get("entities", []),
        )
        MUJOCO_SESSIONS[req.project_id] = sim

    return sim.step(
        throttle=req.throttle,
        steering=req.steering,
        equip_upgrade=req.equip_upgrade,
    )


@app.post("/api/qa-audit")
async def qa_audit(
    project_id: str = Form("job_4091"),
    viewport_png: Optional[UploadFile] = File(None),
) -> Dict[str, Any]:
    """Receive raw viewport PNG from the Rust client, run Visual QA Agent, and patch scene + MuJoCo physics."""
    image_bytes: Optional[bytes] = None
    if viewport_png is not None:
        image_bytes = await viewport_png.read()
        print(f"[Studio Gateway] Received viewport capture ({len(image_bytes)} bytes) for {project_id}")

    current_scene = ACTIVE_PROJECTS.get(project_id, {"project_id": project_id, "entities": []})
    report = qa_tester_agent.audit_viewport(
        project_id=project_id,
        image_bytes=image_bytes,
        current_scene=current_scene,
    )

    if report.get("qa_status") == "hotfix_required":
        hotfix_map = {hf["id"]: hf for hf in report.get("hotfixes", [])}
        for ent in current_scene.get("entities", []):
            if ent["id"] in hotfix_map:
                hf = hotfix_map[ent["id"]]
                ent["x"] = hf.get("x", ent["x"])
                ent["y"] = hf.get("y", ent["y"])
                ent["z"] = hf.get("z", ent["z"])

        sim = MUJOCO_SESSIONS.get(project_id)
        if sim is not None:
            sim.level_entities = current_scene.get("entities", [])
            if "physics_hotfix" in report:
                sim.apply_physics_hotfix(report["physics_hotfix"])
                report["sanity_check"] = sim.compute_physics_sanity_check()

    return report


@app.post("/api/export-build")
async def export_build(req: ExportBuildRequest) -> Dict[str, Any]:
    """Export the physical manufacturing bundle: M3-calibrated STLs + URDF + BOM.csv + MicroPython Firmware."""
    contract = ACTIVE_PROJECTS.get(req.project_id, {})
    return export_physical_build_package(
        project_id=req.project_id,
        car_spec=contract.get("car_physics", {}),
        reward_part=contract.get("reward_part", {}),
    )
