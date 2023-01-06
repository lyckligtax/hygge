//! blazingly fast in-memory authorization checker
//!
//! ## inheritance based access
//! CanDo allows for multiple levels of inheritance
//! eg. User1 -> Group1 -> Group2 -> Action1 <- Action2 <- Action3
//! => User1 can perform Action3
mod error;
mod replay;
mod types;

use arena::Arena;
use std::collections::HashMap;
use std::hash::Hash;
use types::{Action, Grantee};

pub use error::*;
pub use replay::*;

#[derive(Clone)]
pub struct CanDo<GranteeId, ActionId> {
    grantees: HashMap<GranteeId, usize>,
    grantees_arena: Arena<Grantee>,
    actions: HashMap<ActionId, usize>,
    actions_arena: Arena<Action>,
}

impl<GranteeId: Hash + Eq + Copy, ActionId: Hash + Eq + Copy> CanDo<GranteeId, ActionId> {
    /// returns a new empty CanDo
    pub fn new() -> Self {
        CanDo {
            grantees: HashMap::new(),
            grantees_arena: Arena::new(),
            actions: HashMap::new(),
            actions_arena: Arena::new(),
        }
    }

    /// clears all grants and inheritances
    pub fn clear(&mut self) {
        *self = CanDo::new();
    }

    /// returns the index of the grantee_id in the arena
    /// if it does not exist a new Grantee will be created and inserted into the arena
    /// its new index will then be returned
    fn get_grantee(&mut self, grantee_id: &GranteeId) -> usize {
        match self.grantees.get(grantee_id) {
            None => {
                let index = self.grantees_arena.insert(Grantee::default());
                self.grantees.insert(*grantee_id, index);
                index
            }
            Some(&grantee) => grantee,
        }
    }

    /// returns the index of the action_id in the arena
    /// if it does not exist a new Action will be created and inserted into the arena
    /// its new index will then be returned
    fn get_action(&mut self, action_id: &ActionId) -> usize {
        match self.actions.get(action_id) {
            None => {
                let index = self.actions_arena.insert(Action::default());
                self.actions.insert(*action_id, index);
                index
            }
            Some(&action) => action,
        }
    }

    /// removes a grantee
    ///
    /// this removes connections between grantees as well as grantees and actions
    /// this might lead to orphaned grantees and actions
    ///
    /// see [CanDo::compact()]
    pub fn remove_grantee(&mut self, grantee_id: &GranteeId) -> Result<(), CanDoError> {
        let Some(grantee_to_delete_index) = self.grantees.remove(grantee_id) else { return Err(CanDoError::GranteeNotFound); };
        let retain_predicate = |el: &usize| el != &grantee_to_delete_index;

        // remove grantee from arena and return it
        let removed_grantee = self.grantees_arena.remove(grantee_to_delete_index);

        // cut grantee connections;
        for &grantee_of in &removed_grantee.grantee_of {
            let _ = &self
                .grantees_arena
                .get_mut(grantee_of)
                .grantees
                .retain(retain_predicate);
        }

        for &grantee_index in &removed_grantee.grantees {
            let _ = &self
                .grantees_arena
                .get_mut(grantee_index)
                .grantee_of
                .retain(retain_predicate);
        }
        // remove bidirectional action connections
        for &action_index in &removed_grantee.actions {
            let _ = &self
                .actions_arena
                .get_mut(action_index)
                .grantees
                .retain(retain_predicate);
        }
        Ok(())
    }

    /// removes an action
    ///
    /// this removes connections between grantees and actions
    /// this might lead to orphaned grantees
    ///
    /// see [CanDo::compact()]
    pub fn remove_action(&mut self, action_id: &ActionId) -> Result<(), CanDoError> {
        let Some(action_to_remove_index) = self.actions.remove(action_id) else { return Err(CanDoError::ActionNotFound); };
        let removed_action = self.actions_arena.remove(action_to_remove_index);
        // cut grantee connections
        for grantee_index in removed_action.grantees {
            self.grantees_arena
                .get_mut(grantee_index)
                .actions
                .retain(|el: &usize| el != &action_to_remove_index);
        }
        // cut action connections
        for action_index in removed_action.sub_action_of {
            self.actions_arena
                .get_mut(action_index)
                .main_action_of
                .retain(|el: &usize| el != &action_to_remove_index);
        }

        Ok(())
    }

