use crate::services::auth::account::Account;
#[cfg(test)]
use mockall::{automock, predicate::*};
use uuid::Uuid;
/// store and retrieve accounts by their external id  
/// creates a mapping from an external to an internal id
///
/// internal ids should be used throughout other parts of the application
///
/// implementors may choose their own storage solutions and how to handle sensitive data
#[cfg_attr(test, automock(type Ctx=();))]
pub trait AccountIO {
    type Ctx;

    /// create a new account identified by an external_id and a password
    ///
    /// password SHOULD BE hashed+salted by implementors
    async fn create(
        &mut self,
        id: &str,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Uuid, AccountError>;

    /// retrieve account identified by its external_id
    async fn get_by_external(&self, id: &str, ctx: &mut Self::Ctx)
        -> Result<Account, AccountError>;

    /// retrieve account identified by its internal_id
    async fn get_by_internal(
        &self,
        id: &Uuid,
        ctx: &mut Self::Ctx,
    ) -> Result<Account, AccountError>;

    /// retrieve account identified by its external_id
    async fn exists(&self, id: &str, ctx: &mut Self::Ctx) -> Result<Uuid, AccountError>;

    /// retrieve account identified by its external_id
    async fn update(
        &mut self,
        account: Account,
        ctx: &mut Self::Ctx,
    ) -> Result<Account, AccountError>;

    /// remove an account identified by its internal_id
    async fn remove(&mut self, id: &Uuid, ctx: &mut Self::Ctx) -> Option<AccountError>;

    /// verify a given hash against a password
    async fn create_password_hash(
        &self,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<String, AccountError>;

    /// create a hash of a given password
    async fn verify_credentials(
        &self,
        acc: &Account,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Uuid, AccountError>;
}

pub enum AccountError {
    IO,
    NotFound,
    Invalid,
}
