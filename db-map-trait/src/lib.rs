#![doc = include_str!("../README.md")]
#![forbid(future_incompatible)]
#![warn(missing_docs, missing_debug_implementations, bare_trait_objects)]

pub mod error;

pub use crate::{
    error::{Error, Result},
};

#[doc = include_str!("../README.md")]
pub trait DBMap: Clone {
    /// Get the data for a specified key.
    ///
    /// The `get` function always returns the data as an owned `Vec<u8>`. If you will be
    /// transforming the data into another format, consider using [`get_map`][DBMap::get_map], which uses a mapping
    /// function to output any format from the internal slice presented by the underlying database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// // Create an in-memory testing "database" based on a BTreeMap.
    /// let db = BTreeMapDB::open();
    ///
    /// const KEY: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// const DATA: [u8; u8::MAX as usize] = {
    ///     let mut data = [0_u8; u8::MAX as usize];
    ///     let mut i = 0;
    ///     while i < (u8::MAX as usize) { // while loops can be used in constant definition code.
    ///         data[i] = i as u8;
    ///         i += 1;
    ///     }
    ///     data
    /// };
    ///
    /// // Verify that the data doesn't start in the database.
    /// assert!(db.get(&KEY).unwrap().is_none());
    ///
    /// // Insert some data into the database.
    /// db.insert(&KEY, &DATA);
    ///
    /// // Verify that the data is now in that key.
    /// assert_eq!(db.get(&KEY).unwrap(), Some(DATA.to_vec()));
    /// ```
    fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Vec<u8>>>
    {
        self.get_map(key, |k| k.to_vec())
    }

    /// Get the data for a specified key and use a function to transform it into the proper format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// // Create an in-memory testing "database" based on a BTreeMap.
    /// let db = BTreeMapDB::open();
    ///
    /// const KEY: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// const DATA: u64 = 0x123456789ABCDEF0;
    ///
    /// // Verify that the data doesn't start in the database.
    /// assert!(db.get(&KEY).unwrap().is_none());
    ///
    /// // Insert some data into the database.
    /// db.insert(&KEY, DATA.to_le_bytes());
    ///
    /// // Verify that the data is now in that key.
    /// assert_eq!(db.get_map(&KEY, |d| {
    ///     let mut buf = [0u8; 8];
    ///     buf.copy_from_slice(d);
    ///     u64::from_le_bytes(buf)
    /// }).unwrap(), Some(DATA));
    /// ```
    fn get_map<K, F, T>(&self, key: K, mapper: F) -> Result<Option<T>>
        where
            K: AsRef<[u8]>,
            F: FnOnce(&[u8]) -> T;

