import pytest
from unittest.mock import MagicMock, AsyncMock, patch
from arkhe_autonomous_economic_agent_substrate_980.zeroex_trading_module import ZeroExTradingModule

class MagicMockSession:
    """
    Custom mock class to correctly mock aiohttp.ClientSession as an async context manager.
    """
    def __init__(self, responses=None):
        self.responses = responses or []
        self.call_count = 0

    def get(self, url, **kwargs):
        class AsyncContextManager:
            def __init__(self, response):
                self.response = response

            async def __aenter__(self):
                return self.response

            async def __aexit__(self, exc_type, exc, tb):
                pass

        if self.call_count < len(self.responses):
            response = self.responses[self.call_count]
            self.call_count += 1
            return AsyncContextManager(response)

        # Default empty successful response if not enough responses provided
        mock_resp = AsyncMock()
        mock_resp.status = 200
        mock_resp.json = AsyncMock(return_value={})
        return AsyncContextManager(mock_resp)

    async def close(self):
        pass


@pytest.fixture
def mock_zvec():
    zvec = MagicMock()
    zvec.store_transaction_embedding = MagicMock()
    return zvec

@pytest.fixture
def mock_world_model():
    model = MagicMock()
    model.update_personality_from_reward = MagicMock()
    return model

@pytest.mark.asyncio
async def test_execute_swap_success(mock_zvec, mock_world_model):
    # Setup mock responses for quote and approval
    quote_resp = AsyncMock()
    quote_resp.status = 200
    quote_resp.json = AsyncMock(return_value={
        "buyAmount": "1000",
        "transaction": {"to": "0x123", "data": "0xabc"}
    })

    approval_resp = AsyncMock()
    approval_resp.status = 200
    approval_resp.json = AsyncMock(return_value={})

    # Initialize module with our custom session mock
    module = ZeroExTradingModule("test_api_key", 1, "0xWallet", mock_zvec, mock_world_model)
    module.session = MagicMockSession(responses=[quote_resp, approval_resp])

    result = await module.execute_swap("USDC", "WETH", 500)

    # Assertions
    assert result is not None
    assert "tx_hash" in result
    assert result["buy_amount"] == "1000"

    # Check if memory persistence was called
    mock_zvec.store_transaction_embedding.assert_called_once()
    mock_world_model.update_personality_from_reward.assert_called_once_with(1.0)

    await module.close()

@pytest.mark.asyncio
async def test_execute_swap_failed_quote(mock_zvec, mock_world_model):
    # Setup mock response for quote failure
    quote_resp = AsyncMock()
    quote_resp.status = 400

    module = ZeroExTradingModule("test_api_key", 1, "0xWallet", mock_zvec, mock_world_model)
    module.session = MagicMockSession(responses=[quote_resp])

    result = await module.execute_swap("USDC", "WETH", 500)

    # Assertions
    assert result is None

    mock_zvec.store_transaction_embedding.assert_not_called()
    mock_world_model.update_personality_from_reward.assert_not_called()

    await module.close()

@pytest.mark.asyncio
async def test_execute_swap_liquidity_issue(mock_zvec, mock_world_model):
    # Setup mock response with liquidity issues
    quote_resp = AsyncMock()
    quote_resp.status = 200
    quote_resp.json = AsyncMock(return_value={
        "issues": {"liquidityAvailable": False}
    })

    module = ZeroExTradingModule("test_api_key", 1, "0xWallet", mock_zvec, mock_world_model)
    module.session = MagicMockSession(responses=[quote_resp])

    result = await module.execute_swap("USDC", "WETH", 500)

    # Assertions
    assert result is None

    mock_zvec.store_transaction_embedding.assert_not_called()
    mock_world_model.update_personality_from_reward.assert_not_called()

    await module.close()
