# Argon Game Studio — Live Demo & Walkthrough Guide (`DEMO_GUIDE.md`)

This guide walks you step-by-step through running, configuring, and demonstrating **Argon Game Studio** (**Jorge's Rust/Bevy 3D Engine** + **Gretchen's FastAPI + MuJoCo + Gemini Studio Backend**).

It is designed around the core live-demo flow:
1. **Start driving in a Neon Cyberpunk City (`Neon Overdrive 2099`)** with a GTA-style 3D chase camera and solid street/building physics.
2. **Press `[Tab]` while driving and prompt `"take me to medieval times"`** — in a moment, the **entire scene, music (`theme_music.wav`), street cobblestones, castle keeps, torches, and vehicle skin transform around you** while your exact street position, velocity, and MuJoCo physics stay continuous.
3. **Prompt for live upgrades or skins** (e.g., `"give me a royal gold dragon skin with monster wheels and a battering ram"`) or open the **`[V]` 3D Car & Robot Designer Showroom**.
4. **Press `[T]` (or prompt `"transform into a bipedal mech"`) for Transformers-style MuJoCo Robot Morphing** — switching on the fly between a **4WD Rover Car**, an **Upright Bipedal Transformer Mech**, a **4-Legged Spider Quadruped**, and a **Tracked Siege Tank** while keeping physics and street pose intact.

---

## 1. Quick Start (One Command Local Demo)

From the repository root (`/Users/jorgeajimenez/hackathons/cv/argon4`):

```bash
cp .env_sample .env
./run_local.sh
```

What `./run_local.sh` does automatically:
1. Loads `.env` and prints the active `STUDIO_EXECUTION_MODE` and `STUDIO_BACKEND_URL`.
2. Creates/updates `backend/.venv` via `uv` and starts the FastAPI + MuJoCo backend on `http://127.0.0.1:8000`.
3. Waits for `GET /health` to pass, then launches the Rust/Bevy 3D game client (`game_client`) connected to the FastAPI backend.

---

## 2. Configurable Cloud vs. Local Switches (`.env` / `.env_sample`)

