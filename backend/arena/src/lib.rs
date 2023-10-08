//! blazingly fast in-memory arena-based allocator
//!
//! ## limitations
//! ### not concurrently writable
//! Arena does not lock internally in any way  
//! For concurrent writes it has to be locked from the outside
//! ```rust
//! use std::sync::RwLock;
//! use arena::Arena;
//! let arena_lock: RwLock<Arena<u32>> = RwLock::new(Arena::new());
//! ```
//! ### unchecked access
//! To make it blazingly fast and create a nicer ergonomic there are no checks when accessing data
//! No error will be returned but Arena will panic instead on:
//! - reading removed `T`
//! - reading out of bound indices
//! ```should_panic
//! use arena::Arena;
//! let arena: Arena<u32> = Arena::new();
//! arena.get(1);
//! ```
//! ### Memory is not freed on remove
//! If `T` would be removed and freed from the Arena every index inside the Arena would shift. Thus every index used, by relying algorithms, would need to change.
//!
//! To counter this Arena does:
//! - reuse removed indices
//! - offer a way to compact its data [`Arena::compact()`]

use std::vec;

#[derive(Clone)]
pub struct Arena<T> {
    // Option is used for easier deletion and compaction
    // it has no runtime or memory overhead due to compiler magic :-)
    // size_of::<T>() == size_of::<Option<T>>()
    fields: Vec<Option<T>>,
    empty_fields: Vec<usize>,
}

impl<T> Arena<T> {
    /// Returns index of the added `T`
    ///
    /// will reuse previously removed indices
    #[inline]
    pub fn insert(&mut self, value: T) -> usize {
        match self.empty_fields.pop() {
            None => {
                self.fields.push(Some(value));
                // return last index
                self.fields.len() - 1
            }
            Some(reused_index) => {
                self.fields[reused_index] = Some(value);
                // return reused_index
                reused_index
            }
        }
    }

    /// Returns `&T` at the given index
    ///
    /// # Safety
    /// this will panic if:
    /// - the index is out of bounds
    /// - the underlying `T` has been removed
    #[inline]
    pub fn get(&self, index: usize) -> &T {
        self.fields[index]
            .as_ref()
            .expect("Cannot get non existing value")
    }

    /// Returns `&mut T` at the given index
    ///
    /// # Safety
    /// this will panic if:
    /// - the index is out of bounds
    /// - the underlying `T` has been removed
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.fields[index]
            .as_mut()
            .expect("Cannot get_mut non existing value")
    }

    /// Removes `T` at the given index and returns it
    ///
    /// Removing `T` does not change indices of other `T` in the backing storage.  
    /// The memory location will be reused with later [insertions][`Arena::insert()`] or can be [compacted][`Arena::compact()`].
    ///
    /// # Safety
    /// Accessing the index will panic if it had not been reused.
    ///
    /// this will panic if:
    /// - the index is out of bounds
    /// - the underlying `T` has been removed
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        // return T from the backing storage and fill the index with None
        let value = self.fields[index]
            .take()
            .expect("Cannot remove non existing value");

        self.empty_fields.push(index);
        value
    }

    /// Compacts the backing storage to free up unused memory  
    /// Returns a list of minimal index changes that had to be performed
    pub fn compact(&mut self) -> Vec<(usize, usize)> {
        let mut compactions = vec![];

        if self.empty_fields.is_empty() {
            return compactions;
        }

        // all data has been deleted
        if self.empty_fields.len() == self.fields.len() {
            self.fields.clear();
            self.empty_fields.clear();
            return compactions;
        }

        // sort_unstable is faster for usize but still correct
        self.empty_fields.sort_unstable();
        let mut empty_fields_counter = 0;

        // move backwards len-1 -> 0
        for index in (0..self.fields.len()).rev() {
            // we checked before if there is at least one deleted element
            let empty_field_index = self.empty_fields[empty_fields_counter];
            if index < empty_field_index {
                // all empty values are at the end or are already moved
                // truncate to index + 1 as truncate takes the new length
                self.fields.truncate(index + 1);
                self.empty_fields.clear();
                break;
            }

            // skip None values
            if self.fields[index].is_none() {
                continue;
            }

            // value at index is Some(T)
            compactions.push((index, empty_field_index));
            self.fields.swap(index, empty_field_index);
            empty_fields_counter += 1;
        }
        compactions
    }

    /// Returns a new empty Arena
    pub fn new() -> Self {
        Self {
            fields: vec![],
            empty_fields: vec![],
        }
    }
}

