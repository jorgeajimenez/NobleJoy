"""Managed Agent Sandbox Artifact Puller & Skin/Texture Generator.

Implements Pattern 1 from Gretchen's Architecture Notes:
1. Pulls generated files (.xml MJCF/URDF, .stl/.obj meshes, .png skins/textures, .json contracts)
   from the Managed Agent's remote Linux microVM via `interaction.environment_id`:
   GET https://generativelanguage.googleapis.com/v1beta/files/environment-{env_id}:download?alt=media
2. Generates custom car/rover skins & track textures using `gemini-nano-banana-2.1`
   (or `gemini-3.1-flash-lite-image`) and saves them locally for the engine to load.
"""

import base64
import io
import os
from pathlib import Path
import tarfile
from typing import Dict, List, Optional
import urllib.request


GENERATED_ASSETS_DIR = Path(__file__).parent.parent / "assets" / "generated"
GENERATED_ASSETS_DIR.mkdir(parents=True, exist_ok=True)


def pull_managed_agent_workspace(
    environment_id: Optional[str],
    project_id: str,
    api_key: Optional[str] = None,
) -> List[str]:
    """Download and extract the `/workspace` .tar archive from a Gemini Managed Agent sandbox."""
    key = api_key or os.getenv("GEMINI_API_KEY")
    target_dir = GENERATED_ASSETS_DIR / project_id
    target_dir.mkdir(parents=True, exist_ok=True)

    if not environment_id or not key:
        return []

    url = (
        f"https://generativelanguage.googleapis.com/v1beta/files/"
        f"environment-{environment_id}:download?alt=media"
    )
    req = urllib.request.Request(url, headers={"x-goog-api-key": key})
    extracted_files: List[str] = []

    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            tar_bytes = resp.read()
        with tarfile.open(fileobj=io.BytesIO(tar_bytes), mode="r:*") as tar:
            # Safe extraction into project asset folder
            for member in tar.getmembers():
                if member.isfile() and not member.name.startswith(("/", "..")):
                    tar.extract(member, path=target_dir)
                    extracted_files.append(str(target_dir / member.name))
        print(f"[ArtifactPuller] Extracted {len(extracted_files)} files from env {environment_id}")
    except Exception as exc:
        print(f"[ArtifactPuller] Workspace download skipped/failed ({exc})")

    return extracted_files


def generate_vehicle_skin_texture(
    prompt: str,
    project_id: str,
    skin_name: str = "chassis_skin.png",
    api_key: Optional[str] = None,
) -> Dict[str, str]:
    """Generate a UV/albedo skin texture for the MuJoCo car/rover using Nano Banana 2.1."""
    key = api_key or os.getenv("GEMINI_API_KEY")
    target_dir = GENERATED_ASSETS_DIR / project_id
    target_dir.mkdir(parents=True, exist_ok=True)
    out_path = target_dir / skin_name

    if key:
        try:
            from google import genai

            client = genai.Client(api_key=key)
            interaction = client.interactions.create(
                model="gemini-nano-banana-2.1",
                input=(
                    f"Create a seamless top-down game vehicle chassis texture / decal sheet for: {prompt}. "
                    "High contrast, clean PBR albedo texture, 1:1 square."
                ),
            )
            # Extract image output if returned by Interactions API
            for item in getattr(interaction, "outputs", []) or []:
                if getattr(item, "mime_type", "").startswith("image/"):
                    raw_bytes = base64.b64decode(item.data)
                    out_path.write_bytes(raw_bytes)
                    return {
                        "skin_name": skin_name,
                        "local_path": str(out_path),
                        "url": f"/assets/generated/{project_id}/{skin_name}",
                    }
        except Exception as exc:
            print(f"[SkinGenerator] Nano Banana fallback triggered ({exc})")

    # Procedural fallback PNG texture using Pillow so the engine always receives a valid PNG skin
    from PIL import Image, ImageDraw

    img = Image.new("RGBA", (256, 256), (24, 34, 48, 255))
    draw = ImageDraw.Draw(img)
    # Racing stripes & cyber chassis grid
    draw.rectangle([0, 0, 255, 255], outline=(0, 210, 255, 255), width=6)
    draw.rectangle([108, 0, 148, 255], fill=(255, 90, 30, 255))
    draw.line([0, 128, 255, 128], fill=(0, 255, 140, 200), width=4)
    img.save(out_path, format="PNG")

    return {
        "skin_name": skin_name,
        "local_path": str(out_path),
        "url": f"/assets/generated/{project_id}/{skin_name}",
    }
