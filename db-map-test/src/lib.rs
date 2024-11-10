#![doc = include_str!("../README.md")]
#![forbid(future_incompatible)]
#![warn(missing_docs, missing_debug_implementations, bare_trait_objects)]

use db_key_macro::db_key;
use db_map_trait::DBMap;
use proptest::prelude::*;
use std::fmt::Debug;
type Result<T> = std::result::Result<T, TestCaseError>;

pub mod strategy;

pub use strategy::*;

/// A simple key used for test cases.
#[db_key]
pub struct TestKey {
    /// An identifier for the record.
    id: u32,
    /// An indexed value.
    index: u32,
}

/// A simple value used for test cases.
#[db_key]
pub struct TestValue {
    /// An unsigned 8-bit value.
    #[default = 0x12]
    byte: u8,
    /// An unsigned 16-bit value.
    #[default = 0x3456]
    word: u16,
    /// An unsigned 32-bit value.
    #[default = 0x789ABCDE]
    long: u32,
    /// An unsigned 64-bit value.
    #[default = 0xFEDCBA9876543210]
    quad: u64,
    /// An unsigned 128-bit value.
    #[default = 0x13579BDFECA864201F2E3D4C5B6A7988]
    octo: u128,
    /// An 9-byte array of unsigned 8-bit values.
    #[default = [b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O']]
    array: [u8; 9],
}

macro_rules! tt_to_concat_params {
    (()) => {""};
    (($min:literal, $max:literal)) => { concat!(stringify!($min), ", ", stringify!($max)) }
}

macro_rules! make_test_docs {
    ($func:ident, $test_params:tt) => {
        make_test_docs!{$func, $test_params, ()}
    };
    ($func:ident, ($($var:ident in $strat:ident$params:tt,)*)) => {
        make_test_docs!{$func, ($("" $var "" in $strat$params,)*), ()}
    };
    ($func:ident, ($($var:ident in $strat:ident$params:tt,)*), $extra_vars:tt) => {
        make_test_docs!{$func, ($("" $var "" in $strat$params,)*), $extra_vars}
    };
    ($func:ident, ($($pre:literal $var:ident $post:literal in $strat:ident$params:tt,)*), ($($extra_var:expr),*)) => {
        concat!(r#"
# Examples

```rust
use db_map_btreemap::BTreeMapDB;
use db_map_test::*;
use proptest::prelude::*;

proptest! {
    fn example_test("#,
        $(
            "\n        ", stringify!($var), " in ", stringify!($strat), "(",
            tt_to_concat_params!{$params}, "),",
        )*
        r#"
) {
        let db = BTreeMapDB::open();
        "#, stringify!($func), "(&db", $(", ", $pre, stringify!($var), $post, )*$(", ", stringify!($extra_var), )*r#")?;
    }
}

// Manually run the proptest.
example_test();
```"#)
    };
}

/// This is a simple test using one key and value. It tests get(), insert(), and remove().
#[doc = make_test_docs!{insert_test, (
    key in random_key(),
    value in random_value(),
)}]
pub fn insert_test<M, K, V>(db: &M, key: K, value: V) -> Result<()>
where
    M: DBMap,
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    prop_assert!(db.get(key.as_ref())?.is_none());
    db.insert(key.as_ref(), value.as_ref())?;
    prop_assert_eq!(db.get(key.as_ref())?, Some(value.as_ref().to_vec()));
    db.remove(key.as_ref())?;
    prop_assert!(db.get(key)?.is_none());
    Ok(())
}

/// Test the `Clone` expectations for the `DBMap` trait.
///
/// The `DBMap` type is expected to be able to be cloned and any cloned databse handle access the
/// same database as the original.
#[doc = make_test_docs!{clone_test, (
    keys in random_keys(2, 2),
    values in random_values(2, 2),
)}]
pub fn clone_test<M, K, V>(db: &M, keys: Vec<K>, values: Vec<V>) -> Result<()>
where
    M: DBMap,
    K: AsRef<[u8]> + Clone,
    V: AsRef<[u8]> + Clone,
{
    prop_assert_eq!(2, keys.len());
    prop_assert_eq!(2, values.len());
    let db_clone = db.clone();
    // Verify that the keys don't already exist.
    assert!(db.get(&keys[0])?.is_none());
    assert!(db.get(&keys[1])?.is_none());
    assert!(db_clone.get(&keys[0])?.is_none());
    assert!(db_clone.get(&keys[1])?.is_none());
    db.insert(&keys[0], &values[0])?;
    db_clone.insert(&keys[1], &values[1])?;
    assert_eq!(db_clone.get(&keys[0])?, Some(values[0].as_ref().to_vec()));
    Ok(())
}

