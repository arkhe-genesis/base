// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title FordefiBridgeAnchor
 * @dev Contrato de ancoragem de ZK-proofs de operações Fordefi na RBB Chain.
 *      Substrato 1066.1 — Fordefi Bridge Orchestrator
 *      Selo: FORDEFI-BRIDGE-1066.1-v1.0.0-2026-06-05
 *      Chain ID: 12120014
 *      Deidades: Hefesto (forja), Plutão (tesouro)
 */

contract FordefiBridgeAnchor {
    struct Anchor {
        bytes32 merkleRoot;
        string operationType;
        string vaultId;
        string axiarquiaStatus;
        uint256 timestamp;
        address submitter;
        bool verified;
    }

    mapping(bytes32 => Anchor) public anchors;
    mapping(string => bytes32[]) public vaultAnchors;

    bytes32[] public anchorList;

    event AnchorCreated(
        bytes32 indexed proofId,
        bytes32 merkleRoot,
        string vaultId,
        string operationType,
        uint256 timestamp
    );

    event AnchorVerified(
        bytes32 indexed proofId,
        bool valid,
        uint256 verifyTimestamp
    );

    modifier onlyAuthorized() {
        // Em produção: multi-sig 3/5 BNDES/TCU
        _;
    }

    function createAnchor(
        bytes32 _proofId,
        bytes32 _merkleRoot,
        string memory _operationType,
        string memory _vaultId,
        string memory _axiarquiaStatus
    ) external onlyAuthorized returns (bool) {
        require(anchors[_proofId].timestamp == 0, "Anchor already exists");

        anchors[_proofId] = Anchor({
            merkleRoot: _merkleRoot,
            operationType: _operationType,
            vaultId: _vaultId,
            axiarquiaStatus: _axiarquiaStatus,
            timestamp: block.timestamp,
            submitter: msg.sender,
            verified: false
        });

        vaultAnchors[_vaultId].push(_proofId);
        anchorList.push(_proofId);

        emit AnchorCreated(_proofId, _merkleRoot, _vaultId, _operationType, block.timestamp);

        return true;
    }

    function verifyAnchor(
        bytes32 _proofId,
        bytes32 _expectedMerkleRoot
    ) external onlyAuthorized returns (bool) {
        Anchor storage anchor = anchors[_proofId];
        require(anchor.timestamp != 0, "Anchor not found");

        bool valid = anchor.merkleRoot == _expectedMerkleRoot;
        anchor.verified = valid;

        emit AnchorVerified(_proofId, valid, block.timestamp);

        return valid;
    }

    function getAnchor(bytes32 _proofId) external view returns (Anchor memory) {
        return anchors[_proofId];
    }

    function getVaultAnchors(string memory _vaultId) external view returns (bytes32[] memory) {
        return vaultAnchors[_vaultId];
    }

    function getAnchorCount() external view returns (uint256) {
        return anchorList.length;
    }

    function isVerified(bytes32 _proofId) external view returns (bool) {
        return anchors[_proofId].verified;
    }
}
