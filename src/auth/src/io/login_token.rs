#[cfg(test)]
use mockall::{automock, predicate::*};

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
