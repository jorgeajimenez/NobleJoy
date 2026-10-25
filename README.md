# Argon Game Studio — Autonomous AI Game Studio & Native Rust Engine

**Target:** Gemini 4 Argon SF Hackathon  
**Track:** Track 2 — More Dimensions  
**Prize Target:** Best Use of Managed Agents  

---

## Overview

Instead of hardcoding game logic or relying on blind procedural noise, **Argon Game Studio** treats **Gemini 4 Argon Managed Agents (`antigravity-preview-09-2026`)** as an **autonomous external game development studio**.

- **JORGE (`game_client/` — Rust / Bevy Desktop Engine):** A high-performance native desktop client built in Rust + Bevy 0.14. Handles 3D PBR rendering, player collision physics, spatial input, and real-time viewport frame capture, outsourcing all level design, asset staging, and QA verification to the studio.
- **GRETCHEN (`backend/` — Gemini 4 Argon Managed Agents Studio):** An autonomous multi-agent backend powered by the `google-genai` Interactions API (`gemini-4-argon` & `antigravity-preview-09-2026`):
  1. **Studio Director Agent (`director.py`):** Scopes the user's brief, enforces pacing, aesthetic themes, and game objectives.
  2. **Level Architect Agent (`architect.py`):** Calculates procedural spatial layouts, world coordinate bounds, and entity definitions.
  3. **Visual QA / Playtester Agent (`qa_tester.py`):** Inspects raw viewport PNG frames streamed from the Rust engine, audits the 3D scene for navigation blockers or balance issues, and dispatches live hotfix transform deltas back to the engine without reloading.

---

## Architecture

```
                        GRETCHEN: THE OUTSOURCED STUDIO (BACKEND)
       ┌────────────────────────────────────────────────────────────────────────┐
       │                                                                        │
Client │   [Studio Director Agent]                                              │
Brief ─┼──► (Scopes brief & theme)                                              │
       │              │                                                         │
       │              ▼                                                         │
       │   [Level Architect Agent]                                              │
       │   (Drafts spatial graph) ──► [Asset Pipeline]                          │
       │              │                        │                                │
       │              ▼                        ▼                                │
       │      [API Gateway: Work Order JSON Dispatcher]                         │
       │              ▲                        │                                │
       │              │                        │                                │
       │   [Visual QA Tester Agent]            │                                │
       │   (Reviews frame captures,            │                                │
       │    issues hotfix patches)             │                                │
       └──────────────▲────────────────────────┼────────────────────────────────┘
                      │                        │
       Viewport Image │                        │ Scene Contract JSON /
         (PNG stream) │                        │ Transform Deltas
                      │                        ▼
       ┌──────────────┴─────────────────────────────────────────────────────────┐
       │                                                                        │
       │                    JORGE: CLIENT ENGINE (RUST)                         │
       │                                                                        │
       │  [Network / Poller] ──► [Scene Parser] ──► [ECS Spawner]               │
       │                                                  │                     │
       │  [Frame Capture Subsystem] ◄────────────── [Render / Gameplay]         │
       │                                                                        │
       └────────────────────────────────────────────────────────────────────────┘
```

---

## Quick Start

See [INSTALL_GUIDE.md](./INSTALL_GUIDE.md) for full setup and build instructions using `uv` (for Python) and `cargo` (for Rust).
