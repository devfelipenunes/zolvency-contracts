# Fix Axelar Interoperability Plan

## Objective
Ensure that cross-chain reputation messages sent from Stellar are correctly funded and successfully relayed to the EVM testnet via Axelar. Resolve the deployment script errors that currently prevent end-to-end testing.

## Key Files & Context
- `contracts/adapters/axelar/src/lib.rs`: The Axelar Adapter contract on Stellar.
- `contracts/adapters/axelar/src/test.rs`: The unit tests for the Axelar Adapter.
- `scripts/testnet_deploy_and_test.js`: Deployment and testing script.

## Implementation Steps

### 1. Enable Unit Tests
- Open `contracts/adapters/axelar/src/lib.rs`.
- Add `#[cfg(test)] mod test;` at the end of the file to ensure the tests in `test.rs` are compiled and executed when running `cargo test`.

### 2. Implement Axelar Gas Payment
- In `contracts/adapters/axelar/src/lib.rs`, update the `call_axelar` function to pay for cross-chain execution gas.
- Retrieve the `GasService`, `GasToken` (or use native gas), and the `estimate_fee` (or accept a fee parameter).
- Call the `pay_gas` (or equivalent `pay_native_gas_for_contract_call`) function on the Axelar Gas Service contract *before* calling the Axelar Gateway.
- Update `test.rs` if necessary to ensure `test_send_flow` correctly asserts that the gas payment was made.

### 3. Fix Deployment Script
- Open `scripts/testnet_deploy_and_test.js`.
- Locate the contract creation logic (around line 88-92).
- Update the instantiation of `xdr.ContractIdPreimage` from the deprecated `new xdr.ContractIdPreimage(...)` syntax to the correct static factory method:
  ```javascript
  sourceId: xdr.ContractIdPreimage.contractIdPreimageFromAddress(
      new xdr.ContractIdPreimageFromAddress({
          address: Address.fromString(deployerAddress).toScAddress(),
          salt: crypto.randomBytes(32)
      })
  )
  ```

## Verification & Testing
1. Run `cargo test -p zolvency-axelar-adapter` and verify that `test_send_flow` passes.
2. Run `node scripts/testnet_deploy_and_test.js` (or the equivalent testnet deployment script) and verify that the `XdrWriterError` no longer occurs and the contracts are successfully deployed.
3. Trigger a test cross-chain message and verify on Axelarscan (testnet) that the message is picked up and executed.