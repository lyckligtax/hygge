use auth::PermissionIO;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct Provider;

impl Provider {
    pub fn new() -> Self {
        Provider
    }
}

impl PermissionIO for Provider {
    type InternalId = Uuid;
    type PermissionCtx = ();

    async fn create_account(
        &mut self,
        _user_id: Option<Self::InternalId>,
        _ctx: &mut Self::PermissionCtx,
    ) -> bool {
        true
    }
    async fn remove_account(
        &mut self,
        user_id: &Self::InternalId,
        account_to_remove_id: &Self::InternalId,
        _ctx: &mut Self::PermissionCtx,
    ) -> bool {
        user_id == account_to_remove_id
    }
}