/// This is a simple test using one key and value. It tests get(), insert(), and remove().
#[doc = make_test_docs!{get_test, (
    key in random_key(),
    value in random_value(),
)}]
pub fn get_test<M, K, V>(db: &M, key: K, value: V) -> Result<()>
where
    M: DBMap,
    K: AsRef<[u8]> + Clone,
    V: AsRef<[u8]> + Clone,
{
    prop_assert!(db.get(key.clone())?.is_none());
    db.insert(key.clone(), value.clone())?;
    prop_assert_eq!(db.get(key.clone())?, Some(value.as_ref().to_vec()));
    db.remove(key.clone())?;
    prop_assert!(db.get(key.clone())?.is_none());
    Ok(())
}

/// This is a simple test using one key and value. It tests get_map(), insert(), and remove().
#[doc = make_test_docs!{get_map_test, (
    key in test_key(),
    value in test_value(),
), (|v| TestValue::from(v))}]
pub fn get_map_test<M, K, V, F, O>(db: &M, key: K, value: V, f: F) -> Result<()>
where
    M: DBMap,
    K: AsRef<[u8]> + Clone,
    V: AsRef<[u8]> + Clone + PartialEq<O> + Debug,
    F: Fn(&[u8]) -> O,
    O: PartialEq<V> + Debug,
    Option<O>: PartialEq<Option<V>>,
{
    prop_assert!(db.get(key.clone())?.is_none());
    db.insert(key.clone(), value.clone())?;
    prop_assert_eq!(db.get_map(key.clone(), f)?, Some(value.clone()));
    db.remove(key.clone())?;
    prop_assert!(db.get(key.clone())?.is_none());
    Ok(())
}

/// This is a simple test using one key and several data items. It tests fetch_and_replace(), and remove().
#[doc = make_test_docs!{fetch_and_replace_test, (
    "" key "" in test_key(),
    "&" values "" in test_values(2, 6),
)}]
pub fn fetch_and_replace_test<M, K, V>(db: &M, key: K, values: &[V]) -> Result<()>
where
    M: DBMap,
    K: AsRef<[u8]> + Clone,
    V: AsRef<[u8]> + Clone,
{
    prop_assert!(db.get(key.clone())?.is_none());
    let mut last_insert: Option<Vec<u8>> = None;
    for value in values {
        prop_assert_eq!(db.fetch_and_replace(key.clone(), value.clone())?, last_insert);
        last_insert = Some(value.as_ref().to_vec());
        prop_assert_eq!(db.get(key.clone())?, Some(value.as_ref().to_vec()));
    }
    db.remove(key.clone())?;
    prop_assert!(db.get(key.clone())?.is_none());
    Ok(())
}

/// This is a simple test using one key and several data items. It tests fetch_and_replace_map(),
/// and remove().
#[doc = make_test_docs!{fetch_and_replace_map_test, (
    "" key "" in string_key(),
    "&" values "" in string_values(2, 6),
), (|v| String::from_utf8(v.to_vec()).unwrap())}]
pub fn fetch_and_replace_map_test<M, K, V, F, O>(db: &M, key: K, values: &[V], f: F) -> Result<()>
where
    M: DBMap,
    K: AsRef<[u8]> + Clone,
    V: AsRef<[u8]> + Clone + PartialEq<O> + Debug,
    F: Fn(&[u8]) -> O,
    O: PartialEq<V> + Debug,
    Option<O>: PartialEq<Option<V>>,
{
    prop_assert!(db.get_map(key.clone(), &f)?.is_none());
    let mut last_insert: Option<V> = None;
    for value in values {
        prop_assert_eq!(db.fetch_and_replace_map(key.clone(), value, &f)?, last_insert);
        last_insert = Some(value.clone());
        prop_assert_eq!(db.get_map(key.clone(), &f)?, Some(value.clone()));
    }
    db.remove(key.clone())?;
    prop_assert!(db.get_map(key.clone(), f)?.is_none());
    Ok(())
}

