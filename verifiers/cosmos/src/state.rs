use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

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

pub const CONFIG: Item<Config> = Item::new("config");
pub const WILLS: Map<u32, WillPermission> = Map::new("wills");
