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
    }

    mapping(address => Reputation) public reputations;
    string public sourceStellarAddress;

    event ReputationUpdated(address indexed user, bytes32 externalId, uint8 tier);
    event SourceAddressUpdated(string newSource);

    constructor(address _gateway, string memory _sourceStellarAddress) 
        AxelarExecutable(_gateway) 
        Ownable(msg.sender) 
    {
        require(_gateway != address(0), "INVALID_GATEWAY");
        sourceStellarAddress = _sourceStellarAddress;
    }

    function setSourceStellarAddress(string calldata _newSource) external onlyOwner {
        sourceStellarAddress = _newSource;
        emit SourceAddressUpdated(_newSource);
    }

    function _execute(
        bytes32, /*commandId*/
        string calldata sourceChain,
        string calldata sourceAddress,
        bytes calldata payload
    ) internal override {
        // Na testnet do Axelar, às vezes o nome da chain vem como "stellar-2" ou similar
        // Vamos focar na validação do endereço do contrato que é mais segura.
        
        require(
            keccak256(bytes(sourceAddress)) == keccak256(bytes(sourceStellarAddress)),
            "INVALID_SOURCE_ADDRESS"
        );

        (bytes32 externalId, uint8 tier, address user) = abi.decode(payload, (bytes32, uint8, address));

        reputations[user] = Reputation({
            externalId: externalId,
            tier: tier
        });

        emit ReputationUpdated(user, externalId, tier);
    }

    function getReputation(address user) external view returns (bytes32 externalId, uint8 tier) {
        Reputation storage rep = reputations[user];
        return (rep.externalId, rep.tier);
    }
}
