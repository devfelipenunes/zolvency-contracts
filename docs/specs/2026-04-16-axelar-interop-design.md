# Design Specification: Axelar Interoperability for Zolvency SBTs
**Date:** 2026-04-16
**Status:** Approved

## 1. Objective
Enable cross-chain verification of Zolvency Soulbound Tokens (SBTs) using Axelar General Message Passing (GMP). This allows reputation earned on Stellar (e.g., GitHub activity) to be programmatically verified on EVM-compatible networks.

## 2. Architecture

### 2.1 Components
- **Stellar (Source):** `GithubIdentityContract` (modified to act as a GMP sender).
- **Axelar Infrastructure:** 
    - `Axelar Gateway`: Handles outgoing cross-chain messages.
    - `Axelar Gas Service`: Manages prepayment of cross-chain execution fees.
- **EVM (Destination):** `ZolvencyVerifier.sol` (implements `IAxelarExecutable` to receive and store reputation data).

### 2.2 Data Flow (Push Model)
1. **Trigger:** A user calls `mint` or `update_token` on the `GithubIdentityContract` (Stellar).
2. **Execution:**
    - The contract mints/updates the SBT locally.
    - The contract encodes the reputation data (`external_id`, `tier`) and the user's destination address into a payload.
    - The contract pays the cross-chain gas fee to the `AxelarGasService`.
    - The contract calls `call_contract` on the `AxelarGateway`.
3. **Relay:** Axelar Validators verify the Stellar transaction and relay the message to the destination chain.
4. **Verification:** The `ZolvencyVerifier` contract on the destination chain receives the message, verifies the source, and updates the user's reputation mapping.

## 3. Technical Implementation

### 3.1 Stellar Contract Changes
- **Configuration:** Add storage for `gateway_address`, `gas_service_address`, and `gas_token_address`.
- **Interface Update:**
    - `mint` and `update_token` will accept:
        - `destination_chain: String` (e.g., "ethereum-sepolia").
        - `destination_address: String` (The EVM Verifier contract address).
        - `user_destination_address: Bytes` (The 20-byte EVM wallet address).
- **Logic:**
    - Prepay gas: `gas_service.pay_gas(...)`.
    - Send message: `gateway.call_contract(...)`.

### 3.2 EVM Contract (`ZolvencyVerifier.sol`)
- **State:** `mapping(address => Reputation) public reputations`.
- **Reputation Struct:** `{ bytes32 externalIdHash, uint8 tier }`.
- **Function:** `_execute(string calldata sourceChain, string calldata sourceAddress, bytes calldata payload)`
    - Validate `sourceChain == "stellar"` and `sourceAddress == [Stellar GithubIdentityContract Address]`. (Note: This address will be set during deployment or initialization).
    - Decode `payload` using `abi.decode`.
    - Update the `reputations` mapping.

## 4. Security Considerations
- **Source Authentication:** The destination contract MUST strictly verify the `sourceAddress` to prevent spoofed reputation updates.
- **Sybil Resistance:** The mapping on the destination chain should ensure that a single `external_id_hash` (GitHub ID) cannot be used to boost multiple addresses.
- **Fee Management:** Users are responsible for both the Stellar mint fee and the Axelar cross-chain gas fee.

## 5. Success Criteria
- A user can mint a GitHub SBT on Stellar and see their reputation reflected in the EVM contract within minutes.
- Only authorized updates from the Stellar contract are accepted by the EVM contract.
