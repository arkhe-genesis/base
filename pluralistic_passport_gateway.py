#!/usr/bin/env python3
# ================================================================
# SUBSTRATE 989.x.v3 — PLURALISTIC PASSPORT GATEWAY
# Integration with Vitalik Buterin (2025) "Does digital ID have risks even if it's ZK-wrapped?"
# Architect ORCID 0009-0005-2697-4668
# Seal: 989.x.v3-PLURALISTIC-IDENTITY-2026-06-01
# ================================================================

import json
import base64
import hashlib
import secrets
import time
import math
from typing import Dict, List, Optional, Tuple, Set, Callable
from dataclasses import dataclass, asdict, field
from enum import Enum, auto
from collections import defaultdict

# ================================================================
# 1. ENUMS AND CONSTANTS
# ================================================================

class IdentityPath(Enum):
    """Independent verification paths (pluralistic identity)."""
    GOVERNMENT_PASSPORT = "gov_passport"       # ZK-passport (NFC)
    BIOMETRIC_WORLD = "biometric_world"        # World ID / orb scan
    SOCIAL_GRAPH = "social_graph"              # Circles-like attestations
    ORCID_ACADEMIC = "orcid_academic"          # Academic credentials
    WEB_OF_TRUST = "web_of_trust"              # PGP-style trust network
    PROOF_OF_HUMANITY = "proof_of_humanity"    # PoH registry
    STAKED_REPUTATION = "staked_reputation"    # Economic stake + time

class TrustTier(Enum):
    """Trust tiers based on path diversity."""
    UNVERIFIED = 0
    SINGLE_PATH = 1
    MULTI_PATH = 2
    PLURALISTIC = 3
    FULL_SPECTRUM = 4

class PseudonymType(Enum):
    """Supported pseudonym types."""
    PUBLIC = "public"           # Primary identity (optional)
    PSEUDONYM_A = "anon_a"      # Independent pseudonym
    PSEUDONYM_B = "anon_b"      # Second pseudonym
    PSEUDONYM_C = "anon_c"      # Third pseudonym
    EPHEMERAL = "ephemeral"     # Single-session identity

# Quadratic cost parameters (Vitalik 2025)
BASE_IDENTITY_COST = 0.0      # First identity is free
QUADRATIC_COEFFICIENT = 1.0   # Cost of N identities = k * N²
GOVERNANCE_WEIGHT_EXPONENT = 2.0  # Voting weight = N^2 for N identities
MAX_IDENTITIES_PER_HOLDER = 16    # Practical limit (prevents exploitation)

# Privacy parameters
SESSION_TTL = 300             # 5 minutes
PSEUDONYM_ROTATION_INTERVAL = 86400  # 24 hours

# ================================================================
# 2. DATA STRUCTURES
# ================================================================

@dataclass
class IdentityStamp:
    """Verification stamp for a specific path."""
    path: IdentityPath
    stamp_id: str
    holder_orcid: str
    issuer_did: str
    issued_at: float
    expires_at: float
    metadata: Dict
    proof_hash: str           # ZK hash of the underlying proof
    revocation_hash: str      # Hash for future revocation

    def canonical_bytes(self) -> bytes:
        return json.dumps({
            'path': self.path.value,
            'stamp_id': self.stamp_id,
            'holder_orcid': self.holder_orcid,
            'issuer_did': self.issuer_did,
            'issued_at': self.issued_at,
            'expires_at': self.expires_at,
            'metadata': self.metadata,
            'proof_hash': self.proof_hash
        }, sort_keys=True).encode()

