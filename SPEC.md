# Technical Specification: Argon Game Studio

## 1. Studio Data Contracts

### A. Level Contract (`POST /api/commission-level`)

Request:
```json
{
  "prompt": "Construct a sci-fi reactor bay with defensive blockades.",
  "intentally_flawed_first_pass": true
}
```

Response:
```json
{
  "project_id": "job_4091",
  "theme": "industrial_facility",
  "world_bounds": {"x": 24.0, "z": 24.0},
  "entities": [
    {"id": "spawn_pad", "asset_key": "player", "x": 0.0, "y": 0.5, "z": 0.0, "scale": 1.0, "solid": false},
    {"id": "data_core", "asset_key": "goal", "x": 10.0, "y": 0.5, "z": 10.0, "scale": 1.0, "solid": false},
    {"id": "blast_door", "asset_key": "obstacle", "x": 5.0, "y": 1.5, "z": 5.0, "scale": 2.0, "solid": true}
  ]
}
```

### B. QA Bug Report & Hotfix (`POST /api/qa-audit`)

The Rust client uploads the current viewport render as multipart PNG data (`file`) alongside `project_id` and current level JSON state. The Visual QA Tester returns:

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

## 2. Gemini 4 Argon & Managed Agents Integration

- Uses `google-genai >= 2.3.0` Interactions API (`client.interactions.create(...)`).
- **Studio Director & Level Architect:** Uses `gemini-4-argon` and/or Managed Agent `antigravity-preview-09-2026` with `thinking_level="medium"`.
- **Visual QA Playtester:** Multimodal verification loop inspecting live engine PNG frame captures combined with entity coordinate graphs to detect navigation blockers and issue deterministic transform deltas.
- **No Legacy Parameters:** Strictly omits deprecated `temperature`, `top_p`, `top_k`, and `thinking_budget` parameters per the Gemini 4 Argon specification.
