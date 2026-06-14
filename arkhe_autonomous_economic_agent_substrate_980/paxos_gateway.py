#!/usr/bin/env python3
"""
Cathedral ARKHE v17.2 – Paxos USDG Gateway
Integration with Paxos API (mint, redeem, convert) and ERC-20 contract on the blockchain.
Security: increaseAllowance, address validation, gas estimation.
Monitoring: Prometheus metrics + alert callbacks.
"""

import asyncio
import hashlib
import hmac
import json
import time
import logging
from dataclasses import dataclass
from typing import Dict, List, Optional, Callable, Any
from decimal import Decimal
from urllib.parse import urljoin

import aiohttp
import web3
from web3 import Web3
from web3.middleware import ExtraDataToPOAMiddleware
from eth_account import Account
from prometheus_client import Counter, Histogram, Gauge

logger = logging.getLogger("cathedral.paxos")

# ============================================================================
# Data Structures
# ============================================================================
@dataclass
class PaxosCredentials:
    client_id: str
    client_secret: str
    api_base_url: str = "https://api.paxos.com/v2"
    web3_provider_url: str = "https://mainnet.infura.io/v3/YOUR_KEY"
    usdg_contract_address: str = "0x... (mainnet) or 0x... (testnet)"
    private_key: Optional[str] = None  # For on-chain transfers

@dataclass
class PaxosBalance:
    currency: str
    total: Decimal
    available: Decimal
    pending: Decimal

@dataclass
class PaxosTransaction:
    id: str
    type: str  # mint, redeem, convert, withdrawal, deposit
    status: str  # pending, completed, failed
    amount: Decimal
    currency: str
    created_at: int
    updated_at: int

@dataclass
class OnChainTransactionReceipt:
    tx_hash: str
    block_number: int
    status: bool  # 1 = success
    gas_used: int
    logs: List[Dict]

