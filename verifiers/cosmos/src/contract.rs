#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WillResponse};
use crate::state::{Config, WillPermission, Reputation, CONFIG, WILLS, REPUTATIONS};
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
    
    // REPUTATION = 1
    if payload_bytes[0] == 1 {
        let mut data = &payload_bytes[1..];
        
        let soul_id = u32::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let user_addr_bytes = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let external_id_hash = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let tier = u32::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let nonce = u64::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let token_type_hash = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;

        let user_hex = hex::encode(user_addr_bytes);
        let token_type_hex = hex::encode(token_type_hash);
        let external_id = hex::encode(external_id_hash);

        let reputation = Reputation {
            soul_id,
            external_id,
            tier,
            nonce,
        };

        REPUTATIONS.save(deps.storage, (&user_hex, &token_type_hex), &reputation)?;

        return Ok(Response::new()
            .add_attribute("action", "update_reputation")
            .add_attribute("soul_id", soul_id.to_string())
            .add_attribute("user", user_hex)
            .add_attribute("token_type", token_type_hex));
    }

    // WILL_AUTH = 2
    if payload_bytes[0] == 2 {
        let mut data = &payload_bytes[1..];
        
        let soul_id = u32::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let will_address_bytes = <[u8; 32]>::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
        let _permissions = u64::deserialize(&mut data).map_err(|_| ContractError::InvalidPayload {})?;
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
        QueryMsg::GetReputation { user_hex, token_type_hex } => 
            to_json_binary(&query_reputation(deps, user_hex, token_type_hex)?),
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

fn query_reputation(deps: Deps, user_hex: String, token_type_hex: String) -> StdResult<Reputation> {
    let reputation = REPUTATIONS.load(deps.storage, (&user_hex, &token_type_hex))?;
    Ok(reputation)
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn test_execute_will_auth() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { admin: "admin".to_string() };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let soul_id = 123u32;
        let will_address_bytes = [0u8; 32];
        let permissions = 0xFFFFu64;
        let expiry = 1714521600u64;

        let mut payload_data = vec![2u8]; // WILL_AUTH
        payload_data.extend(borsh::to_vec(&soul_id).unwrap());
        payload_data.extend(borsh::to_vec(&will_address_bytes).unwrap());
        payload_data.extend(borsh::to_vec(&permissions).unwrap());
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

    #[test]
    fn test_execute_reputation() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { admin: "admin".to_string() };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let soul_id = 456u32;
        let user_addr_bytes = [1u8; 32];
        let external_id_hash = [2u8; 32];
        let tier = 3u32;
        let nonce = 100u64;
        let token_type_hash = [4u8; 32];

        let mut payload_data = vec![1u8]; // REPUTATION
        payload_data.extend(borsh::to_vec(&soul_id).unwrap());
        payload_data.extend(borsh::to_vec(&user_addr_bytes).unwrap());
        payload_data.extend(borsh::to_vec(&external_id_hash).unwrap());
        payload_data.extend(borsh::to_vec(&tier).unwrap());
        payload_data.extend(borsh::to_vec(&nonce).unwrap());
        payload_data.extend(borsh::to_vec(&token_type_hash).unwrap());

        let msg = ExecuteMsg::Execute {
            source_chain: "stellar".to_string(),
            source_address: "nexus_addr".to_string(),
            payload: Binary::from(payload_data),
        };

        let info = mock_info("axelar_relayer", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes[0].value, "update_reputation");
        assert_eq!(res.attributes[1].value, "456");

        // Verify storage
        let user_hex = hex::encode(user_addr_bytes);
        let token_type_hex = hex::encode(token_type_hash);
        let rep = query_reputation(deps.as_ref(), user_hex, token_type_hex).unwrap();
        assert_eq!(rep.soul_id, 456);
        assert_eq!(rep.tier, 3);
        assert_eq!(rep.nonce, 100);
        assert_eq!(rep.external_id, hex::encode(external_id_hash));
    }
}
