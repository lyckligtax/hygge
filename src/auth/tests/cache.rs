use auth::{Authentication, AuthenticationError, AuthenticationIO};
use std::collections::HashMap;

pub struct Cache<InternalId, CacheId> {
    data: HashMap<CacheId, Authentication<InternalId>>,
    internal_id: u32,
}

impl AuthenticationIO for Cache<u32, u32> {
    type InternalId = u32;
    type LoginToken = u32;
    fn insert(&mut self, internal_id: &u32) -> Result<u32, AuthenticationError> {
        let id = self.internal_id;
        self.internal_id += 1;
        self.data.insert(id, Authentication::from(*internal_id));
        Ok(id)
    }

    fn remove(&mut self, _id: &u32) -> Result<(), AuthenticationError> {
        todo!()
    }

    fn get(&self, id: &u32) -> Option<&Authentication<u32>> {
        self.data.get(id)
    }

    fn get_mut(&mut self, id: &u32) -> Option<&mut Authentication<u32>> {
        self.data.get_mut(id)
    }
}

impl Default for Cache<u32, u32> {
    fn default() -> Self {
        Self {
            internal_id: 0,
            data: HashMap::new(),
        }
    }
}