/// Macro that generates the standard test suite for implementations of the [`DBMap`] trait.
///
/// # Examples
///
/// ```rust
/// use db_map_test::impl_db_map_tests;
/// use db_map_btreemap::BTreeMapDB;
///
/// impl_db_map_tests! {
///     let db = BTreeMapDB::open();
/// }
/// ```
///
/// ```rust
/// use db_map_test::impl_db_map_tests;
/// use db_map_lmdb::{LMDB, LMDBArgs};
///
/// impl_db_map_tests! {
///     let db = {
///         let temp_dir = tempfile::Builder::new()
///             .prefix("lmdb_test_dir_")
///             .rand_bytes(5)
///             .tempdir().unwrap();
///         LMDB::open(temp_dir.path(), db_name, LMDBArgs::default())
///     };
/// }
/// ```
#[macro_export]
macro_rules! impl_db_map_tests {
    (let db = $let_db:expr;) => {
        mod db_map_tests {
            use super::*;
            use proptest::prelude::*;
            use db_map_test::*;

            proptest! {
                #[test]
                fn clone_random_data(
                    keys in random_keys(2, 2),
                    values in random_values(2, 2),
                ) {
                    let db = $let_db;
                    clone_test(&db, keys, values)?;
                }
            }

            proptest! {
                #[test]
                fn clone_test_data(
                    keys in test_keys(2, 2),
                    values in test_values(2, 2),
                ) {
                    let db = $let_db;
                    clone_test(&db, keys, values)?;
                }
            }

            proptest! {
                #[test]
                fn clone_string_data(
                    keys in string_keys(2, 2),
                    values in string_values(2, 2),
                ) {
                    let db = $let_db;
                    clone_test(&db, keys, values)?;
                }
            }

            proptest! {
                #[test]
                fn insert_random_data(
                    key in random_key(),
                    value in random_value(),
                ) {
                    let db = $let_db;
                    insert_test(&db, key, &value)?;
                }
            }

            proptest! {
                #[test]
                fn insert_test_data(
                    key in test_key(),
                    value in test_value(),
                ) {
                    let db = $let_db;
                    insert_test(&db, key, &value)?;
                }
            }

            proptest! {
                #[test]
                fn insert_string_data(
                    key in string_key(),
                    value in string_value(),
                ) {
                    let db = $let_db;
                    insert_test(&db, key, &value)?;
                }
            }

            proptest! {
                #[test]
                fn get_random_data(
                    key in random_key(),
                    value in random_value(),
                ) {
                    let db = $let_db;
                    get_test(&db, &key, &value)?;
                }
            }

            proptest! {
                #[test]
                fn fetch_and_replace_random_data(
                    key in random_key(),
                    values in random_values(2, 5),
                ) {
                    let db = $let_db;
                    fetch_and_replace_test(&db, &key, &values)?;
                }
            }

            proptest! {
                #[test]
                fn fetch_and_replace_test_data(
                    key in test_key(),
                    values in test_values(2, 5),
                ) {
                    let db = $let_db;
                    fetch_and_replace_test(&db, &key, &values)?;
                }
            }

            proptest! {
                #[test]
                fn fetch_and_replace_string_data(
                    key in string_key(),
                    values in string_values(2, 5),
                ) {
                    let db = $let_db;
                    fetch_and_replace_test(&db, &key, &values)?;
                }
            }

            proptest! {
                #[test]
                fn fetch_and_replace_map_test_data(
                    key in test_key(),
                    values in test_values(2, 5),
                ) {
                    let db = $let_db;
                    fetch_and_replace_map_test(&db, key, &values, |v| TestValue::from(v))?;
                }
            }

            proptest! {
                #[test]
                fn fetch_and_replace_map_string_data(
                    key in string_key(),
                    values in string_values(2, 5),
                ) {
                    let db = $let_db;
                    fetch_and_replace_map_test(&db, key, &values, |v| String::from_utf8(v.to_vec()).unwrap())?;
                }
            }
        }
    };
}
