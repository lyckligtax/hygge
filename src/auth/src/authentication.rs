use crate::authentication_error::AuthError;
use crate::{AccountError, AccountIO, LoginTokenIO, PermissionIO};

// TODO: documentation
#[derive(Clone)]
pub struct Authentication<Accounts: AccountIO, Tokens: LoginTokenIO, Permissions: PermissionIO> {
    tokens: Tokens,
    accounts: Accounts,
    permissions: Permissions,
}

impl<
        InternalId,
        ExternalId,
        AccountCtx,
        LoginCtx,
        PermissionCtx,
        LoginToken,
        Accounts,
        Tokens,
        Permissions,
    > Authentication<Accounts, Tokens, Permissions>
where
    InternalId: Copy,
    InternalId: Copy,
    Accounts: AccountIO<InternalId = InternalId, ExternalId = ExternalId, AccountCtx = AccountCtx>,
    Tokens: LoginTokenIO<InternalId = InternalId, LoginToken = LoginToken, LoginCtx = LoginCtx>,
    Permissions: PermissionIO<InternalId = InternalId, PermissionCtx = PermissionCtx>,
{
    pub fn new(accounts: Accounts, tokens: Tokens, permissions: Permissions) -> Self {
        Self {
            tokens,
            accounts,
            permissions,
        }
    }

    /// create a new account identified by an external_id and a password
    ///
    /// returns CouldNotCreate if the account already exists
    /// returns a new internal id on success
    pub async fn create_account(
        &mut self,
        external_id: &Accounts::ExternalId,
        password: &str,
        internal_id: Option<Accounts::InternalId>,
        account_ctx: &mut AccountCtx,
        permission_ctx: &mut PermissionCtx,
    ) -> Result<InternalId, AuthError> {
        if !self
            .permissions
            .create_account(internal_id, permission_ctx)
            .await
        {
            return Err(AuthError::Credentials);
        }
        match self.accounts.get_login(external_id, account_ctx).await {
            Err(AccountError::NotFound) => {}
            Ok(_) | Err(_) => {
                return Err(AuthError::CouldNotCreate);
            }
        }
        Ok(self
            .accounts
            .create(external_id, password, account_ctx)
            .await?)
    }

    pub fn update_account() {
        // possibly update external id and/or password

        // on password change:
        // needs to log out all existing tokens connected to this internal id but this one
        unimplemented!()
    }

    /// deletes an account identified by its internal_id
    ///
    /// also removes all login_tokens for this internal_id
    pub async fn delete_account(
        &mut self,
        id: &Accounts::InternalId,
        account_ctx: &mut AccountCtx,
        login_ctx: &mut LoginCtx,
    ) -> Result<(), AuthError> {
        // TODO: do I have permissions to delete an account?
        // in theory I should always be able to delete my own account
        // but this api should be usable by a support member as well because we want to always use the same api a user might
        // integrate permission here? new permission IO trait?
        self.tokens.remove_all(id, login_ctx).await?;
        Ok(self.accounts.remove(id, account_ctx).await?)
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
        login_ctx: &mut LoginCtx,
    ) -> Result<Tokens::LoginToken, AuthError> {
        let (internal_id, hash) = self.accounts.get_login(external_id, account_ctx).await?;
        self.accounts
            .verify_password(&hash, password, account_ctx)
            .await?;

        Ok(self
            .tokens
            .insert(&internal_id as &Tokens::InternalId, login_ctx)
            .await?)
    }

    /// log out a client identified by the login_token
    ///
    /// other clients of this account might still have a valid token
    pub async fn logout(
        &mut self,
        login_token: &Tokens::LoginToken,
        login_ctx: &mut LoginCtx,
    ) -> Result<(), AccountError> {
        self.tokens
            .remove(login_token, login_ctx)
            .await
            .or(Err(AccountError::Credentials))
    }

    /// checks if a login token is valid
    /// - it needs to exist
    /// - it had to be used within a configured [Duration](Tokens::new)
    ///
    /// returns the internal id connected with the login token
    pub async fn verify_token(
        &self,
        login_token: &Tokens::LoginToken,
        login_ctx: &mut LoginCtx,
    ) -> Result<InternalId, AuthError> {
        self.tokens
            .get(login_token, login_ctx)
            .await
            .ok_or(AuthError::NotLoggedIn)
    }
}