Because cloud Managed Agent sandboxes or specific preview model endpoints may or may not be reachable during a live demo, **every execution mode, model ID, and URI template is configurable via environment variables** in [`.env_sample`](file:///Users/jorgeajimenez/hackathons/cv/argon4/.env_sample) (copied to `.env`).

### A. Master Execution Mode Switch (`STUDIO_EXECUTION_MODE`)

| `STUDIO_EXECUTION_MODE` | When to Use | What Runs Under the Hood |
| :--- | :--- | :--- |
| **`local_fast`** | **Zero-latency offline / stage demo** (no cloud dependency required) | Uses FastAPI + native **MuJoCo 3.x** physics + instant procedural **PBR PNG skin synthesis** + **loopable WAV soundtrack synthesis** (`theme_music.wav`) + watertight STL/URDF/MicroPython generation. |
| **`gemini_direct`** | **Standard Gemini API key available**, no Managed Agent sandbox required | Calls `client.models.generate_content` (`GEMINI_MODEL_ID`) for world plot, level JSON, and visual QA + `SKIN_MODEL_ID` for vehicle skin textures, while running MuJoCo & CAD synthesis locally in FastAPI. |
| **`managed_agent`** | **Full Cloud Managed Agent Sandbox available** | Launches a stateful `client.interactions.create(agent=MANAGED_AGENT_ID)` cloud container session and pulls sandbox files via `SANDBOX_DOWNLOAD_URI_TEMPLATE`. Automatically falls back if the endpoint is unreachable. |
| **`auto`** *(default)* | **Automatic cascade** | Tries `managed_agent` (if `USE_MANAGED_AGENT=true`) -> `gemini_direct` -> `local_fast` without ever crashing or blocking the game client. |

### B. Configurable Endpoints, Model IDs, and URIs

Edit `.env` at any time (no code changes needed):

```dotenv
# 1. Execution Mode ("auto" | "local_fast" | "gemini_direct" | "managed_agent")
STUDIO_EXECUTION_MODE=auto
USE_MANAGED_AGENT=true

# 2. Configurable Model & Agent Identifiers
GEMINI_MODEL_ID=gemini-4-argon
MANAGED_AGENT_ID=antigravity-preview-09-2026
MANAGED_AGENT_TYPE=antigravity
SKIN_MODEL_ID=gemini-nano-banana-2.1

# 3. Configurable Cloud / Sandbox Endpoints & URI Templates
# Leave GEMINI_API_BASE_URL blank for default Google GenAI endpoint, or set a custom proxy/vertex URL
GEMINI_API_BASE_URL=
SANDBOX_DOWNLOAD_URI_TEMPLATE=https://generativelanguage.googleapis.com/v1beta/files/{session_id}/content?path={remote_path}

# 4. FastAPI Studio Backend URL (Switch between Localhost and GCP Cloud Run!)
STUDIO_PORT=8000
STUDIO_BACKEND_URL=http://127.0.0.1:8000
# Example when deployed to Google Cloud Run:
# STUDIO_BACKEND_URL=https://argon-studio-backend-uc.a.run.app
```

> **Tip:** You can inspect the active mode and endpoints live at any time by running:
> ```bash
> curl -s http://127.0.0.1:8000/health | jq .
> ```

---

## 3. Step-by-Step Live Demo Script

### Act I — Driving in Cyberpunk City (`Neon Overdrive 2099`)
1. When the Bevy window opens, you start **immediately on Main Cyber Avenue** in **Neon Cyberpunk 2099**:
   - Rain-slicked dark asphalt with glowing yellow center lane dashes and streetlamps.
   - Towering cyan-lit cyberpunk skyscrapers lining both sides of the avenue.
   - Solid curb boundaries, orange chicane barriers, and a glowing red plasma blast door roadblock ahead (`x = 19.0`).
2. Hold **`[W]`** to accelerate and **`[A]` / `[D]`** to steer partway down the street (e.g., weave past the first chicane barrier around `x = 12.0`).
   - Notice the **GTA-style 3D chase camera** smoothly tracking behind and above your vehicle, and notice that **solid AABB collision + wall-sliding** prevents clipping through any building or barrier.

---

### Act II — The Dream Scene Swap: `"take me to medieval times"` (In-Place!)
1. While parked or driving down the cyberpunk street, press **`[Tab]`** to open the **Live In-Game Scene, Skin & Robot Prompt Bar**.
2. Notice the prompt input is **already pre-filled with**:
   ```text
   > take me to medieval times_
   ```
   *(You can also type any custom prompt here, or press `[Backspace]` to edit it.)*
3. Press **`[Enter]`**.
4. **Watch what happens immediately & when the FastAPI response arrives:**
   - **Your vehicle stays at the exact same `(x, z)` street position, yaw heading, and velocity — physics never resets!**
   - **The Street & Scenery Morph Around You:**
     - Wet neon asphalt and yellow lane dashes turn into **rough medieval cobblestones** and mossy green terrain.
     - Cyberpunk skyscrapers transform into **stone castle keeps and timber taverns** with warm amber windows and battlements.
     - Modern streetlamps become **flickering medieval roadside torches**.
     - The plasma blast door becomes an **Iron Portcullis Gate** (`iron_portcullis_gate_blockade`).
   - **The Music Changes (`theme_music.wav`):**
     - The backend synthesizes and streams `/assets/{project_id}/theme_music.wav` (`medieval_lute_march` in D-Dorian fifths), and Bevy loops the new soundtrack automatically.
   - **The Vehicle Skin & Chassis Update:**
     - Your car updates to the **Royal Siege Wagon** (`medieval_wagon`) wrapped in the newly generated medieval oak-and-gold PBR skin (`/assets/{project_id}/rover_skin.png`).

*(You can also trigger instant in-place world morphs at any time with **`[F1]` Medieval**, **`[F2]` Cyberpunk**, **`[F3]` Desert Mars Canyon**, and **`[F4]` Arctic Glacier**!)*

---

### Act III — Live Prompting for Upgrades, Skins & The 3D Showroom (`[V]`)
While driving in Medieval Times (or any world), you can prompt the FastAPI backend for live upgrades and skins:
1. Press **`[Tab]`**, type a prompt such as:
   ```text
   give me a royal gold dragon skin with monster wheels and a battering ram
   ```
   and press **`[Enter]`**.
   - The FastAPI Director + Artifact Puller parses your upgrade keywords (`gold`, `monster`, `ram`), generates a matching PBR skin texture, updates the vehicle's wheel scale (`1.35x`), ride height (`+0.09m`), and front **Iron Battering Ram**, and hot-reloads the car in-place!
2. Or press **`[V]`** while driving to enter the **360° Interactive 3D Car & Robot Designer Showroom**:
   - **`[1]`** Cycle Chassis Body (`street_muscle` / `medieval_wagon` / `armored_crawler` / `cyber_interceptor`)
   - **`[2]`** Cycle Paint & Trim Palette (Cyber Cyan / Medieval Oak & Gold / Crimson GT / Stealth Obsidian / Wasteland Copper)
   - **`[3]`** Cycle Wheel Size & Suspension Lift (Standard `1.0x` / Monster Crawler `1.32x` / Lowrider Drift `0.86x`)
   - **`[4]`** Cycle Roof/Front Hardware (`m3_servo_arm` / `battering_ram` / `twin_turbo`)
   - **`[5]`** Request a fresh AI PBR Skin from FastAPI (`SKIN_MODEL_ID`)
   - **`[6]` or `[T]`** Transform MuJoCo Robot Morphology (see Act IV below!)
   - Press **`[Enter]`** or **`[V]`** to drop right back onto the street.

---

### Act IV — Transformers-Style MuJoCo Robot Morphing (`[T]` or Prompt)
Once you have switched scenery, you can morph the **MuJoCo robot morphology itself** while preserving position, velocity, and physics!

1. **Method A — Instant Hotkey (`[T]`):**
   - While driving (or in the `[V]` showroom), press **`[T]`**.
   - Every press of **`[T]`** cycles your vehicle + backend MuJoCo MJCF model through 4 articulated robot morphologies at your exact street coordinates:
     1. **`rover_car`** — 4-Wheel Ackermann Rover / Siege Wagon (`rover.xml`)
     2. **`bipedal_mech`** — **Upright Humanoid Transformer Mech** with articulated left/right legs, knee joints, armored torso, shoulder pauldrons, dual arm cannons, and glowing cyan head visor
     3. **`quadruped_crawler`** — **4-Legged Articulated Spider/Hound Crawler Robot** with splay-hip/knee legs and sensor head pod
     4. **`siege_tank`** — **Heavy Armored Tracked Siege Tank** with continuous side treads, rotating turret, and dual cannons
2. **Method B — Via Natural Language Prompt (`[Tab]`):**
   - Press **`[Tab]`**, type:
     ```text
     transform into a bipedal mech in medieval times
     ```
     (or `"spider quadruped crawler"` / `"siege tank"`), and press **`[Enter]`**.
   - Both the FastAPI MuJoCo engine (`POST /mujoco_step` / `POST /commission_level`) and the Rust 3D viewport morph to the requested robot morphology without losing your place on the street.

---

### Act V — WGPU Compute G-Buffer Artifact Detector + Visual QA Playtester (`[Q]`)
1. Walk or drive up toward `x = 16.0 – 19.0` where:
   - A glowing magenta coplanar **Z-fighting artifact patch** flickers on the road surface (`x = 16.2`), and
   - The **Iron Portcullis / Blast Door (`blast_door`)** blocks the center lane (`x = 19.0, z = 0.0`).
2. Press **`[Q]`**:
   - Runs the WGPU compute shader audit ([`artifact_detector.wgsl`](file:///Users/jorgeajimenez/hackathons/cv/argon4/game_client/assets/shaders/artifact_detector.wgsl) + [`gbuffer_detector.rs`](file:///Users/jorgeajimenez/hackathons/cv/argon4/game_client/src/gbuffer_detector.rs)) inspecting normal discontinuities and depth buffer variance.
   - Captures a viewport PNG (`audit_snap.png`) and sends it to `POST /qa_audit`.
   - **Auto-Repairs the Scene Live:** Despawns the Z-fighting geometry patch, shifts `blast_door` off the road onto the sidewalk (`z = 6.5`), and boosts MuJoCo drive torque so you can drive/walk straight through to the glowing green Extraction Beacon!

---

### Act VI — Snap M3 Hardware (`[U]`), Next Campaign Level (`[N]`), & Physical Export (`[E]`)
- **`[U]` — Snap Modular M3 Hardware Upgrade:** Toggles the `high_torque_wheel_hub_v2` (`MG996R_servo`, `11.0 kg·cm` rated torque > `8.8 kg·cm` required torque) in MuJoCo so the HUD flips from `[STALL]` to `[PASS]`.
- **`[N]` — Advance to Next Campaign Level:** Generates Level 2, Level 3, etc. with an extended corridor, additional chicanes, and the next story chapter.
- **`[E]` — Export Physical Build Package:** Calls `POST /export_build` to write a complete fabrication bundle to `backend/exports/{project_id}_build_package/`:
  - Watertight M3-calibrated STL meshes (`chassis_body.stl`, `high_torque_wheel_hub_v2.stl`)
  - `rover.xml` (MuJoCo MJCF for the active robot form) & `rover.urdf`
  - `BOM.csv` (M3x12mm bolts, MG996R servos, ESP32-S3, 2S LiPo)
  - `main.py` (MicroPython PWM firmware)

---

## 4. Complete Controls Reference

| Key | Mode | Action |
| :--- | :--- | :--- |
| **`W` / `S`** or **`Up` / `Down`** | Driving | Throttle Forward / Reverse (synced with MuJoCo `POST /mujoco_step`) |
| **`A` / `D`** or **`Left` / `Right`** | Driving | Steer Left / Right |
| **`Tab`** or **`Esc`** | Any | Toggle **Live In-Game AI Scene, Skin & Robot Prompt Bar** (pre-filled with `"take me to medieval times"`) |
| **`Enter`** | Prompt Bar | Send prompt to FastAPI Studio & morph scene/music/skin/robot **in-place** |
| **`T`** | Driving / Showroom | **Transformers Robot Morph** (`rover_car` -> `bipedal_mech` -> `quadruped_crawler` -> `siege_tank`) |
| **`V`** | Driving / Showroom | Toggle **360° 3D Car & Robot Designer Showroom** (`[1]`-`[6]` to customize) |
| **`F1` / `F2` / `F3` / `F4`** | Any | Instant in-place world swap: **Medieval** / **Cyberpunk** / **Desert Canyon** / **Arctic Glacier** |
| **`Q`** | Driving | Run **WGPU Compute G-Buffer Artifact Audit** + **Visual QA Roadblock Hotfix** |
| **`U`** | Driving | Snap / Unsnap **M3 Modular Hardware Upgrade** (`11.0 kg·cm` servo hub) |
| **`N`** | Driving | Generate **Next Campaign Level** (`Level + 1`) |
| **`E`** | Driving | Export **Physical Build Bundle** (STLs + URDF + BOM.csv + MicroPython) |
| **`R`** | Driving | Reset vehicle/robot to street start (`x = 0, z = 0`) |
