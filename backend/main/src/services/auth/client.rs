use crate::services::auth::error::AuthError;
use crate::services::auth::io::{AccountIO, TokenIO};
use crate::services::auth::Auth;
use crate::types::Services;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// TODO: documentation
pub struct AuthenticationClient<AccountCtx, TokenCtx, Accounts, Tokens> {
    tokens: Arc<RwLock<Tokens>>,
    accounts: Arc<RwLock<Accounts>>,
    account_ctx: AccountCtx,
    token_ctx: TokenCtx,
}

impl<'a, AccountCtx, TokenCtx, Accounts, Tokens>
    AuthenticationClient<AccountCtx, TokenCtx, Accounts, Tokens>
where
    Accounts: AccountIO<Ctx = AccountCtx>,
    Tokens: TokenIO<Ctx = TokenCtx>,
{
    pub fn new(
        accounts: Arc<RwLock<Accounts>>,
        tokens: Arc<RwLock<Tokens>>,
        account_ctx: AccountCtx,
        token_ctx: TokenCtx,
    ) -> Self {
        Self {
            tokens,
            accounts,
            account_ctx,
            token_ctx,
        }
    }

    /// create a new account identified by an external_id and a password
    ///
    /// returns Credentials if the account already exists
    /// returns a new internal id on success
    pub async fn create_account(
        &mut self,
        external_id: &str,
        password: &str,
        account_ctx: &mut AccountCtx,
    ) -> Result<Uuid, AuthError> {
        let mut accounts = self.accounts.write().await;
        match accounts.exists(external_id, account_ctx).await {
            Ok(_) => Err(AuthError::Credentials),
            _ => Ok(accounts.create(external_id, password, account_ctx).await?),
        }
    }

    /// deletes an account identified by its internal_id
    ///
    /// also removes all login_tokens for this internal_id
    pub async fn delete_account(
        &mut self,
        user_id: &Uuid,
        account_ctx: &mut AccountCtx,
        token_ctx: &mut TokenCtx,
    ) -> Result<(), AuthError> {
        if self
            .tokens
            .write()
            .await
            .revoke_all(user_id, token_ctx)
            .await
            .is_some()
        {
            return Err(AuthError::Credentials);
        }
        if self
            .accounts
            .write()
            .await
            .remove(user_id, account_ctx)
            .await
            .is_some()
        {
            return Err(AuthError::Credentials);
        }

        Ok(())
    }

    /// logs in an external id authenticated by its password
    /// returns a login token on success
    ///
    /// caches login tokens for a configured [Duration](Tokens::new)
    pub async fn login(
        &mut self,
        external_id: &str,
        password: &str,
        account_ctx: &mut AccountCtx,
        token_ctx: &mut TokenCtx,
    ) -> Result<String, AuthError> {
        let accounts = self.accounts.read().await;
        let account = accounts.get_by_external(external_id, account_ctx).await?;

        if let Ok(internal_id) = accounts
            .verify_credentials(&account, password, account_ctx)
            .await
        {
            Ok(self
                .tokens
                .write()
                .await
                .create(&internal_id, token_ctx)
                .await?)
        } else {
            Err(AuthError::Credentials)
        }
    }

    /// log out a client identified by the login_token
    ///
    /// other clients of this account might still have a valid token
    pub async fn logout(
        &mut self,
        login_token: &str,
        token_ctx: &mut TokenCtx,
    ) -> Result<(), AuthError> {
        if self
            .tokens
            .write()
            .await
            .revoke(login_token, token_ctx)
            .await
            .is_none()
        {
            Ok(())
        } else {
            Err(AuthError::Credentials)
        }
    }

    /// checks if a login token is valid
    /// - it needs to exist
    /// - it had to be used within a configured [Duration](Tokens::new)
    ///
    /// returns the internal id connected with the login token
    pub async fn verify_token(
        &mut self,
        login_token: &str,
        token_ctx: &mut TokenCtx,
    ) -> Result<Uuid, AuthError> {
        self.tokens
            .read()
            .await
            .verify(login_token, token_ctx)
            .await
            .or(Err(AuthError::Credentials))
    }
}

