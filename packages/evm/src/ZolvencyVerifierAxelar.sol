// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { AxelarExecutable } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/executable/AxelarExecutable.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ZolvencyVerifierAxelar
 * @dev Receiver contract for cross-chain reputation updates from Stellar.
 */
contract ZolvencyVerifierAxelar is AxelarExecutable, Ownable {
    struct Reputation {
        bytes32 externalId;
        uint8 tier;
        uint64 nonce;
    }

    // mapping(user => mapping(tokenType => Reputation))
    mapping(address => mapping(bytes32 => Reputation)) public reputations;
    string public sourceStellarChain;
    string public sourceStellarAddress;

    event ReputationUpdated(address indexed user, bytes32 indexed tokenType, bytes32 externalId, uint8 tier);
    event SourceConfigUpdated(string newChain, string newSource);

    constructor(address _gateway, string memory _sourceStellarChain, string memory _sourceStellarAddress) 
        AxelarExecutable(_gateway) 
        Ownable(msg.sender) 
    {
        require(_gateway != address(0), "INVALID_GATEWAY");
        sourceStellarChain = _sourceStellarChain;
        sourceStellarAddress = _sourceStellarAddress;
    }

    function setSourceConfig(string calldata _newChain, string calldata _newSource) external onlyOwner {
        sourceStellarChain = _newChain;
        sourceStellarAddress = _newSource;
        emit SourceConfigUpdated(_newChain, _newSource);
    }

    function _execute(
        bytes32, /*commandId*/
        string calldata sourceChain,
        string calldata sourceAddress,
        bytes calldata payload
    ) internal override {
        require(
            keccak256(bytes(sourceChain)) == keccak256(bytes(sourceStellarChain)),
            "INVALID_SOURCE_CHAIN"
        );
        require(
            keccak256(bytes(sourceAddress)) == keccak256(bytes(sourceStellarAddress)),
            "INVALID_SOURCE_ADDRESS"
        );

        (bytes32 externalId, uint256 tier, address user, uint256 nonce, bytes32 tokenType) = abi.decode(
            payload, 
            (bytes32, uint256, address, uint256, bytes32)
        );

        // Proteção contra replay ou fora de ordem
        require(nonce > reputations[user][tokenType].nonce, "INVALID_NONCE");

        reputations[user][tokenType] = Reputation({
            externalId: externalId,
            tier: uint8(tier),
            nonce: uint64(nonce)
        });

        emit ReputationUpdated(user, tokenType, externalId, uint8(tier));
    }

    function getReputation(address user, bytes32 tokenType) external view returns (bytes32 externalId, uint8 tier) {
        Reputation storage rep = reputations[user][tokenType];
        return (rep.externalId, rep.tier);
    }
}
