# Technical Spec: Agentic Proof of Authority (PoA)

## 1. Overview
The **Agentic PoA** is a mechanism that allows a Soul ID owner (Human) to delegate specific transaction rights to an autonomous Agent (AI) across multiple blockchains.

## 2. The Delegation Workflow

### A. Authorization (Stellar Hub)
1. **User Request:** The user signs a message (using their Ed25519 key or Passkey) authorizing an agent address (`agent_evm_addr`) for specific `permissions`.
2. **Registry Entry:** The `Nexus` on Stellar stores this delegation:
   ```rust
   struct AgentAuthorization {
       agent_address: Bytes, // EVM or other chain address
       soul_id: u32,
       permissions: u64, // Bitmask of allowed actions
       expiry: u64,
       nonce: u64,
   }
   ```
3. **ZK Proof (Optional):** The authorization can be wrapped in a ZK proof to hide the link between the human and the agent on-chain if desired.

### B. Export (Axelar Bridge)
1. The user (or the agent) triggers `export_agent_authority` on the Stellar Registry.
2. The Registry calls the `AxelarAdapter`, passing the delegation details.
3. Axelar sends a Cross-Chain Message to the destination chain (e.g., Ethereum).

### C. Verification & Execution (Target Chain)
1. **Reception:** The `ZolvencyVerifierAxelar` on the target chain receives the message.
2. **State Update:** It maps the agent address to its permissions and the linked Soul ID.
3. **Gating:** A target DeFi contract calls `ZolvencyVerifierAxelar.canExecute(agent, action)` before allowing the agent to transact.

## 3. Data Schema (EVM)
On the Solidity side, we store the authorizations as follows:

```solidity
struct AgentPermission {
    uint32 soulId;
    uint64 permissions;
    uint64 expiry;
}

mapping(address => AgentPermission) public authorizedAgents;
```

## 4. Monetization & Payments (x402)
To ensure the sustainability of the infrastructure and provide agents with autonomous financial capabilities, we integrate the **x402 protocol**:

1. **Trigger:** When an Agent requests a cross-chain export but hasn't paid the fee, the Zolvency API/Gateway returns an `HTTP 402 Payment Required`.
2. **Payment:** The Agent signs a Stellar transaction to pay the `export_fee` to the Zolvency Facilitator.
3. **Execution:** Once the payment is verified by the x402 facilitator, the `export_agent_authority` function is triggered on-chain, and the Axelar message is sent.

This allows agents to be self-sufficient, paying for their own trust verification without requiring constant human intervention for every gas fee.

## 5. Security Considerations
- **Nonce Protection:** Every delegation must have a nonce to prevent replay attacks across chains.
- **Expiry:** All delegations must have an expiration timestamp to limit the risk of long-term agent compromise.
- **Revocation:** A user can issue a "Revocation" message on Stellar, which is then propagated cross-chain to disable the agent.

## 5. Phase 1 Implementation Goals
- Add `authorize_agent` to `Nexus`.
- Extend `ZolvencyVerifierAxelar.sol` to handle the `AGENT_AUTHORIZATION` payload type.
- Implement a basic `AgentGated` modifier for third-party contracts to consume.
