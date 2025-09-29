
//! Defines the `Value` enum, which represents any possible JSON value.
//!
//! This file also provides a comprehensive set of `From` trait implementations
//! to allow easy conversion from Rust primitive types into a `json::Value`.

use std::collections::HashMap;
use std::iter::FromIterator;

/// Represents any valid JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represents a JSON `null`.
    Null,
    /// Represents a JSON boolean (`true` or `false`).
    Bool(bool),
    /// Represents a JSON number. All numbers are stored as `f64`.
    Number(f64),
    /// Represents a JSON string.
    String(String),
    /// Represents a JSON array (a sequence of values).
    Array(Vec<Value>),
    /// Represents a JSON object (a collection of key-value pairs).
    Object(HashMap<String, Value>),
}

// Macro to implement `From` for all numeric types.
macro_rules! impl_from_num_for_value {
    ( $( $t:ty ),* ) => {
        $(
            impl From<$t> for Value {
                /// Converts a numeric type into a `Value::Number`.
                fn from(n: $t) -> Self {
                    Value::Number(n as f64)
                }
            }
        )*
    };
}

impl_from_num_for_value!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);

impl From<bool> for Value {
    /// Converts a boolean into a `Value::Bool`.
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<String> for Value {
    /// Converts a `String` into a `Value::String`.
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    /// Converts a `&str` into a `Value::String`.
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Converts a `Vec<T>` where `T` can be converted into a `Value`
    /// into a `Value::Array`.
    fn from(arr: Vec<T>) -> Self {
        Value::Array(arr.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<Value>> From<HashMap<K, V>> for Value {
    /// Converts a `HashMap<K, V>` where `K` can be converted into a `String`
    /// and `V` into a `Value` into a `Value::Object`.
    fn from(map: HashMap<K, V>) -> Self {
        Value::Object(
            map.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Value {
    /// Creates a `Value::Object` from an iterator of key-value pairs.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Value::Object(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