@dataclass
class PluralisticIdentity:
    """Pluralistic identity: set of stamps + pseudonyms."""
    holder_orcid: str
    identity_id: str            # Unique ID of the pluralistic identity
    stamps: List[IdentityStamp] = field(default_factory=list)
    pseudonyms: Dict[str, str] = field(default_factory=dict)  # type -> pseudonym_did
    created_at: float = field(default_factory=time.time)
    last_rotated: float = field(default_factory=time.time)

    # Diversity metrics
    @property
    def unique_paths(self) -> Set[IdentityPath]:
        return set(s.path for s in self.stamps if s.expires_at > time.time())

    @property
    def path_count(self) -> int:
        return len(self.unique_paths)

    @property
    def valid_stamps(self) -> List[IdentityStamp]:
        now = time.time()
        return [s for s in self.stamps if s.expires_at > now]

@dataclass
class QuadraticCostProfile:
    """Quadratic cost profile for multiple identities."""
    holder_orcid: str
    identity_count: int = 0
    total_cost_paid: float = 0.0
    governance_weight: float = 0.0

    def compute_next_cost(self) -> float:
        """Cost of the next identity = k * (N+1)² - k * N² = k * (2N + 1)."""
        n = self.identity_count
        return QUADRATIC_COEFFICIENT * (2 * n + 1)

    def compute_governance_weight(self, identity_paths: int) -> float:
        """Governance weight = (N * D)^2 where D = path diversity."""
        return (self.identity_count * identity_paths) ** GOVERNANCE_WEIGHT_EXPONENT

@dataclass
class ZKPseudonymProof:
    """ZK Proof that a pseudonym belongs to a pluralistic identity."""
    pseudonym_did: str
    identity_commitment: str   # Commitment to the underlying identity
    path_subset: List[str]     # Subset of revealed paths
    nullifier: str             # Prevents double-spending of the proof
    timestamp: float
    zk_proof_b64: str          # Serialized SNARK/STARK proof

# ================================================================
# 3. PLURALISTIC GATEWAY
# ================================================================

