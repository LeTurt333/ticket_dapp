#[cfg(not(feature = "library"))]
use crate::contract_imports::*;

const CONTRACT_NAME: &str = "crates.io:ticket-dapp";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const TICKET_PRICE_DENOM: &str = "ujuno";
const NORMAL_PRICE: Uint128 = Uint128::new(1000000);
const VIP_PRICE: Uint128 = Uint128::new(5000000);
const ULTRA_VIP_PRICE: Uint128 = Uint128::new(10000000);

//////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////////////// Instantiate
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());

    let validated_admin = deps.api.addr_validate(&admin)?;

    CONFIGURATION.save(
        deps.storage,
        &Configuration {
            admin: validated_admin,
            ticket_nft_addr: None,
        },
    )?;

    TICKER.save(deps.storage, &1)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}

//////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////////////// Execute
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitTicketContract {
            code_id,
            name,
            symbol,
        } => execute_init_ticket(deps, env, &info.sender, code_id, name, symbol),

        ExecuteMsg::BuyTicket { package_option } => {
            execute_buy_ticket(deps, info, env, package_option)
        }

        ExecuteMsg::ReceiveNft(receive_nft_msg) => {
            execute_receive_nft(deps, env.block.height, info, receive_nft_msg)
        }
    }
}

pub fn execute_init_ticket(
    deps: DepsMut,
    env: Env,
    sender: &Addr,
    code_id: u64,
    name: String,
    symbol: String,
) -> Result<Response, ContractError> {
    let config = CONFIGURATION.load(deps.storage)?;

    if &config.admin != sender {
        return Err(ContractError::GenericError {});
    }

    let bin_init = to_binary(&TicketNftInstantiateMsg {
        name,
        symbol,
        minter: env.contract.address.to_string(),
    })?;

    let cosmos_msg: CosmosMsg<Empty> = CosmosMsg::from(WasmMsg::Instantiate {
        admin: None,
        code_id: code_id,
        msg: bin_init,
        funds: vec![],
        label: "Nft Ticket Contract".to_string(),
    });

    let sub_msg = SubMsg {
        id: 1,
        msg: cosmos_msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new()
        .add_attribute("Innit NFT ticket contract", "innit")
        .add_submessage(sub_msg))
}

