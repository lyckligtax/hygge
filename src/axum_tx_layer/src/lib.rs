mod layers;
mod lazy;
mod lease;
mod slot;
use crate::lease::Lease;
use crate::slot::Slot;
use axum::async_trait;
pub use layers::*;

#[derive(Clone)]
pub(crate) struct Pool<T>(pub T);
pub(crate) struct TxSlot<Tx>(Slot<Option<Slot<Tx>>>);

#[async_trait]
pub trait TxPool: Send + Sync {
    type Tx: Send + Sync;

    async fn begin(&mut self) -> Option<Self::Tx>;
}

// will be used as parameter inside axum handlers
pub struct Transaction<Tx>(pub Lease<Tx>);

impl<Tx> AsRef<Tx> for Transaction<Tx> {
    fn as_ref(&self) -> &Tx {
        self.0.as_ref()
    }
}

impl<Tx> AsMut<Tx> for Transaction<Tx> {
    fn as_mut(&mut self) -> &mut Tx {
        &mut self.0
    }
}

impl<Tx> std::ops::Deref for Transaction<Tx> {
    type Target = Tx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Tx> std::ops::DerefMut for Transaction<Tx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
