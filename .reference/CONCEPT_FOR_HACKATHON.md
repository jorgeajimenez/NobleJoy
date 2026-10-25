# Autonomous AI Game Studio & Native Rust Engine Spec

Target: **Gemini 4 Argon SF Hackathon**

Track: **Track 2 — More Dimensions**

Prize Target: **Best Use of Managed Agents ($5,000)**

Submission Deadline: **5:00 PM PT**

---

## 1. Concept: The Outsourced AI Game Studio

Instead of hardcoding game logic or relying on blind procedural noise, the native desktop application treats **Gemini 4 Argon Managed Agents** as an **autonomous external game development studio**.

* **The Client (Jorge / Rust Game Client):** A lightweight, high-performance desktop engine built in Rust. It handles low-level rendering, player physics, and spatial input, but outsources all level design, asset staging, and quality assurance to the studio.
* **The Studio (Gretchen / Managed Agents Backend):** An autonomous agency composed of specialized AI agents:
1. **Studio Executive / Director Agent:** Scopes the user's brief, enforces pacing, aesthetic themes, and sets game objectives.


2. **Level Architect Agent:** Calculates procedural spatial layouts, coordinate bounds, and entity definitions.
3. **Visual QA / Playtester Agent (Track 2 Verification Loop):** Inspects raw viewport frames sent from the client, audits the scene for balance or navigation blockers, and dispatches hotfix patches back to the engine.





---

## 2. System Architecture & Studio Workflow

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

## 3. Monorepo Directory Layout

```text
argon-game-studio/
├── .gitignore
├── Cargo.lock
├── README.md
├── SPEC.md
├── slides/
│   └── presentation.md       # Slide deck content for finals / repo overview
├── backend/                  # GRETCHEN: Autonomous Studio Service
│   ├── .env.example
│   ├── main.py               # API Gateway & WebSocket endpoints
│   ├── requirements.txt
│   ├── agents/
│   │   ├── __init__.py
│   │   ├── director.py       # Studio Director: Scoping & constraints
│   │   ├── architect.py      # Level Architect: Procedural coordinate generator
│   │   └── qa_tester.py      # Visual QA: Multimodal critique & hotfix engine
│   └── assets/
│       └── asset_manifest.json
└── game_client/              # JORGE: Desktop Engine (Rust)
    ├── Cargo.toml
    ├── assets/
    │   └── models/           # Local GLB / primitive 3D assets
    │       ├── player.glb
    │       ├── obstacle.glb
    │       └── goal.glb
    └── src/
        ├── main.rs           # Engine entry point & main game loop
        ├── components.rs     # ECS components (Position, Collider, EntityType)
        ├── systems.rs        # Player controls, camera motion, win state
        ├── network.rs        # Async polling & WebSocket communication
        ├── scene_parser.rs   # Serde deserialization of studio contracts
        ├── level_builder.rs  # Instantiation & real-time transform updates
        └── capture.rs        # Frame buffer capture (viewport to PNG bytes)

```

---

## 4. Studio Data Contracts

### A. Level Contract (`POST /api/commission-level` Output)

```json
{
  "project_id": "job_4091",
  "theme": "industrial_facility",
  "world_bounds": {"x": 24.0, "z": 24.0},
  "entities": [
    {"id": "spawn_pad", "asset_key": "player", "x": 0.0, "y": 0.5, "z": 0.0, "solid": false},
    {"id": "data_core", "asset_key": "goal", "x": 10.0, "y": 0.5, "z": 10.0, "solid": false},
    {"id": "blast_door", "asset_key": "obstacle", "x": 5.0, "y": 1.5, "z": 5.0, "scale": 2.0, "solid": true}
  ]
}

```

### B. QA Bug Report & Hotfix (`POST /api/qa-audit` Output)

The client uploads the current viewport render as multipart PNG data. The Visual QA Tester returns:

```json
{
  "project_id": "job_4091",
  "qa_status": "hotfix_required",
  "critique": "blast_door completely seals off the corridor to data_core. Shifting asset 4 units along X to open a navigable path.",
  "hotfixes": [
    {
      "id": "blast_door",
      "action": "reposition",
      "x": 9.0,
      "y": 1.5,
      "z": 5.0
    }
  ]
}

```

---

## 5. Technology Stack

### Studio Backend (`backend/`)

* **Language & Framework:** Python 3.11+, FastAPI, Uvicorn
* **AI Core:** `gemini-4-argon` with Google Cloud Managed Agents


* **Multimodal Engine:** Visual QA inspection feeding engine frame buffers to Gemini vision



### Game Client (`game_client/`)

