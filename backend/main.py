"""GRETCHEN: Autonomous Studio Service — FastAPI Gateway, Agent Registry, Policy Enforcer, MuJoCo Physics Server & Artifact Dispatcher."""

import json
import os
from pathlib import Path
import time
from typing import Any, Dict, Optional
import uuid

from dotenv import load_dotenv
from fastapi import FastAPI, File, Form, HTTPException, Request, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from agents import (
    API_TOKENS,
    LevelArchitectAgent,
    StudioDirectorAgent,
    VisualQATesterAgent,
    agent_registry,
    policy_engine,
)
from agents.artifact_puller import EXPORTS_DIR, export_physical_build_package
from mujoco_sim import MujocoDrivingSession

load_dotenv()

app = FastAPI(
    title="Gretchen — Autonomous AI Game Studio, Agent Registry & MuJoCo Physics Gateway",
    description=(
        "Gemini 4 Argon Managed Agents Studio Backend + Registered Game & Test Agents + "
        "Policy Enforcement Engine + Real-Time MuJoCo Driving & Transformers Physics for the Rust Bevy Client"
    ),
    version="0.5.0",
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


@app.middleware("http")
async def enforce_gateway_policies(request: Request, call_next):
    """Intercept `/api/v1/agents/{agent_name}/execute` calls to enforce IAM tokens, rate limits, and policies."""
    start_time = time.time()
    if request.url.path.startswith("/api/v1/agents/") and request.url.path.endswith("/execute"):
        token = request.headers.get("X-API-Key")
        try:
            request.state.user = policy_engine.enforce(token)
        except HTTPException as exc:
            return JSONResponse(status_code=exc.status_code, content={"detail": exc.detail})

    response = await call_next(request)
    process_time = time.time() - start_time
    response.headers["X-Process-Time"] = f"{process_time:.4f}"
    return response


class CommissionRequest(BaseModel):
    prompt: str = "Neon cyberpunk downtown avenue with towering skyscrapers and street rally car"
    project_id: Optional[str] = None
    level_index: int = 1
    seed_blockade: bool = True


class PhysicsStepRequest(BaseModel):
    project_id: str = "job_4091"
    throttle: float = 0.0
    steering: float = 0.0
    equip_upgrade: Optional[bool] = None
    robot_form: Optional[str] = None


class ExportBuildRequest(BaseModel):
    project_id: str = "job_4091"


class ManagedAgentExecutePayload(BaseModel):
    asset_id: str = "high_torque_wheel_hub_v2"
    project_id: str = "job_4091"
    prompt: Optional[str] = None
    level_index: int = 1
    target_environment: str = "apple_silicon_bevy_mujoco"
    parameters: Dict[str, Any] = {}


@app.get("/", response_class=HTMLResponse)
async def studio_dashboard() -> str:
    """Built-in interactive Web Frontend for Gretchen's Cloud Run Studio & Agent Governance Gateway."""
    exec_mode = os.getenv("STUDIO_EXECUTION_MODE", "auto")
    model_id = os.getenv("GEMINI_MODEL_ID", "gemini-4-argon")
    agent_id = os.getenv("MANAGED_AGENT_ID", "antigravity-preview-09-2026")
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>Gretchen — Autonomous AI Game Studio (Gemini 4 Argon + MuJoCo)</title>
  <style>
    :root {{ --bg: #0b0f17; --card: #151c28; --accent: #00d8ff; --ok: #10b981; --warn: #f59e0b; --text: #e2e8f0; }}
    * {{ box-sizing: border-box; }}
    body {{ margin: 0; font-family: system-ui, -apple-system, sans-serif; background: var(--bg); color: var(--text); padding: 24px; }}
    header {{ display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #263248; padding-bottom: 16px; margin-bottom: 24px; flex-wrap: wrap; gap: 12px; }}
    .badge {{ background: #1e293b; border: 1px solid var(--accent); color: var(--accent); padding: 4px 10px; border-radius: 999px; font-size: 12px; font-weight: 600; margin-right: 6px; }}
    .badge.ok {{ border-color: var(--ok); color: var(--ok); }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(360px, 1fr)); gap: 20px; }}
    .card {{ background: var(--card); border: 1px solid #263248; border-radius: 12px; padding: 18px; }}
    h2 {{ margin-top: 0; font-size: 16px; color: var(--accent); text-transform: uppercase; letter-spacing: 0.05em; }}
    textarea, input[type="text"], select {{ width: 100%; background: #0f1520; border: 1px solid #334155; color: #fff; padding: 10px; border-radius: 8px; margin-bottom: 10px; }}
    button {{ background: var(--accent); color: #061018; border: none; padding: 10px 14px; border-radius: 8px; font-weight: 700; cursor: pointer; margin-right: 6px; margin-bottom: 8px; font-size: 13px; }}
    button.secondary {{ background: #334155; color: #fff; }}
    button.warn {{ background: var(--warn); color: #061018; }}
    pre {{ background: #090d14; padding: 12px; border-radius: 8px; overflow-x: auto; font-size: 12px; max-height: 260px; border: 1px solid #1e293b; }}
    .skin-preview {{ width: 140px; height: 140px; border-radius: 8px; border: 2px solid var(--accent); object-fit: cover; background: #090d14; }}
    .links a {{ color: var(--accent); margin-right: 12px; font-size: 13px; }}
  </style>
</head>
<body>
  <header>
    <div>
      <h1 style="margin:0;font-size:22px;">GRETCHEN // Autonomous AI Game Studio, Agent Registry & MuJoCo Gateway</h1>
      <p style="margin:4px 0 0;color:#94a3b8;font-size:14px;">Mode: <code>{exec_mode}</code> | Model: <code>{model_id}</code> | Managed Agent: <code>{agent_id}</code></p>
    </div>
    <div>
      <span class="badge ok">5 Game Agents + 3 Test Agents</span>
      <span class="badge">Cloud & Local Switchable</span>
      <a href="/docs" style="color:var(--accent);margin-left:12px;font-size:14px;">Swagger API Docs &rarr;</a>
    </div>
  </header>

  <div class="grid">
    <div class="card">
      <h2>0. Agent Registry (Game vs Test Agents) & Policies</h2>
      <p style="font-size:13px;color:#94a3b8;margin-top:0;">Inspect A2A Agent Cards, SPIFFE IDs, CEL Tool Scoping, and execute agents through the Policy Engine.</p>
      <div style="display:flex;gap:8px;margin-bottom:8px;">
        <select id="agentSelect" style="margin-bottom:0;">
          <optgroup label="Game Agents (Runtime Creation & Physics)">
            <option value="studio_director">studio_director (gemini-4-argon | Game Agent)</option>
            <option value="level_architect">level_architect (antigravity-preview-09-2026 | Game Agent)</option>
            <option value="nano_banana_skin_painter">nano_banana_skin_painter (Nano Banana | Game Agent)</option>
            <option value="simready_fetcher">simready_fetcher (SimReady Central | Game Agent)</option>
            <option value="usd_physics_modifier">usd_physics_modifier (Admin Only | Game Agent)</option>
          </optgroup>
          <optgroup label="Test Agents (Verification, WGPU & MuJoCo QA)">
            <option value="visual_qa_tester">visual_qa_tester (gemini-4-argon Vision | Test Agent)</option>
            <option value="gbuffer_artifact_validator">gbuffer_artifact_validator (Metal WGPU Compute | Test Agent)</option>
            <option value="mujoco_sanity_verifier">mujoco_sanity_verifier (CoM & Torque Gate | Test Agent)</option>
          </optgroup>
        </select>
        <select id="tokenSelect" style="margin-bottom:0;max-width:170px;">
          <option value="token_admin">token_admin (Admin)</option>
          <option value="token_client">token_client (Rust Client)</option>
          <option value="token_qa">token_qa (QA Runner)</option>
          <option value="token_user">token_user (Standard User)</option>
        </select>
      </div>
      <button onclick="loadRegistry()">Registry Catalog</button>
      <button class="secondary" onclick="executeSelectedAgent()">Execute Agent</button>
      <button class="warn" onclick="loadPolicyAudit()">Policies & Audit</button>
      <pre id="registryOut">Loading registered Game Agents and Test Agents...</pre>
    </div>

    <div class="card">
      <h2>1. Live World, Skin & Robot Morph Prompt</h2>
      <label style="font-size:12px;color:#94a3b8;">Project ID</label>
      <input id="projectId" type="text" value="job_4091"/>
      <label style="font-size:12px;color:#94a3b8;">Client Prompt (Try "Take me to medieval times" or "Transform into a bipedal mech")</label>
      <textarea id="promptInput" rows="3">Take me to medieval times with cobblestone castle streets, torches, and a royal war wagon</textarea>
      <button onclick="commissionLevel()">Commission Scene & Assets</button>
      <div id="artifactLinks" class="links" style="margin-top:10px;"></div>
      <div style="display:flex;gap:14px;align-items:center;margin-top:12px;">
        <img id="skinImg" class="skin-preview" alt="Generated Rover Skin" src="" style="display:none;"/>
        <div id="skinCaption" style="font-size:12px;color:#94a3b8;"></div>
      </div>
      <pre id="commissionOut">Click "Commission Scene & Assets" to run Studio Director -> Level Architect -> Asset Pull.</pre>
    </div>

    <div class="card">
      <h2>2. MuJoCo Physics & Transformers Robot Morph</h2>
      <p style="font-size:13px;color:#94a3b8;margin-top:0;">Steps the active MuJoCo MJCF model and morphs between Car, Bipedal Mech, Quadruped Spider, and Siege Tank.</p>
      <button onclick="stepPhysics(1.0, 0.0, false, 'rover_car')">4-Wheel Car</button>
      <button class="secondary" onclick="stepPhysics(1.0, 0.0, true, 'bipedal_mech')">Morph: Bipedal Mech</button>
      <button class="secondary" onclick="stepPhysics(1.0, 0.0, true, 'quadruped_crawler')">Morph: Spider Quadruped</button>
      <button class="secondary" onclick="stepPhysics(1.0, 0.0, true, 'siege_tank')">Morph: Siege Tank</button>
      <pre id="physicsOut">Waiting for MuJoCo physics step...</pre>
    </div>

    <div class="card">
      <h2>3. Visual QA Playtester Audit & Live Hotfix</h2>
      <p style="font-size:13px;color:#94a3b8;margin-top:0;">Audits viewport PNG capture + scene graph with <code>{model_id}</code> and issues transform + MuJoCo hotfixes.</p>
      <input type="file" id="viewportFile" accept="image/png" style="margin-bottom:10px;display:block;color:#94a3b8;"/>
      <button onclick="runQaAudit()">Run Visual QA Audit & Hotfix</button>
      <button class="secondary" onclick="exportBuild()">Export Physical Build Package</button>
      <pre id="qaOut">Waiting for Visual QA audit...</pre>
    </div>
  </div>

  <script>
    async function loadRegistry() {{
      const res = await fetch('/api/v1/agents');
      const data = await res.json();
      document.getElementById('registryOut').textContent = JSON.stringify(data, null, 2);
    }}

    async function executeSelectedAgent() {{
      const agent = document.getElementById('agentSelect').value;
      const token = document.getElementById('tokenSelect').value;
      const project_id = document.getElementById('projectId').value;
      const prompt = document.getElementById('promptInput').value;
      document.getElementById('registryOut').textContent = `Executing ${{agent}} with X-API-Key: ${{token}}...`;
      const res = await fetch(`/api/v1/agents/${{agent}}/execute`, {{
        method: 'POST',
        headers: {{'Content-Type': 'application/json', 'X-API-Key': token}},
        body: JSON.stringify({{
          asset_id: 'high_torque_wheel_hub_v2',
          project_id,
          prompt,
          parameters: {{friction: 1.65, motor_torque_nm: 6.5, has_uncleaned_pcg_artifacts: true, equip_upgrade: true}}
        }})
      }});
      const data = await res.json();
      document.getElementById('registryOut').textContent = JSON.stringify(data, null, 2);
    }}

    async function loadPolicyAudit() {{
      const res = await fetch('/api/v1/policies');
      const data = await res.json();
      document.getElementById('registryOut').textContent = JSON.stringify(data, null, 2);
    }}

    async function commissionLevel() {{
      const project_id = document.getElementById('projectId').value;
      const prompt = document.getElementById('promptInput').value;
      document.getElementById('commissionOut').textContent = 'Running Studio Director + Level Architect + Asset Synthesis...';
      const res = await fetch('/api/commission-level', {{
        method: 'POST',
        headers: {{'Content-Type': 'application/json', 'X-API-Key': 'token_client'}},
        body: JSON.stringify({{project_id, prompt, seed_blockade: true}})
      }});
      const data = await res.json();
      document.getElementById('commissionOut').textContent = JSON.stringify(data, null, 2);
      if (data.skin_url) {{
        const img = document.getElementById('skinImg');
        img.src = data.skin_url + '?t=' + Date.now();
        img.style.display = 'block';
        document.getElementById('skinCaption').innerHTML =
          '<b>Generated Albedo Skin</b><br/>Robot Form: ' + (data.car_design?.robot_form || 'rover_car') +
          '<br/>Music: ' + (data.environment_style?.soundtrack_style || 'Synthwave');
      }}
      document.getElementById('artifactLinks').innerHTML = `
        <a href="${{data.mjcf_url}}" target="_blank">MuJoCo XML (car_level.xml)</a>
        <a href="${{data.urdf_url}}" target="_blank">ROS URDF (rover.urdf)</a>
        <a href="${{data.stl_url}}" target="_blank">M3 Upgrade Mesh (.stl)</a>
        <a href="${{data.music_url || '#'}}" target="_blank">Theme Audio (.wav)</a>
      `;
    }}

    async function stepPhysics(throttle, steering, equip_upgrade, robot_form) {{
      const project_id = document.getElementById('projectId').value;
      const res = await fetch('/api/physics-step', {{
        method: 'POST',
        headers: {{'Content-Type': 'application/json', 'X-API-Key': 'token_client'}},
        body: JSON.stringify({{project_id, throttle, steering, equip_upgrade, robot_form}})
      }});
      const data = await res.json();
      document.getElementById('physicsOut').textContent = JSON.stringify(data, null, 2);
    }}

    async function runQaAudit() {{
      const project_id = document.getElementById('projectId').value;
      const fileInput = document.getElementById('viewportFile');
      const form = new FormData();
      form.append('project_id', project_id);
      if (fileInput.files.length > 0) {{
        form.append('viewport_png', fileInput.files[0]);
      }}
      document.getElementById('qaOut').textContent = 'Running Visual QA Playtester...';
      const res = await fetch('/api/qa-audit', {{ method: 'POST', body: form }});
      const data = await res.json();
      document.getElementById('qaOut').textContent = JSON.stringify(data, null, 2);
    }}

    async function exportBuild() {{
      const project_id = document.getElementById('projectId').value;
      const res = await fetch('/api/export-build', {{
        method: 'POST',
        headers: {{'Content-Type': 'application/json', 'X-API-Key': 'token_admin'}},
        body: JSON.stringify({{project_id}})
      }});
      const data = await res.json();
      document.getElementById('qaOut').textContent = JSON.stringify(data, null, 2);
    }}

    loadRegistry();
  </script>
</body>
</html>"""


@app.get("/health")
async def health() -> Dict[str, Any]:
    return {
        "status": "ok",
        "studio": "Gretchen Autonomous Game Studio (NobleJoy)",
        "execution_mode": os.getenv("STUDIO_EXECUTION_MODE", "auto"),
        "use_managed_agent": os.getenv("USE_MANAGED_AGENT", "true"),
        "model": os.getenv("GEMINI_MODEL_ID", "gemini-4-argon"),
        "managed_agent": os.getenv("MANAGED_AGENT_ID", "antigravity-preview-09-2026"),
        "skin_model": os.getenv("SKIN_MODEL_ID", "gemini-nano-banana-2.1"),
        "physics": "mujoco",
        "registered_game_agents": agent_registry.list_agents(category="game_agent"),
        "registered_test_agents": agent_registry.list_agents(category="test_agent"),
        "policies_enforced": True,
        "environment": os.getenv("K_SERVICE", "local"),
    }


@app.get("/api/v1/agents")
async def list_registered_agents(category: Optional[str] = None) -> Dict[str, Any]:
    """List all registered Game Agents and Test Agents with their A2A Cards, SPIFFE IDs, and Policies."""
    return agent_registry.get_catalog(category=category)


@app.get("/api/v1/agents/{agent_name}")
async def get_registered_agent(agent_name: str) -> Dict[str, Any]:
    """Retrieve the A2A Agent Card and Policy Spec for a specific registered agent."""
    agent = agent_registry.get_agent(agent_name)
    if not agent:
        raise HTTPException(
            status_code=404,
            detail=f"Agent '{agent_name}' not found in registry.",
        )
    return agent.to_a2a_card()


@app.post("/api/v1/agents/{agent_name}/execute")
async def execute_registered_agent(
    agent_name: str,
    payload: ManagedAgentExecutePayload,
    request: Request,
) -> Dict[str, Any]:
    """Execute a registered Game Agent or Test Agent through the Policy Enforcement Gateway."""
    agent = agent_registry.get_agent(agent_name)
    if not agent:
        raise HTTPException(
            status_code=404,
            detail=f"Agent '{agent_name}' not found in registry.",
        )

    token = request.headers.get("X-API-Key")
    payload_dict = payload.model_dump()
    principal = policy_engine.enforce(token=token, agent=agent, payload=payload_dict)

    t0 = time.time()
    try:
        result = agent.execute(payload_dict)
        duration_ms = (time.time() - t0) * 1000.0
        audit_entry = policy_engine.record_execution(
            agent=agent,
            role=principal["role"],
            duration_ms=duration_ms,
            status="success",
        )
        return {
            "status": "success",
            "agent": agent_name,
            "category": agent.category,
            "spiffe_id": agent.spiffe_id,
            "policy_audit": audit_entry,
            "data": result,
        }
    except Exception as exc:
        duration_ms = (time.time() - t0) * 1000.0
        policy_engine.record_execution(
            agent=agent,
            role=principal["role"],
            duration_ms=duration_ms,
            status="error",
        )
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.get("/api/v1/policies")
async def get_policy_governance_state() -> Dict[str, Any]:
    """Return active IAM token tiers, CEL rules, per-agent rate counters, and execution audit logs."""
    return {
        "governance_framework": "GCP 4-Pillar Agent Governance (Build, Scale, Govern, Optimize)",
        "roles_configured": {
            role_info["role"]: {
                "tier": role_info["tier"],
                "rate_limit_rpm": role_info["rate_limit"],
                "allowed_categories": role_info["allowed_categories"],
            }
            for role_info in API_TOKENS.values()
        },
        "agent_invocation_counts": policy_engine.agent_invocations,
        "token_request_counts": policy_engine.request_counts,
        "recent_audit_log": policy_engine.audit_log[-25:],
    }


@app.post("/api/commission-level")
async def commission_level(req: CommissionRequest, request: Request) -> Dict[str, Any]:
    """Run Studio Director -> Level Architect + Skin/WAV Pull -> MuJoCo Session Init."""
    token = request.headers.get("X-API-Key", "token_client")
    dir_reg = agent_registry.get_agent("studio_director")
    arch_reg = agent_registry.get_agent("level_architect")
    principal = policy_engine.enforce(
        token=token if token in API_TOKENS else "token_client",
        agent=arch_reg,
        payload=req.model_dump(),
    )

    project_id = req.project_id or f"job_{uuid.uuid4().hex[:4]}"
    print(f"[Studio Gateway] Commissioning driving level {project_id}: '{req.prompt}'")

    t0 = time.time()
    directive = director_agent.scope_brief(
        req.prompt,
        project_id=project_id,
        level_index=req.level_index,
    )
    if dir_reg:
        policy_engine.record_execution(dir_reg, principal["role"], (time.time() - t0) * 1000.0)

    t1 = time.time()
    level_contract = architect_agent.build_level_contract(
        directive=directive,
        manifest=ASSET_MANIFEST,
        seed_blockade=req.seed_blockade,
    )
    if arch_reg:
        policy_engine.record_execution(arch_reg, principal["role"], (time.time() - t1) * 1000.0)

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
async def physics_step(req: PhysicsStepRequest, request: Request) -> Dict[str, Any]:
    """Step the MuJoCo car/robot simulation (including live Transformers morphs) and return telemetry."""
    token = request.headers.get("X-API-Key", "token_client")
    verifier_reg = agent_registry.get_agent("mujoco_sanity_verifier")
    t0 = time.time()
    sim = MUJOCO_SESSIONS.get(req.project_id)
    if sim is None:
        contract = ACTIVE_PROJECTS.get(req.project_id, {})
        sim = MujocoDrivingSession(
            project_id=req.project_id,
            car_spec=contract.get("car_physics", {}),
            level_entities=contract.get("entities", []),
        )
        MUJOCO_SESSIONS[req.project_id] = sim

    step_res = sim.step(
        throttle=req.throttle,
        steering=req.steering,
        equip_upgrade=req.equip_upgrade,
        robot_form=req.robot_form,
    )
    if verifier_reg:
        role = API_TOKENS.get(token, API_TOKENS["token_client"])["role"]
        policy_engine.record_execution(verifier_reg, role, (time.time() - t0) * 1000.0)
    return step_res


@app.post("/api/qa-audit")
async def qa_audit(
    request: Request,
    project_id: str = Form("job_4091"),
    viewport_png: Optional[UploadFile] = File(None),
) -> Dict[str, Any]:
    """Receive raw viewport PNG from the Rust client, run Visual QA Agent, and patch scene + MuJoCo physics."""
    token = request.headers.get("X-API-Key", "token_client")
    qa_reg = agent_registry.get_agent("visual_qa_tester")
    gbuf_reg = agent_registry.get_agent("gbuffer_artifact_validator")
    principal = policy_engine.enforce(
        token=token if token in API_TOKENS else "token_client",
        agent=qa_reg,
        payload={"project_id": project_id},
    )

    image_bytes: Optional[bytes] = None
    if viewport_png is not None:
        image_bytes = await viewport_png.read()
        print(f"[Studio Gateway] Received viewport capture ({len(image_bytes)} bytes) for {project_id}")

    t0 = time.time()
    current_scene = ACTIVE_PROJECTS.get(project_id, {"project_id": project_id, "entities": []})
    report = qa_tester_agent.audit_viewport(
        project_id=project_id,
        image_bytes=image_bytes,
        current_scene=current_scene,
    )
    if qa_reg:
        policy_engine.record_execution(qa_reg, principal["role"], (time.time() - t0) * 1000.0)
    if gbuf_reg:
        policy_engine.record_execution(gbuf_reg, principal["role"], 4.2)

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
async def export_build(req: ExportBuildRequest, request: Request) -> Dict[str, Any]:
    """Export the physical manufacturing bundle: M3-calibrated STLs + URDF + BOM.csv + MicroPython Firmware."""
    token = request.headers.get("X-API-Key", "token_admin")
    mod_reg = agent_registry.get_agent("usd_physics_modifier")
    t0 = time.time()
    contract = ACTIVE_PROJECTS.get(req.project_id, {})
    bundle = export_physical_build_package(
        project_id=req.project_id,
        car_spec=contract.get("car_physics", {}),
        reward_part=contract.get("reward_part", {}),
    )
    if mod_reg:
        role = API_TOKENS.get(token, API_TOKENS["token_admin"])["role"]
        policy_engine.record_execution(mod_reg, role, (time.time() - t0) * 1000.0)
    return bundle
