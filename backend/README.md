# Gretchen — Autonomous AI Game Studio & MuJoCo Backend (`backend/`)

This directory contains **Gretchen's** half of the monorepo:
- **Gemini 4 Argon Managed Agents Studio Backend** (`antigravity-preview-09-2026` + `gemini-4-argon` + `gemini-nano-banana-2.1`)
- **Built-in Interactive Studio Web Frontend** (served directly at **`GET /`**)
- **MuJoCo Driving & Robotics Physics Server** (`mujoco_sim.py`)
- **Sandbox Artifact Puller & Physical Export Pipeline** (`agents/artifact_puller.py`)
- **Cloud Run Production Container** (`Dockerfile`)

You can develop, run, test, and deploy `backend/` to **Google Cloud Run** **100% independently** of the Rust game client.

---

## 1. Local Quick Start (Using `uv`)

```bash
cd backend
cp .env.example .env
# Edit .env and set GEMINI_API_KEY=your_key_here

# Create virtualenv and install dependencies
uv venv
uv pip install -r requirements.txt

# Start the FastAPI server + Web Dashboard with hot-reload
uv run uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

### Built-In Web Frontend & API Docs
- **Interactive Studio Control Dashboard (Web Frontend):** Open **`http://127.0.0.1:8000/`** in your browser to commission levels, preview generated vehicle skins (`chassis_skin.png`), download `car_level.xml` (MuJoCo MJCF) / `rover.urdf` / `.stl` meshes, step the MuJoCo torque gate, upload viewport PNGs for Visual QA, and export manufacturing bundles.
- **Swagger UI API Docs:** Open **`http://127.0.0.1:8000/docs`**.

---

## 2. Explicit Google Cloud Run Deployment Instructions (GCP)

`backend/` includes a production [Dockerfile](./Dockerfile) pre-configured with headless OSMesa OpenGL (`MUJOCO_GL=osmesa`) so MuJoCo, Pillow, and FastAPI run out-of-the-box on **Google Cloud Run**.

### Step 1: Authenticate & Configure Your GCP Project

```bash
# Log in to Google Cloud
gcloud auth login

# Set your GCP Project ID (replace with your hackathon GCP project ID)
export PROJECT_ID="your-gcp-project-id"
export REGION="us-central1"
gcloud config set project "$PROJECT_ID"

# Enable required Cloud Run, Cloud Build, and Artifact Registry APIs
gcloud services enable \
  run.googleapis.com \
  cloudbuild.googleapis.com \
  artifactregistry.googleapis.com
```

### Step 2: Deploy `backend/` Directly to Cloud Run

From inside the `backend/` directory, run:

```bash
cd backend

# Set your Gemini API key from your .env
export GEMINI_API_KEY="your_gemini_api_key_here"

# Build & deploy to Cloud Run in one command
gcloud run deploy argon-studio-backend \
  --source . \
  --region "$REGION" \
  --platform managed \
  --allow-unauthenticated \
  --memory 2Gi \
  --cpu 2 \
  --timeout 300 \
  --set-env-vars "GEMINI_API_KEY=${GEMINI_API_KEY},USE_MANAGED_AGENT=true"
```

### Step 3: Verify Your Cloud Run Deployment & Connect the Rust Client

Once `gcloud run deploy` completes, it prints a Service URL like `https://argon-studio-backend-xxxxxx-uc.a.run.app`:

1. **Open the Web Frontend in Browser:**
   Visit `https://argon-studio-backend-xxxxxx-uc.a.run.app/` to use the live **Gretchen Studio Control Dashboard** directly on Cloud Run!
2. **Health Check via CLI:**
   ```bash
   curl https://argon-studio-backend-xxxxxx-uc.a.run.app/health
   ```
3. **Connect Jorge's Rust Client to Cloud Run:**
   Share the Cloud Run URL with Jorge so the Rust desktop engine connects straight to your GCP backend:
   ```bash
   cd game_client
   STUDIO_BACKEND_URL=https://argon-studio-backend-xxxxxx-uc.a.run.app cargo run
   ```

---

## 3. Independent Testing (Without the Rust Game Client)

### Option A: Use the Web Dashboard (`GET /`)
Open `http://127.0.0.1:8000/` (local) or your Cloud Run URL in Chrome to test all 4 stages visually with one-click buttons.

### Option B: Run the Automated CLI Test Script
```bash
cd backend
uv run python test_studio_pipeline.py
```

### Option C: Test Endpoints with `curl`
```bash
# Set BASE_URL to local or your deployed Cloud Run URL
export BASE_URL="http://127.0.0.1:8000"

# 1. Commission a level + generate vehicle skin + MuJoCo XML + URDF
curl -s -X POST "$BASE_URL/api/commission-level" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Design a slag-heap rover track with a low clearance gap and high-torque wheel upgrade.", "project_id": "job_test"}' | python3 -m json.tool

# 2. Step the MuJoCo Rover Physics & check CoM / Servo Torque Gate
curl -s -X POST "$BASE_URL/api/physics-step" \
  -H "Content-Type: application/json" \
  -d '{"project_id": "job_test", "throttle": 1.0, "steering": 0.2, "equip_upgrade": true}' | python3 -m json.tool

# 3. Trigger Visual QA Audit & Live Hotfix
curl -s -X POST "$BASE_URL/api/qa-audit" \
  -F "project_id=job_test" | python3 -m json.tool

# 4. Export Printable STLs, URDF, BOM.csv, and MicroPython Firmware
curl -s -X POST "$BASE_URL/api/export-build" \
  -H "Content-Type: application/json" \
  -d '{"project_id": "job_test"}' | python3 -m json.tool
```

---

## 4. Architecture & File Ownership (`backend/`)

```text
backend/
├── Dockerfile                   # Cloud Run container (Python 3.11 + OSMesa headless MuJoCo)
├── .dockerignore
├── main.py                      # FastAPI Gateway + Web Dashboard (GET /) + Static Asset Server
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
