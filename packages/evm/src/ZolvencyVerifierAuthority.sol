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

    mapping(address => Reputation) public reputations;
    address public authorityAddress;

    event ReputationUpdated(address indexed user, bytes32 externalId, uint8 tier);

    constructor(address _authorityAddress) Ownable(msg.sender) {
        require(_authorityAddress != address(0), "INVALID_AUTHORITY_ADDRESS");
        authorityAddress = _authorityAddress;
    }

    function setAuthority(address _newAuthority) external onlyOwner {
        require(_newAuthority != address(0), "INVALID_AUTHORITY_ADDRESS");
        authorityAddress = _newAuthority;
    }

    /**
     * @dev Verifies a signature from the authority and updates reputation.
     * @param user The address of the user.
     * @param externalId The user's external ID (e.g. GitHub ID hash).
     * @param tier The reputation tier.
     * @param signature The signature from the authority.
     */
    function verifyAndSetReputation(
        address user,
        bytes32 externalId,
        uint8 tier,
        bytes calldata signature
    ) external {
        // Construct the hash that was signed
        bytes32 messageHash = keccak256(abi.encodePacked(user, externalId, tier));
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();

        // Verify the signer is our authority
        address signer = ethSignedMessageHash.recover(signature);
        require(signer == authorityAddress, "INVALID_AUTHORITY_SIGNATURE");

        // Update reputation
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
