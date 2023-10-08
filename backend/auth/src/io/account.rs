#[cfg(test)]
use mockall::{automock, predicate::*};

/// store and retrieve accounts by their external id  
/// creates a mapping from an external to an internal id
///
/// internal ids should be used throughout other parts of the application
///
/// implementors may choose their own storage solutions and how to handle sensitive data
#[cfg_attr(test, automock(type InternalId=u32; type ExternalId=u32; type Account=(); type Ctx=();))]
pub trait AccountIO {
    type InternalId;
    type ExternalId;
    type Account;
    type Ctx;

    /// create a new account identified by an external_id and a password
    ///
    /// password SHOULD BE hashed+salted by implementors
    async fn create(
        &mut self,
        id: &Self::ExternalId,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::InternalId, AccountError>;

    /// retrieve account identified by its external_id
    async fn get_by_external(
        &self,
        id: &Self::ExternalId,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::Account, AccountError>;

    /// retrieve account identified by its internal_id
    async fn get_by_internal(
        &self,
        id: &Self::InternalId,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::Account, AccountError>;

    /// retrieve account identified by its external_id
    async fn exists(
        &self,
        id: &Self::ExternalId,
        ctx: &mut Self::Ctx,
    ) -> Result<bool, AccountError>;

    /// retrieve account identified by its external_id
    async fn update(
        &self,
        account: Self::Account,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::Account, AccountError>;

    /// remove an account identified by its internal_id
    async fn remove(&mut self, id: &Self::InternalId, ctx: &mut Self::Ctx) -> Option<AccountError>;

    /// verify a given hash against a password
    async fn create_password_hash(
        &self,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<String, AccountError>;

    /// create a hash of a given password
    async fn verify_credentials(
        &self,
        acc: &Self::Account,
        password: &str,
        ctx: &mut Self::Ctx,
    ) -> Result<Self::InternalId, AccountError>;
}

pub enum AccountError {
    IO,
    NotFound,
    Invalid,
}
