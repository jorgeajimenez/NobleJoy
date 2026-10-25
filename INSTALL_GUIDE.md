# Installation & Run Guide

## 1. Backend Setup (`backend/` — Gretchen Studio & MuJoCo Physics Service)

We use `uv` for Python environment and dependency management.

```bash
cd backend
cp .env.example .env
# Edit .env and set GEMINI_API_KEY=your_api_key_here

# Create environment and install dependencies (FastAPI, google-genai, mujoco, pillow) with uv
uv venv
uv pip install -r requirements.txt

# Run the FastAPI Studio & MuJoCo Gateway
uv run uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

## 2. Rust Game Client Setup (`game_client/` — Jorge Bevy 3D Driving Engine)

```bash
cd game_client
cargo build
cargo run
```

### Controls in the Rust Driving Client
- **W / S** or **Up / Down**: Throttle / Reverse (stepped through MuJoCo physics)
- **A / D** or **Left / Right**: Steer Rover Wheels
- **C**: Commission a new driving level + vehicle skin + MuJoCo MJCF/URDF from the Managed Agent (`POST /api/commission-level`)
- **Q**: Capture viewport PNG and trigger Visual QA + MuJoCo Physics Hotfix (`POST /api/qa-audit`)
- **U**: Snap unlocked M3 Modular Part onto the rover chassis & run the MuJoCo Center-of-Mass / Torque ($\tau = r \times F$) Sanity Check
- **E**: Export the Endgame Manufacturing Package (`exports/{project_id}/`: M3 STLs + `rover.urdf` + `BOM.csv` + MicroPython `firmware/main.py`)
- **R**: Reset rover to spawn pad
