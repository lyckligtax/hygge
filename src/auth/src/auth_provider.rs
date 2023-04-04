use crate::{AccountError, AccountIO, AuthenticationError, AuthenticationIO};
use std::sync::Mutex;
use std::time::{Duration, Instant};
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

pub struct AuthProvider<Account: AccountIO, Authentication: AuthenticationIO> {
    authentication_cache: Mutex<Authentication>,
    account_provider: Mutex<Account>,
    login_ttl: Duration,
}

impl<InternalId, ExternalId, LoginToken, Account, Authentication>
    AuthProvider<Account, Authentication>
where
    InternalId: Copy,
    InternalId: Copy,
    Account: AccountIO<InternalId = InternalId, ExternalId = ExternalId>,
    Authentication: AuthenticationIO<InternalId = InternalId, LoginToken = LoginToken>,
{
    pub fn new(
        account_io: Account,
        authentication_io: Authentication,
        login_ttl: Duration,
    ) -> Self {
        Self {
            authentication_cache: Mutex::new(authentication_io),
            account_provider: Mutex::new(account_io),
            login_ttl,
        }
    }

    /// creates a new account and returns its internal id on success
    ///
    /// passwords are salted and hashed with [Argon2](argon2)
    pub fn create_account(
        &mut self,
        id: Account::ExternalId,
        password: &[u8],
    ) -> Result<InternalId, AuthError> {
        Ok(self
            .account_provider
            .get_mut()
            .expect("Expected to get persistence lock")
            .insert(id, password)?)
    }

    pub fn update_user() {
        // possibly update external id and/or password

        // on password change:
        // needs to log out all existing tokens connected to this internal id but this one
        unimplemented!()
    }
    pub fn delete_user() {
        unimplemented!()
    }

    /// logs in an external id authenticated by its password
    /// returns a login token on success
    ///
    /// caches login tokens for a configured [Duration](AuthProvider::new)
    pub fn login(
        &mut self,
        id: Account::ExternalId,
        password: &[u8],
    ) -> Result<Authentication::LoginToken, AuthError> {
        let accounts = self
            .account_provider
            .lock()
            .expect("Expected to get persistence lock");

        let user_account = accounts.get_verified(&id, password)?;

        let cache = self
            .authentication_cache
            .get_mut()
            .expect("Expected to get caching lock");

        Ok(cache.insert(&user_account.id_internal as &Authentication::InternalId)?)
    }

    pub fn logout() {
        unimplemented!()
    }

    /// checks if a login token is valid
    /// - it needs to exist
    /// - it had to be used with a configured [Duration](AuthProvider::new)
    ///
    /// returns the internal id connected with the login token
    pub fn verify_token(
        &mut self,
        login_token: Authentication::LoginToken,
    ) -> Result<InternalId, AuthError> {
        let cache = self
            .authentication_cache
            .get_mut()
            .expect("Expected to get caching lock");

        let mut authentication = cache.get_mut(&login_token).ok_or(AuthError::NotLoggedIn)?;

        let now = Instant::now();

        if authentication.last_seen - now < self.login_ttl {
            authentication.last_seen = now;
            Ok(authentication.id)
        } else {
            let _ = cache.remove(&login_token);
            Err(AuthError::Expired)
        }
    }
}
