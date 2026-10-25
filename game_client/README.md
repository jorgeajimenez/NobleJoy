# Jorge — Native Rust 3D Generative Driving & Vehicle Studio (`game_client/`)

This directory contains **Jorge's** half of the monorepo: the **Rust + Bevy 0.14 3D Desktop Engine** + **WGPU Compute G-Buffer Artifact Validator (`artifact_detector.wgsl`)**.

---

## 1. Quick Start & Full-Stack Orchestration

From the repo root (recommended — launches both Gretchen's Python backend + Jorge's Rust client with `.env`):
```bash
./run_local.sh
```

Or run `game_client` standalone:
```bash
cd game_client
cargo run
```

---

## 2. Welcome Studio Screen, AI Game Generator & 3D Car Designer

- **Welcome & AI World Generator Menu (`[Tab]` or `[Esc]` to open/close anytime):**
  - **Talk Directly to Gretchen's Agent:** Type any natural-language game prompt directly in the menu (e.g., *"Medieval cobblestone castle streets with dragon gate and royal war wagon"*, *"Neon cyberpunk downtown avenue"*, *"Mars desert canyon convoy"*, *"Arctic glacier outpost"*) and press **`[Enter]`** to generate a complete game plot, themed 3D street world, roadside architecture, and custom vehicle from scratch!
  - **1-Key World Theme Presets (work in menu OR while driving):**
    - **`[F1]` Medieval Cobblestone Kingdom:** Mossy terrain, cobblestone road, limestone castle curbs, stone castle keeps with battlements, timber taverns, flaming torches, merchant cart chicanes, Iron Portcullis blockade, and Royal Siege Wagon.
    - **`[F2]` Neon Cyber-City 2099:** Wet asphalt avenue, yellow center dashes, cyber-skyscrapers with neon window bands, sodium streetlamps, and plasma blast door.
    - **`[F3]` Desert Wasteland Canyon:** Red-ochre desert sand, dirt canyon trail, sandstone mesas, flare posts, and lifted 4WD Armored Crawler.
    - **`[F4]` Arctic Glacier Outpost:** Snowfield terrain, blue-ice causeway, geodesic cryo-domes, plasma pylons, and Thermal Interceptor.
- **Interactive 3D Car Designer Showroom (`[V]` to toggle):**
  - Smoothly orbits the camera 360° around your car while you customize:
    - **`[1]` Chassis Body Style:** `street_muscle` / `medieval_wagon` / `armored_crawler` / `cyber_interceptor`
    - **`[2]` Paint & Trim Palette:** Cyber Cyan / Medieval Oak & Royal Gold / Crimson GT / Stealth Obsidian / Wasteland Copper
    - **`[3]` Wheel Size & Suspension Lift:** Standard `1.0x` / Monster Crawler `1.32x` (+lift) / Lowrider Drift `0.86x`
    - **`[4]` Hardware Attachment:** `M3 Servo Arm` / `Iron Battering Ram` / `Twin Turbo Supercharger`
    - **`[5]` AI Texture Skin (`gemini-nano-banana-2.1`):** Requests a fresh PBR texture skin from Gretchen's backend and wraps it onto the car.

---

## 3. WGPU Compute Artifact Detector (`gbuffer_detector.rs` + `artifact_detector.wgsl`)

Pressing **`[Q]`** runs a two-stage verification loop:
1. **Apple/AAA-Tier WGPU Compute Pass (`artifact_detector.wgsl`):** Inspects the scene's depth, normal, instance ID, and shadow buffers on the GPU to detect **Coplanar Z-Fighting Clusters (Type 1)** and **Shadow Acne Anomalies (Type 2)**.
2. **Live Scene Repair + Visual QA Hotfix:** Automatically removes coplanar Z-fighting geometry (`z_fighting_hazard_patch`), repositions the central street roadblock (`blast_door`) onto the sidewalk (`z = 6.5`), boosts MuJoCo servo torque (`11.0 kg·cm`), and streams the viewport capture to Gretchen's `/api/qa-audit` agent.

---

## 4. Controls Reference

| Key | Action |
| :--- | :--- |
| **`Tab` / `Esc`** | Toggle **Welcome & AI Game Generator Prompt Menu** |
| **`Type + Enter`** | (In Menu) Send custom game/world prompt to Gretchen's Agent & generate world |
| **`F1` / `F2` / `F3` / `F4`** | Instant World Generators: **Medieval** (`F1`), **Cyber-City** (`F2`), **Desert** (`F3`), **Arctic** (`F4`) |
| **`V`** | Toggle **3D Car Designer Showroom** (`1` Body, `2` Paint, `3` Wheels/Lift, `4` Attachment, `5` AI Skin) |
| **`W` / `S` / `A` / `D`** | Drive forward / reverse & steer along the street corridor (with solid wall/building collisions) |
| **`Q`** | **Run WGPU G-Buffer Artifact Detector + Visual QA Roadblock Hotfix** |
| **`N`** | **Next Campaign Level:** Advance to `Level N+1` with new plot chapter & escalating street hazards |
| **`U`** | **Snap M3 Upgrade (`high_torque_wheel_hub_v2`):** Toggle modular part & run CoM / $\tau = r \times F$ gate |
| **`E`** | **Export Manufacturing Bundle:** Generate M3 STLs + `rover.urdf` + `BOM.csv` + MicroPython `main.py` |
| **`R`** | **Reset Vehicle:** Return vehicle to the start of the street `(0, 0.35, 0)` |
