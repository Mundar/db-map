#![doc = include_str!("../README.md")]
#![forbid(future_incompatible)]
#![warn(missing_docs, missing_debug_implementations, bare_trait_objects)]

use parking_lot::Mutex;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::Arc,
};
use db_map_trait::{
    DBMap,
    Result,
};

#[doc = include_str!("../README.md")]
#[derive(Clone, Debug, Default)]
pub struct BTreeMapDB(Arc<Mutex<RefCell<BTreeMap<Vec<u8>, Vec<u8>>>>>);

impl BTreeMapDB {
    /// Open a `BTreeMapDB` "database".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// const KEY: &[u8] = &[0x13, 0x57, 0x9B, 0xDF, 0xEC, 0xA8, 0x64, 0x20];
    /// const DATA: &[u8] = &[0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10];
    ///
    /// let db = BTreeMapDB::open();
    ///
    /// db.insert(KEY, DATA).unwrap();
    /// assert_eq!(db.get_map(KEY, |x| {
    ///     let mut buf = [0_u8; 8];
    ///     buf.copy_from_slice(x);
    ///     u64::from_be_bytes(buf)
    /// }).unwrap(), Some(0xFEDCBA9876543210_u64));
    /// db.remove(KEY);
    /// assert!(db.get(KEY).unwrap().is_none());
    /// ```
    pub fn open() -> Self {
        Self::default()
    }
}

impl DBMap for BTreeMapDB {
    fn get_map<K, F, T>(&self, key: K, mapper: F) -> Result<Option<T>>
        where
            K: AsRef<[u8]>,
            F: FnOnce(&[u8]) -> T,
    {
        let map_lock = self.0.lock();
        let map = map_lock.borrow();
        Ok(map.get(key.as_ref()).map(|v| mapper(v)))
    }

    fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, key: K, value: V) -> Result<()> {
        let map_lock = self.0.lock();
        let mut map = map_lock.borrow_mut();
        map.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        Ok(())
    }

    fn fetch_and_replace<K, V>(&self, key: K, value: V) -> Result<Option<Vec<u8>>>
        where
            K: AsRef<[u8]>,
            V: AsRef<[u8]>,
    {
        let map_lock = self.0.lock();
        let mut map = map_lock.borrow_mut();
        Ok(map.insert(key.as_ref().to_vec(), value.as_ref().to_vec()))
    }

    fn fetch_and_replace_map<K, V, F, T>(&self, key: K, value: V, mapper: F) -> Result<Option<T>>
        where
            K: AsRef<[u8]>,
            V: AsRef<[u8]>,
            F: FnOnce(&[u8]) -> T
    {
        let map_lock = self.0.lock();
        let mut map = map_lock.borrow_mut();
        let prev_value = map.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        Ok(prev_value.map(|v| mapper(v.as_ref())))
    }

    fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<()> {
        let map_lock = self.0.lock();
        let mut map = map_lock.borrow_mut();
        map.remove(key.as_ref());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use db_map_test::impl_db_map_tests;

    impl_db_map_tests! {
        let db = BTreeMapDB::open();
    }
}
