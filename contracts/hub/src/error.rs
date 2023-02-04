use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Generic Error")]
    GenericError {},

    #[error("Invalid payment")]
    InvalidPayment {},

    #[error("User has already used a ticket")]
    AlreadyUsed {}

}