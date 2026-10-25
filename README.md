# Argon Game Studio (NobleJoy) — Autonomous AI Game Studio, MuJoCo Rover Physics & Native Rust Engine

**Target:** Gemini 4 Argon SF Hackathon  
**Track:** Track 2 — More Dimensions  
**Prize Target:** Best Use of Managed Agents  

---

## Team Work Split & Independent Project Guides

We have structured the monorepo so **Gretchen** (`backend/`) and **Jorge** (`game_client/`) can develop and test completely independently or connected together:

1. **[`backend/README.md`](./backend/README.md) — Gretchen's Autonomous Studio & MuJoCo Physics Service**
   - Gemini 4 Argon Managed Agents (`antigravity-preview-09-2026`), `gemini-nano-banana-2.1` vehicle skin generator, `/workspace` `.tar` artifact puller, MuJoCo 4-wheel rover simulation (`mujoco_sim.py`), and physical manufacturing exporter (M3 STLs + `rover.urdf` + `BOM.csv` + MicroPython firmware).
   - Includes standalone test script (`uv run python test_studio_pipeline.py`) and `curl` commands so Gretchen can test everything without compiling Rust.
2. **[`game_client/README.md`](./game_client/README.md) — Jorge's Native Rust / Bevy 0.14 3D Driving Engine**
   - Hardware-accelerated 3D rendering on Apple Silicon (`wgpu` Metal), 4-wheel articulated rover + 2-DOF actuator arm, live PBR skin loader, viewport PNG screenshot capture, and HUD telemetry.
   - Includes **Standalone Offline Preview Mode** so Jorge can run `cargo run` and drive/test immediately even when the Python backend is offline.

---

## Architecture Overview

```
                        GRETCHEN: THE OUTSOURCED STUDIO (BACKEND)
       ┌────────────────────────────────────────────────────────────────────────┐
       │                                                                        │
Client │   [Studio Director Agent (gemini-4-argon)]                             │
Brief ─┼──► (Scopes track, hazards, MuJoCo physics & M3 reward part)            │
       │              │                                                         │
       │              ▼                                                         │
       │   [Level Architect + Managed Agent (antigravity-preview-09-2026)]      │
       │   (Builds MJCF/URDF/STLs in /workspace + Nano Banana 2.1 Skin PNG)     │
       │              │                        │                                │
       │              ▼                        ▼                                │
       │      [FastAPI Gateway + MuJoCo Physics Server (mujoco_sim.py)]         │
       │              ▲                        │                                │
       │              │                        │                                │
       │   [Visual QA Tester Agent]            │                                │
       │   (Reviews viewport PNG + telemetry,  │                                │
       │    patches obstacles & MuJoCo torque) │                                │
       └──────────────▲────────────────────────┼────────────────────────────────┘
                      │                        │
       Viewport Image │                        │ Level Contract JSON +
         (PNG stream) │                        │ Skin PNG + MuJoCo Poses
                      │                        ▼
       ┌──────────────┴─────────────────────────────────────────────────────────┐
       │                                                                        │
       │                    JORGE: CLIENT ENGINE (RUST / BEVY)                  │
       │                                                                        │
       │  [Network / Poller] ──► [Scene Parser] ──► [4-Wheel Rover ECS Spawner] │
       │                                                  │                     │
       │  [Frame Capture Subsystem] ◄────────────── [Render / Driving Loop]     │
       │                                                                        │
       └────────────────────────────────────────────────────────────────────────┘
```

---

## Reference Docs
- [`.reference/CONCEPT_FOR_HACKATHON.md`](./.reference/CONCEPT_FOR_HACKATHON.md)
- [`.reference/Gretchen_Brainstorming_Car_Conversation.pdf`](./.reference/Gretchen_Brainstorming_Car_Conversation.pdf)
- [`.reference/[External] Gemini 4 Argon_ Hackathon Builder Guide.pdf`](./.reference/%5BExternal%5D%20Gemini%204%20Argon_%20Hackathon%20Builder%20Guide.pdf)
- [`SPEC.md`](./SPEC.md) & [`INSTALL_GUIDE.md`](./INSTALL_GUIDE.md)