pub fn execute_buy_ticket(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    package_option: u8,
) -> Result<Response, ContractError> {
    let config: Configuration = CONFIGURATION.load(deps.storage)?;

    let token_id = TICKER.load(deps.storage)?;

    let Some(nft_addr) = config.ticket_nft_addr else {
        return Err(ContractError::GenericError {  });
    };

    TICKER.update(deps.storage, |old| -> Result<u32, ContractError> {
        if old >= u32::MAX - 1 {
            Ok(1)
        } else {
            Ok(old + 1)
        }
    })?;

    // 1 = normal , 2 = VIP, 3 = Ultra VIP
    match package_option {
        1 => {
            // check has correct funds for purchase option
            if !has_coins(&info.funds, &coin(NORMAL_PRICE.u128(), TICKET_PRICE_DENOM)) {
                return Err(ContractError::InvalidPayment {});
            }
            if info.funds.len() > 1 {
                return Err(ContractError::GenericError {});
            }

            let submsg = make_mint_submsg(nft_addr, info.sender.to_string(), token_id, 1u8)?;

            Ok(Response::new().add_submessage(submsg))
        }
        2 => {
            if !has_coins(&info.funds, &coin(VIP_PRICE.u128(), TICKET_PRICE_DENOM)) {
                return Err(ContractError::InvalidPayment {});
            }
            if info.funds.len() > 1 {
                return Err(ContractError::GenericError {});
            }

            let submsg = make_mint_submsg(nft_addr, info.sender.to_string(), token_id, 2u8)?;

            Ok(Response::new().add_submessage(submsg))
        }
        3 => {
            if !has_coins(
                &info.funds,
                &coin(ULTRA_VIP_PRICE.u128(), TICKET_PRICE_DENOM),
            ) {
                return Err(ContractError::InvalidPayment {});
            }
            if info.funds.len() > 1 {
                return Err(ContractError::GenericError {});
            }

            let submsg = make_mint_submsg(nft_addr, info.sender.to_string(), token_id, 3u8)?;

            Ok(Response::new().add_submessage(submsg))
        }
        _ => return Err(ContractError::GenericError {}),
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    block: u64,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveNftMsg = from_binary(&wrapper.msg)?;
    let user_wallet = deps.api.addr_validate(&wrapper.sender)?;

    // If nft contract address isn't correct error
    let config: Configuration = CONFIGURATION.load(deps.storage)?;

    if config.ticket_nft_addr != Some(info.sender.clone()) {
        return Err(ContractError::GenericError {});
    }

    // If token_id already used error
    if (used_tickets()
        .idx
        .ticket_id
        .item(deps.storage, wrapper.token_id.clone())?)
    .is_some()
    {
        return Err(ContractError::GenericError {});
    }

    // Optionally check if sender has already used a ticket
    if {
        let used_tickets: StdResult<Vec<_>> = used_tickets()
            .prefix(&user_wallet)
            .range(deps.storage, None, None, Order::Ascending)
            .collect();
        used_tickets.is_err() || used_tickets.unwrap().len() > 1
    } {
        return Err(ContractError::AlreadyUsed {});
    }

    match msg {
        ReceiveNftMsg::UseTicket {} => use_ticket(
            deps,
            block,
            user_wallet,
            wrapper.token_id,
            info.sender.to_string(),
        ),
    }
}

pub fn use_ticket(
    deps: DepsMut,
    block_used: u64,
    user: Addr,
    ticket_id: String,
    nft_contract: String,
) -> Result<Response, ContractError> {
    // deps, key (Addr user, Token ID), TokenInfo
    used_tickets().save(
        deps.storage,
        (&user, ticket_id.clone()),
        &TicketInfo {
            ticket_id: ticket_id.clone(),
            block_used,
        },
    )?;

    // dispatch burn message
    let burn = make_burn_msg(nft_contract, ticket_id)?;

    Ok(Response::new().add_message(burn))
}

//////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////////////// Submessage Reply
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        // Reply from Ticket contract init
        1 => {
            let res = parse_reply_instantiate_data(msg.clone())
                .map_err(|_| ContractError::GenericError {})?;

            let ticket_contract = deps.api.addr_validate(&res.contract_address)?;

            CONFIGURATION.update(
                deps.storage,
                |old| -> Result<Configuration, ContractError> {
                    Ok(Configuration {
                        ticket_nft_addr: Some(ticket_contract.clone()),
                        ..old
                    })
                },
            )?;

            return Ok(Response::new().add_attribute(
                "Instantiated NFT Ticket contract",
                format!("Address: {}", ticket_contract.to_string()),
            ));
        }

        // Reply from Ticket Mint
        2 => {
            let _res = parse_reply_execute_data(msg.clone())
                .map_err(|_| ContractError::GenericError {})?;

            return Ok(Response::new().add_attribute("Mint Ticket", "Success"));
        }

        // Invalid reply msg.id
        _ => {
            return Err(ContractError::GenericError {});
        }
    }
}

//////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////////////// Query
///////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUserStatus { user_address } => {
            to_binary(&get_user_status(deps, user_address)?)
        }
    }
}

pub fn get_user_status(deps: Deps, wallet: String) -> StdResult<Binary> {
    let config: Configuration = CONFIGURATION.load(deps.storage)?;
    let Some(nft_addr) = config.ticket_nft_addr else {
        return Err(cosmwasm_std::StdError::GenericErr { msg: "Error checking for user".to_string() });
    };

    let user_wallet = deps.api.addr_validate(&wallet)?;

    let checked: StdResult<Vec<(_, TicketInfo)>> = used_tickets()
        .prefix(&user_wallet)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let Ok(checked_tickets) = checked else {
        return Err(cosmwasm_std::StdError::GenericErr { msg: "Error checking for user".to_string() });
    };

    if checked_tickets.len() == 0 {
        return to_binary(&UserStatusResponse {
            checked_in: false,
            ticket_data: None,
        });
    };

    let resp: NftInfoResponse<TicketNftExtension> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: nft_addr.to_string(),
            msg: to_binary(&format!(
                "NftInfo {{ token_id: {} }}",
                checked_tickets[0].1.ticket_id
            ))?,
        }))?;

    let Some(ext) = resp.extension else {
        return Err(cosmwasm_std::StdError::GenericErr { msg: "Should always have metadata".to_string() });
    };

    let Some(attributes) = ext.attributes else {
        return Err(cosmwasm_std::StdError::GenericErr { msg: "Should always have metadata".to_string() });
    };

    let traits: Vec<_> = attributes
        .iter()
        .map(|t| format!("{}: {}", t.trait_type, t.value))
        .collect();

    to_binary(&UserStatusResponse {
        checked_in: true,
        ticket_data: Some(traits),
    })
}
