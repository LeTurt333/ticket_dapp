use cosmwasm_std::{Addr, SubMsg, to_binary, CosmosMsg, Empty, WasmMsg, ReplyOn};
use crate::error::*;

use ticket_nft::{
    ExecuteMsg as TicketExecuteMsg,
    MintMsg as TicketMintMsg,
    TicketNftExtension,
    CreateTicketMetadata,
};


pub fn make_mint_submsg(
    contract: Addr,
    ticket_holder: String,
    token_id: u32,
    package_option: u8,

) -> Result<SubMsg, ContractError> {
    let metadata = TicketNftExtension::create(package_option, ticket_holder.clone());

    let mint_msg: TicketMintMsg<TicketNftExtension> = TicketMintMsg {
        count: token_id,
        token_id: token_id.to_string(),
        owner: ticket_holder,
        token_uri: None,
        extension: metadata,
    };

    let bin = to_binary(&TicketExecuteMsg::Mint(mint_msg))?;

    let cosmos_msg: CosmosMsg<Empty> = CosmosMsg::from(WasmMsg::Execute {
        contract_addr: contract.to_string(),
        funds: vec![],
        msg: bin,
    });

    
    Ok(SubMsg {
        id: 2,
        msg: cosmos_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    })

}

pub fn make_burn_msg(
    nft_contract: String,
    token_id: String,

) -> Result<CosmosMsg, ContractError> {

    let bin = to_binary(&TicketExecuteMsg::Burn { token_id })?;

    Ok(CosmosMsg::from(WasmMsg::Execute {
        contract_addr: nft_contract,
        funds: vec![],
        msg: bin,
    }))
}


