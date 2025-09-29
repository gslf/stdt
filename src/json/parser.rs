
//! Implements the JSON parsing logic.
//!
//! This module provides the `from_str` function, which serves as the public
//! entry point for parsing a JSON string into a `json::Value`. It defines
//! a `ParseError` enum for detailed error reporting and a `Parser` struct
//! that implements a recursive descent parser.

use super::value::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// An error that can occur during JSON parsing.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The input ended unexpectedly.
    UnexpectedEndOfInput,
    /// An unexpected character was found.
    UnexpectedToken(char),
    /// A string literal was not properly terminated.
    UnterminatedString,
    /// An invalid escape sequence was found in a string.
    InvalidEscapeSequence(char),
    /// A number was not in a valid format.
    InvalidNumber,
    /// A literal (true, false, null) was malformed.
    InvalidLiteral(String),
    /// Trailing characters were found after a valid JSON value.
    TrailingCharacters,
}

// By implementing the std::error::Error trait, ParseError becomes a type
// that integrates well with Rust's broader error handling ecosystem.
impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::UnexpectedToken(c) => write!(f, "Unexpected token: '{}'", c),
            ParseError::UnterminatedString => write!(f, "Unterminated string"),
            ParseError::InvalidEscapeSequence(c) => write!(f, "Invalid escape sequence: '\\{}'", c),
            ParseError::InvalidNumber => write!(f, "Invalid number format"),
            ParseError::InvalidLiteral(s) => write!(f, "Invalid literal: {}", s),
            ParseError::TrailingCharacters => write!(f, "Trailing characters after valid JSON"),
        }
    }
}