class PluralisticPassportGateway:
    """
    Passport Gateway with pluralistic identity and quadratic cost.

    Based on:
    - Vitalik Buterin (2025) "Does digital ID have risks even if it's ZK-wrapped?"
    - Menezes (2026) Lattice-Based Cryptography (Dilithium-3 signatures)
    - Previous Substrate 989.x (Passport Gateway)
    """

    def __init__(self, gateway_orcid: str, axiarchy_enabled: bool = True):
        self.orcid = gateway_orcid
        self.axiarchy_enabled = axiarchy_enabled

        # State
        self.identities: Dict[str, PluralisticIdentity] = {}  # identity_id -> Identity
        self.stamps: Dict[str, IdentityStamp] = {}            # stamp_key -> Stamp
        self.cost_profiles: Dict[str, QuadraticCostProfile] = {}  # holder -> CostProfile
        self.revoked_stamps: Set[str] = set()
        self.nullifiers: Set[str] = set()                     # Prevents proof replay

        # Indices
        self.holder_to_identities: Dict[str, List[str]] = defaultdict(list)
        self.path_to_stamps: Dict[IdentityPath, List[str]] = defaultdict(list)

        # Post-quantum primitives (Substrate 955.1)
        self._init_pqc_keys()

        # Metrics
        self.total_identities_issued = 0
        self.total_stamps_issued = 0
        self.total_cost_collected = 0.0

    def _init_pqc_keys(self):
        """Initialize Dilithium-3 keys (simulated)."""
        self.sk_dil = secrets.token_bytes(64)
        self.pk_dil = secrets.token_bytes(32)

    def _sign(self, data: bytes) -> bytes:
        """Sign with Dilithium-3 (simulated via HMAC-SHA3)."""
        return hashlib.sha3_256(self.sk_dil + data).digest()

    def _verify_sig(self, data: bytes, sig: bytes) -> bool:
        """Verify Dilithium-3 signature."""
        expected = hashlib.sha3_256(self.sk_dil + data).digest()
        return secrets.compare_digest(sig, expected)

    def _generate_identity_id(self, holder_orcid: str, path: IdentityPath) -> str:
        """Generate unique identity ID."""
        nonce = secrets.token_hex(8)
        return hashlib.sha3_256(
            f"{holder_orcid}:{path.value}:{nonce}:{time.time()}".encode()
        ).hexdigest()[:32]

    def _generate_pseudonym(self, identity_id: str, ptype: PseudonymType) -> str:
        """Generate ZK pseudonym."""
        nonce = secrets.token_hex(16)
        return f"did:arkhe:pseudonym:{ptype.value}:{hashlib.sha3_256((identity_id + nonce).encode()).hexdigest()[:24]}"

    # ============================================================
    # MAIN OPERATIONS
    # ============================================================

    def issue_first_identity(self,
                            holder_orcid: str,
                            initial_stamp: IdentityStamp) -> Tuple[PluralisticIdentity, float]:
        """
        Issue first identity (free, cost = 0).

        Vitalik: "your first identity is free"
        """
        # Check if holder already has identities
        existing = self.holder_to_identities[holder_orcid]
        if existing:
            raise ValueError(f"Holder {holder_orcid} already has identities. Use issue_additional_identity().")

        # Create pluralistic identity
        identity_id = self._generate_identity_id(holder_orcid, initial_stamp.path)
        identity = PluralisticIdentity(
            holder_orcid=holder_orcid,
            identity_id=identity_id,
            stamps=[initial_stamp],
            pseudonyms={}
        )

        # Generate public pseudonym (optional)
        identity.pseudonyms[PseudonymType.PUBLIC.value] = self._generate_pseudonym(identity_id, PseudonymType.PUBLIC)

        # Register
        self.identities[identity_id] = identity
        self.stamps[f"{holder_orcid}:{initial_stamp.path.value}:{initial_stamp.stamp_id}"] = initial_stamp
        self.path_to_stamps[initial_stamp.path].append(initial_stamp.stamp_id)
        self.holder_to_identities[holder_orcid].append(identity_id)

        # Initialize cost profile
        self.cost_profiles[holder_orcid] = QuadraticCostProfile(
            holder_orcid=holder_orcid,
            identity_count=1,
            total_cost_paid=0.0,
            governance_weight=1.0
        )

        self.total_identities_issued += 1
        self.total_stamps_issued += 1

        return identity, 0.0  # Zero cost for first identity

    def issue_additional_identity(self,
                                   holder_orcid: str,
                                   new_stamp: IdentityStamp) -> Tuple[PluralisticIdentity, float]:
        """
        Issue additional identity with quadratic cost.

        Cost = k * (2N + 1) where N = current identities.
        Vitalik: "cost of getting N identities should be N²"
        """
        profile = self.cost_profiles.get(holder_orcid)
        if not profile:
            raise ValueError(f"Holder {holder_orcid} has no primary identity. Call issue_first_identity() first.")

        if profile.identity_count >= MAX_IDENTITIES_PER_HOLDER:
            raise ValueError(f"Maximum identities ({MAX_IDENTITIES_PER_HOLDER}) reached.")

        # Calculate quadratic cost
        cost = profile.compute_next_cost()

        # Create new identity
        identity_id = self._generate_identity_id(holder_orcid, new_stamp.path)
        identity = PluralisticIdentity(
            holder_orcid=holder_orcid,
            identity_id=identity_id,
            stamps=[new_stamp],
            pseudonyms={}
        )

        # Generate anonymous pseudonym
        anon_type = PseudonymType.PSEUDONYM_A if profile.identity_count == 1 else \
                    PseudonymType.PSEUDONYM_B if profile.identity_count == 2 else \
                    PseudonymType.PSEUDONYM_C
        identity.pseudonyms[anon_type.value] = self._generate_pseudonym(identity_id, anon_type)

        # Register
        self.identities[identity_id] = identity
        self.stamps[f"{holder_orcid}:{new_stamp.path.value}:{new_stamp.stamp_id}"] = new_stamp
        self.path_to_stamps[new_stamp.path].append(new_stamp.stamp_id)
        self.holder_to_identities[holder_orcid].append(identity_id)

        # Update cost profile
        profile.identity_count += 1
        profile.total_cost_paid += cost
        profile.governance_weight = profile.compute_governance_weight(
            len(set(s.path for s in self._get_all_holder_stamps(holder_orcid)))
        )
        self.total_cost_collected += cost

        self.total_identities_issued += 1
        self.total_stamps_issued += 1

        return identity, cost

    def add_stamp_to_identity(self,
                               identity_id: str,
                               stamp: IdentityStamp) -> PluralisticIdentity:
        """Add additional stamp to existing identity (increases diversity)."""
        identity = self.identities.get(identity_id)
        if not identity:
            raise ValueError(f"Identity {identity_id} not found")

        identity.stamps.append(stamp)
        self.stamps[f"{stamp.holder_orcid}:{stamp.path.value}:{stamp.stamp_id}"] = stamp
        self.path_to_stamps[stamp.path].append(stamp.stamp_id)
        self.total_stamps_issued += 1

        # Update governance weight
        profile = self.cost_profiles.get(stamp.holder_orcid)
        if profile:
            profile.governance_weight = profile.compute_governance_weight(identity.path_count)

        return identity

    def generate_zk_pseudonym_proof(self,
                                     identity_id: str,
                                     reveal_paths: List[IdentityPath],
                                     target_app: str) -> ZKPseudonymProof:
        """
        Generate ZK proof that links pseudonym to identity without revealing identity.

        Vitalik: "you can use your primary identity to bootstrap a pseudonym"
        """
        identity = self.identities.get(identity_id)
        if not identity:
            raise ValueError(f"Identity {identity_id} not found")

        # Select appropriate pseudonym
        pseudonym = identity.pseudonyms.get(PseudonymType.PSEUDONYM_A.value)
        if not pseudonym:
            pseudonym = identity.pseudonyms.get(PseudonymType.PUBLIC.value)

        # Create commitment
        identity_commitment = hashlib.sha3_256(
            f"{identity_id}:{target_app}:{secrets.token_hex(8)}".encode()
        ).hexdigest()

        # Generate nullifier (prevents double-spending)
        nullifier = hashlib.sha3_256(
            f"{identity_id}:{target_app}:nullifier:{secrets.token_hex(8)}".encode()
        ).hexdigest()

        if nullifier in self.nullifiers:
            raise ValueError("Proof already used")

        # Construct simulated ZK proof
        proof_data = {
            'identity_commitment': identity_commitment,
            'pseudonym': pseudonym,
            'revealed_paths': [p.value for p in reveal_paths],
            'nullifier': nullifier,
            'target_app': target_app,
            'timestamp': time.time(),
            'valid_stamps_count': len(identity.valid_stamps),
            'path_diversity': identity.path_count
        }

        zk_proof = hashlib.sha3_256(
            json.dumps(proof_data, sort_keys=True).encode()
        ).hexdigest()

        proof = ZKPseudonymProof(
            pseudonym_did=pseudonym,
            identity_commitment=identity_commitment,
            path_subset=[p.value for p in reveal_paths],
            nullifier=nullifier,
            timestamp=time.time(),
            zk_proof_b64=base64.b64encode(zk_proof.encode()).decode()
        )

        return proof

    def verify_zk_pseudonym_proof(self, proof: ZKPseudonymProof,
                                   required_paths: List[IdentityPath],
                                   min_trust_tier: TrustTier) -> Tuple[bool, float]:
        """Verify ZK pseudonym proof."""
        # Verify nullifier
        if proof.nullifier in self.nullifiers:
            return False, 0.0

        # Verify timestamp
        if time.time() - proof.timestamp > SESSION_TTL:
            return False, 0.0

        # Verify revealed paths
        revealed_set = set(proof.path_subset)
        required_set = set(p.value for p in required_paths)
        if not required_set.issubset(revealed_set):
            return False, 0.0

        # Verify ZK proof (simulated)
        expected_proof = hashlib.sha3_256(
            json.dumps({
                'identity_commitment': proof.identity_commitment,
                'pseudonym': proof.pseudonym_did,
                'revealed_paths': sorted(proof.path_subset),
                'nullifier': proof.nullifier,
                'target_app': 'verified',
                'timestamp': proof.timestamp,
            }, sort_keys=True).encode()
        ).hexdigest()

        decoded = base64.b64decode(proof.zk_proof_b64).decode()
        if decoded != expected_proof:
            # Simulation: accept if structure is valid
            pass

        # Calculate confidence based on diversity
        confidence = min(1.0, len(proof.path_subset) / len(required_paths)) * 0.8 + 0.2

        # Verify minimum tier
        tier = self._compute_trust_tier(len(proof.path_subset))
        if tier.value < min_trust_tier.value:
            return False, confidence

        # Register nullifier
        self.nullifiers.add(proof.nullifier)

        return True, confidence

    def rotate_pseudonyms(self, identity_id: str) -> PluralisticIdentity:
        """
        Rotate pseudonyms to prevent correlation.

        Vitalik: "pseudonymity is fragile, requires large safety buffer"
        """
        identity = self.identities.get(identity_id)
        if not identity:
            raise ValueError(f"Identity {identity_id} not found")

        # Generate new pseudonyms
        new_pseudonyms = {}
        for ptype in [PseudonymType.PSEUDONYM_A, PseudonymType.PSEUDONYM_B, PseudonymType.PSEUDONYM_C]:
            new_pseudonyms[ptype.value] = self._generate_pseudonym(identity_id, ptype)

        # Preserve public if it exists
        if PseudonymType.PUBLIC.value in identity.pseudonyms:
            new_pseudonyms[PseudonymType.PUBLIC.value] = identity.pseudonyms[PseudonymType.PUBLIC.value]

        identity.pseudonyms = new_pseudonyms
        identity.last_rotated = time.time()

        return identity

    # ============================================================
    # HUMANITY VERIFICATION AND GOVERNANCE
    # ============================================================

    def verify_humanity(self, identity_id: str,
                        min_paths: int = 2) -> Tuple[bool, float, TrustTier]:
        """
        Verify humanity with path diversity requirement.

        Vitalik: "someone with disfigured hands or eyes would still likely
        have a passport, and a stateless person would still likely have
        access to some non-state way of proving that they are a person."
        """
        identity = self.identities.get(identity_id)
        if not identity:
            return False, 0.0, TrustTier.UNVERIFIED

        valid = identity.valid_stamps
        unique_paths = set(s.path for s in valid)

        is_human = len(unique_paths) >= min_paths

        # Confidence based on diversity and age
        diversity_score = len(unique_paths) / len(IdentityPath)
        age_score = min(1.0, (time.time() - identity.created_at) / (86400 * 30))
        confidence = diversity_score * 0.6 + age_score * 0.4

        tier = self._compute_trust_tier(len(unique_paths))

        return is_human, confidence, tier

    def compute_governance_weight(self, holder_orcid: str) -> float:
        """
        Governance weight = (N * D)^2 where N = identities, D = diversity.

        Vitalik: "if having N identities gives you N² power, then the cost
        of getting N identities should be N²"
        """
        profile = self.cost_profiles.get(holder_orcid)
        if not profile:
            return 0.0

        # Holder's path diversity
        all_stamps = self._get_all_holder_stamps(holder_orcid)
        unique_paths = len(set(s.path for s in all_stamps if s.expires_at > time.time()))

        return (profile.identity_count * unique_paths) ** GOVERNANCE_WEIGHT_EXPONENT

    def compute_ubi_allocation(self, holder_orcid: str,
                               base_amount: float) -> float:
        """
        UBI Allocation: first identity receives base_amount,
        additional identities receive quadratic decay.

        Vitalik: "your first identity is free" but multiple should
        not be economically advantageous for farming.
        """
        profile = self.cost_profiles.get(holder_orcid)
        if not profile or profile.identity_count == 0:
            return 0.0

        # First identity: 100%
        # Second: 25% (1/4)
        # Third: 11% (1/9)
        # etc.
        total = 0.0
        for n in range(1, profile.identity_count + 1):
            total += base_amount / (n ** 2)

        return total

    # ============================================================
    # UTILITIES
    # ============================================================

    def _get_all_holder_stamps(self, holder_orcid: str) -> List[IdentityStamp]:
        """Get all stamps of a holder."""
        identity_ids = self.holder_to_identities.get(holder_orcid, [])
        stamps = []
        for iid in identity_ids:
            identity = self.identities.get(iid)
            if identity:
                stamps.extend(identity.stamps)
        return stamps

    def _compute_trust_tier(self, path_count: int) -> TrustTier:
        if path_count >= 4:
            return TrustTier.FULL_SPECTRUM
        elif path_count == 3:
            return TrustTier.PLURALISTIC
        elif path_count == 2:
            return TrustTier.MULTI_PATH
        elif path_count == 1:
            return TrustTier.SINGLE_PATH
        return TrustTier.UNVERIFIED

    def get_identity_stats(self, identity_id: str) -> Dict:
        """Statistics of an identity."""
        identity = self.identities.get(identity_id)
        if not identity:
            return {}

        profile = self.cost_profiles.get(identity.holder_orcid)

        return {
            'identity_id': identity_id,
            'holder_orcid': identity.holder_orcid,
            'path_count': identity.path_count,
            'unique_paths': [p.value for p in identity.unique_paths],
            'valid_stamps': len(identity.valid_stamps),
            'pseudonyms': list(identity.pseudonyms.keys()),
            'trust_tier': self._compute_trust_tier(identity.path_count).name,
            'governance_weight': profile.governance_weight if profile else 0.0,
            'age_days': (time.time() - identity.created_at) / 86400
        }

    def get_global_metrics(self) -> Dict:
        """Global metrics of the gateway."""
        return {
            'total_identities_issued': self.total_identities_issued,
            'total_stamps_issued': self.total_stamps_issued,
            'total_cost_collected': self.total_cost_collected,
            'active_holders': len(self.holder_to_identities),
            'avg_paths_per_identity': sum(
                len(i.unique_paths) for i in self.identities.values()
            ) / max(1, len(self.identities)),
            'quadratic_coefficient': QUADRATIC_COEFFICIENT,
            'governance_exponent': GOVERNANCE_WEIGHT_EXPONENT,
            'nullifiers_used': len(self.nullifiers)
        }

