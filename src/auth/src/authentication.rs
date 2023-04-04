use std::time::Instant;

pub struct Authentication<InternalId> {
    pub(crate) id: InternalId,
    pub(crate) last_seen: Instant,
}

impl<InternalId> From<InternalId> for Authentication<InternalId> {
    fn from(id: InternalId) -> Self {
        Self {
            id,
            last_seen: Instant::now(),
        }
    }
}
