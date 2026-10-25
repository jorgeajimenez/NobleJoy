# Gretchen — Autonomous AI Game Studio & MuJoCo Backend (`backend/`)

This directory contains **Gretchen's** half of the monorepo: the **Gemini 4 Argon Managed Agents Studio Backend**, **MuJoCo Driving & Robotics Physics Server**, **Sandbox Artifact Puller**, and **Physical Export Pipeline**.

You can develop, run, and test everything in `backend/` **100% independently** of the Rust game client.

---

## 1. Quick Start (Using `uv`)

```bash
cd backend
cp .env.example .env
# Open .env and set GEMINI_API_KEY=your_key_here

# Create virtualenv and install dependencies
uv venv
uv pip install -r requirements.txt

# Start the FastAPI server with hot-reload
uv run uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

Once running, open **`http://127.0.0.1:8000/docs`** in your browser for interactive Swagger UI testing of every endpoint.

---

## 2. Independent Testing (Without the Rust Game Client)

### Option A: Run the Automated Studio Pipeline Test
With the server running (or using the in-process test runner), run:

```bash
cd backend
uv run python test_studio_pipeline.py
```

This tests:
1. **Director + Architect + Managed Agent Artifact Pull** (`POST /api/commission-level`)
2. **MuJoCo 4-Wheel Rover Step + Center-of-Mass & $\tau = r \times F$ Torque Gate** (`POST /api/physics-step`)
3. **Multimodal Visual QA Audit & Live Physics Hotfix** (`POST /api/qa-audit`)
4. **Endgame Manufacturing Bundle Export (STLs + URDF + BOM.csv + MicroPython)** (`POST /api/export-build`)

### Option B: Test Individual Endpoints with `curl`

```bash
# 1. Commission a level + generate vehicle skin + MuJoCo XML + URDF
curl -s -X POST http://127.0.0.1:8000/api/commission-level \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Design a slag-heap rover track with a low clearance gap and high-torque wheel upgrade.", "project_id": "job_test"}' | python3 -m json.tool

# 2. Step the MuJoCo Rover Physics & check CoM / Servo Torque Gate
curl -s -X POST http://127.0.0.1:8000/api/physics-step \
  -H "Content-Type: application/json" \
  -d '{"project_id": "job_test", "throttle": 1.0, "steering": 0.2, "equip_upgrade": true}' | python3 -m json.tool

# 3. Trigger Visual QA Audit & Live Hotfix
curl -s -X POST http://127.0.0.1:8000/api/qa-audit \
  -F "project_id=job_test" | python3 -m json.tool

# 4. Export Printable STLs, URDF, BOM.csv, and MicroPython Firmware
curl -s -X POST http://127.0.0.1:8000/api/export-build \
  -H "Content-Type: application/json" \
  -d '{"project_id": "job_test"}' | python3 -m json.tool
```

---

## 3. Architecture & File Ownership (`backend/`)

```text
backend/
├── main.py                      # FastAPI Gateway & Static Asset Server (/assets/generated/...)
├── mujoco_sim.py                # Real-time MuJoCo MJCF stepper + CoM & Torque (tau = r x F) gate
├── test_studio_pipeline.py      # Standalone end-to-end test script (no Rust client required)
├── agents/
│   ├── director.py              # StudioDirectorAgent: scopes brief, hazards, car_physics & reward_part
│   ├── architect.py             # LevelArchitectAgent: builds course & triggers sandbox pull + skin gen
│   ├── qa_tester.py             # VisualQATesterAgent: audits viewport PNG + patches entities & MuJoCo
│   ├── artifact_puller.py       # Downloads Managed Agent /workspace .tar, generates Nano Banana skins & STLs
│   └── mujoco_builder.py        # Generates MuJoCo MJCF (car_level.xml) for the 4-wheel rover
├── assets/
│   ├── asset_manifest.json      # Base asset registry
│   └── generated/{project_id}/  # Returned artifacts: chassis_skin.png, car_level.xml, rover.urdf, *.stl
└── exports/{project_id}/        # Endgame manufacturing bundle: stls/*.stl, rover.urdf, BOM.csv, firmware/main.py
```

---

## 4. How Managed Agents Return Files to Local Disk

Per the Gemini 4 Argon Builder Guide and Gretchen's architecture notes:
1. **Managed Agent Sandbox (`antigravity-preview-09-2026`):**
   ```python
   interaction = client.interactions.create(
       agent="antigravity-preview-09-2026",
       agent_config={"type": "antigravity", "model": "gemini-4-argon"},
       input=prompt,
       environment="remote",
   )
   ```
2. **Pulling `/workspace` Artifacts (`agents/artifact_puller.py`):**
   Uses `interaction.environment_id` to download the `.tar` snapshot from:
   `GET https://generativelanguage.googleapis.com/v1beta/files/environment-{env_id}:download?alt=media`
   and extracts generated `.xml`, `.urdf`, `.stl`, and `.png` files into `backend/assets/generated/{project_id}/`.
3. **Vehicle Skins (`gemini-nano-banana-2.1`):**
   Generates `chassis_skin.png` and serves it at `http://127.0.0.1:8000/assets/generated/{project_id}/chassis_skin.png` so Jorge's Rust client downloads and applies it directly to the 3D rover mesh.
