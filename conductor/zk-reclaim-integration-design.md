# ZK Proof (Reclaim) On-Chain Integration Design

## 1. Objective
Integrate the on-chain verification of Reclaim Protocol (zkTLS) proofs into the `github-identity` contract. This will allow users to mint a Zolvency GitHub Soulbound Token (SBT) only by presenting a valid cryptographic proof of their GitHub account ownership, fully complying with the ZTS-01 Sybil Resistance standard.

## 2. Architecture Fit (Hub & Spoke Model)
Currently, `github-identity` accepts a `proof_data: Bytes` parameter but does not cryptographically verify it. 

### Proposed Flow:
1. **Frontend (zkTLS):** User logs into GitHub via Reclaim. Reclaim's network generates a signed proof containing the user's GitHub ID and username.
2. **Proof Submission:** The frontend sends this proof as a serialized `Bytes` payload to `github-identity`'s `mint` function.
3. **On-Chain Verification:** The `github-identity` contract decodes the proof and verifies the digital signature against a trusted Reclaim Witness Public Key (or via a delegated `zk_verifier` contract).
4. **Data Cross-Check & Mint:** The contract asserts that the `soul_id` inside the proof's context matches the caller's `soul_id` (preventing front-running), then executes the ZTS-01 standard logic (Sybil check, increment counter, emit reputation).

## 3. Data Structures for Soroban
To verify Reclaim proofs natively on Soroban, we need to parse the serialized proof. A simplified representation of a Reclaim proof for Soroban:

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimInfo {
    pub provider: String,        // e.g., "github-account-verification"
    pub parameters: String,      // JSON string with github ID/Username
    pub context: String,         // Custom context (MUST contain user's soul_id)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReclaimProof {
    pub claim_info: ClaimInfo,
    pub signed_claim: BytesN<32>, // The hash of the claim data
    pub signatures: Vec<BytesN<64>>, // Ed25519 signatures from Reclaim Validators
}
```

## 4. Verification Logic (Core Implementation)
The verification relies on Soroban's native host functions, keeping compute costs low.

1. **Hash the Claim:** Compute the hash of `claim_info`.
2. **Verify Signature:** 
   ```rust
   // Verify that the signature is from a trusted Reclaim witness
   env.crypto().ed25519_verify(
       &trusted_witness_pubkey, 
       &proof.signed_claim, 
       &proof.signatures.get(0).unwrap()
   );
   ```
3. **Validate Context (Anti-Front-Running):**
   Ensure the proof was generated specifically for the `soul_id` calling the contract.
   ```rust
   // Pseudo-code
   let context_data: Context = parse_json(&proof.claim_info.context);
   if context_data.soul_id != caller_soul_id {
       return Err(Error::ProofIdentityMismatch);
   }
   ```

## 5. Required Contract Changes (`github-identity/src/lib.rs`)

1. **Update `get_source`:** Change from `"zk-email"` to `"reclaim-zktls"`.
2. **Update `mint` logic:**
   - Deserialize `params.proof_data` into `ReclaimProof`.
   - Implement the `verify_proof` logic (either inline or via `config.zk_verifier` if delegating).
   - Extract `external_id` (GitHub ID) and `username` from the verified `claim_info.parameters` instead of trusting the user's `params` input.
   - Enforce `ZTS-01` Sybil resistance mapping using the verified `external_id`.

## 6. Security Considerations
- **Witness Management:** The contract must store or have access to the valid Reclaim Witness Public Keys. We can store this in the `Config` or `Instance` storage.
- **Proof Replay:** A Reclaim proof shouldn't be used twice. The `ZTS-01` rule mappings (`storage::set_sybil_mapping`) already prevent the same `external_id` from being minted twice, implicitly mitigating proof replay for identity.

## 7. Next Steps for Implementation
If approved, the next steps are to write the Rust code to decode the Reclaim proof and integrate the Ed25519 verification loop into the `github-identity` contract.