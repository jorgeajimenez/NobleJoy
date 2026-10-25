"""Level Architect Agent: Generates the driving course, invokes the Managed Agent sandbox, and pulls artifacts."""

import json
import os
from typing import Any, Dict, List

from .artifact_puller import (
    GENERATED_ASSETS_DIR,
    generate_vehicle_skin_texture,
    pull_managed_agent_workspace,
    write_modular_part_stl,
)


class LevelArchitectAgent:
    """Drafts the spatial driving course & pulls generated MJCF/URDF/STL/Skin files from the Managed Agent sandbox."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def build_level_contract(
        self,
        directive: Dict[str, Any],
        manifest: Dict[str, Any],
        seed_blockade: bool = True,
    ) -> Dict[str, Any]:
        """Run the Managed Agent to build the level JSON + workspace artifacts, then pull them locally."""
        project_id = directive.get("project_id", "job_4091")
        theme = directive.get("theme", "industrial_sorting_bay_track")
        bounds = directive.get("world_bounds", {"x": 24.0, "z": 24.0})
        car_physics = directive.get(
            "car_physics",
            {
                "chassis_mass_kg": 4.5,
                "wheel_friction": 1.2,
                "motor_torque_nm": 3.5,
                "ground_clearance_m": 0.18,
                "servo_torque_kg_cm": 1.8,
            },
        )
        reward_part = directive.get(
            "reward_part",
            {
                "part_id": "high_torque_wheel_hub_v2",
                "mount_type": "M3_faceplate_40mm",
                "compatible_motor": "MG996R_servo",
                "mass_grams": 180.0,
                "torque_capacity_kg_cm": 11.0,
                "stl_manifest_id": "stl_wheel_hub_m3_02",
            },
        )

        prompt = (
            "You are the Level Architect & Robotics Systems Integrator Agent in an autonomous AI game studio.\n"
            f"Studio Directive: {json.dumps(directive)}\n"
            "When running inside the Managed Agent Linux sandbox (/workspace):\n"
            "1. Write `/workspace/level_contract.json` with the 3D driving course entities.\n"
            "2. Write `/workspace/rover.urdf` and `/workspace/car_level.xml` (MuJoCo MJCF) for the 4-wheel rover.\n"
            "3. Print ONLY the valid JSON for the LevelContract to stdout.\n"
        )
        if seed_blockade:
            prompt += (
                "\nIMPORTANT FOR FIRST-PASS GENERATION: Place 'blast_door' directly across the driving lane "
                "at (4.0, 1.5, 4.0) with scale 2.5 so the Visual QA Tester Agent visually detects and patches it live."
            )

        extracted_sandbox_files: List[str] = []
        parsed_entities: List[Dict[str, Any]] = []

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
                    env_id = getattr(interaction, "environment_id", None)
                    extracted_sandbox_files = pull_managed_agent_workspace(
                        environment_id=env_id,
                        project_id=project_id,
                        api_key=self.api_key,
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
                if isinstance(parsed.get("entities"), list):
                    parsed_entities = parsed["entities"]
            except Exception as exc:
                print(f"[ArchitectAgent] Fallback triggered ({exc})")

        if not parsed_entities:
            parsed_entities = [
                {
                    "id": "spawn_pad",
                    "asset_key": "player",
                    "x": 0.0,
                    "y": 0.35,
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
                    "id": "track_barrier_west",
                    "asset_key": "obstacle",
                    "x": 2.0,
                    "y": 1.2,
                    "z": 5.5,
                    "scale": 2.0,
                    "solid": True,
                },
                {
                    "id": "track_barrier_east",
                    "asset_key": "obstacle",
                    "x": 6.0,
                    "y": 1.2,
                    "z": 2.5,
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
            ]

        skin_info = generate_vehicle_skin_texture(
            prompt=directive.get("skin_prompt", f"Rover skin for {theme}"),
            project_id=project_id,
            skin_name="chassis_skin.png",
            api_key=self.api_key,
        )

        part_id = reward_part.get("part_id", "high_torque_wheel_hub_v2")
        stl_path = write_modular_part_stl(
            GENERATED_ASSETS_DIR / project_id / f"{part_id}.stl",
            part_id=part_id,
        )

        return {
            "project_id": project_id,
            "theme": theme,
            "world_bounds": bounds,
            "environmental_hazard": directive.get("environmental_hazard", "low_clearance_gap_40cm"),
            "required_capability": directive.get("required_capability", "high_torque_4wd"),
            "car_physics": car_physics,
            "reward_part": reward_part,
            "skin_url": skin_info["url"],
            "mjcf_url": f"/assets/generated/{project_id}/car_level.xml",
            "urdf_url": f"/assets/generated/{project_id}/rover.urdf",
            "stl_url": f"/assets/generated/{project_id}/{part_id}.stl",
            "sandbox_files": extracted_sandbox_files + [skin_info["local_path"], stl_path],
            "entities": parsed_entities,
        }
