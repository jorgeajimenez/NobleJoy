"""Level Architect Agent: Generates theme-adaptive 3D street courses, invokes Managed Agent sandbox, and pulls artifacts."""

import json
import os
from typing import Any, Dict, List

from .artifact_puller import (
    GENERATED_ASSETS_DIR,
    generate_vehicle_skin_texture,
    pull_managed_agent_workspace,
    write_modular_part_stl,
)


def default_city_street_entities(theme: str = "neon_downtown_avenue", level_index: int = 1) -> List[Dict[str, Any]]:
    """Return a GTA-style street corridor lined with theme-appropriate structures, chicanes, a central roadblock, and a goal beacon."""
    goal_x = 38.0 + (level_index - 1) * 8.0
    entities: List[Dict[str, Any]] = [
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
            "x": goal_x,
            "y": 0.5,
            "z": 0.0,
            "scale": 1.25,
            "solid": False,
        },
        {
            "id": "street_chicane_left",
            "asset_key": "obstacle",
            "x": 11.0,
            "y": 0.9,
            "z": -2.3,
            "scale": 1.3,
            "solid": True,
        },
        {
            "id": "street_chicane_right",
            "asset_key": "obstacle",
            "x": 27.0,
            "y": 0.9,
            "z": 2.3,
            "scale": 1.3,
            "solid": True,
        },
        # Center-avenue blockade that Visual QA Agent + WGPU Validator detects and shifts onto the sidewalk!
        {
            "id": "blast_door",
            "asset_key": "obstacle",
            "x": 19.0,
            "y": 1.5,
            "z": 0.0,
            "scale": 2.6,
            "solid": True,
        },
    ]

    # Add extra slalom obstacles for higher campaign levels
    for extra in range(1, level_index):
        entities.append(
            {
                "id": f"level_{level_index}_hazard_{extra}",
                "asset_key": "obstacle",
                "x": 31.0 + extra * 6.5,
                "y": 0.9,
                "z": -2.1 if extra % 2 == 1 else 2.1,
                "scale": 1.35,
                "solid": True,
            }
        )

    # Roadside architecture lining the left (Z = -9.5) and right (Z = +9.5) sides of the street
    building_heights = (
        [1.8, 2.5, 2.0, 2.8, 2.2, 2.6, 1.9, 2.4]
        if "medieval" in theme or "desert" in theme or "arctic" in theme
        else [2.4, 3.6, 2.8, 4.2, 3.1, 3.8, 2.6, 3.4]
    )
    for idx, bx in enumerate([-6.0, 4.0, 14.0, 24.0, 34.0, 44.0, 54.0, 64.0]):
        sc = building_heights[idx % len(building_heights)]
        entities.append(
            {
                "id": f"city_block_north_{idx}",
                "asset_key": "building",
                "x": bx,
                "y": sc * 1.5,
                "z": -9.5,
                "scale": sc,
                "solid": True,
            }
        )
        entities.append(
            {
                "id": f"city_block_south_{idx}",
                "asset_key": "building",
                "x": bx,
                "y": sc * 1.5,
                "z": 9.5,
                "scale": sc,
                "solid": True,
            }
        )

    return entities


class LevelArchitectAgent:
    """Drafts the spatial street course & pulls generated MJCF/URDF/STL/Skin files from the Managed Agent sandbox."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def build_level_contract(
        self,
        directive: Dict[str, Any],
        manifest: Dict[str, Any],
        seed_blockade: bool = True,
    ) -> Dict[str, Any]:
        """Run the Managed Agent to build the street JSON + workspace artifacts, then pull them locally."""
        project_id = directive.get("project_id", "job_4091")
        level_index = int(directive.get("level_index", 1))
        theme = directive.get("theme", "neon_downtown_avenue")
        bounds = directive.get("world_bounds", {"x": 80.0, "z": 40.0})
        car_physics = directive.get(
            "car_physics",
            {
                "chassis_mass_kg": 4.5,
                "wheel_friction": 1.35,
                "motor_torque_nm": 4.0,
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

        goal_x = 38.0 + (level_index - 1) * 8.0
        prompt = (
            "You are the Level Architect & Robotics Systems Integrator Agent in an autonomous AI game studio.\n"
            f"Studio Directive: {json.dumps(directive)}\n"
            f"Design a 3D street avenue for theme '{theme}' (Level {level_index}) extending along the +X axis from (0, 0, 0) to ({goal_x}, 0, 0).\n"
            "Line both sides of the street (at z = -9.5 and z = +9.5) with theme-appropriate structures (asset_key='building', solid=true).\n"
            f"Place 'spawn_pad' (asset_key='player') at (0, 0.35, 0) and 'data_core' (asset_key='goal') at ({goal_x}, 0.5, 0).\n"
            "When running inside the Managed Agent Linux sandbox (/workspace):\n"
            "1. Write `/workspace/level_contract.json` with the 3D street entities.\n"
            "2. Write `/workspace/rover.urdf` and `/workspace/car_level.xml` (MuJoCo MJCF) for the 4-wheel vehicle.\n"
            "3. Print ONLY the valid JSON for the LevelContract to stdout.\n"
        )
        if seed_blockade:
            prompt += (
                "\nIMPORTANT FOR FIRST-PASS GENERATION: Place 'blast_door' (asset_key='obstacle', scale=2.6, solid=true) "
                "directly in the center of the street at (19.0, 1.5, 0.0) so the Visual QA Tester Agent + WGPU Artifact Detector "
                "visually detects the blockade and shifts it onto the sidewalk (z = 6.5) during the live verification loop."
            )

        extracted_sandbox_files: List[str] = []
        parsed_entities: List[Dict[str, Any]] = []

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
                if isinstance(parsed.get("entities"), list) and len(parsed["entities"]) >= 3:
                    parsed_entities = parsed["entities"]
            except Exception as exc:
                print(f"[ArchitectAgent] Fallback triggered ({exc})")

        if not parsed_entities:
            parsed_entities = default_city_street_entities(theme=theme, level_index=level_index)

        skin_info = generate_vehicle_skin_texture(
            prompt=directive.get("skin_prompt", f"GTA street rally rover skin for {theme}"),
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
            "level_index": level_index,
            "game_title": directive.get("game_title", "Neon Overdrive 2099"),
            "plot_summary": directive.get(
                "plot_summary",
                "Breach the street blockade and deliver the core to the checkpoint beacon.",
            ),
            "objective": directive.get(
                "objective",
                "Drive down the main street corridor and reach the checkpoint beacon.",
            ),
            "theme": theme,
            "world_bounds": bounds,
            "environment_style": directive.get("environment_style", {}),
            "car_design": directive.get("car_design", {}),
            "environmental_hazard": directive.get("environmental_hazard", "mid_avenue_blast_door_blockade"),
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