/// Parses a JSON string slice into a `Value`.
///
/// # Errors
///
/// Returns a `ParseError` if the input string is not valid JSON.
pub fn from_str(s: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(s);
    let value = parser.parse_value()?;
    parser.consume_whitespace();
    if parser.peek().is_some() {
        // If there's more content after a valid value, it's an error.
        Err(ParseError::TrailingCharacters)
    } else {
        Ok(value)
    }
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given input string.
    fn new(input: &'a str) -> Self {
        Parser {
            chars: input.chars().peekable(),
        }
    }

    /// Retrieves the next character from the input stream.
    fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    /// Peeks at the next character without consuming it.
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Consumes whitespace characters until a non-whitespace character is found.
    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    /// The main dispatch function for parsing any JSON value.
    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.consume_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some('t') | Some('f') | Some('n') => self.parse_literal(),
            Some(c) if c.is_digit(10) || *c == '-' => self.parse_number(),
            Some(&c) => Err(ParseError::UnexpectedToken(c)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    /// Parses a JSON string literal: "..."
    fn parse_string(&mut self) -> Result<Value, ParseError> {
        self.next(); // Consume opening '"'
        let mut s = String::new();
        while let Some(c) = self.next() {
            match c {
                '"' => return Ok(Value::String(s)),
                '\\' => {
                    let escaped = self.next().ok_or(ParseError::UnterminatedString)?;
                    match escaped {
                        '"' | '\\' | '/' => s.push(escaped),
                        'b' => s.push('\u{0008}'),
                        'f' => s.push('\u{000C}'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                hex.push(self.next().ok_or(ParseError::UnterminatedString)?);
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| ParseError::InvalidEscapeSequence('u'))?;
                            s.push(
                                std::char::from_u32(code)
                                    .ok_or(ParseError::InvalidEscapeSequence('u'))?,
                            );
                        }
                        _ => return Err(ParseError::InvalidEscapeSequence(escaped)),
                    }
                }
                _ => s.push(c),
            }
        }
        Err(ParseError::UnterminatedString)
    }

    /// Parses a JSON number (integer or float).
    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let mut num_str = String::new();
        if let Some(&'-') = self.peek() {
            num_str.push(self.next().unwrap());
        }

        while let Some(&c) = self.peek() {
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                num_str.push(self.next().unwrap());
            } else {
                break;
            }
        }
        num_str
            .parse::<f64>()
            .map(Value::Number)
            .map_err(|_| ParseError::InvalidNumber)
    }

    /// Parses a JSON array literal: [...]
    fn parse_array(&mut self) -> Result<Value, ParseError> {
        self.next(); // Consume '['
        let mut arr = Vec::new();
        self.consume_whitespace();
        if self.peek() == Some(&']') {
            self.next(); // Consume ']'
            return Ok(Value::Array(arr));
        }
        loop {
            arr.push(self.parse_value()?);
            self.consume_whitespace();
            match self.next() {
                Some(']') => return Ok(Value::Array(arr)),
                Some(',') => continue,
                Some(c) => return Err(ParseError::UnexpectedToken(c)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
    }

    /// Parses a JSON object literal: {...}
    fn parse_object(&mut self) -> Result<Value, ParseError> {
        self.next(); // Consume '{'
        let mut obj = HashMap::new();
        self.consume_whitespace();
        if self.peek() == Some(&'}') {
            self.next(); // Consume '}'
            return Ok(Value::Object(obj));
        }
        loop {
            let key = match self.parse_value()? {
                Value::String(s) => s,
                _ => return Err(ParseError::UnexpectedToken('"')), // Keys must be strings
            };

            self.consume_whitespace();
            if self.next() != Some(':') {
                return Err(ParseError::UnexpectedToken(':'));
            }

            let value = self.parse_value()?;
            obj.insert(key, value);

            self.consume_whitespace();
            match self.next() {
                Some('}') => return Ok(Value::Object(obj)),
                Some(',') => continue,
                Some(c) => return Err(ParseError::UnexpectedToken(c)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
    }

    /// Parses the literals: true, false, null.
    fn parse_literal(&mut self) -> Result<Value, ParseError> {
        let mut literal = String::new();
        while let Some(&c) = self.peek() {
            if c.is_alphabetic() {
                literal.push(self.next().unwrap());
            } else {
                break;
            }
        }
        match literal.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "null" => Ok(Value::Null),
            _ => Err(ParseError::InvalidLiteral(literal)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*; 
    use std::collections::HashMap;

    fn obj(pairs: &[(&str, Value)]) -> Value {
        let mut m = HashMap::new();
        for (k, v) in pairs {
            m.insert((*k).to_string(), v.clone());
        }
        Value::Object(m)
    }

    // --- Happy paths
    #[test]
    fn parses_string_basic() {
        let v = from_str(r#""hello""#).unwrap();
        assert_eq!(v, Value::String("hello".into()));
    }

    #[test]
    fn parses_string_with_escapes_and_unicode() {
        let v = from_str(r#""line\n\tquote:\" snow:\u2603""#).unwrap();
        assert_eq!(v, Value::String("line\n\tquote:\" snow:☃".into()));
    }

    #[test]
    fn parses_numbers_int_float_exp() {
        assert_eq!(from_str("0").unwrap(), Value::Number(0.0));
        assert_eq!(from_str("-42").unwrap(), Value::Number(-42.0));
        assert_eq!(from_str("3.1415").unwrap(), Value::Number(3.1415));
        assert_eq!(from_str("1e3").unwrap(), Value::Number(1000.0));
        assert_eq!(from_str("-2.5E-2").unwrap(), Value::Number(-0.025));
    }

    #[test]
    fn parses_arrays() {
        assert_eq!(from_str("[]").unwrap(), Value::Array(vec![]));
        let v = from_str(r#"[1, "x", true, null]"#).unwrap();
        assert_eq!(
            v,
            Value::Array(vec![
                Value::Number(1.0),
                Value::String("x".into()),
                Value::Bool(true),
                Value::Null
            ])
        );
    }

    #[test]
    fn parses_objects_simple_and_nested() {
        let v = from_str(r#"{"a":1,"b":"x","c":false}"#).unwrap();
        assert_eq!(
            v,
            obj(&[
                ("a", Value::Number(1.0)),
                ("b", Value::String("x".into())),
                ("c", Value::Bool(false))
            ])
        );

        let nested = from_str(r#"{"outer":{"inner":[1,2,3]}}"#).unwrap();
        assert_eq!(
            nested,
            obj(&[(
                "outer",
                obj(&[(
                    "inner",
                    Value::Array(vec![
                        Value::Number(1.0),
                        Value::Number(2.0),
                        Value::Number(3.0)
                    ])
                )])
            )])
        );
    }

    #[test]
    fn parses_literals_and_whitespace() {
        assert_eq!(from_str(" true ").unwrap(), Value::Bool(true));
        assert_eq!(from_str("\nfalse\t").unwrap(), Value::Bool(false));
        assert_eq!(from_str("  null  ").unwrap(), Value::Null);
    }

    // --- Common Errors
    #[test]
    fn error_trailing_characters() {
        let err = from_str("null 0").unwrap_err();
        assert_eq!(err, ParseError::TrailingCharacters);
    }

    #[test]
    fn error_unterminated_string() {
        let err = from_str(r#""unterminated"#).unwrap_err();
        assert_eq!(err, ParseError::UnterminatedString);
    }

    #[test]
    fn error_invalid_escape_sequence() {
        let err = from_str(r#""bad \q escape""#).unwrap_err();
        assert_eq!(err, ParseError::InvalidEscapeSequence('q'));
    }

    #[test]
    fn error_invalid_number() {
        let err = from_str("--1").unwrap_err();
        assert_eq!(err, ParseError::InvalidNumber);
    }

    #[test]
    fn error_invalid_literal() {
        let err = from_str("tru").unwrap_err();
        assert_eq!(err, ParseError::InvalidLiteral("tru".into()));
    }

    #[test]
    fn error_object_key_must_be_string() {
        let err = from_str("{1:2}").unwrap_err();
        assert_eq!(err, ParseError::UnexpectedToken('"'));
    }

    #[test]
    fn error_object_missing_colon() {
        let err = from_str(r#"{"a" 1}"#).unwrap_err();
        assert_eq!(err, ParseError::UnexpectedToken(':'));
    }

    #[test]
    fn error_array_missing_comma_or_closing() {
        let err = from_str(r#"[1 "a"]"#).unwrap_err();
        assert_eq!(err, ParseError::UnexpectedToken('"'));
    }
}
