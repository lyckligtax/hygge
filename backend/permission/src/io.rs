use crate::Change;
use thiserror::Error;

pub trait IO<GranteeId, ActionId> {
    fn read_all(&mut self) -> Box<dyn Iterator<Item = Change<GranteeId, ActionId>>>;
    fn write(&mut self, change: &Change<GranteeId, ActionId>) -> Result<(), IOError>;
    fn flush(&mut self) -> Result<(), IOError>;
    fn clear(&mut self) -> Result<(), IOError>;
}

#[derive(Error, Debug)]
pub enum IOError {
    #[error("Error while writing to IO")]
    Write,
    #[error("Error while flushing IO")]
    Flush,
    #[error("Error while clearing IO")]
    Clear,
}
