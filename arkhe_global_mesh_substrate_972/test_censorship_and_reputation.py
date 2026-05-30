#!/usr/bin/env python3
"""
Test script for simulating total censorship (Tor failover) and Nostr reputation integration in consensus.
"""

import asyncio
from bridge_nostr_tor_ipfs import NostrTorIpfsBridge
from consensus.hamiltonian_consensus import HamiltonianConsensus, Vote

import pytest

@pytest.mark.asyncio
async def test_censorship_failover():
    print("=== Testing Total Censorship Scenario (Tor Failover) ===")
    bridge = NostrTorIpfsBridge(
        node_id="test-node-1",
        ed25519_pubkey="a" * 64,
        ed25519_privkey="b" * 64,
    )

    # Target dummy onion
    target_onion = "arkhe12345678901234.onion"

    # Try connecting with censorship simulated
    print("Initiating mesh_connect with simulate_censorship=True")
    success = await bridge.mesh_connect(target_onion=target_onion, simulate_censorship=True)
    if success:
        print("Failover test PASSED. Successfully fell back to NOSTR.")
    else:
        print("Failover test FAILED.")

def test_nostr_reputation_consensus():
    print("\n=== Testing Nostr Reputation Integration in Consensus ===")
    consensus = HamiltonianConsensus()
    prop_id = "prop_1"
    consensus.propose(prop_id, "Update protocol version")

    # Vote 1: High Theosis, Low Nostr Reputation
    vote1 = Vote(node_id="node_a", proposal_id=prop_id, vote=1.0, theosis_weight=0.9, seal="seal1", nostr_reputation=0.2)
    # Vote 2: Low Theosis, High Nostr Reputation
    vote2 = Vote(node_id="node_b", proposal_id=prop_id, vote=0.0, theosis_weight=0.4, seal="seal2", nostr_reputation=0.9)
    # Vote 3: Medium Theosis, Normal Nostr Reputation
    vote3 = Vote(node_id="node_c", proposal_id=prop_id, vote=1.0, theosis_weight=0.6, seal="seal3", nostr_reputation=1.0)

    consensus.vote(prop_id, vote1)
    consensus.vote(prop_id, vote2)
    consensus.vote(prop_id, vote3)

    tally = consensus.tally(prop_id)
    print("Consensus Tally Result:", tally)

    # Node A effective weight: 0.9 * 0.2 = 0.18
    # Node B effective weight: 0.4 * 0.9 = 0.36
    # Node C effective weight: 0.6 * 1.0 = 0.60
    # Total weight: 0.18 + 0.36 + 0.60 = 1.14
    # Weighted vote: (1.0 * 0.18 + 0.0 * 0.36 + 1.0 * 0.60) / 1.14 = 0.78 / 1.14 ≈ 0.684

    if abs(tally["weighted_vote"] - 0.6842) < 0.01:
        print("Consensus test PASSED. Tally weights appropriately adjusted by Nostr reputation.")
    else:
        print("Consensus test FAILED.")

async def main():
    await test_censorship_failover()
    test_nostr_reputation_consensus()

if __name__ == "__main__":
    asyncio.run(main())