#[cfg(test)]
mod tests {
    use crate::MockAccountIO;
    use crate::{AccountError, MockLoginTokenIO};
    use crate::{Authentication, MockPermissionIO};

    #[tokio::test]
    async fn test_create_account() {
        let mut accounts = MockAccountIO::new();
        let cache = MockLoginTokenIO::new();
        let mut permissions = MockPermissionIO::new();

        // given accounts can insert a new account
        permissions.expect_create_account().returning(|_, _| true);
        accounts
            .expect_get_login()
            .returning(|_, _| Err(AccountError::NotFound));
        accounts.expect_create().returning(|_, _, _| Ok(9u32));

        let mut auth = Authentication::new(accounts, cache, permissions);

        auth.create_account(&1u32, "test1234", None, &mut (), &mut ())
            .await
            .expect("Expected Account to be created");
    }
    #[tokio::test]
    #[should_panic(expected = "Expected Account to be created: CouldNotCreate")]
    async fn test_create_account_already_exists() {
        let mut accounts = MockAccountIO::new();
        let cache = MockLoginTokenIO::new();
        let mut permissions = MockPermissionIO::new();

        // given accounts can insert a new account
        permissions.expect_create_account().returning(|_, _| true);
        accounts
            .expect_get_login()
            .returning(|_, _| Ok((9u32, "test1234".to_string())));

        let mut auth = Authentication::new(accounts, cache, permissions);

        auth.create_account(&1u32, "test1234", None, &mut (), &mut ())
            .await
            .expect("Expected Account to be created");
    }

    #[tokio::test]
    async fn test_login() {
        let mut accounts = MockAccountIO::new();
        let mut cache = MockLoginTokenIO::new();
        let permissions = MockPermissionIO::new();
        accounts
            .expect_get_login()
            .returning(|_, _| Ok((9u32, "test1234".to_string())));

        accounts
            .expect_verify_password()
            .returning(|_, _, _| Ok(()));
        cache.expect_insert().returning(|_, _| Ok(18u32));

        let mut auth = Authentication::new(accounts, cache, permissions);

        auth.login(&1u32, "test1234", &mut (), &mut ())
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected LoginToken: Credentials")]
    async fn test_login_failure() {
        let mut accounts = MockAccountIO::new();
        let cache = MockLoginTokenIO::new();
        let permissions = MockPermissionIO::new();
        accounts
            .expect_get_login()
            .returning(|_, _| Err(AccountError::NotFound)); // the given user does not exist

        let mut auth = Authentication::new(accounts, cache, permissions);

        auth.login(&1u32, "test1234", &mut (), &mut ())
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    async fn test_verify_token() {
        let accounts = MockAccountIO::new();
        let mut cache = MockLoginTokenIO::new();
        let permissions = MockPermissionIO::new();
        cache.expect_get().returning(|_, _| Some(9u32));

        let auth = Authentication::new(accounts, cache, permissions);

        auth.verify_token(&18u32, &mut ())
            .await
            .expect("Expected InternalId");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected InternalId: NotLoggedIn")]
    async fn test_verify_token_failure() {
        let accounts = MockAccountIO::new();
        let mut cache = MockLoginTokenIO::new();
        let permissions = MockPermissionIO::new();
        cache.expect_get().returning(|_, _| None);

        let auth = Authentication::new(accounts, cache, permissions);

        auth.verify_token(&18u32, &mut ())
            .await
            .expect("Expected InternalId");
    }
}