    /// grants a grantee the permission to perform an action
    /// eg: grants the user Thomas to read the newspost 9
    pub fn add_grant(&mut self, grantee_id: &GranteeId, action_id: &ActionId) {
        let grantee = self.get_grantee(grantee_id);
        let action = self.get_action(action_id);

        self.actions_arena.get_mut(action).grantees.push(grantee);
        self.grantees_arena.get_mut(grantee).actions.push(action);
    }

    /// removes a grant
    ///
    /// this removes connections between a grantee and an action
    /// this might lead to orphaned grantees and actions
    ///
    /// see [CanDo::compact()]
    pub fn remove_grant(
        &mut self,
        grantee_id: &GranteeId,
        action_id: &ActionId,
    ) -> Result<(), CanDoError> {
        let Some(&grantee_index) = self.grantees.get(grantee_id) else {
            return Err(CanDoError::GranteeNotFound);
        };
        let Some(&action_index) = self.actions.get(action_id) else {
            return Err(CanDoError::ActionNotFound);
        };

        self.grantees_arena
            .get_mut(grantee_index)
            .actions
            .retain(|el: &usize| el != &action_index);
        self.actions_arena
            .get_mut(action_index)
            .grantees
            .retain(|el: &usize| el != &grantee_index);

        Ok(())
    }

    /// removes a connection between to actions
    ///
    /// if a grantee can perform main_action_id it can also perform sub_action_id
    /// this allows for grant inheritance
    pub fn disconnect_actions(
        &mut self,
        main_action_id: &ActionId,
        sub_action_id: &ActionId,
    ) -> Result<(), CanDoError> {
        let Some(&main_action_idx) = self.actions.get(main_action_id) else { return Err(CanDoError::ActionNotFound); };
        let Some(&sub_action_idx) = self.actions.get(sub_action_id) else { return Err(CanDoError::ActionNotFound); };

        if main_action_idx == sub_action_idx {
            return Ok(());
        }

        self.actions_arena
            .get_mut(sub_action_idx)
            .sub_action_of
            .retain(|el: &usize| el != &main_action_idx);
        self.actions_arena
            .get_mut(main_action_idx)
            .main_action_of
            .retain(|el: &usize| el != &sub_action_idx);

        Ok(())
    }

    /// adds a connection between to actions
    ///
    /// if a grantee can perform main_action_id it can also perform sub_action_id
    /// this allows for grant inheritance
    pub fn connect_actions(&mut self, main_action_id: &ActionId, sub_action_id: &ActionId) {
        let main_action_idx = self.get_action(main_action_id);
        let sub_action_idx = self.get_action(sub_action_id);

        if main_action_idx == sub_action_idx {
            return;
        }

        self.actions_arena
            .get_mut(sub_action_idx)
            .sub_action_of
            .push(main_action_idx);
        self.actions_arena
            .get_mut(main_action_idx)
            .main_action_of
            .push(sub_action_idx);
    }

    /// adds a connection between two grantees
    ///
    /// if either grantee does not exist yet it is created
    /// allows for grant inheritance
    pub fn connect_grantees(&mut self, grantee_id: &GranteeId, grantee_of_id: &GranteeId) {
        let grantee_index = self.get_grantee(grantee_id);
        let grantee_of_index = self.get_grantee(grantee_of_id);

        // if both grantees share the same index we assume them to be equal
        if grantee_index == grantee_of_index {
            return;
        }

        self.grantees_arena
            .get_mut(grantee_index)
            .grantee_of
            .push(grantee_of_index);
        self.grantees_arena
            .get_mut(grantee_of_index)
            .grantees
            .push(grantee_index);
    }

