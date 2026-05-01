// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { AxelarExecutable } from "@axelar-network/axelar-gmp-sdk-solidity/contracts/executable/AxelarExecutable.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ZolvencyVerifierAxelar
 * @dev Verifier contract for cross-chain reputation updates from Stellar (Zenith Protocol).
 */
contract ZolvencyVerifierAxelar is AxelarExecutable, Ownable {
    struct Reputation {
        bytes32 externalId;
        uint8 tier;
        uint64 nonce;
    }

    struct WillPermission {
        uint32 soulId;
        uint64 permissions;
        uint64 expiry;
    }

    // mapping(user => mapping(tokenType => Reputation))
    mapping(address => mapping(bytes32 => Reputation)) public reputations;
    
    // mapping(willAddress => WillPermission)
    mapping(address => WillPermission) public authorizedWills;

    string public sourceStellarChain;
    string public sourceStellarAddress;

    event ReputationUpdated(address indexed user, bytes32 indexed tokenType, bytes32 externalId, uint8 tier);
    event WillAuthorized(address indexed will, uint32 soulId, uint64 permissions, uint64 expiry);
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

        uint8 payloadType = uint8(payload[0]);
        bytes memory data = new bytes(payload.length - 1);
        for (uint256 i = 0; i < payload.length - 1; i++) {
            data[i] = payload[i + 1];
        }
        
        if (payloadType == 1) { // REPUTATION
            (bytes32 externalId, uint256 tier, address user, uint256 nonce, bytes32 tokenType) = abi.decode(
                data, 
                (bytes32, uint256, address, uint256, bytes32)
            );
            require(nonce > reputations[user][tokenType].nonce, "INVALID_NONCE");
            reputations[user][tokenType] = Reputation({
                externalId: externalId,
                tier: uint8(tier),
                nonce: uint64(nonce)
            });
            emit ReputationUpdated(user, tokenType, externalId, uint8(tier));
        } 
        else if (payloadType == 2) { // WILL_AUTH
            (address will, uint32 soulId, uint64 permissions, uint64 expiry) = abi.decode(
                data,
                (address, uint32, uint64, uint64)
            );
            authorizedWills[will] = WillPermission({
                soulId: soulId,
                permissions: permissions,
                expiry: expiry
            });
            emit WillAuthorized(will, soulId, permissions, expiry);
        }
        else if (payloadType == 3) { // REVOKE_WILL
            address will = abi.decode(data, (address));
            delete authorizedWills[will];
        }
    }

    function getReputation(address user, bytes32 tokenType) external view returns (bytes32 externalId, uint8 tier) {
        Reputation storage rep = reputations[user][tokenType];
        return (rep.externalId, rep.tier);
    }

    /**
     * @dev Checks if a Will is authorized to perform an action.
     * @param will The address of the will.
     * @param requiredPermission The bitmask of the required permission.
     */
    function canExecute(address will, uint64 requiredPermission) external view returns (bool) {
        WillPermission storage auth = authorizedWills[will];
        if (block.timestamp > auth.expiry) return false;
        return (auth.permissions & requiredPermission) == requiredPermission;
    }
}
