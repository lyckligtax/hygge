use parking_lot::Mutex;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct Lease<T>(Option<(T, Weak<Mutex<Option<T>>>)>);

impl<T> Lease<T> {
    pub fn new(value: T, slot: Weak<Mutex<Option<T>>>) -> Self {
        Self(Some((value, slot)))
    }
}

impl<T> Drop for Lease<T> {
    fn drop(&mut self) {
        match self.0.take() {
            None => panic!("BUG: Lease::drop called twice"),
            Some((value, slot)) => {
                // try to return value to the slot, if it fails just drop value
                if let Some(slot) = slot.upgrade() {
                    if let Some(mut slot) = slot.try_lock() {
                        assert!(slot.is_none(), "BUG: slot repopulated during lease");
                        *slot = Some(value);
                    }
                }
            }
        }
    }
}

impl<T> AsRef<T> for Lease<T> {
    fn as_ref(&self) -> &T {
        match &self.0 {
            None => panic!("BUG: Lease used after drop/steal"),
            Some((value, ..)) => value,
        }
    }
}

impl<T> AsMut<T> for Lease<T> {
    fn as_mut(&mut self) -> &mut T {
        match &mut self.0 {
            None => panic!("BUG: Lease used after drop/steal"),
            Some((value, ..)) => value,
        }
    }
}

impl<T> std::ops::Deref for Lease<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> std::ops::DerefMut for Lease<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
