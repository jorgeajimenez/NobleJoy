"""Studio Director Agent: Scopes game plot, world theme (Medieval, Cyber, Desert, Arctic, etc.), car design, and MuJoCo physics."""

import json
import os
from typing import Any, Dict


def infer_theme_preset(user_prompt: str, level_index: int = 1) -> Dict[str, Any]:
    """Infer a rich world theme, plot, environment palette, and car design from the user's natural-language prompt."""
    p = user_prompt.lower()
    if any(k in p for k in ["medieval", "castle", "knight", "kingdom", "cobble", "dragon", "tavern", "fantasy", "sword"]):
        return {
            "theme": "medieval_cobblestone_kingdom",
            "game_title": "Chronicles of Iron & Cobble",
            "plot_summary": (
                f"Chapter {level_index}: Lord Malakor has lowered the Iron Portcullis across King's Cobblestone Road. "
                "Pilot the Royal Siege Wagon past the timber taverns and stone keep towers, breach the blockade, "
                "and deliver the Dragon Relic to the Citadel Sanctuary."
            ),
            "objective": "Drive down King's Cobblestone Road, clear the Iron Portcullis gate, and reach the Citadel Sanctuary.",
            "environmental_hazard": "iron_portcullis_gate_blockade",
            "required_capability": "siege_battering_ram + high_torque_oak_axles",
            "environment_style": {
                "ground_color": [0.16, 0.26, 0.14],
                "road_color": [0.36, 0.32, 0.27],
                "sidewalk_color": [0.48, 0.45, 0.40],
                "sky_color": [0.14, 0.17, 0.24],
                "building_style": "medieval_keep",
                "prop_style": "torch_post",
                "has_center_dashes": False,
            },
            "car_design": {
                "body_style": "medieval_wagon",
                "primary_color": [0.58, 0.36, 0.16],
                "accent_color": [0.85, 0.68, 0.20],
                "wheel_scale": 1.15,
                "ride_height": 0.04,
                "roof_attachment": "battering_ram",
            },
            "skin_prompt": f"Medieval royal war wagon oak wood and forged Damascus iron armor texture with gold heraldic trim for '{user_prompt}'",
        }
    if any(k in p for k in ["desert", "mars", "canyon", "wasteland", "dune", "sand", "mad", "rust"]):
        return {
            "theme": "wasteland_convoy_canyon",
            "game_title": "Mad Torque: Red Dust Run",
            "plot_summary": (
                f"Chapter {level_index}: Scavenger clans have sealed Fury Canyon with a rusted scrap-metal wall. "
                "Engage 4WD crawler lock, weave between sandstone mesas and oil derricks, and deliver the fuel core."
            ),
            "objective": "Blast through the red-rock canyon trail, clear the scrap barricade, and reach the Refinery Beacon.",
            "environmental_hazard": "rusted_scrap_wall_barricade",
            "required_capability": "lifted_4wd_crawler + desert_beadlock_tires",
            "environment_style": {
                "ground_color": [0.58, 0.31, 0.16],
                "road_color": [0.44, 0.25, 0.14],
                "sidewalk_color": [0.66, 0.39, 0.22],
                "sky_color": [0.32, 0.18, 0.12],
                "building_style": "desert_mesa",
                "prop_style": "torch_post",
                "has_center_dashes": False,
            },
            "car_design": {
                "body_style": "armored_crawler",
                "primary_color": [0.82, 0.42, 0.15],
                "accent_color": [0.22, 0.22, 0.24],
                "wheel_scale": 1.30,
                "ride_height": 0.08,
                "roof_attachment": "m3_servo_arm",
            },
            "skin_prompt": f"Weathered desert wasteland off-road crawler armor plating with hazard stripes for '{user_prompt}'",
        }
    if any(k in p for k in ["arctic", "ice", "snow", "glacier", "frozen", "cryo", "winter"]):
        return {
            "theme": "arctic_glacier_outpost",
            "game_title": "Cryo-Drive: Subzero Protocol",
            "plot_summary": (
                f"Chapter {level_index}: A glacial shear triggered an ice-wall blockade across Sector Zero Causeway. "
                "Pilot the thermal interceptor between geodesic cryo-domes to re-ignite the fusion beacon."
            ),
            "objective": "Navigate the frozen causeway, shatter the glacial ice blockade, and reach the Fusion Core.",
            "environmental_hazard": "glacial_ice_wall_blockade",
            "required_capability": "studded_cryo_treads + thermal_servo_actuator",
            "environment_style": {
                "ground_color": [0.82, 0.90, 0.96],
                "road_color": [0.45, 0.65, 0.82],
                "sidewalk_color": [0.72, 0.84, 0.92],
                "sky_color": [0.10, 0.18, 0.28],
                "building_style": "arctic_dome",
                "prop_style": "cryo_pylon",
                "has_center_dashes": True,
            },
            "car_design": {
                "body_style": "cyber_interceptor",
                "primary_color": [0.92, 0.96, 1.0],
                "accent_color": [0.10, 0.78, 0.98],
                "wheel_scale": 1.20,
                "ride_height": 0.05,
                "roof_attachment": "twin_turbo",
            },
            "skin_prompt": f"Arctic exploration rover white carbon composite with cyan thermal conduits for '{user_prompt}'",
        }

    # Default / Cyber-City / Custom Prompt
    return {
        "theme": "neon_downtown_avenue",
        "game_title": "Neon Overdrive 2099",
        "plot_summary": (
            f"Chapter {level_index}: CorpSec locked down Main Cyber Avenue with an armored blast door. "
            f"Mission Brief ('{user_prompt}'): Thread the needle through downtown skyscrapers, run Visual QA to override "
            "the blockade, and deliver the neural payload to the Extraction Beacon."
        ),
        "objective": "Drive down Main Cyber Avenue through the skyscraper blocks to reach the Extraction Beacon.",
        "environmental_hazard": "mid_avenue_blast_door_blockade",
        "required_capability": "high_torque_4wd + 2-DOF street clearance",
        "environment_style": {
            "ground_color": [0.07, 0.08, 0.11],
            "road_color": [0.14, 0.16, 0.20],
            "sidewalk_color": [0.26, 0.29, 0.34],
            "sky_color": [0.05, 0.07, 0.12],
            "building_style": "cyber_skyscraper",
            "prop_style": "streetlamp",
            "has_center_dashes": True,
        },
        "car_design": {
            "body_style": "street_muscle",
            "primary_color": [0.15, 0.78, 0.98],
            "accent_color": [1.0, 0.45, 0.10],
            "wheel_scale": 1.0,
            "ride_height": 0.0,
            "roof_attachment": "m3_servo_arm",
        },
        "skin_prompt": f"GTA cyber-street rally car hood & chassis decal skin for '{user_prompt}', carbon fiber with neon cyan & orange racing stripes",
    }


