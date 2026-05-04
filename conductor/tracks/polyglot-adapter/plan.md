# Polyglot Adapter & Complete Interoperability Plan

## 1. Objective
Implement a "Polyglot Adapter" architecture (Abordagem 1) within the `AxelarAdapter` on Stellar to dynamically support different payload serializations (EVM ABI vs Borsh) based on the destination ecosystem. This will enable true cross-chain interoperability with Ethereum (Sepolia), Cosmos, and Solana, utilizing a single, centralized Stellar codebase.

## 2. Key Files & Context
- **Contracts (Stellar):**
  - `contracts/nexus/src/lib.rs`: Needs `Ecosystem` enum in `CrossChainParams`.
  - `contracts/adapters/axelar/src/lib.rs`: Needs dynamic encoding functions (ABI vs Borsh).
- **Verifiers (Destination):**
  - `verifiers/evm/src/ZolvencyVerifierAxelar.sol`: Already handles ABI.
  - `verifiers/cosmos/src/contract.rs`: Needs `REPUTATION` handler implementation (Borsh).
  - `verifiers/solana/programs/zolvency-verifier-solana/src/lib.rs`: Needs `REPUTATION` handler implementation (Borsh).
- **Scripts:**
  - `scripts/full_cross_chain_test.sh`: Update to test all 3 chains.

## 3. Implementation Steps

### Phase 1: Soroban Contracts Refactoring (The Polyglot Core)
1. **Define Ecosystem Enum:**
   Create an `Ecosystem` enum (`Evm`, `Cosmos`, `Solana`) in both `Nexus` and `AxelarAdapter`.
2. **Update `CrossChainParams` in Nexus:**
   Add `ecosystem: Ecosystem` to the struct to allow users/clients to specify the target environment format.
3. **Refactor `AxelarAdapter` Serialization:**
   - Modify `send_reputation` and `send_will_auth` to receive the `Ecosystem` type.
   - Implement `encode_reputation_payload_evm` (existing) and `encode_reputation_payload_borsh` (new: Little-Endian, no 32-byte padding for small types).
   - Implement `encode_will_auth_payload_evm` (existing) and `encode_will_auth_payload_borsh` (new: Little-Endian, no padding).

### Phase 2: Verifiers Enhancement (Cosmos & Solana)
1. **Update Cosmos Verifier:**
   - Implement the `REPUTATION` (Type 1) payload handler using `borsh` deserialization.
   - Add state mapping for `reputations` (user + token_type -> tier, nonce, external_id).
2. **Update Solana Verifier:**
   - Implement the `REPUTATION` (Type 1) payload handler using `borsh`.
   - Create a PDA for storing the user's reputation.

### Phase 3: Testing & Automation
1. **Unit Tests:** Add tests in `AxelarAdapter` to verify byte outputs of EVM vs Borsh payloads.
2. **Deployment Script:** Create/update a script (`test_all_networks.sh`) to deploy the EVM, Cosmos (mock/local), and Solana verifiers alongside the Stellar contracts, and execute the full cross-chain flow for each ecosystem.

## 4. Verification & Testing
- Deploy the updated Soroban contracts to Stellar Testnet.
- Trigger `export_reputation` and `export_will_authority` for all three `Ecosystem` targets.
- Validate that the payload emitted to Axelar matches the expected formatting (ABI vs Borsh) without reverting.
