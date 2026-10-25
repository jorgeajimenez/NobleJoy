# Technical Specification: Argon Game Studio (MuJoCo Driving & Modular Rover Edition)

## 1. Unified Architecture (Incorporating Gretchen's Brainstorming Notes)

1. **GRETCHEN (`backend/` — Autonomous Studio & MuJoCo Physics Server):**
   - **Studio Director Agent (`director.py`):** Uses `gemini-4-argon` / Managed Agent `antigravity-preview-09-2026` to scope the driving track theme, environmental hazard (`low_clearance_gap_40cm`, `blast_door_blockade`), MuJoCo vehicle parameters (`chassis_mass_kg`, `wheel_friction`, `motor_torque_nm`), and the unlockable COTS modular reward part (`high_torque_wheel_hub_v2` with `M3_faceplate_40mm` mount).
   - **Level Architect & Artifact Puller (`architect.py`, `artifact_puller.py`, `mujoco_builder.py`):**
     - Invokes the Managed Agent sandbox (`/workspace`) and pulls generated files to the local machine via the **Environments / Files API**:
       `GET https://generativelanguage.googleapis.com/v1beta/files/environment-{env_id}:download?alt=media`
     - Generates the rover's custom UV/albedo skin texture (`chassis_skin.png`) using `gemini-nano-banana-2.1` and serves it over HTTP (`/assets/generated/{project_id}/chassis_skin.png`) to the Rust engine.
     - Emits the MuJoCo XML model (`car_level.xml`), ROS/Gazebo URDF (`rover.urdf`), and M3-calibrated watertight STL mesh (`high_torque_wheel_hub_v2.stl`).
   - **MuJoCo Physics & Verification Gate (`mujoco_sim.py`):**
     - Steps the 4-wheel rover (`car_chassis`, `wheel_fl`, `wheel_fr`, `wheel_rl`, `wheel_rr`) in MuJoCo (`POST /api/physics-step`).
     - Runs Gretchen's **Physics Sanity Check**:
       - **Center-of-Mass (CoM) Equilibrium:** Verifies the combined CoM stays inside the wheelbase polygon when modular arms/parts are mounted.
       - **Torque Validation ($\tau = r \times F$):** Compares lever arm load against rated servo torque (`SG90` = $1.8\text{ kg}\cdot\text{cm}$ vs `MG996R` upgrade = $11.0\text{ kg}\cdot\text{cm}$).
   - **Visual QA Playtester (`qa_tester.py`):**
     - Multimodal verification loop (`POST /api/qa-audit`) inspecting raw PNG viewport frames from the Rust engine + MuJoCo telemetry, issuing both **spatial transform deltas** (moving `blast_door`) and **live MuJoCo physics hotfixes** (boosting `motor_torque_nm` and `wheel_friction`).
   - **Endgame Physical Export Pipeline (`POST /api/export-build`):**
     - Writes a ready-to-manufacture bundle to `backend/exports/{project_id}/`:
       1. `stls/*.stl` (calibrated for 3.2mm M3 pass-through and 4.0mm brass heat-set inserts)
       2. `rover.urdf` (standard Unified Robot Description Format for ROS/Gazebo/Isaac Sim)
       3. `BOM.csv` (MG996R/SG90 servos, 608ZZ bearings, M3x12mm SHCS screws, ESP32-S3, 18650 sled)
       4. `firmware/main.py` (pre-calibrated MicroPython PWM + 2-DOF inverse kinematics script)

2. **JORGE (`game_client/` — Native Rust / Bevy 0.14 Desktop Engine):**
   - Renders the 3D driving course and articulated 4-wheel rover + 2-DOF modular arm on Apple Silicon (`wgpu` Metal backend).
   - Downloads the Managed Agent's generated `chassis_skin.png` texture over HTTP and binds it live to the rover's PBR `StandardMaterial`.
   - Streams driving controls (`WASD`) to the MuJoCo physics server and renders live MuJoCo CoM & $\tau = r \times F$ torque telemetry on the HUD.
