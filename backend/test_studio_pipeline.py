"""Standalone verification script for Gretchen's Backend Pipeline.

Run with:
    cd backend
    uv run python test_studio_pipeline.py
"""

import io
import json
from pathlib import Path
from PIL import Image

from agents import LevelArchitectAgent, StudioDirectorAgent, VisualQATesterAgent
from agents.artifact_puller import export_physical_build_package
from mujoco_sim import MujocoDrivingSession


def main() -> None:
    project_id = "job_gretchen_test"
    prompt = "Design a slag-heap rover track with a low clearance gap and high-torque wheel upgrade."

    manifest_path = Path(__file__).parent / "assets" / "asset_manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    print("1. [Director Agent] Scoping brief...")
    director = StudioDirectorAgent()
    directive = director.scope_brief(prompt, project_id=project_id)
    print("   Directive:", json.dumps(directive, indent=2))

    print("\n2. [Architect Agent + Artifact Puller] Building Level Contract & Pulling Assets...")
    architect = LevelArchitectAgent()
    contract = architect.build_level_contract(directive, manifest, seed_blockade=True)
    print("   Generated Skin URL:", contract["skin_url"])
    print("   Generated MJCF URL:", contract["mjcf_url"])
    print("   Generated URDF URL:", contract["urdf_url"])
    print("   Sandbox/Local Files:", contract["sandbox_files"])

    print("\n3. [MuJoCo Driving Simulation] Initializing & Stepping 4-Wheel Rover...")
    sim = MujocoDrivingSession(
        project_id=project_id,
        car_spec=contract["car_physics"],
        level_entities=contract["entities"],
    )
    step_1 = sim.step(throttle=1.0, steering=0.15, equip_upgrade=False)
    print("   Step (Stock Rover):", json.dumps(step_1, indent=2))

    step_2 = sim.step(throttle=1.0, steering=0.0, equip_upgrade=True)
    print("   Step (Upgraded M3 Wheel Hub Equipped):", json.dumps(step_2, indent=2))

    print("\n4. [Visual QA Playtester Agent] Auditing Viewport PNG & Patching Blockade + MuJoCo Physics...")
    dummy_img = Image.new("RGB", (320, 180), (30, 40, 55))
    buf = io.BytesIO()
    dummy_img.save(buf, format="PNG")
    qa = VisualQATesterAgent()
    qa_report = qa.audit_viewport(
        project_id=project_id,
        image_bytes=buf.getvalue(),
        current_scene=contract,
    )
    print("   QA Report:", json.dumps(qa_report, indent=2))

    print("\n5. [Physical Export Pipeline] Generating Printable STLs + URDF + BOM.csv + MicroPython Firmware...")
    export_bundle = export_physical_build_package(
        project_id=project_id,
        car_spec=contract["car_physics"],
        reward_part=contract["reward_part"],
    )
    print("   Export Bundle:", json.dumps(export_bundle, indent=2))
    print("\nALL BACKEND CHECKS PASSED!")


if __name__ == "__main__":
    main()
