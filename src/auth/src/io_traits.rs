use crate::account::Account;
use crate::Authentication;

/// store and retrieve accounts by their external id  
/// creates a mapping from an external to an internal id
///
/// internal ids should be used throughout other parts of the application
///
/// eg: email (external) -> UUID (internal)
pub trait AccountIO {
    type InternalId;
    type ExternalId;

    // TODO: documentation
    async fn insert(
        &mut self,
        id: Self::ExternalId,
        password: &[u8],
    ) -> Result<Self::InternalId, AccountError>;

    async fn get(
        &self,
        id: &Self::ExternalId,
    ) -> Result<&Account<Self::InternalId, Self::ExternalId>, AccountError>;

    async fn verify_password(&self, hash: &str, password: &[u8]) -> Result<(), AccountError>;
}

pub enum AccountError {
    CouldNotCreate,
    Credentials,
    AlreadyExists,
    NotFound,
}

/// store and retrieve authenticated login tokens  
/// creates new login tokens for a given internal id
pub trait AuthenticationIO {
    type InternalId;
    type LoginToken;
    /// an internal id may be inserted multiple times for different devices  
    /// **must** return a new login token each time
    async fn insert(
        &mut self,
        id: &Self::InternalId,
    ) -> Result<Self::LoginToken, AuthenticationError>;
    // TODO: documentation
    async fn remove(&mut self, id: &Self::LoginToken) -> Result<(), AuthenticationError>;
    async fn get(&self, id: &Self::LoginToken) -> Option<&Authentication<Self::InternalId>>;
    async fn get_mut(
        &mut self,
        id: &Self::LoginToken,
    ) -> Option<&mut Authentication<Self::InternalId>>;
}
pub enum AuthenticationError {
    AlreadyExists,
    NotFound,
}