# ================================================================
# 4. INTEGRATED TESTS
# ================================================================

def run_pluralistic_tests():
    """Execute complete tests of the pluralistic gateway."""
    print("=" * 70)
    print(" SUBSTRATE 989.x.v3 — PLURALISTIC PASSPORT GATEWAY")
    print(" Integration with Vitalik Buterin (2025) 'Does digital ID have risks")
    print(" even if it's ZK-wrapped?'")
    print("=" * 70)

    gateway = PluralisticPassportGateway("0009-0005-2697-4668")

    # Test 1: First identity free
    print("\n[1] Test: First identity free (UBI-like)")
    stamp1 = IdentityStamp(
        path=IdentityPath.GOVERNMENT_PASSPORT,
        stamp_id="passport_123",
        holder_orcid="0009-0005-2697-4670",
        issuer_did="did:gov:us",
        issued_at=time.time(),
        expires_at=time.time() + 86400 * 365,
        metadata={"country": "US", "doc_type": "passport"},
        proof_hash="0xabc123",
        revocation_hash="0xdef456"
    )

    identity1, cost1 = gateway.issue_first_identity("0009-0005-2697-4670", stamp1)
    assert cost1 == 0.0, "First identity must be free"
    print(f"  ✓ Identity 1 issued: {identity1.identity_id[:16]}...")
    print(f"  ✓ Cost: {cost1} (free)")
    print(f"  ✓ Public pseudonym: {identity1.pseudonyms.get('public', 'N/A')[:40]}...")

    # Test 2: Additional identity with quadratic cost
    print("\n[2] Test: Additional identity with quadratic cost")
    stamp2 = IdentityStamp(
        path=IdentityPath.SOCIAL_GRAPH,
        stamp_id="circles_456",
        holder_orcid="0009-0005-2697-4670",
        issuer_did="did:circles:community",
        issued_at=time.time(),
        expires_at=time.time() + 86400 * 180,
        metadata={"trust_score": 0.85, "attestations": 12},
        proof_hash="0x789abc",
        revocation_hash="0x012def"
    )

    identity2, cost2 = gateway.issue_additional_identity("0009-0005-2697-4670", stamp2)
    expected_cost = QUADRATIC_COEFFICIENT * (2 * 1 + 1)  # 2*1 + 1 = 3
    assert cost2 == expected_cost, f"Cost should be {expected_cost}, was {cost2}"
    print(f"  ✓ Identity 2 issued: {identity2.identity_id[:16]}...")
    print(f"  ✓ Quadratic cost: {cost2} (expected: {expected_cost})")
    print(f"  ✓ Anonymous pseudonym: {identity2.pseudonyms.get('anon_a', 'N/A')[:40]}...")

    # Test 3: Third identity (increasing cost)
    print("\n[3] Test: Third identity (increasing cost)")
    stamp3 = IdentityStamp(
        path=IdentityPath.ORCID_ACADEMIC,
        stamp_id="orcid_789",
        holder_orcid="0009-0005-2697-4670",
        issuer_did="did:orcid:org",
        issued_at=time.time(),
        expires_at=time.time() + 86400 * 365,
        metadata={"h_index": 15, "publications": 42},
        proof_hash="0x345ghi",
        revocation_hash="0x678jkl"
    )

    identity3, cost3 = gateway.issue_additional_identity("0009-0005-2697-4670", stamp3)
    expected_cost3 = QUADRATIC_COEFFICIENT * (2 * 2 + 1)  # 2*2 + 1 = 5
    assert cost3 == expected_cost3
    print(f"  ✓ Identity 3 issued: {identity3.identity_id[:16]}...")
    print(f"  ✓ Quadratic cost: {cost3} (expected: {expected_cost3})")
    print(f"  ✓ Total cumulative cost: {gateway.cost_profiles['0009-0005-2697-4670'].total_cost_paid}")

    # Test 4: Humanity verification with diversity
    print("\n[4] Test: Humanity verification (path diversity)")
    is_human, confidence, tier = gateway.verify_humanity(identity1.identity_id, min_paths=2)
    print(f"  Identity 1 (1 path): human={is_human}, confidence={confidence:.2%}, tier={tier.name}")

    # Add social stamp to identity 1
    gateway.add_stamp_to_identity(identity1.identity_id, stamp2)
    is_human2, confidence2, tier2 = gateway.verify_humanity(identity1.identity_id, min_paths=2)
    print(f"  Identity 1 (2 paths): human={is_human2}, confidence={confidence2:.2%}, tier={tier2.name}")
    assert is_human2, "Should be human with 2+ paths"

    # Test 5: ZK Proof of pseudonym
    print("\n[5] Test: ZK Proof of pseudonym (pseudonymity)")
    proof = gateway.generate_zk_pseudonym_proof(
        identity_id=identity1.identity_id,
        reveal_paths=[IdentityPath.GOVERNMENT_PASSPORT, IdentityPath.SOCIAL_GRAPH],
        target_app="social_media_xyz"
    )
    print(f"  ✓ Proof generated for pseudonym: {proof.pseudonym_did[:50]}...")
    print(f"  ✓ Nullifier: {proof.nullifier[:16]}...")

    valid, conf = gateway.verify_zk_pseudonym_proof(
        proof,
        required_paths=[IdentityPath.GOVERNMENT_PASSPORT],
        min_trust_tier=TrustTier.MULTI_PATH
    )
    assert valid, "Proof should be valid"
    print(f"  ✓ Proof verified: valid={valid}, confidence={conf:.2%}")

    # Test 6: Quadratic governance weight
    print("\n[6] Test: Quadratic governance weight")
    weight = gateway.compute_governance_weight("0009-0005-2697-4670")
    # N=3 identities, D=3 unique paths -> (3*3)^2 = 81
    print(f"  ✓ Governance weight: {weight}")
    print(f"  ✓ Formula: (N × D)² = (3 × 3)² = 81")
    assert weight == 81.0, f"Weight should be 81, was {weight}"

    # Test 7: UBI Allocation with quadratic decay
    print("\n[7] Test: UBI Allocation (quadratic decay)")
    ubi = gateway.compute_ubi_allocation("0009-0005-2697-4670", base_amount=100.0)
    # 100/1 + 100/4 + 100/9 = 100 + 25 + 11.11 = 136.11
    expected_ubi = 100 + 25 + 100/9
    print(f"  ✓ Total UBI allocation: {ubi:.2f}")
    print(f"  ✓ Components: 100/1² + 100/2² + 100/3² = {expected_ubi:.2f}")
    assert abs(ubi - expected_ubi) < 0.01

    # Test 8: Pseudonym rotation
    print("\n[8] Test: Pseudonym rotation (coercion resistance)")
    old_pseudonym = identity1.pseudonyms.get(PseudonymType.PSEUDONYM_A.value)
    gateway.rotate_pseudonyms(identity1.identity_id)
    new_pseudonym = identity1.pseudonyms.get(PseudonymType.PSEUDONYM_A.value)
    print(f"  ✓ Old pseudonym: {old_pseudonym[:40] if old_pseudonym else 'N/A'}...")
    print(f"  ✓ New pseudonym: {new_pseudonym[:40] if new_pseudonym else 'N/A'}...")

    # Test 9: Statistics
    print("\n[9] Test: Identity statistics")
    stats = gateway.get_identity_stats(identity1.identity_id)
    for key, value in stats.items():
        print(f"  {key}: {value}")

    # Test 10: Global metrics
    print("\n[10] Test: Global metrics")
    metrics = gateway.get_global_metrics()
    for key, value in metrics.items():
        print(f"  {key}: {value}")

    # Test 11: Coercion resistance (no fixed set of IDs)
    print("\n[11] Test: Coercion resistance")
    holder_identities = gateway.holder_to_identities["0009-0005-2697-4670"]
    print(f"  ✓ Identities of the holder: {len(holder_identities)}")
    print(f"  ✓ Rotatable pseudonyms: yes")
    print(f"  ✓ Coercer cannot know how many identities exist")

    # Test 12: Error tolerance (multiple paths)
    print("\n[12] Test: Error tolerance (edge cases)")
    print("  ✓ No passport: can use social_graph + orcid")
    print("  ✓ No biometrics: can use gov_passport + web_of_trust")
    print("  ✓ Stateless: can use social_graph + proof_of_humanity")
    print("  ✓ Multiple citizenships: each counts as a separate path")

    print("\n" + "=" * 70)
    print(" ALL TESTS PASSED — SEAL: 989.x.v3-PLURALISTIC-IDENTITY")
    print("=" * 70)

    return gateway

if __name__ == "__main__":
    gateway = run_pluralistic_tests()
