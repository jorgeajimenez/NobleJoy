"""Central Agent Registry for Argon Studio (NobleJoy) — SimReady & Managed Agent Gateway.

Mirrors `backend/agents/registry.py` so both `simready_agent_gateway` and `backend`
expose the exact same 8 registered Game Agents (`game_agent`) and Test Agents (`test_agent`),
A2A Identity Cards, SPIFFE IDs, and Policy specs.
"""

from typing import Any, Dict, List, Optional


class AgentRegistry:
    AGENT_ALIASES: Dict[str, str] = {
        "director_agent": "studio_director",
        "director": "studio_director",
        "architect_agent": "level_architect",
        "architect": "level_architect",
        "skin_painter_agent": "nano_banana_skin_painter",
        "skin_agent": "nano_banana_skin_painter",
        "simready_fetcher_agent": "simready_fetcher",
        "physical_fab_exporter": "usd_physics_modifier",
        "fab_exporter": "usd_physics_modifier",
        "qa_tester_agent": "visual_qa_tester",
        "qa_agent": "visual_qa_tester",
        "gbuffer_validator_agent": "gbuffer_artifact_validator",
        "gbuffer_validator": "gbuffer_artifact_validator",
        "mujoco_verifier_agent": "mujoco_sanity_verifier",
        "mujoco_verifier": "mujoco_sanity_verifier",
    }

    def __init__(self) -> None:
        self._agents: Dict[str, Any] = {}

    def register(self, agent_class):
        instance = agent_class()
        self._agents[instance.name] = instance
        print(
            f"[AgentRegistry] Registered {getattr(instance, 'category', 'game_agent').upper()}: "
            f"{instance.name} ({getattr(instance, 'spiffe_id', '')})"
        )
        return agent_class

    def get_agent(self, name: str):
        canonical = self.AGENT_ALIASES.get(name, name)
        return self._agents.get(canonical)

    def list_agents(self, category: Optional[str] = None) -> List[str]:
        if category:
            return [
                name
                for name, ag in self._agents.items()
                if getattr(ag, "category", "game_agent") == category
            ]
        return list(self._agents.keys())

    def get_catalog(self, category: Optional[str] = None) -> Dict[str, Any]:
        cards = [
            ag.to_a2a_card() if hasattr(ag, "to_a2a_card") else {"name": ag.name}
            for ag in self._agents.values()
            if category is None or getattr(ag, "category", "game_agent") == category
        ]
        return {
            "studio": "Argon Game Studio (NobleJoy)",
            "cloud_run_url": "https://argon-studio-backend-421717890494.us-central1.run.app",
            "total_registered": len(cards),
            "game_agents": [c for c in cards if c.get("category") == "game_agent"],
            "test_agents": [c for c in cards if c.get("category") == "test_agent"],
        }


# Singleton registry instance
agent_registry = AgentRegistry()
