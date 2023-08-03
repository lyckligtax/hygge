use crate::lease::Lease;
use parking_lot::Mutex;
use std::sync::Arc;

pub(crate) struct Slot<T>(Arc<Mutex<Option<T>>>);

impl<T> Slot<T> {
    /// Construct a new `Slot` holding the given value.
    pub(crate) fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(Some(value))))
    }

    /// Construct a new `Slot` and an immediately acquired `Lease`.
    ///
    /// This avoids the impossible `None` vs. calling `new()` then `lease()`.
    pub(crate) fn new_leased(value: T) -> (Self, Lease<T>) {
        let mut slot = Self::new(value);
        let lease = slot.lease().expect("BUG: new slot empty");
        (slot, lease)
    }

    /// Lease the value from the slot, leaving it empty.
    ///
    /// Ownership of the contained value moves to the `Lease` for the duration. The value may return
    /// to the slot when the `Lease` is dropped, or the value may be "stolen", leaving the slot
    /// permanently empty.
    pub(crate) fn lease(&mut self) -> Option<Lease<T>> {
        if let Some(value) = self.0.try_lock().and_then(|mut slot| slot.take()) {
            Some(Lease::new(value, Arc::downgrade(&self.0)))
        } else {
            None
        }
    }

    /// Get the inner value from the slot, if any.
    ///
    /// Note that if this returns `Some`, there are no oustanding leases. If it returns `None` then
    /// the value has been leased, and since this consumes the slot the value will be dropped once
    /// the lease is done.
    pub(crate) fn into_inner(self) -> Option<T> {
        self.0.lock().take()
    }
}
