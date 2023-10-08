use crate::IOError;
use can_do::CanDoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Permission entered Failed State")]
    Failed,
    #[error("Error during permission check")]
    Check(CanDoError),
    #[error("Error during IO Performance\n\t{0}")]
    Io(IOError),
}

#[derive(Copy, Clone, Debug)]
pub enum Change<GranteeId, ActionId> {
    Clear,
    RemoveGrantee(GranteeId),
    RemoveAction(ActionId),
    AddGrant(GranteeId, ActionId),
    RemoveGrant(GranteeId, ActionId),
    ConnectGrantees(GranteeId, GranteeId),
    DisconnectGrantees(GranteeId, GranteeId),
    // main - sub
    ConnectActions(ActionId, ActionId),
    DisconnectActions(ActionId, ActionId),
    AddRoot(GranteeId),
    RemoveRoot(GranteeId),
}
