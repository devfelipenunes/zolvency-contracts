# Design Specification: Authority-Pull Interoperability for Zolvency SBTs
**Date:** 2026-04-17
**Status:** Draft (Pending User Review)
**Reference:** Replaces `2026-04-16-axelar-interop-design.md`

## 1. Objective
Enable cost-efficient and provider-agnostic cross-chain verification of Zolvency Soulbound Tokens (SBTs) using an **Authority-Pull** model. This design eliminates dependency on specific cross-chain bridges (like Axelar) and reduces gas costs for users by verifying off-chain attestations on the destination chain.

## 2. Architecture

### 2.1 Components
- **Stellar (Source of Truth):** `GithubIdentityContract` manages identity and reputation (tiers).
- **Zolvency Attestor (Off-chain):** A service that verifies Stellar state and signs ECDSA attestations (EIP-712 compatible).
- **EVM (Destination):** `ZolvencyVerifier.sol` verifies the Attestor's signature and maps reputation to EVM addresses.

### 2.2 Data Flow (Pull Model)
1. **Identity Minting:** User earns/updates reputation on Stellar. No cross-chain message is sent during this transaction.
2. **Attestation Request:** When the user wants to use their reputation on an EVM chain, they request an attestation from the Zolvency Attestor service.
3. **Verification & Signing:** 
   - The Attestor checks the user's current Tier and External ID on the Stellar `GithubIdentityContract`.
   - The Attestor generates a payload: `(address user_evm_addr, uint8 tier, bytes32 external_id_hash, uint256 salt)`.
   - The Attestor signs this payload using a dedicated Private Key.
4. **Claiming on EVM:**
   - The user calls `claimReputation` on the `ZolvencyVerifier` contract on the destination chain, providing the payload and the signature.
   - The contract verifies the signature against the registered `attestorAddress`.
   - If valid, the contract updates its local reputation mapping for the user.

## 3. Technical Implementation

### 3.1 Stellar Contract Changes (Cleanup)
- **Removal:** Remove all Axelar-related code (`axelar.rs`, `gateway` calls, `gas_service` payments).
- **Optimization:** Keep the contract focused on local SBT logic and events.
- **Linkage:** (Optional) Add a function to map a Stellar Address to an EVM Address to strengthen the attestation bond.

### 3.2 EVM Contract (`ZolvencyVerifier.sol`)
- **Storage:**
  - `address public attestorAddress`: The public key of the authorized signer.
  - `mapping(address => Reputation) public reputations`: Locally stored verified reputation.
  - `mapping(bytes32 => bool) public usedSalts`: Prevents replay attacks.
- **Functions:**
  - `claimReputation(uint8 tier, bytes32 externalIdHash, uint256 salt, bytes calldata signature)`:
    - Reconstructs the EIP-712 hash.
    - Recovers the signer via `ecrecover`.
    - Validates `signer == attestorAddress`.
    - Checks `!usedSalts[salt]`.
    - Updates `reputations[msg.sender]`.

## 4. Security Considerations
- **Signer Security:** The Attestor's Private Key is the most sensitive component. It must be managed via a Secure Enclave or KMS.
- **Replay Protection:** The `salt` (or nonce) and the inclusion of `msg.sender` in the signature ensure that a signature cannot be reused or stolen by another address.
- **Data Freshness:** Signatures should include a expiration timestamp to ensure users are using up-to-date reputation data.

## 5. Success Criteria
- Stellar contract is clean and free of bridge-specific dependencies.
- EVM contract can verify a signature from the Attestor in a single, low-gas transaction.
- The system is agnostic to the underlying transport layer (it works as long as the user can provide the signature).
