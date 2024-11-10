#![doc = include_str!("../README.md")]
#![forbid(future_incompatible)]
#![warn(missing_docs, missing_debug_implementations, bare_trait_objects)]

use anyhow::Error;
pub use db_map_trait::{
    DBMap,
    Result,
};
use lmdb::{
    Environment,
    EnvironmentFlags,
    Database,
    DatabaseFlags,
    Transaction,
    Error as LMDBError,
    Result as LMDBResult,
    WriteFlags,
};
use lmdb_sys::mdb_mode_t;
use libc::{c_uint, size_t};
use std::{
    path::Path,
    sync::Arc,
};

/// Arguments sent to [LMDB::open] to define options when opening an LMDB database.
///
/// # Examples
///
/// ```rust
/// use db_map_trait::DBMap;
/// use db_map_lmdb::{LMDB, LMDBArgs};
/// use tempfile;
///
/// // Generate a random directory name for the environment for this test.
/// let temp_dir = tempfile::Builder::new()
///     .prefix("lmdb_test_dir_")
///     .rand_bytes(5)
///     .tempdir()
///     .unwrap();
/// // Open the lmdb default database for the environment.
/// let db = LMDB::open(temp_dir.path(), None, LMDBArgs{
///     max_dbs: Some(5),
///     max_readers: Some(10),
///     ..Default::default()
/// }).unwrap();
///
/// assert!(db.get(b"key").unwrap().is_none());
/// db.insert(b"key", b"value");
/// assert_eq!(db.get_map(b"key", |v| String::from_utf8(v.to_vec()).unwrap())
///     .unwrap().unwrap().as_str(), "value");
/// ```
#[derive(Clone, Debug, Default)]
pub struct LMDBArgs {
    /// Open an environment with the provided UNIX permissions.
    ///
    /// Source: [lmdb::EnvironmentBuilder::open_with_permissions].
    pub file_mode: Option<mdb_mode_t>,
    /// Sets the provided options in the environment.
    ///
    /// Source: [lmdb::EnvironmentBuilder::set_flags].
    pub env_flags: Option<EnvironmentFlags>,
    /// Sets the maximum number of named databases for the environment.
    ///
    /// This function is only needed if multiple databases will be used in the
    /// environment. Simpler applications that use the environment as a single
    /// unnamed database can ignore this option.
    ///
    /// Currently a moderate number of slots are cheap but a huge number gets
    /// expensive: 7-120 words per transaction, and every `Transaction::open_db`
    /// does a linear search of the opened slots.
    ///
    /// Source: [lmdb::EnvironmentBuilder::set_max_dbs].
    pub max_dbs: Option<c_uint>,
    /// Sets the maximum number of threads or reader slots for the environment.
    ///
    /// This defines the number of slots in the lock table that is used to track readers in the
    /// the environment. The default is 126. Starting a read-only transaction normally ties a lock
    /// table slot to the current thread until the environment closes or the thread exits. If
    /// `MDB_NOTLS` is in use, `Environment::open_txn` instead ties the slot to the `Transaction`
    /// object until it or the `Environment` object is destroyed.
    ///
    /// Source: [lmdb::EnvironmentBuilder::set_max_readers].
    pub max_readers: Option<c_uint>,
    /// Sets the size of the memory map to use for the environment.
    ///
    /// The size should be a multiple of the OS page size. The default is
    /// 1048576 bytes. The size of the memory map is also the maximum size
    /// of the database. The value should be chosen as large as possible,
    /// to accommodate future growth of the database. It may be increased at
    /// later times.
    ///
    /// Any attempt to set a size smaller than the space already consumed
    /// by the environment will be silently changed to the current size of the used space.
    ///
    /// Source: [lmdb::EnvironmentBuilder::set_map_size].
    pub map_size: Option<size_t>,
    // This was removed because all of the Database flags will break the expectations of this crate.
    // pub db_flags: Option<DatabaseFlags>,
}

#[doc = include_str!("../README.md")]
#[derive(Clone, Debug)]
pub struct LMDB {
    env: Arc<Environment>,
    db: Arc<Database>,
}

