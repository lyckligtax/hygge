#[cfg(test)]
use mockall::{automock, predicate::*};

/// check if a user has specific account actions
#[cfg_attr(test, automock(type InternalId=u32; type PermissionCtx=();))]
pub trait PermissionIO {
    type InternalId;
    type PermissionCtx;

    /// check if a user can create an account
    async fn create_account(
        &mut self,
        user_id: Option<Self::InternalId>,
        ctx: &mut Self::PermissionCtx,
    ) -> bool;

    /// check if a user might delete a given account
    async fn remove_account(
        &mut self,
        user_id: &Self::InternalId,
        account_to_remove_id: &Self::InternalId,
        ctx: &mut Self::PermissionCtx,
    ) -> bool;
}
