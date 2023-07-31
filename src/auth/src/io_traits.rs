#[cfg(test)]
use mockall::{automock, predicate::*};

/// store and retrieve accounts by their external id  
/// creates a mapping from an external to an internal id
///
/// internal ids should be used throughout other parts of the application
///
/// implementors may choose their own storage solutions and how to handle sensitive data
#[cfg_attr(test, automock(type InternalId=u32; type ExternalId=u32; type AccountCtx=();))]
pub trait AccountIO {
    type InternalId;
    type ExternalId;
    type AccountCtx;

    /// create a new account identified by an external_id and a password
    ///
    /// password SHOULD BE hashed+salted by implementors
    async fn create(
        &mut self,
        id: &Self::ExternalId,
        password: &str,
        ctx: &mut Self::AccountCtx,
    ) -> Result<Self::InternalId, AccountError>;

    /// retrieve account login_data identified by its external_id
    async fn get_login(
        &self,
        id: &Self::ExternalId,
        ctx: &mut Self::AccountCtx,
    ) -> Result<(Self::InternalId, String), AccountError>;

    /// remove an account identified by its internal_id
    async fn remove(
        &mut self,
        id: &Self::InternalId,
        ctx: &Self::AccountCtx,
    ) -> Result<(), AccountError>;

    /// verify a given hash against a password
    ///
    /// implementors SHOULD save the password hashed+salted
    async fn verify_password(
        &self,
        hash: &str,
        password: &str,
        ctx: &Self::AccountCtx,
    ) -> Result<(), AccountError>;
}

pub enum AccountError {
    CouldNotCreate,
    Credentials,
    AlreadyExists,
    NotFound,
}

/// store and retrieve authenticated login tokens  
/// creates new login tokens for a given internal id
#[cfg_attr(test, automock(type InternalId=u32;type LoginToken=u32; type LoginCtx=();))]
pub trait LoginTokenIO {
    type InternalId;
    type LoginToken;
    type LoginCtx;
    /// an internal id may be inserted multiple times for different devices  
    /// **must** return a new login token each time
    async fn insert(
        &mut self,
        id: &Self::InternalId,
        ctx: &mut Self::LoginCtx,
    ) -> Result<Self::LoginToken, AuthenticationError>;

    // TODO: documentation
    async fn remove(
        &mut self,
        id: &Self::LoginToken,
        ctx: &mut Self::LoginCtx,
    ) -> Result<(), AuthenticationError>;

    async fn remove_all(
        &mut self,
        id: &Self::InternalId,
        ctx: &mut Self::LoginCtx,
    ) -> Result<(), AuthenticationError>;

    async fn get(
        &self,
        id: &Self::LoginToken,
        ctx: &mut Self::LoginCtx,
    ) -> Option<Self::InternalId>;
}

#[derive(Debug)]
pub enum AuthenticationError {
    AlreadyExists,
    NotFound,
    IO, //TODO: remove
}