impl LMDB {
    /// Open an LMDB database for use with the [`DBMap`] trait.
    ///
    /// This implementation of the `DBMap` trait use the [`lmdb-rkv`][lmdb] crate.
    ///
    ///
    pub fn open(env_path: &Path, db_name: Option<&str>, lmdb_args: LMDBArgs) -> Result<LMDB> {
        Ok(Self::open_inner(env_path, db_name, lmdb_args)?)
    }

    /// The implementation for the `open` function.
    fn open_inner(env_path: &Path, db_name: Option<&str>, lmdb_args: LMDBArgs) -> anyhow::Result<LMDB> {
        let mut builder = Environment::new();
        if let Some(flags) = lmdb_args.env_flags {
            builder.set_flags(flags);
        }
        if let Some(max_dbs) = lmdb_args.max_dbs {
            builder.set_max_dbs(max_dbs);
        }
        if let Some(max_readers) = lmdb_args.max_readers {
            builder.set_max_readers(max_readers);
        }
        if let Some(map_size) = lmdb_args.map_size {
            builder.set_map_size(map_size);
        }
        let env = Arc::new(match lmdb_args.file_mode {
            None => builder.open(env_path)?,
            Some(mode) => builder.open_with_permissions(env_path, mode)?,
        });
        let db = Arc::new(env.create_db(db_name, DatabaseFlags::empty())?);
        Ok(Self {
            env,
            db,
        })
    }

    /// Open a read-only transaction.
    #[inline]
    fn begin_ro_txn<'env>(&'env self) -> LMDBResult<lmdb::RoTransaction<'env>> {
        self.env.begin_ro_txn()
    }

    /// Begin a read-write transaction.
    #[inline]
    fn begin_rw_txn<'env>(&'env self) -> LMDBResult<lmdb::RwTransaction<'env>> {
        self.env.begin_rw_txn()
    }
}

impl DBMap for LMDB {
    fn get_map<K, F, T>(&self, key: K, mapper: F) -> Result<Option<T>>
        where
            K: AsRef<[u8]>,
            F: FnOnce(&[u8]) -> T
    {
        let db = &self.db;
        let txn = self.begin_ro_txn().map_err(|e| Error::from(e))?;
        match txn.get(**db, &key) {
            Ok(result) => Ok(Some(mapper(result))),
            Err(LMDBError::NotFound) => Ok(None),
            Err(err) => Err(Error::from(err).into()),
        }
    }

    fn insert<K: AsRef<[u8]>, V: AsRef<[u8]>>(&self, key: K, value: V) -> Result<()> {
        let db = &self.db;
        let mut txn = self.begin_rw_txn().map_err(|e| Error::from(e))?;
        txn.put(**db, &key, &value, WriteFlags::empty()).map_err(|e| Error::from(e))?;
        txn.commit().map_err(|e| Error::from(e))?;
        Ok(())
    }

    fn fetch_and_replace_map<K, V, F, T>(&self, key: K, value: V, mapper: F) -> Result<Option<T>>
        where
            K: AsRef<[u8]>,
            V: AsRef<[u8]>,
            F: FnOnce(&[u8]) -> T
    {
        let db = &self.db;
        let mut txn = self.begin_rw_txn().map_err(|e| Error::from(e))?;
        let result = match txn.get(**db, &key) {
            Ok(result) => Some(mapper(result)),
            Err(LMDBError::NotFound) => None,
            Err(err) => { return Err(Error::from(err).into()) }
        };
        txn.put(**db, &key, &value, WriteFlags::empty()).map_err(|e| Error::from(e))?;
        txn.commit().map_err(|e| Error::from(e))?;
        Ok(result)
    }

    fn remove<K: AsRef<[u8]>>(&self, key: K) -> Result<()> {
        let db = &self.db;
        let mut txn = self.begin_rw_txn().map_err(|e| Error::from(e))?;
        txn.del(**db, &key, None).map_err(|e| Error::from(e))?;
        txn.commit().map_err(|e| Error::from(e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use db_map_test::impl_db_map_tests;

    fn open_temp_lmdb(db_name: Option<&str>) -> Result<LMDB> {
        let temp_dir = tempfile::Builder::new()
            .prefix("lmdb_test_dir_")
            .rand_bytes(5)
            .tempdir()?;
        LMDB::open(temp_dir.path(), db_name, LMDBArgs::default())
    }

    impl_db_map_tests! {
        let db = open_temp_lmdb(None).unwrap();
    }
}
