import asyncio
import sys
import os

# Ensure the orchestrator is in the python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "arkhe_full_100t_orchestrator_substrate_989_y_3"))

from api_gateway import APIGateway
from passport_gateway import PassportGateway
from full_100t_orchestrator import Full100TOrchestrator

class ArkheNode:
    def __init__(self, config_path: str = "config.yaml"):
        self.node_id = "demo-node"
        self.config = {"passport_enabled": True, "orchestrator_enabled": True}
        self.passport = PassportGateway()
        self.orchestrator = Full100TOrchestrator()
        self.api = APIGateway(node_id=self.node_id, passport=self.passport)

    async def start(self):
        if self.config.get("passport_enabled", True):
            await self.passport.start()
        if self.config.get("orchestrator_enabled", True):
            await self.orchestrator.run()
        await self.api.start_http_server()

if __name__ == "__main__":
    node = ArkheNode()
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    try:
        loop.run_until_complete(node.start())
        loop.run_forever()
    except KeyboardInterrupt:
        pass
    finally:
        if node.config.get("orchestrator_enabled", True):
            loop.run_until_complete(node.orchestrator.stop())
        if node.config.get("passport_enabled", True):
            loop.run_until_complete(node.passport.stop())
        loop.close()