    /// Insert data for a specified key into the database.
    ///
    /// This behaves differently than `BTreeMap::insert` in that it doesn't return the old value.
    /// This behavior is likely almost free for a BTreeMap, but for databases it may require an
    /// explicit `get` before writing the new data in addition allocating memory for returning the
    /// old data. Since most inserts don't care about what was in there before, there is an
    /// explicit `fetch_and_replace` (or `fetch_and_replace_map`) method for when you want the old
    /// data returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// // Create an in-memory testing "database" based on a BTreeMap.
    /// let db = BTreeMapDB::open();
    ///
    /// const KEY: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// const DATA_SIZE: usize = 26;
    /// const DATA: [u8; DATA_SIZE] = {
    ///     let mut data = [0_u8; DATA_SIZE];
    ///     let mut i = 0;
    ///     while i < DATA_SIZE { // while loops can be used in constant definition code.
    ///         data[i] = b'A' + (i as u8);
    ///         i += 1;
    ///     }
    ///     data
    /// };
    ///
    /// // Verify that the data doesn't start in the database.
    /// assert!(db.get(&KEY).unwrap().is_none());
    ///
    /// // Insert some data into the database.
    /// db.insert(&KEY, &DATA);
    ///
    /// // Verify that the data is now in that key.
    /// assert_eq!(db.get(&KEY).unwrap(), Some(DATA.to_vec()));
    /// ```
    fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, key: K, value: V) -> Result<()>;

    /// Insert data for a specified key into the database and return the old value as a `Vec<u8>`.
    ///
    /// This behaves like the normal `BTreeMap::insert` function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// // Create an in-memory testing "database" based on a BTreeMap.
    /// let db = BTreeMapDB::open();
    ///
    /// const KEY: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// const DATA_SIZE: usize = 26;
    /// const FIRST_DATA: [u8; DATA_SIZE] = {
    ///     let mut data = [0_u8; DATA_SIZE];
    ///     let mut i = 0;
    ///     while i < DATA_SIZE {
    ///         data[i] = b'A' + (i as u8);
    ///         i += 1;
    ///     }
    ///     data
    /// };
    /// const SECOND_DATA: [u8; DATA_SIZE] = {
    ///     let mut data = [0_u8; DATA_SIZE];
    ///     let mut i = 0;
    ///     while i < DATA_SIZE {
    ///         data[i] = b'Z' - (i as u8);
    ///         i += 1;
    ///     }
    ///     data
    /// };
    ///
    /// // Verify that the data doesn't start in the database.
    /// assert!(db.get(&KEY).unwrap().is_none());
    ///
    /// // Insert the first data into the database.
    /// assert_eq!(db.fetch_and_replace(&KEY, &FIRST_DATA).unwrap(), None);
    ///
    /// // Insert the second data into the database.
    /// assert_eq!(db.fetch_and_replace(&KEY, &SECOND_DATA).unwrap(), Some(FIRST_DATA.to_vec()));
    ///
    /// // Verify that the second data is now in that key.
    /// assert_eq!(db.get(&KEY).unwrap(), Some(SECOND_DATA.to_vec()));
    /// ```
    fn fetch_and_replace<K, V>(&self, key: K, value: V) -> Result<Option<Vec<u8>>>
        where
            K: AsRef<[u8]>,
            V: AsRef<[u8]>,
    {
        self.fetch_and_replace_map(key, value, |k| k.to_vec())
    }

    /// Insert data for a specified key into the database and use a function to transform the old
    /// value into an owned object.
    ///
    /// This behaves like the normal `BTreeMap::insert` function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// // Create an in-memory testing "database" based on a BTreeMap.
    /// let db = BTreeMapDB::open();
    ///
    /// fn slice_to_u64(slice: &[u8]) -> u64 {
    ///     let mut buf = [0u8; 8];
    ///     buf.copy_from_slice(slice);
    ///     u64::from_le_bytes(buf)
    /// }
    ///
    /// const KEY: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// const FIRST_DATA: u64 = 0x123456789ABCDEF0;
    /// const SECOND_DATA: u64 = 0xFEDCBA987654321;
    ///
    /// // Verify that the data doesn't start in the database.
    /// assert!(db.get(&KEY).unwrap().is_none());
    ///
    /// // Insert first data into the database.
    /// assert_eq!(
    ///     db.fetch_and_replace_map(&KEY, &FIRST_DATA.to_le_bytes(), slice_to_u64).unwrap(),
    ///     None);
    ///
    /// // Insert second data into the database.
    /// assert_eq!(
    ///     db.fetch_and_replace_map(&KEY, &SECOND_DATA.to_le_bytes(), slice_to_u64).unwrap(),
    ///     Some(FIRST_DATA));
    ///
    /// // Verify that the data is now in that key.
    /// assert_eq!(db.get_map(&KEY, slice_to_u64).unwrap(), Some(SECOND_DATA));
    /// ```
    fn fetch_and_replace_map<K, V, F, T>(&self, key: K, value: V, mapper: F) -> Result<Option<T>>
        where
            K: AsRef<[u8]>,
            V: AsRef<[u8]>,
            F: FnOnce(&[u8]) -> T;

    /// Remove data for a specified key from the database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_trait::DBMap;
    /// use db_map_btreemap::BTreeMapDB;
    ///
    /// // Create an in-memory testing "database" based on a BTreeMap.
    /// let db = BTreeMapDB::open();
    ///
    /// const KEY: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    /// const DATA_SIZE: usize = 26;
    /// const DATA: [u8; DATA_SIZE] = {
    ///     let mut data = [0_u8; DATA_SIZE];
    ///     let mut i = 0;
    ///     while i < DATA_SIZE { // while loops can be used in constant definition code.
    ///         data[i] = b'A' + (i as u8);
    ///         i += 1;
    ///     }
    ///     data
    /// };
    ///
    /// // Verify that the data doesn't start in the database.
    /// assert!(db.get(&KEY).unwrap().is_none());
    ///
    /// // Insert some data into the database.
    /// db.insert(&KEY, &DATA);
    ///
    /// // Verify that the data is now in that key.
    /// assert_eq!(db.get(&KEY).unwrap(), Some(DATA.to_vec()));
    ///
    /// // Remove that data from the database.
    /// db.remove(&KEY);
    ///
    /// // Verify that the data isn't in the database anymore.
    /// assert!(db.get(&KEY).unwrap().is_none());
    /// ```
    fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<()>;
}
