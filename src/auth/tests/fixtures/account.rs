use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use auth::AccountError::{CouldNotCreate, Credentials};
use auth::{Account, AccountError, AccountIO, AccountStatus};
use rand_core::OsRng;
use std::collections::HashMap;

pub struct AccountProvider<InternalId, ExternalId> {
    data: HashMap<InternalId, Account<InternalId, ExternalId>>,
    argon: Argon2<'static>,
}

impl AccountIO for AccountProvider<u32, u32> {
    type InternalId = u32;
    type ExternalId = u32;

    async fn insert(&mut self, id: u32, password: &[u8]) -> Result<u32, AccountError> {
        let salt = SaltString::generate(&mut OsRng);
        let Ok(password_hash) = self.argon.hash_password(password, &salt) else {
            return Err(CouldNotCreate);
        };

        let account = Account {
            id_internal: id,
            id_external: id,
            password_hash: password_hash.to_string(),
            status: AccountStatus::Ok,
        };
        // not thread safe but doesnt matter in tests
        if self.data.contains_key(&id) {
            Err(AccountError::AlreadyExists)
        } else {
            self.data.insert(id, account);
            Ok(id)
        }
    }

    async fn get(&self, id: &u32) -> Result<&Account<u32, u32>, AccountError> {
        self.data.get(id).ok_or(AccountError::NotFound)
    }

    async fn verify_password(
        &self,
        password_hash: &str,
        password: &[u8],
    ) -> Result<(), AccountError> {
        let parsed_hash = PasswordHash::new(password_hash).or(Err(Credentials))?;

        self.argon
            .verify_password(password, &parsed_hash)
            .or(Err(Credentials))
    }
}

impl Default for AccountProvider<u32, u32> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            argon: Argon2::default(),
        }
    }
}
