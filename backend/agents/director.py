"""Studio Director Agent: Scopes the user's brief, enforces pacing, aesthetic themes, and game objectives."""

import json
import os
from typing import Any, Dict


class StudioDirectorAgent:
    """Uses Gemini 4 Argon / Managed Agents to scope a level brief into structured design constraints."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def scope_brief(self, user_prompt: str, project_id: str) -> Dict[str, Any]:
        """Translate a raw user prompt into a structured Studio Directive."""
        system_directive = (
            "You are the Studio Executive Director Agent at Gretchen Autonomous Game Studio.\n"
            "Given a client prompt, output ONLY valid JSON defining the level directive with keys:\n"
            '  "project_id": string,\n'
            '  "theme": string (e.g. "industrial_facility", "cyber_reactor", "abandoned_hangar"),\n'
            '  "objective": string,\n'
            '  "pacing": string,\n'
            '  "world_bounds": {"x": 24.0, "z": 24.0},\n'
            '  "architect_instructions": string\n'
            f"\nClient Prompt: {user_prompt}\nProject ID: {project_id}"
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
                        input=system_directive,
                        environment="remote",
                    )
                else:
                    interaction = client.interactions.create(
                        model="gemini-4-argon",
                        input=system_directive,
                        generation_config={"thinking_level": "low"},
                    )
                raw = interaction.output_text.strip()
                if "```" in raw:
                    raw = raw.split("```")[1].removeprefix("json").strip()
                return json.loads(raw)
            except Exception as exc:
                print(f"[DirectorAgent] Fallback triggered ({exc})")

        return {
            "project_id": project_id,
            "theme": "industrial_facility",
            "objective": "Reach the shielded reactor data_core from spawn_pad.",
            "pacing": "tactical_corridor",
            "world_bounds": {"x": 24.0, "z": 24.0},
            "architect_instructions": (
                f"Fulfill prompt '{user_prompt}': place spawn_pad at origin, "
                "data_core at (10, 0.5, 10), and defensive blast_door obstacles."
            ),
        }
