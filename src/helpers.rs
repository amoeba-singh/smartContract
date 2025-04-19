use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
};

use crate::msg::{ExecuteMsg, QueryMsg, ProfileResponse};

/// TippingContract is a wrapper around Addr that provides helpers
/// for working with the tipping profiles contract
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TippingContract(pub Addr);

impl TippingContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    /// Get user profile
    pub fn get_profile<Q, T, CQ>(&self, querier: &Q, username: T) -> StdResult<ProfileResponse>
    where
        Q: Querier,
        T: Into<String>,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetProfile { username: username.into() };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: ProfileResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }
}

// Convert a string to an Addr, returning a StdResult
pub fn addr_validate(api: &dyn cosmwasm_std::Api, addr: &str) -> StdResult<Addr> {
    api.addr_validate(addr)
}

// Generate a unique tip key
pub fn generate_tip_key(from_username: &str, to_username: &str, timestamp_nanos: u64) -> String {
    format!("{}:{}:{}", from_username, to_username, timestamp_nanos)
}

// Parse a tip key to extract components
pub fn parse_tip_key(key: &str) -> Option<(String, String, u64)> {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() == 3 {
        let from = parts[0].to_string();
        let to = parts[1].to_string();
        let timestamp = parts[2].parse::<u64>().ok()?;
        Some((from, to, timestamp))
    } else {
        None
    }
}
