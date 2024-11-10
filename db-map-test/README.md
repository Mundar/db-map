# DBMap: Testing

Any databse that is written for `DBMap` should run the standard test suite.
Since all databases that implement `DBMap` should behave the same, they should
all pass. If any type of database strays from the correct behavior, it defeats
the ability to write code for `DBMap` and easily switch between different
database implementations.

```rust
use db_map_test::impl_db_map_tests;
use db_map_btreemap::BTreeMapDB;

impl_db_map_tests! {
    let db = BTreeMapDB::open();
}
```

```rust
use db_map_test::impl_db_map_tests;
use db_map_lmdb::{LMDB, LMDBArgs};

impl_db_map_tests! {
    let db = {
        let temp_dir = tempfile::Builder::new()
            .prefix("lmdb_test_dir_")
            .rand_bytes(5)
            .tempdir().unwrap();
        LMDB::open(temp_dir.path(), db_name, LMDBArgs::default())
    };
}
```
