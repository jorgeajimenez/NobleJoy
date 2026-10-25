"""Studio Director Agent: Scopes game plot, world theme (Medieval, Cyber, Desert, Arctic, etc.), car/robot design, and MuJoCo physics."""

import json
import os
from typing import Any, Dict


def infer_robot_form(user_prompt: str) -> str:
    """Detect if the user is requesting a Transformers-style MuJoCo robot morphology swap."""
    p = user_prompt.lower()
    if any(k in p for k in ["mech", "biped", "transformer", "optimus", "walker", "humanoid", "gundam", "striker"]):
        return "bipedal_mech"
    if any(k in p for k in ["spider", "quadruped", "dog", "spot", "crawler_legs", "insect", "arachnid"]):
        return "quadruped_crawler"
    if any(k in p for k in ["tank", "tread", "tracked", "panzer", "artillery", "behemoth"]):
        return "siege_tank"
    return "rover_car"


def infer_theme_preset(user_prompt: str, level_index: int = 1) -> Dict[str, Any]:
    """Infer a rich world theme, plot, environment palette, soundtrack, and car/robot design from the user's prompt."""
    p = user_prompt.lower()
    robot_form = infer_robot_form(user_prompt)

    if any(k in p for k in ["medieval", "castle", "knight", "kingdom", "cobble", "dragon", "tavern", "fantasy", "sword"]):
        preset = {
            "theme": "medieval_cobblestone_kingdom",
            "game_title": "Chronicles of Iron & Cobble",
            "plot_summary": (
                f"Chapter {level_index}: Lord Malakor has lowered the Iron Portcullis across King's Cobblestone Road. "
                "Pilot the Royal Siege Wagon past timber taverns and stone castle keeps, breach the blockade, "
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
                "soundtrack_style": "Medieval Tavern Lute & War Drums",
                "has_center_dashes": False,
            },
            "car_design": {
                "body_style": "medieval_wagon",
                "robot_form": robot_form,
                "primary_color": [0.58, 0.36, 0.16],
                "accent_color": [0.85, 0.68, 0.20],
                "wheel_scale": 1.15,
                "ride_height": 0.04,
                "roof_attachment": "battering_ram",
            },
            "skin_prompt": f"Medieval royal war wagon oak wood and forged Damascus iron armor texture with gold heraldic trim for '{user_prompt}'",
        }
    elif any(k in p for k in ["desert", "mars", "canyon", "wasteland", "dune", "sand", "mad", "rust"]):
        preset = {
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
                "soundtrack_style": "Fury Canyon Industrial Bass Pulse",
                "has_center_dashes": False,
            },
            "car_design": {
                "body_style": "armored_crawler",
                "robot_form": robot_form,
                "primary_color": [0.82, 0.42, 0.15],
                "accent_color": [0.22, 0.22, 0.24],
                "wheel_scale": 1.30,
                "ride_height": 0.08,
                "roof_attachment": "m3_servo_arm",
            },
            "skin_prompt": f"Weathered desert wasteland off-road crawler armor plating with hazard stripes for '{user_prompt}'",
        }
    elif any(k in p for k in ["arctic", "ice", "snow", "glacier", "frozen", "cryo", "winter"]):
        preset = {
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
                "soundtrack_style": "Subzero Crystalline Ambient Synth",
                "has_center_dashes": True,
            },
            "car_design": {
                "body_style": "cyber_interceptor",
                "robot_form": robot_form,
                "primary_color": [0.92, 0.96, 1.0],
                "accent_color": [0.10, 0.78, 0.98],
                "wheel_scale": 1.20,
                "ride_height": 0.05,
                "roof_attachment": "twin_turbo",
            },
            "skin_prompt": f"Arctic exploration rover white carbon composite with cyan thermal conduits for '{user_prompt}'",
        }
    else:
        preset = {
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
                "soundtrack_style": "Cyberpunk 2099 Synthwave Arpeggio",
                "has_center_dashes": True,
            },
            "car_design": {
                "body_style": "street_muscle",
                "robot_form": robot_form,
                "primary_color": [0.15, 0.78, 0.98],
                "accent_color": [1.0, 0.45, 0.10],
                "wheel_scale": 1.0,
                "ride_height": 0.0,
                "roof_attachment": "m3_servo_arm",
            },
            "skin_prompt": f"GTA cyber-street rally car hood & chassis decal skin for '{user_prompt}', carbon fiber with neon cyan & orange racing stripes",
        }

    # Apply prompt-driven paint/upgrade overrides on top of the theme
    if "gold" in p:
        preset["car_design"]["primary_color"] = [0.92, 0.75, 0.18]
        preset["car_design"]["accent_color"] = [0.98, 0.92, 0.55]
    elif "red" in p or "crimson" in p:
        preset["car_design"]["primary_color"] = [0.88, 0.14, 0.18]
        preset["car_design"]["accent_color"] = [0.95, 0.95, 0.98]
    elif "stealth" in p or "obsidian" in p or "black" in p:
        preset["car_design"]["primary_color"] = [0.12, 0.13, 0.16]
        preset["car_design"]["accent_color"] = [0.10, 0.95, 0.55]

    if "ram" in p:
        preset["car_design"]["roof_attachment"] = "battering_ram"
    elif "turbo" in p or "boost" in p:
        preset["car_design"]["roof_attachment"] = "twin_turbo"

    if "monster" in p or "lifted" in p or "big wheel" in p:
        preset["car_design"]["wheel_scale"] = 1.35
        preset["car_design"]["ride_height"] = 0.09

    return preset


