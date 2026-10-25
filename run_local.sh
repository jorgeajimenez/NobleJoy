#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
CLIENT_DIR="$ROOT_DIR/game_client"
STUDIO_PORT="${STUDIO_PORT:-8000}"
BACKEND_PID=""

cleanup() {
  if [[ -n "$BACKEND_PID" ]] && kill -0 "$BACKEND_PID" 2>/dev/null; then
    echo ""
    echo "[Orchestrator] Shutting down Gretchen Studio Backend (PID $BACKEND_PID)..."
    kill "$BACKEND_PID" 2>/dev/null || true
    wait "$BACKEND_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "=================================================================="
echo "  ARGON GAME STUDIO — Local End-to-End Orchestrator"
echo "  Gretchen (Gemini 4 Argon + MuJoCo) <-> Jorge (Rust Bevy GTA View)"
echo "=================================================================="

# 1. Load .env variables from root .env or backend/.env
if [[ -f "$ROOT_DIR/.env" ]]; then
  echo "[Orchestrator] Loading environment from $ROOT_DIR/.env"
  set -a
  # shellcheck disable=SC1091
  source "$ROOT_DIR/.env"
  set +a
  # Mirror to backend/.env if not present
  if [[ ! -f "$BACKEND_DIR/.env" ]]; then
    cp "$ROOT_DIR/.env" "$BACKEND_DIR/.env"
  fi
elif [[ -f "$BACKEND_DIR/.env" ]]; then
  echo "[Orchestrator] Loading environment from $BACKEND_DIR/.env"
  set -a
  # shellcheck disable=SC1091
  source "$BACKEND_DIR/.env"
  set +a
else
  echo "[Orchestrator] WARNING: No .env found in root or backend/."
  echo "               Copy .env_sample to .env and set GEMINI_API_KEY for live Gemini 4 Argon calls."
fi

EXEC_MODE="${STUDIO_EXECUTION_MODE:-auto}"
echo "[Orchestrator] STUDIO_EXECUTION_MODE=${EXEC_MODE} (options: auto | local_fast | gemini_direct | managed_agent)"

if [[ -n "${GEMINI_API_KEY:-}" && "${GEMINI_API_KEY}" != "your_gemini_key" && "${GEMINI_API_KEY}" != "your_gemini_api_key_here" ]]; then
  echo "[Orchestrator] GEMINI_API_KEY detected (${GEMINI_API_KEY_NAME:-active}). Live Generative Studio enabled!"
else
  echo "[Orchestrator] GEMINI_API_KEY not set or placeholder — backend will run in deterministic local_fast studio mode."
fi

# 2. Ensure Python uv virtual environment & dependencies are ready
echo "[Orchestrator] Preparing Python backend environment with uv..."
cd "$BACKEND_DIR"
if [[ ! -d ".venv" ]]; then
  uv venv
fi
uv pip install -q -r requirements.txt

# 3. Start Gretchen Studio + MuJoCo Backend in background
echo "[Orchestrator] Starting Gretchen Studio Backend on http://127.0.0.1:${STUDIO_PORT} ..."
uv run uvicorn main:app --host 127.0.0.1 --port "$STUDIO_PORT" > "$ROOT_DIR/backend_studio.log" 2>&1 &
BACKEND_PID=$!

# Wait for /health endpoint
echo -n "[Orchestrator] Waiting for backend health check"
for _ in {1..30}; do
  if curl -fsS "http://127.0.0.1:${STUDIO_PORT}/health" >/dev/null 2>&1; then
    echo " -> READY!"
    break
  fi
  echo -n "."
  sleep 0.5
done

ACTIVE_BACKEND_URL="${STUDIO_BACKEND_URL:-http://127.0.0.1:${STUDIO_PORT}}"
echo "[Orchestrator] Web Dashboard: http://127.0.0.1:${STUDIO_PORT}/"
echo "[Orchestrator] Active Client Endpoint: ${ACTIVE_BACKEND_URL}"
echo "[Orchestrator] Backend Logs:  $ROOT_DIR/backend_studio.log"
echo "[Orchestrator] Launching Jorge Rust 3D GTA Street Driving Client..."
echo "=================================================================="

cd "$CLIENT_DIR"
STUDIO_BACKEND_URL="${ACTIVE_BACKEND_URL}" cargo run
