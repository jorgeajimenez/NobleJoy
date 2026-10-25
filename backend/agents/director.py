"""Studio Director Agent: Scopes the GTA-style city street driving brief, hazards, and modular upgrades."""

import json
import os
from typing import Any, Dict


class StudioDirectorAgent:
    """Uses Gemini 4 Argon / Managed Agents to scope a city street driving & modular rover brief."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def scope_brief(self, user_prompt: str, project_id: str) -> Dict[str, Any]:
        """Translate a raw user prompt into a structured Studio Directive with MuJoCo vehicle specs & city street layout."""
        system_directive = (
            "You are the Studio Executive Director Agent at Gretchen Autonomous Game Studio.\n"
            "We are building a GTA-style 3D street driving & physics-aware MuJoCo vehicle game.\n"
            "Given a client prompt, output ONLY valid JSON with keys:\n"
            '  "project_id": string,\n'
            '  "theme": string (e.g. "neon_downtown_avenue", "industrial_harbor_streets"),\n'
            '  "objective": string,\n'
            '  "environmental_hazard": string (e.g. "mid_avenue_blast_door_blockade", "narrow_alley_chicane"),\n'
            '  "required_capability": string (e.g. "high_torque_4wd + street_drift_grip"),\n'
            '  "world_bounds": {"x": 80.0, "z": 40.0},\n'
            '  "car_physics": {\n'
            '    "chassis_mass_kg": 4.5,\n'
            '    "wheel_friction": 1.35,\n'
            '    "motor_torque_nm": 4.0,\n'
            '    "ground_clearance_m": 0.18,\n'
            '    "servo_torque_kg_cm": 1.8\n'
            '  },\n'
            '  "reward_part": {\n'
            '    "part_id": "high_torque_wheel_hub_v2",\n'
            '    "mount_type": "M3_faceplate_40mm",\n'
            '    "compatible_motor": "MG996R_servo",\n'
            '    "mass_grams": 180.0,\n'
            '    "torque_capacity_kg_cm": 11.0,\n'
            '    "stl_manifest_id": "stl_wheel_hub_m3_02"\n'
            '  },\n'
            '  "skin_prompt": string\n'
            f"\nClient Prompt: {user_prompt}\nProject ID: {project_id}"
        )

        if self.api_key and self.api_key not in ("your_gemini_key", "your_gemini_api_key_here"):
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
                parsed = json.loads(raw)
                if "car_physics" in parsed:
                    return parsed
            except Exception as exc:
                print(f"[DirectorAgent] Fallback triggered ({exc})")

        return {
            "project_id": project_id,
            "theme": "neon_downtown_avenue",
            "objective": "Drive down Main Cyber Avenue through the city blocks to reach the Reactor Checkpoint Beacon.",
            "environmental_hazard": "mid_avenue_blast_door_blockade",
            "required_capability": "high_torque_4wd + 2-DOF street clearance",
            "world_bounds": {"x": 80.0, "z": 40.0},
            "car_physics": {
                "chassis_mass_kg": 4.5,
                "wheel_friction": 1.35,
                "motor_torque_nm": 4.0,
                "ground_clearance_m": 0.18,
                "servo_torque_kg_cm": 1.8,
            },
            "reward_part": {
                "part_id": "high_torque_wheel_hub_v2",
                "mount_type": "M3_faceplate_40mm",
                "compatible_motor": "MG996R_servo",
                "mass_grams": 180.0,
                "torque_capacity_kg_cm": 11.0,
                "stl_manifest_id": "stl_wheel_hub_m3_02",
            },
            "skin_prompt": f"GTA cyber-street rally car hood & chassis decal skin for '{user_prompt}', carbon fiber with neon cyan & orange racing stripes",
        }
