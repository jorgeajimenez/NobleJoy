"""Level Architect Agent: Calculates procedural spatial layouts, coordinate bounds, and entity definitions."""

import json
import os
from typing import Any, Dict


class LevelArchitectAgent:
    """Drafts the spatial entity graph from the Studio Director's brief using Gemini 4 Argon."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def build_level_contract(
        self,
        directive: Dict[str, Any],
        manifest: Dict[str, Any],
        seed_blockade: bool = True,
    ) -> Dict[str, Any]:
        """Generate the Level Contract JSON matching the Rust client's SceneParser contract."""
        project_id = directive.get("project_id", "job_4091")
        theme = directive.get("theme", "industrial_facility")
        bounds = directive.get("world_bounds", {"x": 24.0, "z": 24.0})

        prompt = (
            "You are the Level Architect Agent in an autonomous AI game studio.\n"
            f"Studio Directive: {json.dumps(directive)}\n"
            f"Available Assets Manifest: {json.dumps(manifest)}\n"
            "Return ONLY valid JSON matching the exact Level Contract schema:\n"
            "{\n"
            f'  "project_id": "{project_id}",\n'
            f'  "theme": "{theme}",\n'
            '  "world_bounds": {"x": 24.0, "z": 24.0},\n'
            '  "entities": [\n'
            '    {"id": "spawn_pad", "asset_key": "player", "x": 0.0, "y": 0.5, "z": 0.0, "scale": 1.0, "solid": false},\n'
            '    {"id": "data_core", "asset_key": "goal", "x": 10.0, "y": 0.5, "z": 10.0, "scale": 1.0, "solid": false},\n'
            '    {"id": "blast_door", "asset_key": "obstacle", "x": 5.0, "y": 1.5, "z": 5.0, "scale": 2.0, "solid": true}\n'
            "  ]\n"
            "}\n"
        )
        if seed_blockade:
            prompt += (
                "\nIMPORTANT FOR FIRST-PASS GENERATION: Place at least one obstacle ('blast_door') "
                "directly along the path at (5.0, 1.5, 5.0) with scale 2.5 so the Visual QA Tester "
                "Agent can visually detect and hotfix the navigation blocker during the live verification loop."
            )

        if self.api_key:
            try:
                from google import genai

                client = genai.Client(api_key=self.api_key)
                if self.use_managed_agent:
                    interaction = client.interactions.create(
                        agent="antigravity-preview-09-2026",
                        agent_config={
                            "type": "antigravity",
                            "model": "gemini-4-argon",
                        },
                        input=prompt,
                        environment="remote",
                    )
                else:
                    interaction = client.interactions.create(
                        model="gemini-4-argon",
                        input=prompt,
                        generation_config={"thinking_level": "medium"},
                    )
                raw = interaction.output_text.strip()
                if "```" in raw:
                    raw = raw.split("```")[1].removeprefix("json").strip()
                parsed = json.loads(raw)
                if "entities" in parsed:
                    return parsed
            except Exception as exc:
                print(f"[ArchitectAgent] Fallback triggered ({exc})")

        return {
            "project_id": project_id,
            "theme": theme,
            "world_bounds": bounds,
            "entities": [
                {
                    "id": "spawn_pad",
                    "asset_key": "player",
                    "x": 0.0,
                    "y": 0.5,
                    "z": 0.0,
                    "scale": 1.0,
                    "solid": False,
                },
                {
                    "id": "data_core",
                    "asset_key": "goal",
                    "x": 8.0,
                    "y": 0.5,
                    "z": 8.0,
                    "scale": 1.0,
                    "solid": False,
                },
                {
                    "id": "corridor_wall_left",
                    "asset_key": "obstacle",
                    "x": 2.0,
                    "y": 1.5,
                    "z": 5.0,
                    "scale": 2.0,
                    "solid": True,
                },
                {
                    "id": "corridor_wall_right",
                    "asset_key": "obstacle",
                    "x": 6.0,
                    "y": 1.5,
                    "z": 3.0,
                    "scale": 2.0,
                    "solid": True,
                },
                {
                    "id": "blast_door",
                    "asset_key": "obstacle",
                    "x": 4.0,
                    "y": 1.5,
                    "z": 4.0,
                    "scale": 2.5,
                    "solid": True,
                },
            ],
        }
