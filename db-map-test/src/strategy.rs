//! # Proptest strategies used for db-map tests.
//!
//!

use super::{TestKey, TestKeyArgs, TestValue, TestValueArgs};
use proptest::prelude::*;
use std::collections::HashMap;

/// Maximum length of the random keys generated by `random_key()`.
pub const RANDOM_KEY_MAX: usize = 20;
/// Maximum length of the random value generated by `random_value()`.
pub const RANDOM_VALUE_MAX: usize = 100;

macro_rules! prop_compose_single_multi {
    (
        $doc_name:literal,
        $extra_use:literal,
        $doc_test:literal,
        $single_fn:ident,
        $multi_fn:ident,
        $single:ident,
        $multi:ident,
        $out_type:ty,
        $single_strat:expr
    ) => {
        prop_compose_single_multi!{
            $doc_name,
            $extra_use,
            $doc_test,
            $single_fn,
            $multi_fn,
            $single,
            $multi,
            $out_type,
            $single,
            $single_strat,
            { $single }
        }
    };
    (
        $doc_name:literal,
        $extra_use:literal,
        $doc_test:literal,
        $single_fn:ident,
        $multi_fn:ident,
        $single:ident,
        $multi:ident,
        $out_type:ty,
        $single_param:ident,
        $single_strat:expr,
        $single_code:tt
    ) => {
        prop_compose! {
            #[doc = concat!("A proptest strategy for generating a random ", $doc_name, " ",
                stringify!($single), ".", r#"

# Examples

```rust
use db_map_test::strategy::"#, stringify!($single_fn), ";", $extra_use, r#"
use proptest::prelude::*;

proptest! {
    fn "#, stringify!($single_fn), r#"_test("#, stringify!($single), r#" in "#,
        stringify!($single_fn), r#"()) {"#, $doc_test, r#"
    }
}

// Run the proptest.
"#, stringify!($single_fn), r#"_test();
```"#)]
            pub fn $single_fn()($single_param in $single_strat) -> $out_type
                $single_code
        }

        prop_compose! {
            #[doc = concat!("A proptest strategy for generating multiple random ", $doc_name, " ",
                stringify!($multi), ".", r#"

# Examples

```rust
use db_map_test::strategy::"#, stringify!($multi_fn), ";", $extra_use, r#"
use proptest::prelude::*;

proptest! {
    fn "#, stringify!($multi_fn), r#"_test("#, stringify!($multi), r#" in "#,
        stringify!($multi_fn), r#"(1, 10)) {
        for "#, stringify!($single), r#" in "#, stringify!($multi), r#" {"#, $doc_test, r#"
        }
    }
}

// Run the proptest.
"#, stringify!($multi_fn), r#"_test();
```"#)]
            pub fn $multi_fn(min: usize, max: usize)(
                $multi in proptest::collection::hash_set($single_fn(), min..=max)
            ) -> Vec<$out_type> {
                $multi.into_iter().collect()
            }
        }
    };
}

macro_rules! prop_compose_key_value {
    (
        $doc_key_name:literal,
        $extra_key_use:literal,
        $doc_key_test:literal,
        $single_key_fn:ident,
        $multi_key_fn:ident,
        $key_type:ty,
        $single_key_strat:expr,
        $doc_value_name:literal,
        $extra_value_use:literal,
        $doc_value_test:literal,
        $single_value_fn:ident,
        $multi_value_fn:ident,
        $value_type:ty,
        $single_value_strat:expr,
        $multi_key_and_value_fn:ident
    ) => {
        prop_compose_key_value!{
            $doc_key_name,
            $extra_key_use,
            $doc_key_test,
            $single_key_fn,
            $multi_key_fn,
            $key_type,
            key,
            $single_key_strat,
            { key },
            $doc_value_name,
            $extra_value_use,
            $doc_value_test,
            $single_value_fn,
            $multi_value_fn,
            $value_type,
            value,
            $single_value_strat,
            { value },
            $multi_key_and_value_fn
        }
    };
    (
        $doc_key_name:literal,
        $extra_key_use:literal,
        $doc_key_test:literal,
        $single_key_fn:ident,
        $multi_key_fn:ident,
        $key_type:ty,
        $single_key_param:ident,
        $single_key_strat:expr,
        $single_key_code:tt,
        $doc_value_name:literal,
        $extra_value_use:literal,
        $doc_value_test:literal,
        $single_value_fn:ident,
        $multi_value_fn:ident,
        $value_type:ty,
        $single_value_param:ident,
        $single_value_strat:expr,
        $single_value_code:tt,
        $multi_key_and_value_fn:ident
    ) => {
        prop_compose_single_multi!{
            $doc_key_name,
            $extra_key_use,
            $doc_key_test,
            $single_key_fn,
            $multi_key_fn,
            key,
            keys,
            $key_type,
            $single_key_param,
            $single_key_strat,
            $single_key_code
        }

        prop_compose_single_multi!{
            $doc_value_name,
            $extra_value_use,
            $doc_value_test,
            $single_value_fn,
            $multi_value_fn,
            value,
            values,
            $value_type,
            $single_value_param,
            $single_value_strat,
            $single_value_code
        }

        prop_compose! {
            #[doc = concat!("A proptest strategy for generating multiple random ", $doc_key_name,
                " keys and ", $doc_value_name, "values.", r#"

# Examples

```rust
use db_map_test::strategy::"#, stringify!($multi_key_and_value_fn), ";", $extra_key_use,
    $extra_value_use, r#"
use proptest::prelude::*;

proptest! {
    fn "#, stringify!($multi_key_and_value_fn), r#"_test(keys_and_values in "#,
        stringify!($multi_key_and_value_fn), r#"(1, 10)) {
        for (key, value) in keys_and_values {"#, $doc_key_test, $doc_value_test, r#"
        }
    }
}

// Run the proptest.
"#, stringify!($multi_key_and_value_fn), r#"_test();
```"#)]
            pub fn $multi_key_and_value_fn(min: usize, max: usize)(
                map in proptest::collection::hash_map($single_key_fn(), $single_value_fn(), min..=max)
            ) -> HashMap<$key_type, $value_type> {
                map
            }
        }
    };
}

