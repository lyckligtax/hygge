use crate::services::auth::account::Account;
use crate::services::auth::io::{AccountError, AccountIO};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct LocalAccountIO {
    hasher: Argon2<'static>,
}

impl LocalAccountIO {
    pub fn new() -> Self {
        LocalAccountIO {
            hasher: Argon2::default(),
        }
    }
}

impl AccountIO for LocalAccountIO {
    type Ctx = PgConnection;

    async fn create(
        &mut self,
        id: &str,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Uuid, AccountError> {
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
        external_id: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Account, AccountError> {
        //TODO: look at why query_as macro cannot use status directly
        sqlx::query_as!(
            Account,
            r#"SELECT id, id_external, hash, status as "status: _" from public.account WHERE id_external = $1"#,
            external_id
        )
        .fetch_one(ctx)
        .await
        .map_err(|_| AccountError::NotFound)
    }

    async fn get_by_internal(
        &self,
        id: &Uuid,
        ctx: &mut Self::Ctx,
    ) -> Result<Account, AccountError> {
        //TODO: look at why query_as macro cannot use status directly
        sqlx::query_as!(
            Account,
            r#"SELECT id, id_external, hash, status as "status: _" from public.account WHERE id = $1"#,
            id
        )
        .fetch_one(ctx)
        .await
        .map_err(|_| AccountError::NotFound)
    }
    async fn exists(&self, external_id: &str, ctx: &mut Self::Ctx) -> Result<Uuid, AccountError> {
        if let Ok(rec) = sqlx::query!(
            r#"SELECT id from public.account where id_external = $1"#,
            external_id
        )
        .fetch_one(ctx)
        .await
        {
            Ok(rec.id)
        } else {
            Err(AccountError::NotFound)
        }
    }

    async fn update(
        &mut self,
        _account: Account,
        _ctx: &mut Self::Ctx,
    ) -> Result<Account, AccountError> {
        todo!()
    }

    async fn remove(&mut self, internal_id: &Uuid, ctx: &mut Self::Ctx) -> Option<AccountError> {
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
        acc: &Account,
        password: &str,
        _ctx: &mut Self::Ctx,
    ) -> Result<Uuid, AccountError> {
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
