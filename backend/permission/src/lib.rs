use can_do::CanDo;
use left_right::{ReadHandle, WriteHandle};
use std::hash::Hash;
use std::sync::Mutex;

mod io;
mod lr;
mod types;

pub use io::*;
pub use types::*;

pub struct Permission<GranteeId, ActionId>
where
    GranteeId: Hash + Eq + Copy,
    ActionId: Hash + Eq + Copy,
{
    is_failed: bool,
    writer: Mutex<WriteHandle<CanDo<GranteeId, ActionId>, Change<GranteeId, ActionId>>>,
    reader: ReadHandle<CanDo<GranteeId, ActionId>>,
    io: Mutex<Box<dyn IO<GranteeId, ActionId>>>,
}

impl<GranteeId, ActionId> Permission<GranteeId, ActionId>
where
    GranteeId: Hash + Eq + Copy,
    ActionId: Hash + Eq + Copy,
{
    pub fn new(mut io: Box<dyn IO<GranteeId, ActionId>>) -> Self {
        let (mut writer, reader) =
            left_right::new::<CanDo<GranteeId, ActionId>, Change<GranteeId, ActionId>>();

        for change in &mut io.read_all() {
            writer.append(change);
        }
        writer.publish();

        Permission {
            is_failed: false,
            writer: Mutex::new(writer),
            reader,
            io: Mutex::new(io),
        }
    }

    /// persist event and apply changes to database
    /// batch a list of changes
    pub fn change(
        &mut self,
        changes: Vec<Change<GranteeId, ActionId>>,
    ) -> Result<(), PermissionError> {
        if self.is_failed {
            return Err(PermissionError::Failed);
        }
        let io = &mut self
            .io
            .get_mut()
            .expect("Expected to get exclusive write lock");

        // batch write event into IO
        for change in &changes {
            if let Err(write_error) = io.write(change) {
                return Err(PermissionError::Io(write_error));
            }
        }

        let can_do_writer = &mut self
            .writer
            .get_mut()
            .expect("Expected to get exclusive write lock");

        // batch write event into CanDo
        for change in changes {
            can_do_writer.append(change);
        }

        if let Err(flush_error) = io.flush() {
            if let Err(clear_error) = io.clear() {
                // could neither flush nor clear
                self.is_failed = true;
                return Err(PermissionError::Io(clear_error));
            };

            return Err(PermissionError::Io(flush_error));
        }
        // publish pending changes to CanDo
        can_do_writer.flush();
        Ok(())
    }

    pub fn check(
        &self,
        grantee_id: &GranteeId,
        action_id: &ActionId,
    ) -> Result<bool, PermissionError> {
        if !self.is_failed {
            return Err(PermissionError::Failed);
        }

        match self
            .reader
            .enter()
            .expect("Expected to get ReadGuard on CanDo")
            .can_grantee_do(grantee_id, action_id)
        {
            Ok(result) => Ok(result),
            Err(check_error) => Err(PermissionError::Check(check_error)),
        }
    }
}
