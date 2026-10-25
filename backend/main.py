"""GRETCHEN: Autonomous Studio Service — FastAPI Gateway, MuJoCo Physics Server & Artifact Dispatcher."""

import json
from pathlib import Path
from typing import Any, Dict, Optional
import uuid

from dotenv import load_dotenv
from fastapi import FastAPI, File, Form, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from agents import LevelArchitectAgent, StudioDirectorAgent, VisualQATesterAgent
from agents.artifact_puller import export_physical_build_package
from mujoco_sim import MujocoDrivingSession

load_dotenv()

app = FastAPI(
    title="Gretchen — Autonomous AI Game Studio & MuJoCo Physics Gateway",
    description="Gemini 4 Argon Managed Agents Studio Backend + Real-Time MuJoCo Driving Physics for the Rust Bevy Client",
    version="0.2.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

ASSETS_DIR = Path(__file__).parent / "assets"
ASSETS_DIR.mkdir(parents=True, exist_ok=True)
app.mount("/assets", StaticFiles(directory=str(ASSETS_DIR)), name="assets")

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
    seed_blockade: bool = True


class PhysicsStepRequest(BaseModel):
    project_id: str = "job_4091"
    throttle: float = 0.0
    steering: float = 0.0
    equip_upgrade: Optional[bool] = None


class ExportBuildRequest(BaseModel):
    project_id: str = "job_4091"


@app.get("/health")
async def health() -> Dict[str, str]:
    return {
        "status": "ok",
        "studio": "Gretchen Autonomous Game Studio",
        "model": "gemini-4-argon",
        "managed_agent": "antigravity-preview-09-2026",
        "physics": "mujoco",
    }


@app.post("/api/commission-level")
async def commission_level(req: CommissionRequest) -> Dict[str, Any]:
    """Run Studio Director -> Level Architect -> Managed Agent Artifact Pull -> MuJoCo Session Init."""
    project_id = req.project_id or f"job_{uuid.uuid4().hex[:4]}"
    print(f"[Studio Gateway] Commissioning driving level {project_id}: '{req.prompt}'")

    directive = director_agent.scope_brief(req.prompt, project_id=project_id)
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
