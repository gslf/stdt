mod value;
mod parser;
mod serializer;

pub use value::Value;
pub use parser::{from_str, ParseError};

/// A macro to create a `json::Value` with a JSON-like syntax.
///
/// ## Usage
///
/// ### Literals
/// Create null, boolean, number, or string values.
///
/// ```
/// # use stdt::json;
/// let a = json!(null);
/// let b = json!(true);
/// let c = json!(123.45);
/// let d = json!("hello world");
/// ```
///
/// ### Arrays
/// Create an array from a list of values.
///
/// ```
/// # use stdt::json;
/// let arr = json!([1, "two", false, null, ["nested"]]);
/// ```
///
/// ### Objects
/// Create an object from key-value pairs. Keys must be string literals or expressions
/// that evaluate to a `String` or `&str`.
///
/// ```
/// # use stdt::json;
/// let obj = json!({
///     "name": "John Doe",
///     "age": 43,
///     "is_developer": true,
///     "phones": [
///         "+44 1234567",
///         "+44 2345678"
///     ]
/// });
/// ```
#[macro_export]
macro_rules! json {
    // Null literal
    (null) => {
        $crate::json::Value::Null
    };

    // Array literal
    ([ $( $element:tt ),* ]) => {
        $crate::json::Value::Array(vec![ $( $crate::json!($element) ),* ])
    };

    // Object literal
    ({ $( $key:literal : $value:tt ),* }) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert(String::from($key), $crate::json!($value));
            )*
            $crate::json::Value::Object(map)
        }
    };

    // Any other expression is converted into a Value.
    ($other:expr) => {
        $crate::json::Value::from($other)
    };
}