prop_compose_key_value!{
    "length", r#"
use db_map_test::RANDOM_KEY_MAX;"#, r#"
            prop_assert!(key.len() > 0);
            prop_assert!(key.len() <= RANDOM_KEY_MAX);"#,
    random_key, random_keys, Vec<u8>,
    proptest::collection::vec(0..=u8::MAX, 1..RANDOM_KEY_MAX),
    "length", r#"
use db_map_test::RANDOM_VALUE_MAX;"#, r#"
            prop_assert!(value.len() > 0);
            prop_assert!(value.len() <= RANDOM_VALUE_MAX);"#,
    random_value, random_values, Vec<u8>,
    proptest::collection::vec(0..=u8::MAX, 1..RANDOM_VALUE_MAX),
    random_keys_and_values
}

prop_compose_key_value!{
    "string", r#"
use db_map_test::RANDOM_KEY_MAX;"#, r#"
            prop_assert!(key.len() > 0);
            prop_assert!(key.len() <= RANDOM_KEY_MAX);"#,
    string_key, string_keys, String,
    s, "\\PC{1, 20}",
    {
        let mut vec = s.into_bytes();
        vec.truncate(RANDOM_KEY_MAX);
        match std::str::from_utf8(&vec) {
            Ok(_) => String::from_utf8(vec).unwrap(),
            Err(err) => {
                vec.truncate(err.valid_up_to());
                String::from_utf8(vec).unwrap()
            }
        }
    },
    "string", r#"
use db_map_test::RANDOM_VALUE_MAX;"#, r#"
            prop_assert!(value.len() > 0);
            prop_assert!(value.len() <= RANDOM_VALUE_MAX);"#,
    string_value, string_values, String,
    s, "\\PC{1, 50}",
    {
        let mut vec = s.into_bytes();
        vec.truncate(RANDOM_VALUE_MAX);
        match std::str::from_utf8(&vec) {
            Ok(_) => String::from_utf8(vec).unwrap(),
            Err(err) => {
                vec.truncate(err.valid_up_to());
                String::from_utf8(vec).unwrap()
            }
        }
    },
    string_keys_and_values
}

prop_compose! {
    /// A proptest strategy for generating random test key arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_test::{*, strategy::*};
    /// use proptest::prelude::*;
    ///
    /// proptest! {
    ///     fn test_value_args_test(
    ///         args in test_value_args(),
    ///     ) {
    ///         let value = TestValue::from(args);
    ///         assert_eq!(args.byte, value.byte());
    ///         assert_eq!(args.word, value.word());
    ///         assert_eq!(args.long, value.long());
    ///         assert_eq!(args.quad, value.quad());
    ///         assert_eq!(args.octo, value.octo());
    ///         assert_eq!(&args.array, value.array());
    ///     }
    /// }
    ///
    /// // Manually run the proptest.
    /// test_value_args_test();
    /// ```
    pub fn test_key_args()(id in 0..=u32::MAX, index in 0..=u32::MAX) -> TestKeyArgs {
        TestKeyArgs {
            id,
            index,
        }
    }
}

prop_compose! {
    /// A proptest strategy for generating random test value arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use db_map_test::*;
    /// use proptest::prelude::*;
    ///
    /// proptest! {
    ///     fn test_value_args_test(
    ///         args in test_value_args(),
    ///     ) {
    ///         let value = TestValue::from(args);
    ///         assert_eq!(args.byte, value.byte());
    ///         assert_eq!(args.word, value.word());
    ///         assert_eq!(args.long, value.long());
    ///         assert_eq!(args.quad, value.quad());
    ///         assert_eq!(args.octo, value.octo());
    ///         assert_eq!(&args.array, value.array());
    ///     }
    /// }
    ///
    /// // Manually run the proptest.
    /// test_value_args_test();
    /// ```
    pub fn test_value_args()(
        byte in 0..=u8::MAX,
        word in 0..=u16::MAX,
        long in 0..=u32::MAX,
        quad in 0..=u64::MAX,
        octo in 0..=u128::MAX,
        array_vec in proptest::collection::vec(0..u8::MAX, 9..=9),
    ) -> TestValueArgs {
        let mut array = [0_u8; 9];
        array.copy_from_slice(&array_vec);
        TestValueArgs {
            byte,
            word,
            long,
            quad,
            octo,
            array,
        }
    }
}

prop_compose_key_value!{
    "TestKey", r#"
use db_map_test::TestKey;"#, r#"
            prop_assert_eq!(key.as_ref().len(), TestKey::KEY_LENGTH);"#,
    test_key, test_keys, TestKey,
    args, test_key_args(),
    { TestKey::from(args) },
    "TestValue", r#"
use db_map_test::TestValue;"#, r#"
            prop_assert_eq!(value.as_ref().len(), TestValue::KEY_LENGTH);"#,
    test_value, test_values, TestValue,
    args, test_value_args(),
    { TestValue::from(args) },
    test_keys_and_values
}
