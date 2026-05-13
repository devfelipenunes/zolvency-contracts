// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import { ECDSA } from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import { MessageHashUtils } from "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title ZolvencyVerifierAuthority
 * @dev Verifies reputation using off-chain signatures from an authorized authority.
 */
contract ZolvencyVerifierAuthority is Ownable {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;

    struct Reputation {
        bytes32 externalId;
        uint8 tier;
    }

    // mapping(user => mapping(tokenType => Reputation))
    mapping(address => mapping(bytes32 => Reputation)) public reputations;
    // mapping(user => nonce)
    mapping(address => uint256) public nonces;
    address public authorityAddress;

    event ReputationUpdated(address indexed user, bytes32 indexed tokenType, bytes32 externalId, uint8 tier);

    constructor(address _authorityAddress) Ownable(msg.sender) {
        require(_authorityAddress != address(0), "INVALID_AUTHORITY_ADDRESS");
        authorityAddress = _authorityAddress;
    }

    function setAuthority(address _newAuthority) external onlyOwner {
        require(_newAuthority != address(0), "INVALID_AUTHORITY_ADDRESS");
        authorityAddress = _newAuthority;
    }

    /**
     * @notice Verifies authority signature and updates user reputation
     * @param user User address
     * @param tokenType Token type hash
     * @param externalId External platform ID hash
     * @param tier Reputation tier
     * @param nonce Current user nonce
     * @param signature Authority signature
     */
    function verifyAndSetReputation(
        address user,
        bytes32 tokenType,
        bytes32 externalId,
        uint8 tier,
        uint256 nonce,
        bytes calldata signature
    ) external {
        require(nonce == nonces[user], "INVALID_NONCE");
        
        bytes32 messageHash = keccak256(abi.encodePacked(
            block.chainid,
            address(this),
            user, 
            tokenType, 
            externalId, 
            tier,
            nonce
        ));
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();

        address signer = ethSignedMessageHash.recover(signature);
        require(signer == authorityAddress, "INVALID_AUTH");

        nonces[user]++;
        reputations[user][tokenType] = Reputation({
            externalId: externalId,
            tier: tier
        });

        emit ReputationUpdated(user, tokenType, externalId, tier);
    }

    function getReputation(address user, bytes32 tokenType) external view returns (bytes32 externalId, uint8 tier) {
        Reputation storage rep = reputations[user][tokenType];
        return (rep.externalId, rep.tier);
    }
}
