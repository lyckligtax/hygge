use crate::lease::Lease;
use crate::slot::Slot;
use crate::TxPool;

pub struct Lazy<Pool: TxPool> {
    pub(crate) pool: Pool,
    pub(crate) tx: Lease<Option<Slot<Pool::Tx>>>,
}

impl<Pool: TxPool> Lazy<Pool> {
    pub(crate) async fn get_or_begin(&mut self) -> Result<Lease<Pool::Tx>, ()> {
        let tx = if let Some(tx) = self.tx.as_mut() {
            tx
        } else {
            let Some(tx) = self.pool.begin().await else {
                return Err(());
            };
            self.tx.insert(Slot::new(tx))
        };

        tx.lease().ok_or(())
    }
}