// TODO: what tests do we actually need? Some of these do not make sense
// TODO: check if tests are correct and use correct asserts instead of expects
#[cfg(test)]
mod tests {
    use crate::services::auth::account::{Account, AccountStatus};
    use crate::services::auth::client::AuthenticationClient;
    use crate::services::auth::io::{AccountError, MockAccountIO, MockTokenIO, TokenError};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_account() {
        let mut accounts = MockAccountIO::new();
        let tokens = MockTokenIO::new();

        // given accounts can insert a new account
        accounts
            .expect_exists()
            .returning(|_, _| Err(AccountError::NotFound));
        accounts
            .expect_create()
            .returning(|_, _, _| Ok(Uuid::new_v4()));

        let mut auth = AuthenticationService::new(accounts, tokens);

        auth.create_account("some mail", "test1234", &mut ())
            .await
            .expect("Expected Account to be created");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected Account to be created: Credentials")]
    async fn test_create_account_already_exists() {
        let mut accounts = MockAccountIO::new();
        let tokens = MockTokenIO::new();

        accounts
            .expect_exists()
            .returning(|_, _| Ok(Uuid::new_v4()));

        let mut auth = AuthenticationService::new(accounts, tokens);

        auth.create_account("some mail", "test1234", &mut ())
            .await
            .expect("Expected Account to be created");
    }

    #[tokio::test]
    async fn test_login() {
        let mut accounts = MockAccountIO::new();
        let mut tokens = MockTokenIO::new();
        accounts.expect_get_by_external().returning(|_, _| {
            Ok(Account {
                id: Default::default(),
                id_external: "some mail".to_string(),
                hash: "some hash".to_string(),
                status: AccountStatus::Active,
            })
        });

        accounts
            .expect_verify_credentials()
            .returning(|_, _, _| Ok(Uuid::new_v4()));
        tokens
            .expect_create()
            .returning(|_, _| Ok("some hash".to_string()));

        let mut auth = AuthenticationService::new(accounts, tokens);

        auth.login("some mail", "test1234", &mut (), &mut ())
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected LoginToken: Credentials")]
    async fn test_login_failure() {
        let mut accounts = MockAccountIO::new();
        let tokens = MockTokenIO::new();
        accounts
            .expect_get_by_external()
            .returning(|_, _| Err(AccountError::NotFound)); // the given user does not exist

        let mut auth = AuthenticationService::new(accounts, tokens);

        auth.login("some mail", "test1234", &mut (), &mut ())
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    async fn test_verify_token() {
        let accounts = MockAccountIO::new();
        let mut tokens = MockTokenIO::new();
        tokens.expect_verify().returning(|_, _| Ok(Uuid::new_v4()));

        let mut auth = AuthenticationService::new(accounts, tokens);

        auth.verify_token("some token", &mut ())
            .await
            .expect("Expected Token to be valid");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected Token to be valid: Credentials")]
    async fn test_verify_token_failure() {
        let accounts = MockAccountIO::new();
        let mut tokens = MockTokenIO::new();
        tokens
            .expect_verify()
            .returning(|_, _| Err(TokenError::Invalid));

        let mut auth = AuthenticationService::new(accounts, tokens);

        auth.verify_token("some token", &mut ())
            .await
            .expect("Expected Token to be valid");
    }
}

#[async_trait]
impl FromRequestParts<Arc<Services>> for Auth {
    type Rejection = ();

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<Services>,
    ) -> Result<Self, Self::Rejection> {
        let token_provider = &state.token_provider;
        let account_provider = &state.account_provider;

        //TODO: how can we then providers into  a new client?
        // cloning would allow us to simply work without refs
        //  rwlock?
        Ok(AuthenticationClient::new(
            account_provider.get_mut(),
            token_provider.get_mut(),
        ))
    }
}
