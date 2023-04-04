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

    fn insert(
        &mut self,
        id: Self::ExternalId,
        password: &[u8],
    ) -> Result<Self::InternalId, AccountError>;

    fn get(
        &self,
        id: &Self::ExternalId,
    ) -> Result<&Account<Self::InternalId, Self::ExternalId>, AccountError>;

    /// returns the account if the password matches
    fn get_verified(
        &self,
        id: &Self::ExternalId,
        password: &[u8],
    ) -> Result<&Account<Self::InternalId, Self::ExternalId>, AccountError>;
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
    fn insert(&mut self, id: &Self::InternalId) -> Result<Self::LoginToken, AuthenticationError>;
    fn remove(&mut self, id: &Self::LoginToken) -> Result<(), AuthenticationError>;
    fn get(&self, id: &Self::LoginToken) -> Option<&Authentication<Self::InternalId>>;
    fn get_mut(&mut self, id: &Self::LoginToken) -> Option<&mut Authentication<Self::InternalId>>;
}
pub enum AuthenticationError {
    AlreadyExists,
    NotFound,
}
