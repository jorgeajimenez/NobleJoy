# Jorge — Native Rust 3D Driving & Rover Client (`game_client/`)

This directory contains **Jorge's** half of the monorepo: the **Rust + Bevy 0.14 3D Desktop Engine** (hardware-accelerated on Apple Silicon via `wgpu` Metal).

You can preview, drive, and iterate on `game_client/` **immediately and independently** — even when Gretchen's Python backend is not running!

---

## 1. Quick Start & Preview

```bash
cd game_client
cargo run
```

> **Tip for Fast Iterative Builds:** Need faster incremental compiles while tweaking visuals or controls? Run `cargo run` (debug mode) or connect to a remote backend on LAN via:
> ```bash
> STUDIO_BACKEND_URL=http://127.0.0.1:8000 cargo run
> ```

---

## 2. Standalone Offline Mode vs. Connected Studio Mode

- **Standalone Offline Preview Mode (Automatic):**
  - If Gretchen's backend (`http://127.0.0.1:8000`) is not running when you launch `cargo run`, the Rust client **automatically spawns a built-in Local Preview Driving Track** with the 4-wheel articulated rover, 2-DOF actuator arm, blast door obstacle, and reactor goal beacon so you can test driving, collisions, and camera/HUD immediately.
  - You can also press **`[L]`** at any time to load/reset the Local Preview Track, or press **`[Q]`** in offline mode to preview a simulated hotfix reposition!
- **Connected Studio Mode (With `backend/` running):**
  - When Gretchen's FastAPI + MuJoCo server is running on port `8000`, pressing **`[C]`** commissions a live level from the Gemini 4 Argon Managed Agents, downloads the generated `chassis_skin.png` texture into `game_client/assets/generated/`, binds it to the 3D rover's PBR material, and streams driving controls through `/api/physics-step`.

---

## 3. Keyboard Controls

| Key | Action |
| :--- | :--- |
| **`W` / `S`** or **`Up` / `Down`** | Throttle forward / Reverse rover |
| **`A` / `D`** or **`Left` / `Right`** | Steer rover left / right |
| **`C`** | **Commission Studio Level:** Request new level + skin + MuJoCo MJCF from Gretchen's backend |
| **`Q`** | **Visual QA Audit & Hotfix:** Capture viewport PNG (`viewport_capture.png`) & reposition blockers |
| **`U`** | **Snap M3 Upgrade (`high_torque_wheel_hub_v2`):** Toggle modular part & run CoM / $\tau = r \times F$ torque gate |
| **`E`** | **Export Manufacturing Bundle:** Generate M3 STLs + `rover.urdf` + `BOM.csv` + MicroPython `main.py` |
| **`L`** | **Local Offline Preview:** Spawn/reset built-in test track immediately without backend |
| **`R`** | **Reset Rover:** Return rover to `spawn_pad` `(0, 0.35, 0)` |

---

## 4. Architecture & File Ownership (`game_client/`)

```text
game_client/
├── Cargo.toml               # Bevy 0.14, serde, reqwest, crossbeam-channel, image
├── assets/
│   ├── models/              # Optional custom .glb meshes (player.glb, obstacle.glb, goal.glb)
│   └── generated/           # Auto-downloaded vehicle skins ({project_id}_skin.png) from Gretchen's studio
└── src/
    ├── main.rs              # Bevy App entry point, window setup & system scheduling
    ├── components.rs        # ECS components (StudioEntity, EntityRole) & GameSessionState
    ├── scene_parser.rs      # Serde JSON data contracts shared with Gretchen's backend
    ├── network.rs           # Non-blocking background HTTP threads & crossbeam channel events
    ├── level_builder.rs     # Spawns 4-wheel articulated rover + PBR skin texture & applies hotfix deltas
    ├── capture.rs           # Viewport frame buffer screenshot capture to PNG
    └── systems.rs           # Driving controls, offline fallback, HUD telemetry & collision checks
```
