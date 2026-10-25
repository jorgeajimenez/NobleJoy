# Installation & Run Guide

## 1. Backend Setup (`backend/` — Gretchen Studio Service)

We use `uv` for Python environment and dependency management.

```bash
cd backend
cp .env.example .env
# Edit .env and set GEMINI_API_KEY=your_api_key_here

# Create environment and install dependencies with uv
uv venv
uv pip install -r requirements.txt

# Run the FastAPI Studio Gateway
uv run uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

## 2. Rust Game Client Setup (`game_client/` — Jorge Engine)

```bash
cd game_client
cargo build
cargo run
```

### Controls in the Rust Client
- **W / A / S / D** or **Arrow Keys**: Move the player entity
- **C**: Commission a new level from the AI Game Studio (`POST /api/commission-level`)
- **Q**: Capture viewport frame and trigger Visual QA Audit & Live Hotfix (`POST /api/qa-audit`)
- **R**: Reset player to spawn pad
