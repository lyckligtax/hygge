use crate::Change;
use can_do::CanDo;
use left_right::Absorb;
use std::hash::Hash;

impl<GranteeId, ActionId> Absorb<Change<GranteeId, ActionId>> for CanDo<GranteeId, ActionId>
where
    GranteeId: Hash + Eq + Copy,
    ActionId: Hash + Eq + Copy,
{
    /// apply changes to can_do
    fn absorb_first(&mut self, change: &mut Change<GranteeId, ActionId>, _: &Self) {
        match change {
            Change::Clear => self.clear(),
            Change::RemoveGrantee(grantee_id) => {
                let _ = self.remove_grantee(grantee_id);
            }
            Change::RemoveAction(action_id) => {
                let _ = self.remove_action(action_id);
            }
            Change::AddGrant(grantee_id, action_id) => self.add_grant(grantee_id, action_id),
            Change::RemoveGrant(grantee_id, action_id) => {
                self.remove_grant(grantee_id, action_id).unwrap()
            }
            Change::ConnectGrantees(grantee_id, grantee_of_id) => {
                self.connect_grantees(grantee_id, grantee_of_id)
            }
            Change::DisconnectGrantees(grantee_id, grantee_of_id) => {
                let _ = self.disconnect_grantees(grantee_id, grantee_of_id);
            }
            Change::AddRoot(grantee_id) => self.add_root(grantee_id),
            Change::ConnectActions(main_action_id, sub_action_id) => {
                self.connect_actions(main_action_id, sub_action_id)
            }
            Change::DisconnectActions(main_action_id, sub_action_id) => {
                let _ = self.disconnect_actions(main_action_id, sub_action_id);
            }
            Change::RemoveRoot(grantee_id) => self.remove_root(grantee_id),
        }
    }

    /// this is only called once after the very first publish
    /// clone is expensive but in this case does not matter much
    fn sync_with(&mut self, first: &Self) {
        *self = first.clone()
    }
}
