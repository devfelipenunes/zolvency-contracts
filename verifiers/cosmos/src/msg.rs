use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Chamado pelo middleware de IBC do Axelar
    Execute {
        source_chain: String,
        source_address: String,
        payload: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(WillResponse)]
    GetWill { soul_id: u32 },
}

#[cw_serde]
pub struct WillResponse {
    pub soul_id: u32,
    pub will_address: String,
    pub expiry: u64,
}