    /// removes a connection between to grantees
    ///
    /// this might lead to orphaned grantees
    ///
    /// see [CanDo::compact()]
    pub fn disconnect_grantees(
        &mut self,
        grantee_id: &GranteeId,
        grantee_of_id: &GranteeId,
    ) -> Result<(), CanDoError> {
        let Some(&grantee_index) = self.grantees.get(grantee_id) else { return Err(CanDoError::GranteeNotFound); };
        let Some(&grantee_of_index) = self.grantees.get(grantee_of_id) else { return Err(CanDoError::GranteeNotFound); };

        if grantee_index == grantee_of_index {
            return Ok(());
        }
        self.grantees_arena
            .get_mut(grantee_index)
            .grantee_of
            .retain(|el: &usize| el != &grantee_of_index);
        self.grantees_arena
            .get_mut(grantee_of_index)
            .grantees
            .retain(|el: &usize| el != &grantee_of_index);
        Ok(())
    }

    /// updates a grantee to be a root node
    ///
    /// root nodes do not get removed when compacting
    /// this can be used to resync (delete + refill) parts of can_do
    pub fn add_root(&mut self, grantee_id: &GranteeId) {
        let grantee_index = self.get_grantee(grantee_id);
        self.grantees_arena.get_mut(grantee_index).is_root = true;
    }

    /// updates a grantee to NOT be a root node
    ///
    /// root nodes do not get removed when compacting
    /// this can be used to resync (delete + refill) parts of can_do
    pub fn remove_root(&mut self, grantee_id: &GranteeId) {
        let grantee_index = self.get_grantee(grantee_id);
        self.grantees_arena.get_mut(grantee_index).is_root = false;
    }

    fn collect_main_action_idx(&self, action_idx: usize) -> Vec<usize> {
        // simply walk the tree bia BFS and mark all visited
        let mut actions_checked: Vec<bool> = vec![false; self.actions.len()];
        actions_checked[action_idx] = true;

        // iterate over Vec of Vecs for performance
        // no need to copy or clone whole Vecs
        let mut actions_to_check = vec![&self.actions_arena.get(action_idx).sub_action_of];
        while !actions_to_check.is_empty() {
            let mut next_actions_to_check = Vec::<&Vec<usize>>::new();
            for sub_actions in actions_to_check {
                for &action_to_check in sub_actions {
                    if actions_checked[action_to_check] {
                        // prevent loops
                        continue;
                    }

                    actions_checked[action_to_check] = true;
                    next_actions_to_check
                        .push(&self.actions_arena.get(action_to_check).sub_action_of);
                }
            }

            actions_to_check = next_actions_to_check
        }
        // every checked action is also a valid main_action of actions_idx
        actions_checked
            .into_iter()
            .enumerate()
            .filter_map(|(idx, checked)| if checked { Some(idx) } else { None })
            .collect()
    }

    /// check if a user can perform an action
    ///
    /// uses breadth first search over connected grantees
    /// checks for loops
    pub fn can_grantee_do(
        &self,
        grantee_id: &GranteeId,
        action_id: &ActionId,
    ) -> Result<bool, CanDoError> {
        let Some(&grantee_idx) = self.grantees.get(grantee_id) else { return Err(CanDoError::GranteeNotFound); };
        let Some(&sub_action_idx) = self.actions.get(action_id) else { return Err(CanDoError::ActionNotFound); };
        // grantee_idx and action_idx are positions in the arena
        // thus comparing them is easy

        // we know the maximum number of members ahead of time. This is worst case needed
        // bool uses 1 byte instead of 1 bit thus we have size bytes allocated
        // bitvec could store it bitwise but the values would have to be "translated"
        // only for gargantuan sizes this would be needed anyway and we take the performance boost here
        let mut grantees_checked: Vec<bool> = vec![false; self.grantees.len()];

        let mut grantees_to_check = Vec::<&Vec<usize>>::new();
        // as actions are inheritable we need to get a list all all possible actions that might fit and check for their grantees
        for action_idx in self.collect_main_action_idx(sub_action_idx) {
            grantees_to_check.push(&self.actions_arena.get(action_idx).grantees);
        }

        // might be transitive grantee
        while !grantees_to_check.is_empty() {
            // using breadth first search this is a list of all possible grantees at this level in the tree
            let mut next_grantees_to_check = Vec::<&Vec<usize>>::new();
            // grantees to check have previously been checked to not equal grantee
            // check their grantees
            for next_to_check in grantees_to_check {
                for &grantee_to_check in next_to_check {
                    if grantees_checked[grantee_to_check] {
                        continue;
                    }
                    if grantee_to_check == grantee_idx {
                        return Ok(true);
                    }
                    // prevent loops
                    grantees_checked[grantee_to_check] = true;
                    next_grantees_to_check
                        .push(&self.grantees_arena.get(grantee_to_check).grantees);
                }
            }

            grantees_to_check = next_grantees_to_check;
        }

        Ok(false)
    }

