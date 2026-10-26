"""Register all 8 Argon Studio Game & Test Agents + MCP Server in Google Cloud Agent Registry (`agentregistry.googleapis.com`).

Usage:
    python3 backend/register_cloud_agent_registry.py
"""

import json
import subprocess

PROJECT = "gen-lang-client-0221638641"
BASE_URL = "https://argon-studio-backend-421717890494.us-central1.run.app"

AGENTS = [
    {
        "service_id": "studio-director-agent",
        "name": "studio_director",
        "display_name": "Studio Director Game Agent (Gemini 4 Argon)",
        "description": "Game Director Managed Agent: scopes story plot, 3D world theme (Medieval/Cyber/Desert/Arctic), Transformers morphology, MuJoCo physics targets, and COTS M3 hardware upgrades.",
        "skills": [
            {
                "id": "narrative_plot_scoping",
                "name": "Narrative & Plot Scoping",
                "description": "Scopes story plot, 3D world theme, and COTS M3 hardware reward part",
                "tags": ["game_agent", "gemini-4-argon", "director"],
            },
            {
                "id": "mujoco_physics_parameterization",
                "name": "MuJoCo Physics Parameterization",
                "description": "Defines chassis mass, wheel friction, motor torque, and servo torque targets",
                "tags": ["mujoco", "robotics", "physics"],
            },
        ],
    },
    {
        "service_id": "level-architect-agent",
        "name": "level_architect",
        "display_name": "Level Architect Managed Agent (Gemini 4 Argon)",
        "description": "Level Architect Managed Agent: runs inside Google-hosted Linux sandbox (/workspace) to build 3D street scene contracts, MuJoCo MJCF XML, ROS 2 URDF, and M3 STL meshes.",
        "skills": [
            {
                "id": "3d_street_layout_generation",
                "name": "3D Street Layout & ECS Generation",
                "description": "Generates 3D street corridor entities, buildings, chicanes, and goal beacons",
                "tags": ["game_agent", "gemini-4-argon", "architect"],
            },
            {
                "id": "mujoco_mjcf_urdf_stl_synthesis",
                "name": "MuJoCo MJCF, URDF & M3 STL Synthesis",
                "description": "Compiles car_level.xml, rover.urdf, and watertight M3 STL meshes in /workspace",
                "tags": ["mujoco", "urdf", "stl", "managed_agent"],
            },
        ],
    },
    {
        "service_id": "nano-banana-skin-painter",
        "name": "nano_banana_skin_painter",
        "display_name": "Nano Banana 2.1 Vehicle Skin Painter Agent",
        "description": "Nano Banana 2.1 Vehicle Skin Agent: synthesizes top-down 256x256 PBR vehicle chassis textures and streams them over HTTP to the Apple Silicon Rust/Bevy renderer.",
        "skills": [
            {
                "id": "pbr_vehicle_albedo_generation",
                "name": "PBR Vehicle Albedo Generation",
                "description": "Synthesizes UV-mapped chassis_skin.png textures for Bevy 3D vehicles and robots",
                "tags": ["game_agent", "gemini-nano-banana-2.1", "texture"],
            }
        ],
    },
    {
        "service_id": "simready-fetcher-agent",
        "name": "simready_fetcher",
        "display_name": "SimReady OpenUSD & URDF Asset Fetcher Agent",
        "description": "SimReady Asset Fetcher Game Agent: retrieves standardized OpenUSD/URDF/MuJoCo physics-ready 3D asset descriptors with validated mass, friction, and M3 mounting metadata.",
        "skills": [
            {
                "id": "simready_openusd_lookup",
                "name": "SimReady OpenUSD & M3 Lookup",
                "description": "Resolves validated FET_003_STANDARD physics profiles and M3 ISO 4762 mounting specs",
                "tags": ["game_agent", "gemini-3.8-flash", "simready", "openusd"],
            }
        ],
    },
    {
        "service_id": "usd-physics-modifier-agent",
        "name": "usd_physics_modifier",
        "display_name": "USD Physics Modifier & Physical Fab Exporter Agent",
        "description": "Admin-Gated Physics Modifier & Hardware Exporter Agent: applies custom UsdPhysics/MuJoCo overrides and exports printable M3 STLs + URDF + BOM.csv + MicroPython firmware.",
        "skills": [
            {
                "id": "usd_physics_layer_override",
                "name": "UsdPhysics & MuJoCo Override",
                "description": "Applies friction and MG996R 11.0 kg*cm servo torque overrides",
                "tags": ["game_agent", "gemini-4-argon", "admin_only", "usd_physics"],
            },
            {
                "id": "endgame_manufacturing_bundle_export",
                "name": "Physical Robot Fabrication Exporter",
                "description": "Exports M3 STLs, rover.urdf, rover.xml, BOM.csv, and ESP32-S3 MicroPython firmware",
                "tags": ["stl", "urdf", "bom", "micropython"],
            },
        ],
    },
    {
        "service_id": "visual-qa-tester-agent",
        "name": "visual_qa_tester",
        "display_name": "Visual QA Playtester Test Agent (Gemini 4 Argon)",
        "description": "Visual QA Playtester Test Agent (Track 2 Closed Verification Loop): inspects raw viewport PNG captures from the Rust Bevy engine, detects street blockades, and dispatches live ECS + MuJoCo hotfixes.",
        "skills": [
            {
                "id": "multimodal_viewport_png_inspection",
                "name": "Multimodal Viewport PNG Inspection",
                "description": "Audits rendered Bevy 3D frames against the level goal using Gemini 4 Argon vision",
                "tags": ["test_agent", "gemini-4-argon", "visual_qa"],
            },
            {
                "id": "live_ecs_and_mujoco_hotfix",
                "name": "Live ECS & MuJoCo Auto-Repair",
                "description": "Shifts blast_door onto sidewalk (z=6.5) and boosts drive torque to 6.5 N*m",
                "tags": ["test_agent", "hotfix", "closed_loop"],
            },
        ],
    },
    {
        "service_id": "gbuffer-artifact-validator",
        "name": "gbuffer_artifact_validator",
        "display_name": "WGPU Compute G-Buffer Artifact Validator Agent",
        "description": "Apple Silicon Metal / WGPU Compute Test Agent: evaluates G-Buffer depth, normal, instance ID, and shadow textures via artifact_detector.wgsl to catch Z-Fighting and Shadow Acne.",
        "skills": [
            {
                "id": "wgpu_metal_compute_gbuffer_audit",
                "name": "WGPU Metal Compute G-Buffer Audit",
                "description": "Executes 8x8 workgroup WGSL compute shader to detect coplanar Z-fighting and shadow acne",
                "tags": ["test_agent", "gemini-3.8-flash", "wgpu", "metal"],
            }
        ],
    },
    {
        "service_id": "mujoco-sanity-verifier",
        "name": "mujoco_sanity_verifier",
        "display_name": "MuJoCo Robotics Physics Sanity Verifier Agent",
        "description": "High-Frequency Robotics Physics Test Agent: checks Center-of-Mass (CoM) polygon equilibrium inside the 4-wheel footprint and evaluates tau = r x F servo torque limits.",
        "skills": [
            {
                "id": "center_of_mass_and_torque_gate",
                "name": "CoM Stability & Servo Torque Gate",
                "description": "Verifies CoM < 0.45m wheelbase and MG996R 11.0 kg*cm vs SG90 1.8 kg*cm torque limits",
                "tags": ["test_agent", "gemini-3.5-flash-lite", "mujoco", "torque"],
            }
        ],
    },
]


def main() -> None:
    for loc in ["global", "us-central1"]:
        for ag in AGENTS:
            card = {
                "protocolVersion": "0.3.0",
                "name": ag["name"],
                "description": ag["description"],
                "url": f"{BASE_URL}/api/v1/agents/{ag['name']}/execute",
                "version": "1.0.0",
                "capabilities": {"streaming": False},
                "defaultInputModes": ["application/json", "text/plain"],
                "defaultOutputModes": ["application/json"],
                "skills": ag["skills"],
            }
            cmd = [
                "gcloud",
                "alpha",
                "agent-registry",
                "services",
                "create",
                ag["service_id"],
                f"--location={loc}",
                f"--project={PROJECT}",
                f"--display-name={ag['display_name']}",
                f"--description={ag['description']}",
                "--agent-spec-type=a2a-agent-card",
                f"--agent-spec-content={json.dumps(card)}",
                "--async",
            ]
            res = subprocess.run(cmd, capture_output=True, text=True)
            print(f"[{loc}] {ag['service_id']}: code={res.returncode}")


if __name__ == "__main__":
    main()
