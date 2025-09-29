//! Implements the serialization logic for `json::Value`.
//!
//! Serialization is handled by implementing the `std::fmt::Display` trait
//! for the `Value` enum. This allows any `Value` to be converted to a string
//! representation using methods like `to_string()` or by including it in
//! formatting macros like `format!` and `println!`.

use super::value::Value;
use std::fmt;

impl fmt::Display for Value {
    /// Formats a `Value` enum into its JSON string representation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => {
                if n.is_nan() || n.is_infinite() {
                    write!(f, "null") // JSON standard does not support NaN or Infinity
                } else {
                    write!(f, "{}", n)
                }
            },
            Value::String(s) => {
                write!(f, "\"")?;
                for char in s.chars() {
                    match char {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '/' => write!(f, "\\/")?,
                        '\u{0008}' => write!(f, "\\b")?,
                        '\u{000C}' => write!(f, "\\f")?,
                        '\n' => write!(f, "\\n")?,
                        '\r' => write!(f, "\\r")?,
                        '\t' => write!(f, "\\t")?,
                        // Handle control characters according to JSON spec
                        c if c >= '\u{0000}' && c <= '\u{001F}' => write!(f, "\\u{:04x}", c as u32)?,
                        c => write!(f, "{}", c)?,
                    }
                }
                write!(f, "\"")
            }
            Value::Array(arr) => {
                write!(f, "[")?;
                let mut first = true;
                for val in arr {
                    if !first {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", val)?;
                    first = false;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                // Note: HashMap iteration order is not guaranteed. 
                for (key, val) in obj {
                    if !first {
                        write!(f, ",")?;
                    }
                    // An object key is a JSON string, so we format it by wrapping in a Value::String
                    write!(f, "{}:{}", Value::String(key.clone()), val)?;
                    first = false;
                }
                write!(f, "}}")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn assert_contains_all(haystack: &str, needles: &[&str]) {
        for n in needles {
            assert!(
                haystack.contains(n),
                "expected `{}` in `{}`",
                n,
                haystack
            );
        }
    }

    #[test]
    fn null_displays_as_null() {
        assert_eq!(format!("{}", Value::Null), "null");
    }

    #[test]
    fn bool_displays_as_true_false() {
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Bool(false)), "false");
    }

    #[test]
    fn number_formats_and_specials_to_null() {
        assert_eq!(format!("{}", Value::Number(0.0)), "0");
        assert_eq!(format!("{}", Value::Number(-42.0)), "-42");
        
        let pi = format!("{}", Value::Number(3.14));
        assert!(pi == "3.14" || pi == "3.1400000000000001"); 

        assert_eq!(format!("{}", Value::Number(f64::NAN)), "null");
        assert_eq!(format!("{}", Value::Number(f64::INFINITY)), "null");
        assert_eq!(format!("{}", Value::Number(f64::NEG_INFINITY)), "null");
    }

    #[test]
    fn string_escapes_quotes_backslash_slash_and_controls() {
        let s = "\"\\/\u{0008}\u{000C}\n\r\t";
        let out = format!("{}", Value::String(s.into()));
        assert_eq!(out, "\"\\\"\\\\\\/\\b\\f\\n\\r\\t\"");
    }

    #[test]
    fn string_escapes_other_control_chars_as_unicode() {
        let s = "\u{0000}\u{001F}";
        let out = format!("{}", Value::String(s.into()));
        assert_eq!(out, "\"\\u0000\\u001f\"");
    }

    #[test]
    fn array_serializes_as_expected() {
        let v = Value::Array(vec![
            Value::String("a".into()),
            Value::Number(1.0),
            Value::Null,
            Value::Bool(true),
        ]);
        let out = format!("{}", v);
        assert_eq!(out, "[\"a\",1,null,true]");
    }

    #[test]
    fn empty_array_and_empty_object() {
        let v = Value::Array(vec![]);
        assert_eq!(format!("{}", v), "[]");

        let v = Value::Object(HashMap::new());
        assert_eq!(format!("{}", v), "{}");
    }

    #[test]
    fn object_contains_all_pairs_independent_of_order() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), Value::Number(1.0));
        m.insert("b".to_string(), Value::String("x".into()));
        let v = Value::Object(m);

        let out = format!("{}", v);

        assert!(out.starts_with('{') && out.ends_with('}'), "{}", out);
        assert!(out.contains(","), "manca la virgola tra coppie: {}", out);

        assert_contains_all(&out, &["\"a\":1", "\"b\":\"x\""]);
    }

    #[test]
    fn nested_structures_render_correctly() {
        let mut inner = HashMap::new();
        inner.insert(
            "k".into(),
            Value::Array(vec![Value::String("€/\"".into()), Value::Number(2.0)]),
        );

        let v = Value::Array(vec![Value::Object(inner), Value::Bool(false)]);
        let out = format!("{}", v);

        assert!(out.contains("\"k\""), "manca la chiave k: {}", out);
        assert!(out.contains("false"), "manca false: {}", out);
        assert!(out.contains("2"), "manca 2: {}", out);
        assert!(out.contains('€'), "manca il simbolo €: {}", out);
    }

    #[test]
    fn object_keys_are_rendered_as_json_strings() {
        let mut m = HashMap::new();
        m.insert("q\"w\\e".to_string(), Value::Bool(true));
        let v = Value::Object(m);
        let out = format!("{}", v);

        assert_contains_all(&out, &["{\"", "\\\"", "\\\\", "\":true}"]);
    }
}

