use crate::services::auth::io::{AccountError, TokenError};
use thiserror::Error;

//TODO: make custom messages possible to distinguish? Or more different errors?
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid Credentials")]
    Credentials,
    #[error("Error during IO Operations")]
    IO,
}

impl From<TokenError> for AuthError {
    fn from(value: TokenError) -> Self {
        match value {
            TokenError::IO => AuthError::IO,
            TokenError::Invalid => AuthError::Credentials,
        }
    }
}

impl From<AccountError> for AuthError {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::IO => AuthError::IO,
            AccountError::NotFound => AuthError::Credentials,
            AccountError::Invalid => AuthError::Credentials,
        }
    }
}
