# Argon Game Studio — Autonomous AI Game Studio, MuJoCo Rover Physics & Native Rust Engine

**Target:** Gemini 4 Argon SF Hackathon  
**Track:** Track 2 — More Dimensions  
**Prize Target:** Best Use of Managed Agents  

---

## Overview

**Argon Game Studio** pairs a native **Rust / Bevy 0.14 3D desktop engine (`JORGE`)** with an autonomous **Gemini 4 Argon Managed Agents studio & MuJoCo physics backend (`GRETCHEN`)** to power a physics-aware **Modular Rover Driving Game**:

1. **Managed Agent Sandbox Artifact Generation & Local Pull (`antigravity-preview-09-2026` + `gemini-4-argon`):**
   - The Studio Director and Level Architect agents design the driving course, environmental hazards (`low_clearance_gap_40cm`, `blast_door`), and vehicle physics parameters inside their remote Linux microVM (`/workspace`).
   - Our backend pulls the generated artifacts from the Managed Agent via the **Environments / Files API** (`environment-{env_id}:download?alt=media`) and generates custom vehicle albedo skins (`chassis_skin.png`) via **`gemini-nano-banana-2.1`**.
   - Returned artifacts include:
     - **MuJoCo MJCF (`car_level.xml`)** & **ROS/Gazebo URDF (`rover.urdf`)**
     - **Vehicle Albedo Skins (`chassis_skin.png`)** dynamically downloaded and applied to the 3D rover mesh in Rust/Bevy
     - **Watertight M3-calibrated 3D Printable STLs (`*.stl`)**, **Hardware Bill of Materials (`BOM.csv`)**, and **MicroPython Firmware (`firmware/main.py`)**
2. **Real-Time MuJoCo Driving Physics & Robotics Verification Gate:**
   - The 4-wheel rover (`car_chassis`, `wheel_fl`, `wheel_fr`, `wheel_rl`, `wheel_rr`, plus 2-DOF actuator mount) is stepped via **MuJoCo** and verified against Gretchen's **Physics Sanity Check**:
     - **Center-of-Mass (CoM) Footprint Check:** Ensures extended modular arms/payloads do not tip the rover outside its wheelbase polygon.
     - **Servo Torque Gate ($\tau = r \times F$):** Validates lever-arm load against COTS servo ratings (`SG90` $1.8\text{ kg}\cdot\text{cm}$ vs `MG996R` $11.0\text{ kg}\cdot\text{cm}$).
3. **Closed Multimodal Visual QA + Physics Hotfix Loop (Track 2):**
   - The Rust client captures its live 3D viewport frame buffer to PNG and sends it to the **Visual QA Playtester Agent (`gemini-4-argon`)**.
   - The QA Agent visually detects navigation blockades (like `blast_door`) and physics bottlenecks, dispatching live **JSON transform deltas** and **MuJoCo physics hotfixes** (`motor_torque_nm`, `wheel_friction`) without reloading the game.

---

## Quick Start

See [INSTALL_GUIDE.md](./INSTALL_GUIDE.md) and [SPEC.md](./SPEC.md) for full setup and build instructions using `uv` and `cargo`.
