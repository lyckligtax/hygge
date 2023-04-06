use std::time::{Duration, Instant};

use crate::auth_provider_error::AuthError;
use crate::{AccountError, AccountIO, AccountStatus, LoginTokenIO};

// TODO: documentation
pub struct Authentication<Accounts: AccountIO, Tokens: LoginTokenIO> {
    tokens: Tokens,
    accounts: Accounts,
    login_ttl: Duration,
}

impl<InternalId, ExternalId, LoginToken, Accounts, Tokens> Authentication<Accounts, Tokens>
where
    InternalId: Copy,
    InternalId: Copy,
    Accounts: AccountIO<InternalId = InternalId, ExternalId = ExternalId>,
    Tokens: LoginTokenIO<InternalId = InternalId, LoginToken = LoginToken>,
{
    pub fn new(accounts: Accounts, tokens: Tokens, login_ttl: Duration) -> Self {
        Self {
            tokens,
            accounts,
            login_ttl,
        }
    }

    /// create a new account identified by an external_id and a password
    ///
    /// returns CouldNotCreate if the account already exists
    /// returns a new internal id on success
    pub async fn create_account(
        &mut self,
        id: Accounts::ExternalId,
        password: &[u8],
    ) -> Result<InternalId, AuthError> {
        match self.accounts.get(&id).await {
            Err(AccountError::NotFound) => {}
            Ok(_) | Err(_) => return Err(AuthError::CouldNotCreate),
        }
        Ok(self.accounts.create(id, password).await?)
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
    pub async fn delete_account(&mut self, id: &Accounts::InternalId) -> Result<(), AuthError> {
        self.tokens.remove_all(id).await?;
        Ok(self.accounts.remove(id).await?)
    }

    /// logs in an external id authenticated by its password
    /// returns a login token on success
    ///
    /// caches login tokens for a configured [Duration](Tokens::new)
    pub async fn login(
        &mut self,
        id: Accounts::ExternalId,
        password: &[u8],
    ) -> Result<Tokens::LoginToken, AuthError> {
        let user_account = self.accounts.get(&id).await?;
        match user_account.status {
            AccountStatus::Ok => {}
            AccountStatus::Disabled | AccountStatus::Removed => {
                return Err(AuthError::Credentials);
            }
        }
        self.accounts
            .verify_password(&user_account.password_hash, password)
            .await?;

        Ok(self
            .tokens
            .insert(&user_account.id_internal as &Tokens::InternalId)
            .await?)
    }

    /// log out a client identified by the login_token
    ///
    /// other clients of this account might still have a valid token
    pub async fn logout(&mut self, login_token: Tokens::LoginToken) -> Result<(), AccountError> {
        self.tokens
            .remove(&login_token)
            .await
            .or(Err(AccountError::Credentials))
    }

    /// checks if a login token is valid
    /// - it needs to exist
    /// - it had to be used within a configured [Duration](Tokens::new)
    ///
    /// returns the internal id connected with the login token
    pub async fn verify_token(
        &mut self,
        login_token: Tokens::LoginToken,
    ) -> Result<InternalId, AuthError> {
        let (internal_id, last_seen) = self
            .tokens
            .get(&login_token)
            .await
            .ok_or(AuthError::NotLoggedIn)?;

        let now = Instant::now();

        if last_seen - now < self.login_ttl {
            self.tokens.last_seen(&login_token, now);
            Ok(internal_id)
        } else {
            let _ = self.tokens.remove(&login_token).await;
            Err(AuthError::Expired)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Authentication;
    use crate::{Account, AccountStatus, MockAccountIO};
    use crate::{AccountError, MockLoginTokenIO};
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_create_account() {
        let mut accounts = MockAccountIO::new();
        let cache = MockLoginTokenIO::new();

        // given accounts can insert a new account
        accounts
            .expect_get()
            .returning(|_| Err(AccountError::NotFound));
        accounts.expect_create().returning(|_, _| Ok(9u32));

        let mut auth = Authentication::new(accounts, cache, Duration::from_secs(3600));

        auth.create_account(1u32, b"test1234")
            .await
            .expect("Expected Account to be created");
    }
    #[tokio::test]
    #[should_panic(expected = "Expected Account to be created: CouldNotCreate")]
    async fn test_create_account_already_exists() {
        let mut accounts = MockAccountIO::new();
        let cache = MockLoginTokenIO::new();

        // given accounts can insert a new account
        accounts.expect_get().returning(|_| {
            Ok(Account {
                id_internal: 9u32,
                id_external: 1u32,
                password_hash: "test1234".to_string(),
                status: AccountStatus::Ok,
            })
        });

        let mut auth = Authentication::new(accounts, cache, Duration::from_secs(3600));

        auth.create_account(1u32, b"test1234")
            .await
            .expect("Expected Account to be created");
    }

    #[tokio::test]
    async fn test_login() {
        let mut accounts = MockAccountIO::new();
        let mut cache = MockLoginTokenIO::new();
        accounts.expect_get().returning(|_| {
            Ok(Account {
                id_internal: 9u32,
                id_external: 1u32,
                password_hash: "test1234".to_string(),
                status: AccountStatus::Ok,
            })
        });
        accounts.expect_verify_password().returning(|_, _| Ok(()));
        cache.expect_insert().returning(|_| Ok(18u32));

        let mut auth = Authentication::new(accounts, cache, Duration::from_secs(3600));

        auth.login(1u32, b"test1234")
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected LoginToken: Credentials")]
    async fn test_login_failure() {
        let mut accounts = MockAccountIO::new();
        let cache = MockLoginTokenIO::new();
        accounts
            .expect_get()
            .returning(|_| Err(AccountError::NotFound)); // the given user does not exist

        let mut auth = Authentication::new(accounts, cache, Duration::from_secs(3600));

        auth.login(1u32, b"test1234")
            .await
            .expect("Expected LoginToken");
    }

    #[tokio::test]
    async fn test_verify_token() {
        let accounts = MockAccountIO::new();
        let mut cache = MockLoginTokenIO::new();
        cache
            .expect_get()
            .returning(|_| Some((9u32, Instant::now())));
        cache.expect_last_seen().returning(|_, _| Ok(()));

        let mut auth = Authentication::new(accounts, cache, Duration::from_secs(3600));

        auth.verify_token(18u32).await.expect("Expected InternalId");
    }

    #[tokio::test]
    #[should_panic(expected = "Expected InternalId: NotLoggedIn")]
    async fn test_verify_token_failure() {
        let accounts = MockAccountIO::new();
        let mut cache = MockLoginTokenIO::new();
        cache.expect_get().returning(|_| None);

        let mut auth = Authentication::new(accounts, cache, Duration::from_secs(3600));

        auth.verify_token(18u32).await.expect("Expected InternalId");
    }
}
