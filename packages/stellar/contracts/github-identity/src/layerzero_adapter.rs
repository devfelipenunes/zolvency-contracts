use soroban_sdk::{Address, Bytes, Env, String, BytesN};
use crate::interop::MessengerTrait;
use crate::layerzero::{LayerZeroClient};
use crate::types::Error;
use crate::storage;

pub struct LayerZeroAdapter;

impl MessengerTrait for LayerZeroAdapter {
    fn send_reputation(
        env: Env,
        _caller: Address,
        _destination_chain: String,
        _destination_address: String,
        payload: Bytes,
    ) -> Result<(), Error> {
        let lz_config = storage::get_layerzero_config(&env)?;
        let lz_client = LayerZeroClient::new(&env, lz_config.endpoint);

        // 1. Map destination_chain (String) to dst_eid (u32)
        // In a real implementation, this would use a lookup table in storage.
        // For this adapter, we use a placeholder EID (e.g., 40161 for Sepolia).
        let dst_eid: u32 = 40161; 

        // 2. Convert destination_address (String hex) to BytesN<32>
        // In a real implementation, we would parse the hex string.
        // For now, we use a zeroed placeholder.
        let receiver = BytesN::from_array(&env, &[0u8; 32]);

        // 3. Execution Options
        // LayerZero V2 options (e.g., gas limit on destination)
        // For simplicity, we use empty options.
        let options = Bytes::new(&env);

        // 4. Quote Fee
        let fee = lz_client.quote(
            dst_eid,
            receiver.clone(),
            payload.clone(),
            options.clone(),
            false, // pay in native token (XLM)
        );

        // 5. Send Message
        lz_client.send(
            dst_eid,
            receiver,
            payload,
            options,
            fee,
        );

        Ok(())
    }
}
