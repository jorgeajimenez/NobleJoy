# Slide Deck Blueprint: Autonomous Studio

> **Note:** Round 1 judging requires a **3-minute live demonstration without slide decks** (10% pitch score). Use these slides as visual backdrops, in the GitHub README, or during the **Round 2 Top 6 Stage Finals**.

---

## Slide 1: The Hook — Autonomous Studio Outsourcing

* **Header:** Autonomous Studio: Outsourcing Game Dev to AI Agents
* **Visual:** Split graphic — Left: Jorge's high-speed Rust desktop client. Right: Gretchen's multi-agent studio running on Gemini 4 Argon.
* **Core Points:**
  * Procedural generation has always lacked intent, narrative cohesion, and visual awareness.
  * Game engines should render and simulate; Managed Agents should design, direct, and test.
  * Introducing an end-to-end studio pipeline: from player prompt to verified playable level.

---

## Slide 2: The Problem — Blind Procedural Generation

* **Header:** The Problem: Procedural Systems Cannot "See" Their Mistakes
* **Visual:** Diagram of an unplayable level (an obstacle clipping through a goal) with a red "FAILED" badge.
* **Core Points:**
  * Math-based procedural generation often generates broken, impassable geometry.
  * Manual playtesting is slow, expensive, and outside the reach of dynamic runtimes.
  * Without an active multimodal feedback loop, generative game worlds remain static toys.

---

## Slide 3: The Architecture — The Closed Verification Loop

* **Header:** The Agentic Studio: Design, Render, Audit, Patch
* **Visual:** The 4-step circular architecture diagram:
  1. *Director & Architect:* Prompts $\rightarrow$ Spatial Level Spec
  2. *Rust Desktop Client:* Real-time ECS asset placement & lighting
  3. *Viewport Capture:* Snapshot rendered frame buffer
  4. *Visual QA Agent:* Gemini 4 Argon critiques layout & pushes live hotfix deltas
* **Core Points:**
  * Powered by Gemini 4 Argon Managed Agents (`antigravity-preview-09-2026`).
  * Native desktop client renders and streams snapshots back to the studio in sub-second intervals.

---

## Slide 4: Engineering Innovation & Tech Stack

* **Header:** Frontier AI Meets Native Systems Performance
* **Visual:** Grid of technologies (`Gemini 4 Argon`, `Google Cloud Managed Agents`, `Rust`, `Bevy Engine`, `FastAPI`).
* **Core Points:**
  * **Memory & Reasoning:** 1M context handles persistent multi-room layouts and level history.
  * **Zero Reloads:** Differential JSON transforms update Rust ECS components live in memory.
  * **Multimodal Verification:** Autonomous visual playtesting without human intervention.

---

## Slide 5: The Live Demo (Transition to Action)

* **Header:** Live Autonomous Commission
* **Visual:** Live screen-share of the Rust app side-by-side with agent logs.
* **Core Points:**
  * Prompt: *"Design an abandoned hangar where the reactor core is shielded."*
  * Watch the Studio build the layout, detect a navigation blockage, and self-correct on screen.
  * Controller ready: Live playthrough demonstrating real collision, objectives, and movement.
