use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum CanDoError {
    #[error("Could not find Grantee")]
    GranteeNotFound,
    #[error("Could not find Action")]
    ActionNotFound,
}
