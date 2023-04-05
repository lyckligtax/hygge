use std::time::{Duration, Instant};

use crate::auth_provider_error::AuthError;
use crate::{AccountIO, AuthenticationIO};

// TODO: documentation
pub struct AuthProvider<Account: AccountIO, Authentication: AuthenticationIO> {
    authentication_cache: Authentication,
    account_provider: Account,
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
        account_provider: Account,
        authentication_cache: Authentication,
        login_ttl: Duration,
    ) -> Self {
        Self {
            authentication_cache,
            account_provider,
            login_ttl,
        }
    }

    /// creates a new account and returns its internal id on success
    ///
    /// passwords are salted and hashed with [Argon2](argon2)
    pub async fn create_account(
        &mut self,
        id: Account::ExternalId,
        password: &[u8],
    ) -> Result<InternalId, AuthError> {
        Ok(self.account_provider.insert(id, password).await?)
    }

    pub fn update_user() {
        // possibly update external id and/or password

        // on password change:
        // needs to log out all existing tokens connected to this internal id but this one
        unimplemented!()
    }
    pub async fn delete_user() {
        unimplemented!()
    }

    /// logs in an external id authenticated by its password
    /// returns a login token on success
    ///
    /// caches login tokens for a configured [Duration](AuthProvider::new)
    pub async fn login(
        &mut self,
        id: Account::ExternalId,
        password: &[u8],
    ) -> Result<Authentication::LoginToken, AuthError> {
        let user_account = self.account_provider.get(&id).await?;
        self.account_provider
            .verify_password(&user_account.password_hash, password)
            .await?;

        Ok(self
            .authentication_cache
            .insert(&user_account.id_internal as &Authentication::InternalId)
            .await?)
    }

    pub async fn logout() {
        unimplemented!()
    }

    /// checks if a login token is valid
    /// - it needs to exist
    /// - it had to be used with a configured [Duration](AuthProvider::new)
    ///
    /// returns the internal id connected with the login token
    pub async fn verify_token(
        &mut self,
        login_token: Authentication::LoginToken,
    ) -> Result<InternalId, AuthError> {
        let mut authentication = self
            .authentication_cache
            .get_mut(&login_token)
            .await
            .ok_or(AuthError::NotLoggedIn)?;

        let now = Instant::now();

        if authentication.last_seen - now < self.login_ttl {
            authentication.last_seen = now;
            Ok(authentication.id)
        } else {
            let _ = self.authentication_cache.remove(&login_token).await;
            Err(AuthError::Expired)
        }
    }
}
