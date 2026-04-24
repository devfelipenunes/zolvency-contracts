# Design Specification: Multi-Protocol Cross-Chain Adapter for Zolvency
**Date:** 2026-04-17
**Status:** Draft (Multi-Protocol Update)
**Reference:** Replaces previous Axelar-only design.

## 1. Objective
Implement a provider-agnostic interoperability layer for Zolvency SBTs. This allows the protocol to switch between different cross-chain providers (Axelar, LayerZero V2) via administrative configuration, enabling performance and cost comparisons without core logic changes.

## 2. Architecture

### 2.1 The Adapter Pattern
- **Zolvency Identity Contract (Core):** Calls a generic `Messenger` interface.
- **Messenger Trait (Soroban):** Defines the standard `send_reputation` method.
- **Axelar Adapter:** Implements the interface using Axelar GMP.
- **LayerZero Adapter:** Implements the interface using LayerZero V2 OApp/OFT standards.

### 2.2 Components
- **Stellar (Source):** `GithubIdentityContract` + `MessengerRegistry`.
- **Adapters:** Independent modules/contracts for each provider.
- **EVM (Destination):** 
    - `ZolvencyVerifierAxelar.sol` (Executable)
    - `ZolvencyVerifierLayerZero.sol` (OApp)

## 3. Data Flow
1. **Trigger:** `mint` or `update_token` is called on Stellar.
2. **Dispatch:** Core contract checks `active_protocol` in storage.
3. **Execution:** 
    - If `Axelar`: Calls `AxelarAdapter::send`.
    - If `LayerZero`: Calls `LayerZeroAdapter::send`.
4. **Relay:** The selected provider transports the message to its respective EVM verifier.

## 4. Technical Implementation

### 4.1 Storage (Stellar)
```rust
pub enum InteropProtocol {
    None,
    Axelar,
    LayerZero,
}

pub struct InteropConfig {
    pub active_protocol: InteropProtocol,
    pub adapter_address: Address,
}
```

### 4.2 EVM Verifiers
Two distinct verifier contracts will be deployed to evaluate:
1. **Gas Consumption:** Comparing the cost of receiving messages on each protocol.
2. **Time to Finality:** Measuring how long each protocol takes to reflect reputation on the destination chain.

## 5. Success Criteria
- Ability to switch protocols via a single admin transaction.
- Successful reputation push via both Axelar and LayerZero.
- Comparative report on cost and speed for both providers.
