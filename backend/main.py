"""GRETCHEN: Autonomous Studio Service — FastAPI Gateway & Work Order Dispatcher."""

import json
from pathlib import Path
from typing import Any, Dict, List, Optional
import uuid

from dotenv import load_dotenv
from fastapi import FastAPI, File, Form, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from agents import LevelArchitectAgent, StudioDirectorAgent, VisualQATesterAgent

load_dotenv()

app = FastAPI(
    title="Gretchen — Autonomous AI Game Studio API Gateway",
    description="Gemini 4 Argon & Managed Agents Studio Backend for the Native Rust Engine",
    version="0.1.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

MANIFEST_PATH = Path(__file__).parent / "assets" / "asset_manifest.json"
with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    ASSET_MANIFEST: Dict[str, Any] = json.load(f)

director_agent = StudioDirectorAgent()
architect_agent = LevelArchitectAgent()
qa_tester_agent = VisualQATesterAgent()

# In-memory store for active studio commissions and pending hotfixes
ACTIVE_PROJECTS: Dict[str, Dict[str, Any]] = {}


class CommissionRequest(BaseModel):
    prompt: str = "Construct a sci-fi reactor bay with defensive blockades."
    project_id: Optional[str] = None
    seed_blockade: bool = True


class EntityContract(BaseModel):
    id: str
    asset_key: str
    x: float
    y: float
    z: float
    scale: float = 1.0
    solid: bool = True


class LevelContractResponse(BaseModel):
    project_id: str
    theme: str
    world_bounds: Dict[str, float]
    entities: List[EntityContract]


@app.get("/health")
async def health() -> Dict[str, str]:
    return {"status": "ok", "studio": "Gretchen Autonomous Game Studio", "model": "gemini-4-argon"}


@app.post("/api/commission-level", response_model=LevelContractResponse)
async def commission_level(req: CommissionRequest) -> Dict[str, Any]:
    """Run the Studio Director -> Level Architect pipeline to produce a playable 3D scene contract."""
    project_id = req.project_id or f"job_{uuid.uuid4().hex[:4]}"
    print(f"[Studio Gateway] Commissioning {project_id}: '{req.prompt}'")

    directive = director_agent.scope_brief(req.prompt, project_id=project_id)
    level_contract = architect_agent.build_level_contract(
        directive=directive,
        manifest=ASSET_MANIFEST,
        seed_blockade=req.seed_blockade,
    )
    ACTIVE_PROJECTS[project_id] = level_contract
    return level_contract


@app.post("/api/qa-audit")
async def qa_audit(
    project_id: str = Form("job_4091"),
    viewport_png: Optional[UploadFile] = File(None),
) -> Dict[str, Any]:
    """Receive raw viewport PNG from the Rust client, run Visual QA Agent, and return transform hotfixes."""
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

    # Apply hotfixes to our server-side scene copy as well
    if report.get("qa_status") == "hotfix_required":
        hotfix_map = {hf["id"]: hf for hf in report.get("hotfixes", [])}
        for ent in current_scene.get("entities", []):
            if ent["id"] in hotfix_map:
                hf = hotfix_map[ent["id"]]
                ent["x"] = hf.get("x", ent["x"])
                ent["y"] = hf.get("y", ent["y"])
                ent["z"] = hf.get("z", ent["z"])

    return report
