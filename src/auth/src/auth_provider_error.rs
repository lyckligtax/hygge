use crate::{AccountError, AuthenticationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Could not create Account")]
    CouldNotCreate,
    #[error("Credential Error")]
    Credentials,
    #[error("User is not logged in")]
    NotLoggedIn,
    #[error("Token is expired")]
    Expired,
    #[error("Could not cache the authentication")]
    Caching,
}

impl From<AccountError> for AuthError {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::CouldNotCreate | AccountError::AlreadyExists => Self::CouldNotCreate,
            AccountError::Credentials | AccountError::NotFound => Self::Credentials,
        }
    }
}

impl From<AuthenticationError> for AuthError {
    fn from(_value: AuthenticationError) -> Self {
        Self::Caching
    }
}
