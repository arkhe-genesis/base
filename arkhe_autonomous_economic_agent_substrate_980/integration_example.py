# integration_example.py
import asyncio
import logging
from decimal import Decimal
from paxos_gateway import PaxosUSDGGateway, PaxosCredentials

logging.basicConfig(level=logging.INFO)

async def main():
    # Configurations (use environment variables in production)
    creds = PaxosCredentials(
        client_id="YOUR_CLIENT_ID",
        client_secret="YOUR_CLIENT_SECRET",
        api_base_url="https://api.paxos.com/v2",  # or sandbox
        web3_provider_url="https://mainnet.infura.io/v3/YOUR_KEY",
        usdg_contract_address="0x...",  # USDG contract address
        private_key="0x..."  # keep it secret
    )

    async with PaxosUSDGGateway(creds) as gateway:
        # Registers alert callback (e.g., send notification to Windows)
        async def alert_callback(title: str, message: str, severity: str):
            logging.warning(f"[{severity.upper()} ALERT] {title}: {message}")
        gateway.set_alert_callback(alert_callback)

        # Starts monitoring
        await gateway.start_monitoring(interval_seconds=30)

        # 1. Check custodial balances
        balances = await gateway.get_balances()
        for bal in balances:
            print(f"Balance {bal.currency}: total={bal.total}, available={bal.available}")

        # 2. Mint of 1000 USDG
        tx_mint = await gateway.mint(Decimal("1000"), currency="USD")
        print(f"Mint initiated: {tx_mint.id}, status={tx_mint.status}")

        # 3. On-chain transfer
        tx_hash = await gateway.transfer_on_chain("0xDestination...", Decimal("100"))
        print(f"Transfer sent: {tx_hash}")

        # 4. Increase allowance for a swap contract
        spender = "0xSwapContract..."
        await gateway.increase_allowance(spender, Decimal("500"))

        # 5. Check on-chain balance
        onchain_bal = await gateway.get_onchain_balance()
        print(f"Wallet on-chain balance: {onchain_bal} USDG")

        # 6. Redeem (redeem USDG for USD)
        tx_redeem = await gateway.redeem(Decimal("200"), destination_currency="USD")
        print(f"Redeem initiated: {tx_redeem.id}")

        # Wait monitoring for a few seconds
        await asyncio.sleep(10)

if __name__ == "__main__":
    asyncio.run(main())
