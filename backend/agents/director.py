"""Studio Director Agent: Scopes the driving/rover level brief, environmental hazards, and modular upgrades."""

import json
import os
from typing import Any, Dict


class StudioDirectorAgent:
    """Uses Gemini 4 Argon / Managed Agents to scope a driving & modular rover brief."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def scope_brief(self, user_prompt: str, project_id: str) -> Dict[str, Any]:
        """Translate a raw user prompt into a structured Studio Directive with MuJoCo vehicle specs & reward parts."""
        system_directive = (
            "You are the Studio Executive Director Agent at Gretchen Autonomous Game Studio.\n"
            "We are building a physics-aware MuJoCo driving & modular rover game.\n"
            "Given a client prompt, output ONLY valid JSON with keys:\n"
            '  "project_id": string,\n'
            '  "theme": string,\n'
            '  "objective": string,\n'
            '  "environmental_hazard": string (e.g. "low_clearance_gap_40cm", "steep_slag_ramp_28deg", "blast_door_blockade"),\n'
            '  "required_capability": string (e.g. "high_torque_4wd + 2-DOF manipulation"),\n'
            '  "world_bounds": {"x": 24.0, "z": 24.0},\n'
            '  "car_physics": {\n'
            '    "chassis_mass_kg": 4.5,\n'
            '    "wheel_friction": 1.2,\n'
            '    "motor_torque_nm": 3.5,\n'
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
                parsed = json.loads(raw)
                if "car_physics" in parsed:
                    return parsed
            except Exception as exc:
                print(f"[DirectorAgent] Fallback triggered ({exc})")

        return {
            "project_id": project_id,
            "theme": "industrial_sorting_bay_track",
            "objective": "Drive the MuJoCo wheeled rover from spawn_pad to data_core and unlock the M3 modular actuator mount.",
            "environmental_hazard": "low_clearance_gap_40cm + blast_door_blockade",
            "required_capability": "high_torque_4wd + 2-DOF manipulation",
            "world_bounds": {"x": 24.0, "z": 24.0},
            "car_physics": {
                "chassis_mass_kg": 4.5,
                "wheel_friction": 1.2,
                "motor_torque_nm": 3.5,
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
            "skin_prompt": f"Cybernetic off-road rover chassis skin for '{user_prompt}', carbon fiber with orange hazard stripes",
        }
