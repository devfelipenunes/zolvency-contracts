use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};
use borsh::{BorshSerialize, BorshDeserialize};

#[cw_serde]
pub struct Config {
    pub admin: String,
}

#[cw_serde]
pub struct WillPermission {
    pub soul_id: u32,
    pub will_address: String,
    pub expiry: u64,
}

#[cw_serde]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Reputation {
    pub soul_id: u32,
    pub external_id: String,
    pub tier: u32,
    pub nonce: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WILLS: Map<u32, WillPermission> = Map::new("wills");
pub const REPUTATIONS: Map<(&str, &str), Reputation> = Map::new("reputations"); // (user_hex, token_type_hex)