    pub fn compact(&mut self) -> Vec<Replay<GranteeId, ActionId>> {
        // find and remove orphaned actions
        // an action is orphaned if it is
        //  a) not granted to a grantee
        //  b) not a subaction
        // to check this we have to iterate over actions until no orphaned is found
        loop {
            let actions_to_remove: Vec<ActionId> = self
                .actions
                .iter()
                .filter_map(|(id, idx)| {
                    let action = self.actions_arena.get(*idx);
                    if action.grantees.is_empty() && action.main_action_of.is_empty() {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();

            if actions_to_remove.is_empty() {
                break;
            }

            actions_to_remove.iter().for_each(|id| {
                self.remove_action(id)
                    .expect("Expected Action to be removed")
            });
        }

        // find and remove orphaned grantees
        // a grantee is orphaned if it is
        //  a) not granted an action AND has no further grantees
        //  b) is not a grantee of anything
        // to check this we have to iterate over grantees until no orphaned is found
        loop {
            let grantees_to_remove: Vec<GranteeId> = self
                .grantees
                .iter()
                .filter_map(|(id, idx)| {
                    let grantee = self.grantees_arena.get(*idx);
                    if grantee.is_root {
                        // do not remove root grantees
                        return None;
                    }
                    if (
                        grantee.actions.is_empty() && grantee.grantees.is_empty()) // last in chain without any actions
                        || grantee.grantee_of.is_empty()
                    // first in chain but no parent
                    {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();

            if grantees_to_remove.is_empty() {
                // no more changes
                break;
            }

            grantees_to_remove.iter().for_each(|id| {
                self.remove_grantee(id)
                    .expect("Expected Grantee to be removed")
            });
        }

        // compact arenas and apply changes
        let grantee_compactions: HashMap<usize, usize> =
            self.grantees_arena.compact().into_iter().collect();
        for old_idx in self.grantees.values_mut() {
            if let Some(new_idx) = grantee_compactions.get(old_idx) {
                *old_idx = *new_idx
            }
        }

        let action_compactions: HashMap<usize, usize> =
            self.actions_arena.compact().into_iter().collect();
        for old_idx in self.actions.values_mut() {
            if let Some(new_idx) = action_compactions.get(old_idx) {
                *old_idx = *new_idx
            }
        }

        // map current state into Replays
        let reversed_grantees: HashMap<usize, GranteeId> =
            self.grantees.iter().map(|(k, v)| (*v, *k)).collect();
        let reversed_actions: HashMap<usize, ActionId> =
            self.actions.iter().map(|(k, v)| (*v, *k)).collect();

        let grants_iter = self.actions.iter().flat_map(|(action_id, action_idx)| {
            self.actions_arena
                .get(*action_idx)
                .grantees
                .iter()
                .map(|grantee_idx| {
                    Replay::Grant(*reversed_grantees.get(grantee_idx).unwrap(), *action_id)
                })
        });

        let roots_iter = self
            .grantees
            .iter()
            .filter_map(|(grantee_id, grantee_idx)| {
                let grantee = self.grantees_arena.get(*grantee_idx);
                if grantee.is_root {
                    Some(Replay::Root(*grantee_id))
                } else {
                    None
                }
            });

        let connect_grantees_iter = self
            .grantees
            .iter()
            .filter(|(_, grantee_idx)| !self.grantees_arena.get(**grantee_idx).is_root)
            .flat_map(|(grantee_id, grantee_idx)| {
                self.grantees_arena
                    .get(*grantee_idx)
                    .grantee_of
                    .iter()
                    .map(|grantee_of_idx| {
                        Replay::ConnectGrantees(
                            *grantee_id,
                            *reversed_grantees.get(grantee_of_idx).unwrap(),
                        )
                    })
            });

        let connect_actions_iter = self.actions.iter().flat_map(|(action_id, action_idx)| {
            self.actions_arena
                .get(*action_idx)
                .main_action_of
                .iter()
                .map(|action_of_idx| {
                    Replay::ConnectActions(
                        *action_id,
                        *reversed_actions.get(action_of_idx).unwrap(),
                    )
                })
        });

        connect_grantees_iter
            .chain(connect_actions_iter)
            .chain(grants_iter)
            .chain(roots_iter)
            .collect()
    }
}

impl<GranteeId: Hash + Eq + Copy, ActionId: Hash + Eq + Copy> Default
    for CanDo<GranteeId, ActionId>
{
    fn default() -> Self {
        CanDo::<GranteeId, ActionId>::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Id = u32;
    #[derive(Eq, PartialEq, Hash, Copy, Clone)]
    pub enum ActionItem<ItemId> {
        Read(ItemId),
    }

    #[derive(Eq, PartialEq, Hash, Copy, Clone)]
    enum Grantee {
        User(Id),
        Group(Id),
    }

    #[test]
    fn we_can_add_grantee_connections() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();
        let user1 = Grantee::User(1);
        let user2 = Grantee::User(2);

        can_do.connect_grantees(&user1, &user2);

        assert_eq!(2, can_do.grantees.len());

        let user1_grantee_index = can_do.grantees.get(&user1).unwrap();
        let user1_grantee = can_do.grantees_arena.get(*user1_grantee_index);
        assert_eq!(0, user1_grantee.grantees.len());
        assert_eq!(1, user1_grantee.grantee_of.len());

        let user2_grantee_index = can_do.grantees.get(&user2).unwrap();
        let user2_grantee = can_do.grantees_arena.get(*user2_grantee_index);
        assert_eq!(1, user2_grantee.grantees.len());
        assert_eq!(0, user2_grantee.grantee_of.len());
    }

    #[test]
    fn check_that_member_can_do_action() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();
        let read = ActionItem::Read(1);
        let user1 = Grantee::User(1);

        can_do.add_grant(&user1, &read);

        assert!(can_do.can_grantee_do(&user1, &read).unwrap());
    }

    #[test]
    fn check_that_member_can_not_do_action() {
        let can_do = CanDo::<Grantee, ActionItem<Id>>::new();
        let read = ActionItem::Read(1);
        let user1 = Grantee::User(1);

        assert_eq!(
            Err(CanDoError::GranteeNotFound),
            can_do.can_grantee_do(&user1, &read)
        );
    }

    #[test]
    fn check_that_member_can_do_through_transitive_membership_action() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();
        let read = ActionItem::Read(1);
        let user1 = Grantee::User(1);
        let group1 = Grantee::Group(1);
        let group2 = Grantee::Group(2);

        can_do.connect_grantees(&user1, &group1);
        can_do.connect_grantees(&group1, &group2);
        can_do.add_grant(&group2, &read);

        assert!(can_do.can_grantee_do(&user1, &read).unwrap());
    }

    #[test]
    fn check_that_member_can_do_through_transitive_action() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();
        let read1 = ActionItem::Read(1);
        let read2 = ActionItem::Read(2);
        let user = Grantee::User(1);
        can_do.connect_actions(&read1, &read2);
        can_do.add_grant(&user, &read1);

        assert!(can_do.can_grantee_do(&user, &read2).unwrap());
    }

    #[test]
    fn we_can_remove_grantees() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let group1 = Grantee::Group(1);

        can_do.connect_grantees(&user1, &group1);

        assert!(can_do.grantees.contains_key(&user1));
        assert!(can_do.grantees.contains_key(&group1));

        can_do.remove_grantee(&user1).unwrap();

        assert!(!can_do.grantees.contains_key(&user1));
        assert!(can_do.grantees.contains_key(&group1));
    }

    #[test]
    fn removing_grantees_cuts_connections() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let group1 = Grantee::Group(1);

        can_do.connect_grantees(&user1, &group1);

        let &user1_index = can_do.grantees.get(&user1).unwrap();
        let &group1_index = can_do.grantees.get(&group1).unwrap();

        let user1_grantee = can_do.grantees_arena.get(user1_index);
        assert!(user1_grantee.grantee_of.contains(&group1_index));

        let group1_grantee = can_do.grantees_arena.get(group1_index);
        assert!(&group1_grantee.grantees.contains(&user1_index));

        can_do.remove_grantee(&user1).unwrap();

        let group1_grantee = &can_do.grantees_arena.get(group1_index);
        assert!(&group1_grantee.grantees.is_empty());
    }

    #[test]
    fn removing_grantees_removes_them_from_actions() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let read = ActionItem::Read(1);

        can_do.add_grant(&user1, &read);

        let &read_index = can_do.actions.get(&read).unwrap();
        let read_action = can_do.actions_arena.get(read_index);

        let user1_grantee_index = can_do.grantees.get(&user1).unwrap();
        let user1_grantee = can_do.grantees_arena.get(*user1_grantee_index);

        assert!(&user1_grantee.actions.contains(&read_index));
        assert!(&read_action.grantees.contains(user1_grantee_index));

        can_do.remove_grantee(&user1).unwrap();

        let read_action = can_do.actions_arena.get(read_index);
        assert!(&read_action.grantees.is_empty());
    }

