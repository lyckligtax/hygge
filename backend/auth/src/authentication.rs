use crate::authentication_error::AuthError;
use crate::{AccountIO, TokenIO};

// TODO: documentation
#[derive(Clone)]
pub struct Authentication<Accounts: AccountIO, Tokens: TokenIO> {
    tokens: Tokens,
    accounts: Accounts,
}

impl<InternalId, ExternalId, AccountCtx, TokenCtx, LoginToken, Accounts, Tokens>
    Authentication<Accounts, Tokens>
where
    InternalId: Copy,
    InternalId: Copy,
    Accounts: AccountIO<InternalId = InternalId, ExternalId = ExternalId, Ctx = AccountCtx>,
    Tokens: TokenIO<InternalId = InternalId, LoginToken = LoginToken, Ctx = TokenCtx>,
{
    pub fn new(accounts: Accounts, tokens: Tokens) -> Self {
        Self { tokens, accounts }
    }

    /// create a new account identified by an external_id and a password
    ///
    /// returns Credentials if the account already exists
    /// returns a new internal id on success
    pub async fn create_account(
        &mut self,
        external_id: &Accounts::ExternalId,
        password: &str,
        account_ctx: &mut AccountCtx,
    ) -> Result<InternalId, AuthError> {
        match self.accounts.exists(external_id, account_ctx).await {
            Ok(x) if x => Err(AuthError::Credentials),
            _ => Ok(self
                .accounts
                .create(external_id, password, account_ctx)
                .await?),
        }
    }

    /// deletes an account identified by its internal_id
    ///
    /// also removes all login_tokens for this internal_id
    pub async fn delete_account(
        &mut self,
        user_id: &Accounts::InternalId,
        account_ctx: &mut AccountCtx,
        token_ctx: &mut TokenCtx,
    ) -> Result<(), AuthError> {
        if self.tokens.revoke_all(user_id, token_ctx).await.is_some() {
            return Err(AuthError::Credentials);
        }
        if self.accounts.remove(user_id, account_ctx).await.is_some() {
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
        external_id: &Accounts::ExternalId,
        password: &str,
        account_ctx: &mut AccountCtx,
        token_ctx: &mut TokenCtx,
    ) -> Result<Tokens::LoginToken, AuthError> {
        let account = self
            .accounts
            .get_by_external(external_id, account_ctx)
            .await?;

        if let Ok(internal_id) = self
            .accounts
            .verify_credentials(&account, password, account_ctx)
            .await
        {
            Ok(self.tokens.create(&internal_id, token_ctx).await?)
        } else {
            Err(AuthError::Credentials)
        }
    }

    /// log out a client identified by the login_token
    ///
    /// other clients of this account might still have a valid token
    pub async fn logout(
        &mut self,
        login_token: &Tokens::LoginToken,
        token_ctx: &mut TokenCtx,
    ) -> Result<(), AuthError> {
        if self.tokens.revoke(login_token, token_ctx).await.is_none() {
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
        login_token: &Tokens::LoginToken,
        token_ctx: &mut TokenCtx,
    ) -> Result<InternalId, AuthError> {
        self.tokens
            .verify(login_token, token_ctx)
            .await
            .or(Err(AuthError::Credentials))
    }
}

// TODO: what tests do we actually need? Some of these do not make sense
// TODO: check if tests are correct and use correct asserts instead of expects
#[cfg(test)]
mod tests {
    use crate::MockAccountIO;
    use crate::{AccountError, MockTokenIO};
    use crate::{Authentication, TokenError};

    #[tokio::test]
    async fn test_create_account() {
        let mut accounts = MockAccountIO::new();
        let tokens = MockTokenIO::new();

        // given accounts can insert a new account
        accounts
            .expect_exists()
            .returning(|_, _| Err(AccountError::NotFound));
        accounts.expect_create().returning(|_, _, _| Ok(9u32));

        let mut auth = Authentication::new(accounts, tokens);

        auth.create_account(&1u32, "test1234", &mut ())
            .await
            .expect("Expected Account to be created");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected Account to be created: Credentials")]
    async fn test_create_account_already_exists() {
        let mut accounts = MockAccountIO::new();
        let tokens = MockTokenIO::new();

        accounts.expect_exists().returning(|_, _| Ok(true));

        let mut auth = Authentication::new(accounts, tokens);

        auth.create_account(&1u32, "test1234", &mut ())
            .await
            .expect("Expected Account to be created");
    }

    #[tokio::test]
    async fn test_login() {
        let mut accounts = MockAccountIO::new();
        let mut tokens = MockTokenIO::new();
        accounts.expect_get_by_external().returning(|_, _| Ok(()));

        accounts
            .expect_verify_credentials()
            .returning(|_, _, _| Ok(1));
        tokens.expect_create().returning(|_, _| Ok(18u32));

        let mut auth = Authentication::new(accounts, tokens);

        auth.login(&1u32, "test1234", &mut (), &mut ())
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

        let mut auth = Authentication::new(accounts, tokens);

        auth.login(&1u32, "test1234", &mut (), &mut ())
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    async fn test_verify_token() {
        let accounts = MockAccountIO::new();
        let mut tokens = MockTokenIO::new();
        tokens.expect_verify().returning(|_, _| Ok(18));

        let mut auth = Authentication::new(accounts, tokens);

        auth.verify_token(&18u32, &mut ())
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

        let mut auth = Authentication::new(accounts, tokens);

        auth.verify_token(&18u32, &mut ())
            .await
            .expect("Expected Token to be valid");
    }
}
