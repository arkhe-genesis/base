import hashlib
import json
import time
from cathedral.types import TemporalAnchor


class TemporalChain1097:
    def __init__(self, chain_id="12120014"):
        self.chain_id = chain_id; self.anchors = []; self._merkle_leaves = []
        self._current_batch = []; self._batch_size = 10

    def anchor_reading(self, reading, zk_proof=None):
        reading_hash = hashlib.sha3_256(json.dumps(reading, sort_keys=True, default=str).encode()).hexdigest()
        self._merkle_leaves.append(reading_hash); self._current_batch.append(reading)
        merkle_root = self._compute_merkle_root()
        anchor_id = f"ANCHOR-{self.chain_id}-{int(time.time())}-{reading_hash[:8]}"
        anchor = TemporalAnchor(anchor_id=anchor_id, merkle_root=merkle_root,
                                zk_proof_hash=zk_proof.proof_id if zk_proof else "",
                                theosis_reading=reading)
        self.anchors.append(anchor)
        if len(self._current_batch) >= self._batch_size: self._rollup_batch()
        return anchor

    def _compute_merkle_root(self):
        if not self._merkle_leaves: return "0" * 64
        leaves = self._merkle_leaves.copy()
        while len(leaves) > 1:
            if len(leaves) % 2 == 1: leaves.append(leaves[-1])
            new_level = []
            for i in range(0, len(leaves), 2):
                combined = hashlib.sha3_256((leaves[i] + leaves[i+1]).encode()).hexdigest()
                new_level.append(combined)
            leaves = new_level
        return leaves[0]

    def _rollup_batch(self):
        if not self._current_batch: return
        batch_hash = hashlib.sha3_256(json.dumps(self._current_batch, sort_keys=True, default=str).encode()).hexdigest()
        print(f"  [TemporalChain] ZK-Rollup: {len(self._current_batch)} readings -> {batch_hash[:16]}...")
        self._current_batch.clear()

    def get_telemetry(self):
        return {"module": "TemporalChain1097", "version": "2.0.0", "substrate": "1097",
                "seal": "TEMPORALCHAIN-1097-v2.0.0-2026-06-07",
                "total_anchors": len(self.anchors), "pending_batch": len(self._current_batch),
                "merkle_root": self._compute_merkle_root()[:16] + "...", "chain_id": self.chain_id}
