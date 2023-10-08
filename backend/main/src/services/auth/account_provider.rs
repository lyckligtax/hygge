use auth::{AccountError, AccountIO};

use crate::services::auth::account::Account;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct Provider {
    hasher: Argon2<'static>,
}

impl Provider {
    pub fn new() -> Self {
        Provider {
            hasher: Argon2::default(),
        }
    }
}

impl AccountIO for Provider {
    type InternalId = Uuid;
    type ExternalId = String;
    type Account = Account<Self::InternalId, Self::ExternalId>;
    type Ctx = PgConnection;

    async fn create(
        &mut self,
        id: &Self::ExternalId,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::InternalId, AccountError> {
        let hash = self.create_password_hash(password, ctx).await?;

        if let Ok(rec) = sqlx::query!(
            "INSERT INTO public.account (id_external, hash) VALUES ($1, $2) RETURNING id",
            id,
            hash
        )
        .fetch_one(ctx)
        .await
        {
            Ok(rec.id)
        } else {
            Err(AccountError::IO)
        }
    }

    async fn get_by_external(
        &self,
        external_id: &Self::ExternalId,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::Account, AccountError> {
        //TODO: look at why query_as macro cannot use status directly
        sqlx::query_as!(
            Self::Account,
            r#"SELECT id, id_external, hash, status as "status: _" from public.account WHERE id_external = $1"#,
            external_id
        )
        .fetch_one(ctx)
        .await
        .map_err(|_| AccountError::NotFound)
    }

    async fn get_by_internal(
        &self,
        _id: &Self::InternalId,
        _ctx: &mut Self::Ctx,
    ) -> Result<Self::Account, AccountError> {
        todo!()
    }
    async fn exists(
        &self,
        _id: &Self::ExternalId,
        _ctx: &mut Self::Ctx,
    ) -> Result<bool, AccountError> {
        todo!()
    }

    async fn update(
        &self,
        _account: Self::Account,
        _ctx: &mut Self::Ctx,
    ) -> Result<Self::Account, AccountError> {
        todo!()
    }

    async fn remove(
        &mut self,
        internal_id: &Self::InternalId,
        ctx: &mut Self::Ctx,
    ) -> Option<AccountError> {
        match sqlx::query!(
            r#"UPDATE public.account SET status = 'removed' WHERE id = $1"#,
            internal_id
        )
        .execute(ctx)
        .await
        {
            Ok(res) if res.rows_affected() == 1 => None,
            _ => Some(AccountError::NotFound),
        }
    }

    async fn create_password_hash(
        &self,
        password: &str,
        _ctx: &mut Self::Ctx,
    ) -> Result<String, AccountError> {
        let argon2 = &self.hasher;
        let salt = SaltString::generate(&mut OsRng);
        match argon2.hash_password(password.as_ref(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(_) => Err(AccountError::IO),
        }
    }

    async fn verify_credentials(
        &self,
        acc: &Self::Account,
        password: &str,
        _ctx: &mut Self::Ctx,
    ) -> Result<Self::InternalId, AccountError> {
        let Ok(parsed_hash) = PasswordHash::new(&acc.hash) else {
            return Err(AccountError::IO);
        };

        if self
            .hasher
            .verify_password(password.as_ref(), &parsed_hash)
            .is_ok()
        {
            Ok(acc.id)
        } else {
            Err(AccountError::IO)
        }
    }
}