# ============================================================================
# Paxos API Client (Custodial)
# ============================================================================
class PaxosAPIClient:
    """Async client for Paxos REST API with HMAC authentication."""

    def __init__(self, credentials: PaxosCredentials):
        self.creds = credentials
        self.session: Optional[aiohttp.ClientSession] = None
        self._nonce_counter = 0

    async def _get_session(self) -> aiohttp.ClientSession:
        if self.session is None or self.session.closed:
            self.session = aiohttp.ClientSession()
        return self.session

    def _generate_signature(self, method: str, path: str, body: str = "", timestamp: int = None) -> str:
        """Generates HMAC-SHA256 signature for API authentication."""
        if timestamp is None:
            timestamp = int(time.time() * 1000)
        self._nonce_counter += 1
        nonce = str(timestamp) + str(self._nonce_counter)
        message = f"{method}\n{path}\n{nonce}\n{body}"
        signature = hmac.new(
            self.creds.client_secret.encode('utf-8'),
            message.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()
        return signature, nonce

    async def _request(self, method: str, endpoint: str, data: Dict = None) -> Dict:
        session = await self._get_session()
        url = urljoin(self.creds.api_base_url, endpoint)
        body = json.dumps(data) if data else ""
        signature, nonce = self._generate_signature(method, endpoint, body)
        headers = {
            "X-PXS-API-KEY": self.creds.client_id,
            "X-PXS-SIGNATURE": signature,
            "X-PXS-NONCE": nonce,
            "Content-Type": "application/json",
        }
        async with session.request(method, url, headers=headers, data=body) as resp:
            if resp.status >= 400:
                text = await resp.text()
                raise RuntimeError(f"Paxos API error {resp.status}: {text}")
            return await resp.json()

    async def get_balances(self) -> List[PaxosBalance]:
        """Returns institutional account balances."""
        data = await self._request("GET", "balances")
        return [
            PaxosBalance(
                currency=item["currency"],
                total=Decimal(item["total"]),
                available=Decimal(item["available"]),
                pending=Decimal(item["pending"])
            )
            for item in data.get("balances", [])
        ]

    async def mint(self, amount: Decimal, currency: str = "USD") -> PaxosTransaction:
        """Requests USDG mint from fiat balance."""
        data = await self._request("POST", "orchestrations/mint", {
            "amount": str(amount),
            "currency": currency,
            "destination_currency": "USDG",
        })
        return PaxosTransaction(
            id=data["id"],
            type="mint",
            status=data["status"],
            amount=Decimal(data["amount"]),
            currency=data["currency"],
            created_at=data["created_at"],
            updated_at=data["updated_at"]
        )

    async def redeem(self, amount: Decimal, destination_currency: str = "USD") -> PaxosTransaction:
        """Redeems USDG to fiat currency."""
        data = await self._request("POST", "orchestrations/redeem", {
            "amount": str(amount),
            "currency": "USDG",
            "destination_currency": destination_currency,
        })
        return PaxosTransaction(
            id=data["id"],
            type="redeem",
            status=data["status"],
            amount=Decimal(data["amount"]),
            currency="USDG",
            created_at=data["created_at"],
            updated_at=data["updated_at"]
        )

    async def convert(self, from_currency: str, to_currency: str, amount: Decimal) -> PaxosTransaction:
        """Converts between different custodial stablecoins (e.g., USDG -> USDC)."""
        data = await self._request("POST", "orchestrations/convert", {
            "from_currency": from_currency,
            "to_currency": to_currency,
            "amount": str(amount),
        })
        return PaxosTransaction(
            id=data["id"],
            type="convert",
            status=data["status"],
            amount=Decimal(data["amount"]),
            currency=from_currency,
            created_at=data["created_at"],
            updated_at=data["updated_at"]
        )

    async def get_transaction(self, tx_id: str) -> PaxosTransaction:
        data = await self._request("GET", f"orchestrations/{tx_id}")
        return PaxosTransaction(
            id=data["id"],
            type=data["type"],
            status=data["status"],
            amount=Decimal(data["amount"]),
            currency=data["currency"],
            created_at=data["created_at"],
            updated_at=data["updated_at"]
        )

    async def close(self):
        if self.session:
            await self.session.close()

# ============================================================================
# On-Chain Client (Web3) for USDG
# ============================================================================
class OnChainUSDGClient:
    """Interacts with the USDG ERC-20 contract on the blockchain."""

    ERC20_ABI = [
        {"constant": True, "inputs": [{"name": "_owner", "type": "address"}], "name": "balanceOf", "outputs": [{"name": "balance", "type": "uint256"}], "type": "function"},
        {"constant": False, "inputs": [{"name": "_spender", "type": "address"}, {"name": "_value", "type": "uint256"}], "name": "approve", "outputs": [{"name": "", "type": "bool"}], "type": "function"},
        {"constant": False, "inputs": [{"name": "_spender", "type": "address"}, {"name": "_addedValue", "type": "uint256"}], "name": "increaseAllowance", "outputs": [{"name": "", "type": "bool"}], "type": "function"},
        {"constant": True, "inputs": [{"name": "_owner", "type": "address"}, {"name": "_spender", "type": "address"}], "name": "allowance", "outputs": [{"name": "", "type": "uint256"}], "type": "function"},
        {"constant": False, "inputs": [{"name": "_to", "type": "address"}, {"name": "_value", "type": "uint256"}], "name": "transfer", "outputs": [{"name": "", "type": "bool"}], "type": "function"},
        {"constant": False, "inputs": [{"name": "_spender", "type": "address"}, {"name": "_subtractedValue", "type": "uint256"}], "name": "decreaseAllowance", "outputs": [{"name": "", "type": "bool"}], "type": "function"},
    ]

    def __init__(self, credentials: PaxosCredentials):
        self.w3 = Web3(Web3.HTTPProvider(credentials.web3_provider_url))
        if self.w3.is_connected() is False:
            raise ConnectionError("Web3 failed to connect to the network")
        # Add middleware for PoA networks (e.g., Polygon, BSC)
        self.w3.middleware_onion.inject(ExtraDataToPOAMiddleware, layer=0)
        self.contract_address = Web3.to_checksum_address(credentials.usdg_contract_address)
        self.contract = self.w3.eth.contract(address=self.contract_address, abi=self.ERC20_ABI)
        self.private_key = credentials.private_key
        self.account = None
        if self.private_key:
            self.account = Account.from_key(self.private_key)
            self.from_address = self.account.address
        else:
            self.from_address = None

    async def get_balance(self, address: str) -> Decimal:
        address = Web3.to_checksum_address(address)
        balance_wei = self.contract.functions.balanceOf(address).call()
        return Decimal(balance_wei) / Decimal(10**18)

    async def get_allowance(self, owner: str, spender: str) -> Decimal:
        owner = Web3.to_checksum_address(owner)
        spender = Web3.to_checksum_address(spender)
        allowance_wei = self.contract.functions.allowance(owner, spender).call()
        return Decimal(allowance_wei) / Decimal(10**18)

    async def increase_allowance(self, spender: str, added_value: Decimal, gas_limit: int = None) -> str:
        """Increases allowance using `increaseAllowance` (security against race condition)."""
        if not self.account:
            raise ValueError("Private key not configured for on-chain transactions")
        spender = Web3.to_checksum_address(spender)
        value_wei = int(added_value * 10**18)
        tx = self.contract.functions.increaseAllowance(spender, value_wei).build_transaction({
            'from': self.from_address,
            'nonce': self.w3.eth.get_transaction_count(self.from_address),
            'gas': gas_limit or 100000,
            'gasPrice': self.w3.eth.gas_price
        })
        signed = self.account.sign_transaction(tx)
        tx_hash = self.w3.eth.send_raw_transaction(signed.rawTransaction)
        return tx_hash.hex()

    async def transfer(self, to_address: str, amount: Decimal, gas_limit: int = None) -> str:
        """Transfers USDG to another address."""
        if not self.account:
            raise ValueError("Private key not configured for on-chain transactions")
        to_address = Web3.to_checksum_address(to_address)
        value_wei = int(amount * 10**18)
        # Estimates gas with margin
        if gas_limit is None:
            gas_estimate = self.contract.functions.transfer(to_address, value_wei).estimate_gas({'from': self.from_address})
            gas_limit = int(gas_estimate * 1.2)
        tx = self.contract.functions.transfer(to_address, value_wei).build_transaction({
            'from': self.from_address,
            'nonce': self.w3.eth.get_transaction_count(self.from_address),
            'gas': gas_limit,
            'gasPrice': self.w3.eth.gas_price
        })
        signed = self.account.sign_transaction(tx)
        tx_hash = self.w3.eth.send_raw_transaction(signed.rawTransaction)
        return tx_hash.hex()

    async def get_transaction_receipt(self, tx_hash: str) -> OnChainTransactionReceipt:
        receipt = self.w3.eth.wait_for_transaction_receipt(tx_hash, timeout=120)
        return OnChainTransactionReceipt(
            tx_hash=receipt['transactionHash'].hex(),
            block_number=receipt['blockNumber'],
            status=receipt['status'] == 1,
            gas_used=receipt['gasUsed'],
            logs=receipt['logs']
        )

# ============================================================================
# Unified Gateway with Monitoring and Alerts
# ============================================================================
class PaxosUSDGGateway:
    """Unified facade for custodial operations (Paxos API) and on-chain."""

    def __init__(self, credentials: PaxosCredentials):
        self.creds = credentials
        self.api = PaxosAPIClient(credentials)
        self.onchain = OnChainUSDGClient(credentials)
        self._alert_callbacks: List[Callable] = []
        self._monitoring_task: Optional[asyncio.Task] = None
        self._stop_monitoring = False
        self._last_balances = {}

        # Prometheus Metrics
        self.api_requests = Counter('paxos_api_requests_total', 'Total API requests', ['operation', 'status'])
        self.api_latency = Histogram('paxos_api_latency_seconds', 'API latency', ['operation'])
        self.usdg_balance = Gauge('paxos_usdg_balance', 'USDG balance (custodial)', ['type'])
        self.onchain_usdg_balance = Gauge('paxos_onchain_usdg_balance', 'On-chain USDG balance')
        self.allowance_gauge = Gauge('paxos_allowance', 'Token allowance', ['spender'])
        self.onchain_tx_counter = Counter('paxos_onchain_transactions_total', 'On-chain transactions', ['type'])

    def set_alert_callback(self, callback: Callable):
        """Registers a function to receive alerts (e.g., send notification)."""
        self._alert_callbacks.append(callback)

    async def _send_alert(self, title: str, message: str, severity: str = "warning"):
        for cb in self._alert_callbacks:
            try:
                await cb(title, message, severity)
            except Exception as e:
                logger.error(f"Error in alert callback: {e}")

    async def get_balances(self) -> List[PaxosBalance]:
        start = time.time()
        try:
            balances = await self.api.get_balances()
            self.api_requests.labels(operation="get_balances", status="success").inc()
            # Updates metrics
            for bal in balances:
                if bal.currency == "USDG":
                    self.usdg_balance.labels(type="available").set(float(bal.available))
                    self.usdg_balance.labels(type="total").set(float(bal.total))
                self._last_balances[bal.currency] = bal
            return balances
        except Exception as e:
            self.api_requests.labels(operation="get_balances", status="error").inc()
            raise
        finally:
            self.api_latency.labels(operation="get_balances").observe(time.time() - start)

    async def mint(self, amount: Decimal, currency: str = "USD") -> PaxosTransaction:
        start = time.time()
        try:
            tx = await self.api.mint(amount, currency)
            self.api_requests.labels(operation="mint", status="success").inc()
            # Large mint alert
            if amount > 100000:
                await self._send_alert("Large Mint", f"Amount: {amount} {currency}", "info")
            return tx
        except Exception as e:
            self.api_requests.labels(operation="mint", status="error").inc()
            await self._send_alert("Mint Error", str(e), "error")
            raise
        finally:
            self.api_latency.labels(operation="mint").observe(time.time() - start)

    async def redeem(self, amount: Decimal, destination_currency: str = "USD") -> PaxosTransaction:
        start = time.time()
        try:
            tx = await self.api.redeem(amount, destination_currency)
            self.api_requests.labels(operation="redeem", status="success").inc()
            return tx
        except Exception as e:
            self.api_requests.labels(operation="redeem", status="error").inc()
            await self._send_alert("Redeem Error", str(e), "error")
            raise
        finally:
            self.api_latency.labels(operation="redeem").observe(time.time() - start)

    async def convert(self, from_currency: str, to_currency: str, amount: Decimal) -> PaxosTransaction:
        start = time.time()
        try:
            tx = await self.api.convert(from_currency, to_currency, amount)
            self.api_requests.labels(operation="convert", status="success").inc()
            return tx
        except Exception as e:
            self.api_requests.labels(operation="convert", status="error").inc()
            raise
        finally:
            self.api_latency.labels(operation="convert").observe(time.time() - start)

    async def transfer_on_chain(self, to_address: str, amount: Decimal) -> str:
        """Transfers USDG on-chain (self-custody)."""
        tx_hash = await self.onchain.transfer(to_address, amount)
        self.onchain_tx_counter.labels(type="transfer").inc()
        await self._send_alert("On-chain Transfer", f"To {to_address}, amount {amount} USDG", "info")
        return tx_hash

    async def increase_allowance(self, spender: str, added_value: Decimal) -> str:
        """Increases allowance for a spender (recommended over direct approve)."""
        tx_hash = await self.onchain.increase_allowance(spender, added_value)
        self.onchain_tx_counter.labels(type="increase_allowance").inc()
        # Updates metric
        new_allowance = await self.onchain.get_allowance(self.onchain.from_address, spender)
        self.allowance_gauge.labels(spender=spender).set(float(new_allowance))
        return tx_hash

    async def get_onchain_balance(self, address: str = None) -> Decimal:
        if address is None:
            if not self.onchain.from_address:
                raise ValueError("Address not provided and no configured address")
            address = self.onchain.from_address
        balance = await self.onchain.get_balance(address)
        self.onchain_usdg_balance.set(float(balance))
        return balance

    async def start_monitoring(self, interval_seconds: int = 30):
        """Starts continuous monitoring loop for balances and allowances."""
        self._stop_monitoring = False
        self._monitoring_task = asyncio.create_task(self._monitoring_loop(interval_seconds))

    async def _monitoring_loop(self, interval: int):
        while not self._stop_monitoring:
            try:
                # Checks custodial balances
                balances = await self.get_balances()
                for bal in balances:
                    if bal.currency == "USDG" and bal.available < 1000:
                        await self._send_alert("Low USDG Balance", f"Available balance: {bal.available}", "warning")

                # Checks on-chain balance
                if self.onchain.from_address:
                    onchain_bal = await self.get_onchain_balance()
                    if onchain_bal < 500:
                        await self._send_alert("Low On-chain Balance", f"Balance: {onchain_bal} USDG", "warning")

                # Monitors allowances for critical spender (e.g., swap contract)
                # (example: check allowance for 0x...)
                # ...
            except Exception as e:
                logger.error(f"Error in monitoring: {e}")
            await asyncio.sleep(interval)

    async def stop_monitoring(self):
        self._stop_monitoring = True
        if self._monitoring_task:
            await self._monitoring_task
        await self.api.close()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.stop_monitoring()
