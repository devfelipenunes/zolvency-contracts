// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { IAxelarGateway } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/interfaces/IAxelarGateway.sol";
import { AxelarExecutable } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/executable/AxelarExecutable.sol";

/**
 * @title ZolvencyVerifier
 * @dev Receiver contract for cross-chain reputation updates from Stellar.
 */
contract ZolvencyVerifier is AxelarExecutable {
    struct Reputation {
        bytes32 externalId;
        uint8 tier;
    }

    // Mapping from user address to their reputation
    mapping(address => Reputation) public reputations;
    
    // The authorized source contract address on Stellar
    string public sourceStellarAddress;

    /**
     * @param _gateway Address of the Axelar Gateway on the destination chain.
     * @param _sourceStellarAddress The Stellar address of the GithubIdentityContract.
     */
    constructor(address _gateway, string memory _sourceStellarAddress) AxelarExecutable(_gateway) {
        sourceStellarAddress = _sourceStellarAddress;
    }

    /**
     * @dev Internal function called by AxelarExecutable when a message is received.
     * @param sourceChain The name of the source chain (should be "stellar").
     * @param sourceAddress The address of the sender contract (should match sourceStellarAddress).
     * @param payload The encoded data: (bytes32 externalId, uint8 tier, address user).
     */
    function _execute(
        string calldata sourceChain,
        string calldata sourceAddress,
        bytes calldata payload
    ) internal override {
        // Verify source chain
        require(keccak256(bytes(sourceChain)) == keccak256(bytes("stellar")), "INVALID_SOURCE_CHAIN");
        
        // Verify source address
        require(
            keccak256(bytes(sourceAddress)) == keccak256(bytes(sourceStellarAddress)),
            "INVALID_SOURCE_ADDRESS"
        );

        // Decode payload
        // Expected format: (bytes32 externalId, uint8 tier, address user)
        // Each element is padded to 32 bytes as per standard ABI encoding for static types.
        (bytes32 externalId, uint8 tier, address user) = abi.decode(payload, (bytes32, uint8, address));

        // Update storage
        reputations[user] = Reputation({
            externalId: externalId,
            tier: tier
        });
    }

    /**
     * @dev Helper function to check if a user has a specific reputation.
     */
    function getReputation(address user) external view returns (bytes32 externalId, uint8 tier) {
        Reputation storage rep = reputations[user];
        return (rep.externalId, rep.tier);
    }
}
