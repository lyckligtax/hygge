#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum Replay<GranteeId, ActionId> {
    Grant(GranteeId, ActionId),
    // Grantee - GranteeOf
    ConnectGrantees(GranteeId, GranteeId),
    // Main - Sub
    ConnectActions(ActionId, ActionId),
    Root(GranteeId),
}