* **Language:** Rust (2021 Edition)
* **Engine Core:** Bevy `0.14` (3D PBR pipeline, ECS architecture)
* **Networking:** `tokio`, `reqwest`, `crossbeam-channel`
* **Serialization & Imaging:** `serde`, `serde_json`, `image`

---

## 6. Slide Deck Blueprint (3–5 Slides)

> **Important Hackathon Rule Note:** Round 1 judging explicitly requires a **3-minute live demonstration without slide decks** (10% pitch score). Use these slides as visual backdrops on your monitors, in your GitHub README, or during the **Round 2 Top 6 Stage Finals**.
> 
> 

### Slide 1: The Hook — Autonomous Studio Outsourcing

* **Header:** Autonomous Studio: Outsourcing Game Dev to AI Agents
* **Visual:** Split graphic — Left: Jorge's high-speed Rust desktop client. Right: Gretchen's multi-agent studio running on Gemini 4 Argon.


* **Core Points:**
* Procedural generation has always lacked intent, narrative cohesion, and visual awareness.
* Game engines should render and simulate; Managed Agents should design, direct, and test.


* Introducing an end-to-end studio pipeline: from player prompt to verified playable level.



### Slide 2: The Problem — Blind Procedural Generation

* **Header:** The Problem: Procedural Systems Cannot "See" Their Mistakes
* **Visual:** Diagram of an unplayable level (an obstacle clipping through a goal) with a red "FAILED" badge.
* **Core Points:**
* Math-based procedural generation often generates broken, impassable geometry.
* Manual playtesting is slow, expensive, and outside the reach of dynamic runtimes.
* Without an active multimodal feedback loop, generative game worlds remain static toys.





### Slide 3: The Architecture — The Closed Verification Loop

* **Header:** The Agentic Studio: Design, Render, Audit, Patch
* **Visual:** The 4-step circular architecture diagram:
1. *Director & Architect:* Prompts $\rightarrow$ Spatial Level Spec
2. *Rust Desktop Client:* Real-time ECS asset placement & lighting
3. *Viewport Capture:* Snapshot rendered frame buffer
4. *Visual QA Agent:* Gemini 4 Argon critiques layout & pushes live hotfix deltas




* **Core Points:**
* Powered by Gemini 4 Argon Managed Agents.


* Native desktop client renders and streams snapshots back to the studio in sub-second intervals.



### Slide 4: Engineering Innovation & Tech Stack

* **Header:** Frontier AI Meets Native Systems Performance
* **Visual:** Grid of technologies (`Gemini 4 Argon`, `Google Cloud Managed Agents`, `Rust`, `Bevy Engine`, `FastAPI`).


* **Core Points:**
* **Memory & Reasoning:** 1M context handles persistent multi-room layouts and level history.


* **Zero Reloads:** Differential JSON transforms update Rust ECS components live in memory.
* **Multimodal Verification:** Autonomous visual playtesting without human intervention.





### Slide 5: The Live Demo (Transition to Action)

* **Header:** Live Autonomous Commission
* **Visual:** Live screen-share of the Rust app side-by-side with agent logs.
* **Core Points:**
* Prompt: *"Design an abandoned hangar where the reactor core is shielded."*
* Watch the Studio build the layout, detect a navigation blockage, and self-correct on screen.
* Controller ready: Live playthrough demonstrating real collision, objectives, and movement.





---

## 7. 3-Minute Live Demo Pitch Script (No Slides Mode)



* **0:00 – 0:30 (The Outsourcing Analogy):**
*"Modern game development separates the game engine from content design. We built a high-performance Rust desktop game client, but we completely outsourced the level design, asset staging, and QA playtesting to an autonomous game studio powered by Gemini 4 Argon Managed Agents."*


* **0:30 – 1:30 (First-Pass Generation):**
*Send prompt: 'Construct a sci-fi reactor bay with defensive blockades.'*
Show the terminal firing the request to the Director Agent, which coordinates with the Architect to return the initial level contract. Show the Rust client spawning the assets live.


* **1:30 – 2:15 (The Multimodal QA Loop — Track 2 Focus):**
Show the Rust client automatically taking a viewport screenshot and submitting it to the QA Tester Agent.


*"Watch the QA Agent: it visually identifies that our blast wall completely blocks access to the reactor core. Without human input, it generates a hotfix transform delta, and the Rust client moves the obstacle live."*


* **2:15 – 3:00 (Interactive Gameplay):**
Take the controls. WASD the player entity through the newly opened corridor, touch the reactor beacon, and trigger the victory state to prove live working functionality.



---

Would you like the complete implementation of `backend/agents/qa_tester.py` (multimodal vision critique using Gemini 4 Argon) or the Rust `capture.rs` buffer grabber next?