class StudioDirectorAgent:
    """Uses Gemini 4 Argon / Managed Agents to scope a complete game plot, theme, car design & MuJoCo brief."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"

    def scope_brief(self, user_prompt: str, project_id: str, level_index: int = 1) -> Dict[str, Any]:
        """Translate a raw user prompt into a structured Studio Directive with plot, environment palette, car design & MuJoCo specs."""
        preset = infer_theme_preset(user_prompt, level_index=level_index)

        system_directive = (
            "You are the Studio Executive Director Agent at Gretchen Autonomous Game Studio.\n"
            "We are building a generative 3D street driving & physics-aware MuJoCo vehicle game where the player can request ANY world theme "
            "(e.g., medieval cobblestone kingdom, cyberpunk neon downtown, desert canyon convoy, arctic glacier outpost) and a compelling story plot.\n"
            "Given a client prompt, output ONLY valid JSON with keys:\n"
            '  "project_id": string,\n'
            '  "level_index": integer,\n'
            '  "game_title": string,\n'
            '  "plot_summary": string (2-3 exciting sentences setting up the story plot and why the road ahead is perilous),\n'
            '  "theme": string (e.g. "medieval_cobblestone_kingdom", "neon_downtown_avenue", "wasteland_convoy_canyon", "arctic_glacier_outpost"),\n'
            '  "objective": string,\n'
            '  "environmental_hazard": string,\n'
            '  "required_capability": string,\n'
            '  "world_bounds": {"x": 80.0, "z": 40.0},\n'
            '  "environment_style": {\n'
            '    "ground_color": [r, g, b],\n'
            '    "road_color": [r, g, b],\n'
            '    "sidewalk_color": [r, g, b],\n'
            '    "sky_color": [r, g, b],\n'
            '    "building_style": "medieval_keep" | "cyber_skyscraper" | "desert_mesa" | "arctic_dome",\n'
            '    "prop_style": "torch_post" | "streetlamp" | "cryo_pylon",\n'
            '    "has_center_dashes": boolean\n'
            '  },\n'
            '  "car_design": {\n'
            '    "body_style": "street_muscle" | "armored_crawler" | "medieval_wagon" | "cyber_interceptor",\n'
            '    "primary_color": [r, g, b],\n'
            '    "accent_color": [r, g, b],\n'
            '    "wheel_scale": 1.0,\n'
            '    "ride_height": 0.0,\n'
            '    "roof_attachment": "m3_servo_arm" | "battering_ram" | "twin_turbo"\n'
            '  },\n'
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
            f"\nClient Prompt: {user_prompt}\nProject ID: {project_id}\nLevel Index: {level_index}"
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
                    parsed.setdefault("level_index", level_index)
                    parsed.setdefault("game_title", preset["game_title"])
                    parsed.setdefault("plot_summary", preset["plot_summary"])
                    parsed.setdefault("environment_style", preset["environment_style"])
                    parsed.setdefault("car_design", preset["car_design"])
                    return parsed
            except Exception as exc:
                print(f"[DirectorAgent] Fallback triggered ({exc})")

        return {
            "project_id": project_id,
            "level_index": level_index,
            "game_title": preset["game_title"],
            "plot_summary": preset["plot_summary"],
            "theme": preset["theme"],
            "objective": preset["objective"],
            "environmental_hazard": preset["environmental_hazard"],
            "required_capability": preset["required_capability"],
            "world_bounds": {"x": 80.0, "z": 40.0},
            "environment_style": preset["environment_style"],
            "car_design": preset["car_design"],
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
            "skin_prompt": preset["skin_prompt"],
        }
