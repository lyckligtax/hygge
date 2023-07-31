use auth::{Account, AccountError, AccountIO};

use super::account::{ProviderAccount, ProviderAccountStatus};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Clone)]
pub struct AccountProvider {
    hasher: Argon2<'static>,
}

impl AccountProvider {
    pub fn new() -> Self {
        AccountProvider {
            hasher: Argon2::default(),
        }
    }
}

impl AccountIO for AccountProvider {
    type InternalId = Uuid;
    type ExternalId = String;

    type AccountCtx = PgConnection;

    async fn create(
        &mut self,
        id: &Self::ExternalId,
        password: &str,
        ctx: &mut Self::AccountCtx,
    ) -> Result<Self::InternalId, AccountError> {
        let argon2 = &self.hasher;
        let salt = SaltString::generate(&mut OsRng);
        let Ok(password_hash) = argon2.hash_password(password.as_ref(), &salt) else {
            return Err(AccountError::CouldNotCreate);
        };

        if let Ok(rec) = sqlx::query!("INSERT INTO public.auth (id, id_external, hash, created_at, updated_at) VALUES (DEFAULT, $1, $2, DEFAULT, DEFAULT) RETURNING id",
            id, password_hash.to_string()
        ).fetch_one(ctx).await {
            Ok(rec.id)
        } else {
            Err(AccountError::CouldNotCreate)
        }
    }

    async fn get(
        &self,
        id: &Self::ExternalId,
        ctx: &mut Self::AccountCtx,
    ) -> Result<Account<Self::InternalId, Self::ExternalId>, AccountError> {
        let row :ProviderAccount<Self::InternalId, Self::ExternalId> = sqlx::query_as!(
            ProviderAccount,
            r#"SELECT id as id_internal, id_external, hash, status as "status: ProviderAccountStatus" from public.auth WHERE id_external = $1"#,
            id
        )
        .fetch_one(ctx)
        .await
        // no explicit match on the Error needed. It is always RowNotFound
        .map_err(|_| AccountError::NotFound)?;

        Ok(Account::from(row))
    }

    async fn remove(
        &mut self,
        _id: &Self::InternalId,
        _ctx: &Self::AccountCtx,
    ) -> Result<(), AccountError> {
        unimplemented!()
    }

    async fn verify_password(
        &self,
        password_hash: &str,
        password: &str,
        _ctx: &Self::AccountCtx,
    ) -> Result<(), AccountError> {
        let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
            return Err(AccountError::Credentials);
        };

        if self
            .hasher
            .verify_password(password.as_ref(), &parsed_hash)
            .is_ok()
        {
            Ok(())
        } else {
            Err(AccountError::Credentials)
        }
    }
}
