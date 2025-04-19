use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Username '{username}' already exists")]
    UsernameExists { username: String },

    #[error("Username '{username}' not found")]
    UsernameNotFound { username: String },

    #[error("Username '{username}' contains invalid characters. Usernames must contain only letters, numbers, underscores, and hyphens")]
    InvalidUsername { username: String },

    #[error("Username must be between 3 and 30 characters")]
    UsernameTooShortOrLong {},

    #[error("Cannot tip yourself")]
    SelfTipping {},

    #[error("Wallet address already registered with username '{username}'")]
    WalletAlreadyRegistered { username: String },

    #[error("Tip record not found")]
    TipRecordNotFound {},

    #[error("Missing or invalid field: {field}")]
    MissingField { field: String },

    #[error("Custom Error: {message}")]
    CustomError { message: String },
}
