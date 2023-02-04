use cosmwasm_schema::{cw_serde, QueryResponses};
//use cosmwasm_std::{StdError, StdResult, Uint128, Empty};
//pub type QueryMsg = cw721_base::QueryMsg<Empty>;
use cw721::Cw721ReceiveMsg;

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    ReceiveNft(Cw721ReceiveMsg),

    InitTicketContract {
        code_id: u64,
        name: String,
        symbol: String,
    },

    BuyTicket {
        package_option: u8,
    },
}

#[cw_serde]
pub enum ReceiveNftMsg {
    UseTicket {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(UserStatusResponse)]
    GetUserStatus { user_address: String },
}

#[cw_serde]
pub struct UserStatusResponse {
    pub checked_in: bool,
    pub ticket_data: Option<Vec<String>>,
}