    #[test]
    fn removing_actions_removes_them_from_grantees() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let read = ActionItem::Read(1);

        can_do.add_grant(&user1, &read);
        let read_index = can_do.actions.get(&read).unwrap();

        let &user1_grantee_index = can_do.grantees.get(&user1).unwrap();
        let user1_grantee = can_do.grantees_arena.get(user1_grantee_index);
        assert!(&user1_grantee.actions.contains(read_index));

        can_do.remove_action(&read).unwrap();

        let user1_grantee = can_do.grantees_arena.get(user1_grantee_index);
        assert!(&user1_grantee.actions.is_empty());
    }

    #[test]
    fn we_can_revoke_grants() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let read = ActionItem::Read(1);

        can_do.add_grant(&user1, &read);
        let read_index = can_do.actions.get(&read).unwrap();

        let &user1_grantee_index = can_do.grantees.get(&user1).unwrap();
        let user1_grantee = can_do.grantees_arena.get(user1_grantee_index);
        assert!(&user1_grantee.actions.contains(read_index));

        can_do.remove_grant(&user1, &read).unwrap();

        let user1_grantee = can_do.grantees_arena.get(user1_grantee_index);

        assert!(&user1_grantee.actions.is_empty());
        assert_eq!(1, can_do.actions.len());
    }

    #[test]
    fn compact_should_not_need_to_do_anything() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let read = ActionItem::Read(1);

        can_do.add_grant(&user1, &read);
        can_do.add_root(&user1);
        assert_eq!(1, can_do.actions.len());
        assert_eq!(1, can_do.grantees.len());

        let replays = can_do.compact();
        assert_eq!(2, replays.len());
        assert_eq!(1, can_do.actions.len());
        assert_eq!(1, can_do.grantees.len());
    }

    #[test]
    fn compact_should_remove_all_grantees() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let group1 = Grantee::Group(2);

        can_do.connect_grantees(&user1, &group1);
        assert_eq!(0, can_do.actions.len());
        assert_eq!(2, can_do.grantees.len());

        let replays = can_do.compact();
        assert_eq!(0, replays.len());
        assert_eq!(0, can_do.actions.len());
        assert_eq!(0, can_do.grantees.len());
    }

    #[test]
    fn compact_should_remove_all_actions() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let action1 = ActionItem::Read(1);
        let action2 = ActionItem::Read(2);
        let action3 = ActionItem::Read(3);

        can_do.connect_actions(&action1, &action2);
        can_do.connect_actions(&action2, &action3);
        assert_eq!(3, can_do.actions.len());
        assert_eq!(0, can_do.grantees.len());

        let replays = can_do.compact();
        assert_eq!(0, replays.len());
        assert_eq!(0, can_do.actions.len());
        assert_eq!(0, can_do.grantees.len());
    }

    #[test]
    fn compact_should_remove_all_grantees_not_connected_to_root() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let user2 = Grantee::User(2);
        let group1 = Grantee::Group(3);

        can_do.add_root(&user2);
        can_do.connect_grantees(&user1, &group1);

        assert_eq!(0, can_do.actions.len());
        assert_eq!(3, can_do.grantees.len(), "We should have 3 grantees");

        let replays = can_do.compact();
        assert_eq!(
            1,
            replays.len(),
            "We should need to replay 1 Grant for user2"
        );
        assert_eq!(0, can_do.actions.len());
        assert_eq!(1, can_do.grantees.len());
    }

    #[test]
    fn compact_should_remove_group2() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let user1 = Grantee::User(1);
        let group1 = Grantee::Group(2);
        let group2 = Grantee::Group(3);
        let read = ActionItem::Read(1);

        can_do.add_grant(&user1, &read);
        can_do.connect_grantees(&user1, &group1);
        can_do.connect_grantees(&group2, &group1);
        can_do.add_root(&group1);

        assert_eq!(1, can_do.actions.len());
        assert_eq!(3, can_do.grantees.len());

        let replays = can_do.compact();
        assert_eq!(3, replays.len());
        assert_eq!(1, can_do.actions.len());
        assert_eq!(2, can_do.grantees.len());
        assert!(!can_do.grantees.contains_key(&group2));
    }

    #[test]
    fn connect_actions_should_work() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let read1 = ActionItem::Read(1);
        let read2 = ActionItem::Read(2);

        can_do.connect_actions(&read1, &read2);
        assert_eq!(2, can_do.actions.len());

        let &read1_index = can_do.actions.get(&read1).unwrap();
        let read1_action = can_do.actions_arena.get(read1_index);

        let &read2_index = can_do.actions.get(&read2).unwrap();
        let read2_action = can_do.actions_arena.get(read2_index);

        assert!(read1_action.main_action_of.contains(&read2_index));
        assert!(read2_action.sub_action_of.contains(&read1_index));
    }

    #[test]
    fn disconnect_actions_should_work() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let read1 = ActionItem::Read(1);
        let read2 = ActionItem::Read(2);

        can_do.connect_actions(&read1, &read2);
        can_do.disconnect_actions(&read1, &read2).unwrap();
        assert_eq!(2, can_do.actions.len());

        let &read1_index = can_do.actions.get(&read1).unwrap();
        let read1_action = can_do.actions_arena.get(read1_index);

        let &read2_index = can_do.actions.get(&read2).unwrap();
        let read2_action = can_do.actions_arena.get(read2_index);

        assert!(read1_action.main_action_of.is_empty());
        assert!(read2_action.sub_action_of.is_empty());
    }

    #[test]
    fn disconnect_actions_should_only_remove_one_action() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let read1 = ActionItem::Read(1);
        let read2 = ActionItem::Read(2);
        let read3 = ActionItem::Read(3);

        can_do.connect_actions(&read1, &read2);
        can_do.connect_actions(&read1, &read3);
        can_do.disconnect_actions(&read1, &read2).unwrap();
        assert_eq!(3, can_do.actions.len());

        let &read1_index = can_do.actions.get(&read1).unwrap();
        let read1_action = can_do.actions_arena.get(read1_index);

        let &read3_index = can_do.actions.get(&read3).unwrap();
        let read3_action = can_do.actions_arena.get(read3_index);

        assert!(read1_action.main_action_of.contains(&read3_index));
        assert!(read3_action.sub_action_of.contains(&read1_index));
    }

    #[test]
    fn collection_of_subaction_idx() {
        let mut can_do = CanDo::<Grantee, ActionItem<Id>>::new();

        let read1 = ActionItem::Read(1);
        let read2 = ActionItem::Read(2);
        let read3 = ActionItem::Read(3);
        let read4 = ActionItem::Read(4);

        can_do.connect_actions(&read1, &read2);
        can_do.connect_actions(&read2, &read3);
        can_do.connect_actions(&read1, &read4);
        assert_eq!(4, can_do.actions.len());

        let &read1_index = can_do.actions.get(&read1).unwrap();
        let &read2_index = can_do.actions.get(&read2).unwrap();
        let &read3_index = can_do.actions.get(&read3).unwrap();

        let main_action_idx = can_do.collect_main_action_idx(read3_index);
        assert_eq!(vec![read1_index, read2_index, read3_index], main_action_idx);
    }
}