impl<T: Default> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::Arena;

    #[test]
    fn values_should_be_inserted() {
        let mut manager = Arena::<usize>::new();
        manager.insert(1);
        manager.insert(2);

        assert_eq!(2, manager.fields.len())
    }

    #[test]
    fn values_should_be_deleted() {
        let mut manager = Arena::<usize>::new();
        let index1 = manager.insert(1);
        let index2 = manager.insert(2);

        assert_eq!(2, manager.fields.len());

        manager.remove(index1);
        manager.remove(index2);

        assert_eq!(2, manager.fields.len());
        assert_eq!(2, manager.empty_fields.len());
    }

    #[test]
    fn deleted_values_should_be_reused() {
        let mut manager = Arena::<usize>::new();
        manager.insert(1);
        let index_to_delete = manager.insert(2);

        assert_eq!(2, manager.fields.len());

        manager.remove(index_to_delete);
        assert_eq!(1, manager.empty_fields.len());

        let reused_index = manager.insert(3);

        assert_eq!(2, manager.fields.len());
        assert_eq!(0, manager.empty_fields.len());
        assert_eq!(index_to_delete, reused_index);
        assert_eq!(3, *manager.get(reused_index));
    }

    #[test]
    fn compaction_should_return_empty_if_no_deletions() {
        let mut manager = Arena::<usize>::new();
        manager.insert(1);

        assert_eq!(1, manager.fields.len());
        assert_eq!(0, manager.empty_fields.len());

        let compactions = manager.compact();
        assert_eq!(0, compactions.len());

        // nothing has been done
        assert_eq!(1, manager.fields.len());
        assert_eq!(0, manager.empty_fields.len());
    }

    #[test]
    fn compaction_should_return_empty_if_all_fields_have_been_deleted() {
        let mut manager = Arena::<usize>::new();
        let index = manager.insert(1);
        manager.remove(index);

        assert_eq!(1, manager.fields.len());
        assert_eq!(1, manager.empty_fields.len());

        let compactions = manager.compact();
        assert_eq!(0, compactions.len());

        // all fields have been truncated
        assert_eq!(0, manager.fields.len());
        assert_eq!(0, manager.empty_fields.len());
    }

    #[test]
    fn compaction_should_return_empty_if_all_deletions_are_at_the_end() {
        let mut manager = Arena::<usize>::new();
        manager.insert(1);
        let index = manager.insert(2);
        manager.remove(index);

        assert_eq!(2, manager.fields.len());
        assert_eq!(1, manager.empty_fields.len());

        let compactions = manager.compact();
        assert_eq!(0, compactions.len());

        // all fields have been truncated
        assert_eq!(1, manager.fields.len());
        assert_eq!(0, manager.empty_fields.len());
    }

    #[test]
    fn compaction_should_return_correct_compactions() {
        let mut manager = Arena::<usize>::new();

        manager.insert(1);
        let index2 = manager.insert(2);
        let index3 = manager.insert(3);
        manager.insert(4);
        let index5 = manager.insert(5);

        manager.remove(index2);
        manager.remove(index3);
        manager.remove(index5);

        assert_eq!(5, manager.fields.len());
        assert_eq!(3, manager.empty_fields.len());

        let compactions = manager.compact();
        assert_eq!(1, compactions.len());
        assert_eq!((3, 1), compactions[0]);

        assert_eq!(vec![Some(1), Some(4)], manager.fields);

        // all fields have been truncated
        assert_eq!(2, manager.fields.len());
        assert_eq!(0, manager.empty_fields.len());
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 0 but the index is 0")]
    fn out_of_bounds_access_should_panic() {
        let manager = Arena::<usize>::new();
        manager.get(0);
    }

    #[test]
    #[should_panic(expected = "Cannot get non existing value")]
    fn removed_access_should_panic() {
        let mut manager = Arena::<usize>::new();
        let index = manager.insert(1);
        manager.remove(index);
        manager.get(index);
    }

    #[test]
    #[should_panic(expected = "Cannot get_mut non existing value")]
    fn removed_mut_access_should_panic() {
        let mut manager = Arena::<usize>::new();
        let index = manager.insert(1);
        manager.remove(index);
        manager.get_mut(index);
    }

    #[test]
    #[should_panic(expected = "Cannot remove non existing value")]
    fn remove_twice_should_panic() {
        let mut manager = Arena::<usize>::new();
        let index = manager.insert(1);
        manager.remove(index);
        manager.remove(index);
    }
}
