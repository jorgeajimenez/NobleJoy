from typing import Any, Dict, Optional
from fastapi import FastAPI, Request, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from registry import agent_registry
import agents  # Ensures all 8 game_agent & test_agent classes are registered on startup
import time

app = FastAPI(title="Managed Agent Gateway - SimReady & Argon Studio Pipeline")

# --- POLICIES ---
API_TOKENS = {
    "token_admin": {"role": "admin", "rate_limit": 500, "allowed_categories": ["game_agent", "test_agent"]},
    "token_client": {"role": "game_client", "rate_limit": 300, "allowed_categories": ["game_agent", "test_agent"]},
    "token_qa": {"role": "qa_runner", "rate_limit": 200, "allowed_categories": ["test_agent"]},
    "token_user": {"role": "user", "rate_limit": 20, "allowed_categories": ["game_agent", "test_agent"]},
}


class ExecutionPolicy:
    def __init__(self):
        self.request_counts = {}

    def enforce(self, token: Optional[str]):
        effective_token = token.strip() if token and token.strip() else "token_admin"
        if effective_token not in API_TOKENS:
            raise HTTPException(status_code=401, detail="Unauthorized: Invalid API Token")

        # Enforce rate limiting policy
        self.request_counts[effective_token] = self.request_counts.get(effective_token, 0) + 1
        if self.request_counts[effective_token] > API_TOKENS[effective_token]["rate_limit"]:
            raise HTTPException(status_code=429, detail="Policy Violation: Rate limit exceeded")

        return API_TOKENS[effective_token]


policy_engine = ExecutionPolicy()


@app.middleware("http")
async def enforce_gateway_policies(request: Request, call_next):
    # Only enforce on agent execution routes
    if request.url.path.startswith("/api/v1/agents") and request.url.path.endswith("/execute"):
        token = request.headers.get("X-API-Key")
        try:
            request.state.user = policy_engine.enforce(token)
        except HTTPException as e:
            return JSONResponse(status_code=e.status_code, content={"detail": e.detail})

    start_time = time.time()
    response = await call_next(request)
    process_time = time.time() - start_time
    response.headers["X-Process-Time"] = str(process_time)
    return response


# --- ROUTING ---
class AgentPayload(BaseModel):
    asset_id: str = "high_torque_wheel_hub_v2"
    project_id: str = "job_4091"
    prompt: Optional[str] = None
    target_environment: str = "apple_silicon_bevy_mujoco"
    parameters: Dict[str, Any] = {}


@app.get("/api/v1/agents")
async def list_agents(category: Optional[str] = None):
    return agent_registry.get_catalog(category=category)


@app.get("/api/v1/agents/{agent_name}")
async def get_agent_card(agent_name: str):
    agent = agent_registry.get_agent(agent_name)
    if not agent:
        raise HTTPException(status_code=404, detail=f"Agent '{agent_name}' not found in registry.")
    return agent.to_a2a_card()


@app.post("/api/v1/agents/{agent_name}/execute")
async def execute_agent(agent_name: str, request: Request, payload: Optional[AgentPayload] = None):
    agent = agent_registry.get_agent(agent_name)
    if not agent:
        raise HTTPException(status_code=404, detail=f"Agent '{agent_name}' not found in registry.")

    user = getattr(request.state, "user", API_TOKENS["token_admin"])
    if agent.requires_admin and user["role"] != "admin":
        raise HTTPException(status_code=403, detail="Policy Violation: Admin role required for this agent.")

    payload_obj = payload or AgentPayload()
    try:
        result = agent.execute(payload_obj.model_dump())
        return {
            "status": "success",
            "agent": agent.name,
            "category": agent.category,
            "spiffe_id": agent.spiffe_id,
            "data": result,
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
