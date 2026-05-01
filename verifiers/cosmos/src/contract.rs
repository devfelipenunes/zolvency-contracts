#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WillResponse};
use crate::state::{Config, WillPermission, CONFIG, WILLS};
use borsh::BorshDeserialize;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        admin: deps.api.addr_validate(&msg.admin)?.to_string(),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Execute { source_chain, source_address, payload } => {
            execute_axelar_message(deps, info, source_chain, source_address, payload)
        }
    }
}

pub fn execute_axelar_message(
    deps: DepsMut,
    _info: MessageInfo,
    _source_chain: String,
    _source_address: String,
    payload: Binary,
) -> Result<Response, ContractError> {
    let payload_bytes = payload.as_slice();
    
    // WILL_AUTH = 2
    if payload_bytes[0] == 2 {
        let mut data = &payload_bytes[1..];
        
        let soul_id = u32::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let will_address_bytes = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let expiry = u64::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;

        let will_address = hex::encode(will_address_bytes);

        let permission = WillPermission {
            soul_id,
            will_address: will_address.clone(),
            expiry,
        };

        WILLS.save(deps.storage, soul_id, &permission)?;

        return Ok(Response::new()
            .add_attribute("action", "authorize_will")
            .add_attribute("soul_id", soul_id.to_string())
            .add_attribute("will", will_address));
    }

    Err(ContractError::InvalidPayload {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetWill { soul_id } => to_json_binary(&query_will(deps, soul_id)?),
    }
}

fn query_will(deps: Deps, soul_id: u32) -> StdResult<WillResponse> {
    let will = WILLS.load(deps.storage, soul_id)?;
    Ok(WillResponse {
        soul_id: will.soul_id,
        will_address: will.will_address,
        expiry: will.expiry,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use borsh::BorshSerialize;

    #[test]
    fn test_execute_will_auth() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { admin: "admin".to_string() };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let soul_id = 123u32;
        let will_address_bytes = [0u8; 32];
        let expiry = 1714521600u64;

        let mut payload_data = vec![2u8]; // WILL_AUTH
        payload_data.extend(borsh::to_vec(&soul_id).unwrap());
        payload_data.extend(borsh::to_vec(&will_address_bytes).unwrap());
        payload_data.extend(borsh::to_vec(&expiry).unwrap());

        let msg = ExecuteMsg::Execute {
            source_chain: "stellar".to_string(),
            source_address: "nexus_addr".to_string(),
            payload: Binary::from(payload_data),
        };

        let info = mock_info("axelar_relayer", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes[0].value, "authorize_will");
        assert_eq!(res.attributes[1].value, "123");

        // Verify storage
        let will = query_will(deps.as_ref(), 123).unwrap();
        assert_eq!(will.soul_id, 123);
        assert_eq!(will.expiry, expiry);
    }
}
