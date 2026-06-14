import aiohttp
from typing import Optional, Dict, Any, List
import hashlib
import time

class ZeroExTradingModule:
    """
    0x Trading Module for the Cathedral AGI.
    Implements the Fast Brain execution logic for high-frequency trading via 0x API.
    """
    def __init__(self, api_key: str, chain_id: int, wallet_address: str, zvec_memory: Any, world_model: Optional[Any] = None):
        self.api_key = api_key
        self.chain_id = chain_id
        self.wallet = wallet_address
        self.zvec = zvec_memory
        self.world_model = world_model
        self.base_url = "https://api.0x.org/swap/allowance-holder"
        self.session = aiohttp.ClientSession()

    async def execute_swap(self, sell_token: str, buy_token: str, sell_amount: int, slippage_bps: int = 100) -> Optional[Dict[str, Any]]:
        """
        Executes a swap via 0x Swap API v2.
        Includes security validation and memory persistence.
        """
        params = {
            "chainId": self.chain_id,
            "sellToken": sell_token,
            "buyToken": buy_token,
            "sellAmount": sell_amount,
            "taker": self.wallet,
            "slippageBps": slippage_bps,
        }
        headers = {"0x-api-key": self.api_key, "0x-version": "v2"}

        async with self.session.get(f"{self.base_url}/quote", params=params, headers=headers) as resp:
            if resp.status != 200:
                return None
            quote = await resp.json()

        # 1. Security Validation (Z3) - Example condition
        if "issues" in quote and quote["issues"].get("liquidityAvailable") is False:
            return None

        # 2. Token Approval (AllowanceHolder)
        allowance_params = {
            "chainId": self.chain_id,
            "sellToken": sell_token,
            "taker": self.wallet,
            "sellAmount": sell_amount,
        }
        async with self.session.get(f"{self.base_url}/approval", params=allowance_params, headers=headers) as resp:
            approval = await resp.json()

        # 3. Transaction Submission (simplified)
        tx_hash = await self._send_transaction(quote.get("transaction", {}))
        result = {"tx_hash": tx_hash, "buy_amount": quote.get("buyAmount", 0)}

        # 4. Post-processing: Memory and Personality Update
        embedding = await self._get_market_embedding(sell_token, buy_token, sell_amount, quote)

        if hasattr(self.zvec, "store_transaction_embedding"):
            self.zvec.store_transaction_embedding(embedding, result)

        reward = self._calc_reward(result)

        if self.world_model and hasattr(self.world_model, "update_personality_from_reward"):
            self.world_model.update_personality_from_reward(reward)

        return result

    async def _send_transaction(self, transaction_data: Dict[str, Any]) -> str:
        """
        Simulates sending a transaction and returning a mock transaction hash.
        """
        tx_id = hashlib.sha3_256(f"tx-{time.time()}".encode()).hexdigest()[:16]
        return f"0x{tx_id}"

    async def _get_market_embedding(self, sell_token: str, buy_token: str, sell_amount: int, quote: Dict[str, Any]) -> List[float]:
        """
        Simulates getting a market embedding.
        """
        return [0.1, 0.2, 0.3, 0.4]

    def _calc_reward(self, result: Dict[str, Any]) -> float:
        """
        Simulates calculating a reward from the transaction result.
        """
        try:
            buy_amount = float(result.get("buy_amount", 0))
        except (ValueError, TypeError):
            buy_amount = 0.0

        if buy_amount > 0:
            return 1.0
        return -1.0

    async def close(self):
        """Closes the underlying aiohttp session."""
        if self.session and hasattr(self.session, "closed") and not self.session.closed:
            await self.session.close()
