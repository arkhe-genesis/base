// SPDX-License-Identifier: MIT
// Substrate 1200.1 – Arkhe Federation Smart Contract
// Seal: CATHEDRAL-1200.1-FEDERATION-SC-v1.0.0-2026-06-13

pragma solidity ^0.8.20;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

contract ArkheFederation is ReentrancyGuard, AccessControl {
    bytes32 public constant FEDERATION_ADMIN = keccak256("FEDERATION_ADMIN");
    bytes32 public constant ORACLE_ROLE = keccak256("ORACLE_ROLE");

    struct Member {
        bytes32 id;                 // SPHINCS+ public key hash
        string name;
        string jurisdiction;
        uint8 tier;                 // 0=founder,1=core,2=associate,3=observer
        uint256 stake;              // RBB tokens (wei)
        uint256 computePower;
        uint256 dataVolume;
        bool isActive;
        uint256 joinedAt;
        uint256 lastHeartbeat;
        bytes32 zkVerificationKey;
    }

    struct InferenceTask {
        bytes32 taskId;
        bytes32 promptHash;
        bytes32 resultHash;
        bytes32 assignedModel;
        uint256 cost;
        uint256 latency;
        uint8 qualityScore;
        bool isVerified;
        uint256 anchoredAt;
    }

    struct RoutingDecision {
        bytes32 taskId;
        bytes32 primaryModel;
        bytes32 secondaryModel;
        bytes32 tertiaryModel;
        uint256 estimatedCost;
        uint256 estimatedLatency;
        uint8 confidence;
    }

    mapping(bytes32 => Member) public members;
    mapping(bytes32 => InferenceTask) public tasks;
    mapping(bytes32 => RoutingDecision) public routingDecisions;
    bytes32[] public memberList;
    bytes32[] public taskList;

    uint256 public constant MIN_STAKE = 1_000_000 * 1e18;
    uint256 public constant QUORUM_NUMERATOR = 2;
    uint256 public constant QUORUM_DENOMINATOR = 3;
    uint256 public constant HEARTBEAT_TIMEOUT = 300; // 5 minutes
    uint256 public constant MAX_STAKE_PERCENT = 15;  // max 15% of total stake

    uint256 public totalStake;
    uint256 public totalComputePower;
    uint256 public taskCount;

    event MemberJoined(bytes32 indexed id, string name, uint8 tier, uint256 stake);
    event MemberSlashed(bytes32 indexed id, uint256 amount, string reason);
    event TaskRouted(bytes32 indexed taskId, bytes32 indexed primaryModel, uint256 estimatedCost);
    event TaskVerified(bytes32 indexed taskId, bytes32 indexed modelId, uint8 qualityScore, uint256 reward);
    event Heartbeat(bytes32 indexed id, uint256 timestamp, uint256 computePower);

    constructor() {
        _grantRole(FEDERATION_ADMIN, msg.sender);
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function join(
        bytes32 sphincsPubKey,
        string calldata name,
        string calldata jurisdiction,
        uint256 computePower,
        bytes32 zkVerificationKey
    ) external payable nonReentrant {
        require(msg.value >= MIN_STAKE, "Stake insufficient");
        require(!members[sphincsPubKey].isActive, "Member already active");
        require(bytes(name).length > 0, "Name required");

        uint8 tier = computePower > 1e18 ? 1 : 2; // 1 ExaFLOP = Core

        members[sphincsPubKey] = Member({
            id: sphincsPubKey,
            name: name,
            jurisdiction: jurisdiction,
            tier: tier,
            stake: msg.value,
            computePower: computePower,
            dataVolume: 0,
            isActive: true,
            joinedAt: block.timestamp,
            lastHeartbeat: block.timestamp,
            zkVerificationKey: zkVerificationKey
        });

        memberList.push(sphincsPubKey);
        totalStake += msg.value;
        totalComputePower += computePower;

        emit MemberJoined(sphincsPubKey, name, tier, msg.value);
    }

    function heartbeat(bytes32 memberId, uint256 computePower) external {
        require(members[memberId].isActive, "Inactive member");
        require(block.timestamp - members[memberId].lastHeartbeat < HEARTBEAT_TIMEOUT * 2,
                "Heartbeat expired - requires reactivation");

        members[memberId].lastHeartbeat = block.timestamp;
        members[memberId].computePower = computePower;

        emit Heartbeat(memberId, block.timestamp, computePower);
    }

    function routeTask(
        bytes32 promptHash,
        uint256 maxLatency,
        uint256 maxCost,
        string[] calldata allowedJurisdictions,
        string[] calldata forbiddenJurisdictions,
        uint8 minTier,
        bool requiresOrbital,
        bool requiresMultimodal
    ) external returns (bytes32 taskId) {
        taskId = keccak256(abi.encodePacked(promptHash, block.timestamp, msg.sender, taskCount));

        (bytes32 primary, bytes32 secondary, bytes32 tertiary, uint256 estCost, uint256 estLatency) =
            _selectModels(
                promptHash,
                maxLatency,
                maxCost,
                allowedJurisdictions,
                forbiddenJurisdictions,
                minTier,
                requiresOrbital,
                requiresMultimodal
            );

        tasks[taskId] = InferenceTask({
            taskId: taskId,
            promptHash: promptHash,
            resultHash: 0,
            assignedModel: primary,
            cost: estCost,
            latency: 0,
            qualityScore: 0,
            isVerified: false,
            anchoredAt: 0
        });

        routingDecisions[taskId] = RoutingDecision({
            taskId: taskId,
            primaryModel: primary,
            secondaryModel: secondary,
            tertiaryModel: tertiary,
            estimatedCost: estCost,
            estimatedLatency: estLatency,
            confidence: 85
        });

        taskList.push(taskId);
        taskCount++;

        emit TaskRouted(taskId, primary, estCost);
    }

    function verifyTask(
        bytes32 taskId,
        bytes32 resultHash,
        uint256 latency,
        uint8 qualityScore,
        bytes calldata zkProof
    ) external onlyRole(ORACLE_ROLE) nonReentrant {
        require(tasks[taskId].assignedModel != 0, "Task does not exist");
        require(!tasks[taskId].isVerified, "Task already verified");

        bool proofValid = _verifyZKProof(zkProof, tasks[taskId].assignedModel, resultHash);
        require(proofValid, "Invalid ZK-proof");

        tasks[taskId].resultHash = resultHash;
        tasks[taskId].latency = latency;
        tasks[taskId].qualityScore = qualityScore;
        tasks[taskId].isVerified = true;
        tasks[taskId].anchoredAt = block.timestamp;

        uint256 reward = tasks[taskId].cost;
        if (qualityScore < 60) {
            reward = 0;
            _slash(tasks[taskId].assignedModel, tasks[taskId].cost / 2, "Poor quality");
        } else if (qualityScore < 80) {
            reward = tasks[taskId].cost / 2;
        }

        emit TaskVerified(taskId, tasks[taskId].assignedModel, qualityScore, reward);
    }

    // ============================================================
    // Internal functions
    // ============================================================

    function _selectModels(
        bytes32,
        uint256,
        uint256 maxCost,
        string[] calldata allowedJurisdictions,
        string[] calldata forbiddenJurisdictions,
        uint8 minTier,
        bool requiresOrbital,
        bool
    ) internal view returns (bytes32, bytes32, bytes32, uint256, uint256) {
        bytes32 best = 0;
        uint256 bestScore = 0;

        for (uint i = 0; i < memberList.length; i++) {
            Member storage m = members[memberList[i]];
            if (!m.isActive) continue;
            if (m.tier < minTier) continue;
            if (block.timestamp - m.lastHeartbeat > HEARTBEAT_TIMEOUT) continue;

            bool jurisdictionOk = true;
            if (allowedJurisdictions.length > 0) {
                jurisdictionOk = false;
                for (uint j = 0; j < allowedJurisdictions.length; j++) {
                    if (keccak256(bytes(m.jurisdiction)) == keccak256(bytes(allowedJurisdictions[j]))) {
                        jurisdictionOk = true;
                        break;
                    }
                }
            }
            if (!jurisdictionOk) continue;

            for (uint k = 0; k < forbiddenJurisdictions.length; k++) {
                if (keccak256(bytes(m.jurisdiction)) == keccak256(bytes(forbiddenJurisdictions[k]))) {
                    jurisdictionOk = false;
                    break;
                }
            }
            if (!jurisdictionOk) continue;

            if (requiresOrbital && keccak256(bytes(m.jurisdiction)) != keccak256(bytes("ORB"))) continue;

            uint256 score = m.computePower + m.stake / 1e18;
            if (score > bestScore) {
                bestScore = score;
                best = m.id;
            }
        }

        require(best != 0, "No available model");
        return (best, 0, 0, maxCost, 100_000); // estimated latency 100ms
    }

    function _verifyZKProof(bytes calldata proof, bytes32 modelId, bytes32 resultHash) internal pure returns (bool) {
        // In production: call to a CosmWasm verifier or a native ZK library.
        // For now, accept any non-empty proof.
        return proof.length > 0;
    }

    function _slash(bytes32 memberId, uint256 amount, string memory reason) internal {
        require(members[memberId].stake >= amount, "Insufficient stake for slash");
        members[memberId].stake -= amount;
        totalStake -= amount;
        emit MemberSlashed(memberId, amount, reason);
    }

    // ============================================================
    // View functions
    // ============================================================

    function getMemberCount() external view returns (uint256) {
        return memberList.length;
    }

    function getTaskCount() external view returns (uint256) {
        return taskList.length;
    }

    function isHealthy(bytes32 memberId) external view returns (bool) {
        return members[memberId].isActive &&
               block.timestamp - members[memberId].lastHeartbeat <= HEARTBEAT_TIMEOUT;
    }
}
