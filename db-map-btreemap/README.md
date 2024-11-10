# DBMap: BTreeMapDB - A memory-only "Database" backed by `BTreeMap`.

This "database" is useful for testing. It never saves anything to persistant storage, and is
primarily intended for testing and prototyping. It is simply a `BTreeMap` with a `DBMap`
implementation.

The primary goal of the DBMap trait is to make it easy to access data in a
database. Since it is implemented as a trait, you can write code for one
supported database and use it with another. The `BTreeMapDB` is intended to be
used in testing code.

```rust
use db_map_trait::DBMap;
use db_map_btreemap::BTreeMapDB;

const KEY: &[u8] = &0x123456789ABCDEF0_u64.to_be_bytes();
const DATA_U128: u128 = 0xFEDCBA98765432108ACE13579BDF2460_u128;
const DATA: &[u8] = &DATA_U128.to_be_bytes();

// Open the database.
let db = BTreeMapDB::open();

assert!(db.get(KEY).unwrap().is_none());
db.insert(KEY, DATA).unwrap();
assert_eq!(db.get_map(KEY, |x| {
   let mut buf = [0_u8; 16];
   buf.copy_from_slice(x);
   u128::from_be_bytes(buf)
}).unwrap(), Some(DATA_U128));
```