class StudioDirectorAgent:
    """Uses Gemini 4 Argon / Managed Agents (or configurable execution mode) to scope game plot, theme, car/robot design & MuJoCo brief."""

    def __init__(self) -> None:
        self.api_key = os.getenv("GEMINI_API_KEY")
        self.exec_mode = os.getenv("STUDIO_EXECUTION_MODE", "auto").lower()
        self.use_managed_agent = os.getenv("USE_MANAGED_AGENT", "true").lower() == "true"
        self.model_id = os.getenv("GEMINI_MODEL_ID", "gemini-4-argon")
        self.managed_agent_id = os.getenv("MANAGED_AGENT_ID", "antigravity-preview-09-2026")
        self.managed_agent_type = os.getenv("MANAGED_AGENT_TYPE", "antigravity")

    def scope_brief(self, user_prompt: str, project_id: str, level_index: int = 1) -> Dict[str, Any]:
        """Translate a raw user prompt into a structured Studio Directive with plot, environment palette, robot form & MuJoCo specs."""
        preset = infer_theme_preset(user_prompt, level_index=level_index)
        robot_form = preset["car_design"].get("robot_form", "rover_car")

        system_directive = (
            "You are the Studio Executive Director Agent at Gretchen Autonomous Game Studio.\n"
            "We are building a generative 3D street driving & physics-aware MuJoCo vehicle/robot game where the player can request ANY world theme "
            "(e.g., medieval cobblestone kingdom, cyberpunk neon downtown, desert canyon convoy, arctic glacier outpost), upgrades/skins, "
            "AND Transformers-style MuJoCo robot morphs ('rover_car', 'bipedal_mech', 'quadruped_crawler', 'siege_tank').\n"
            "Given a client prompt, output ONLY valid JSON with keys:\n"
            '  "project_id": string,\n'
            '  "level_index": integer,\n'
            '  "game_title": string,\n'
            '  "plot_summary": string,\n'
            '  "theme": string,\n'
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
            '    "soundtrack_style": string,\n'
            '    "has_center_dashes": boolean\n'
            '  },\n'
            '  "car_design": {\n'
            '    "body_style": "street_muscle" | "armored_crawler" | "medieval_wagon" | "cyber_interceptor",\n'
            '    "robot_form": "rover_car" | "bipedal_mech" | "quadruped_crawler" | "siege_tank",\n'
            '    "primary_color": [r, g, b],\n'
            '    "accent_color": [r, g, b],\n'
            '    "wheel_scale": 1.0,\n'
            '    "ride_height": 0.0,\n'
            '    "roof_attachment": "m3_servo_arm" | "battering_ram" | "twin_turbo"\n'
            '  },\n'
            '  "car_physics": {\n'
            '    "robot_form": "rover_car" | "bipedal_mech" | "quadruped_crawler" | "siege_tank",\n'
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

        if (
            self.exec_mode != "local_fast"
            and self.api_key
            and self.api_key not in ("your_gemini_key", "your_gemini_api_key_here")
        ):
            try:
                from google import genai

                client = genai.Client(api_key=self.api_key)
                use_remote = self.exec_mode == "managed_agent" or (
                    self.exec_mode == "auto" and self.use_managed_agent
                )
                if use_remote:
                    interaction = client.interactions.create(
                        agent=self.managed_agent_id,
                        agent_config={
                            "type": self.managed_agent_type,
                            "model": self.model_id,
                        },
                        input=system_directive,
                        environment="remote",
                    )
                else:
                    interaction = client.interactions.create(
                        model=self.model_id,
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
                    parsed["car_design"].setdefault("robot_form", robot_form)
                    parsed["car_physics"].setdefault("robot_form", robot_form)
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
                "robot_form": robot_form,
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
