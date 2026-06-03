//! A JSON parser library built from scratch in Rust.
//!
//! `rust_json_parser` provides a complete JSON parser that tokenizes and parses
//! JSON strings into a structured [`JsonValue`] type. It supports all JSON data types
//! including nested objects and arrays, with detailed error reporting via [`JsonError`].
//!
//! # Features
//!
//! - All six JSON types: objects, arrays, strings, numbers, booleans, and null
//! - Zero-copy parsing: strings without escape sequences borrow directly from the
//!   input rather than being copied, so a parsed [`JsonValue`] holds a
//!   [`Cow`](std::borrow::Cow) and borrows from the source string
//! - Unicode escape sequences (`\uXXXX`) within the Basic Multilingual Plane
//! - Detailed error messages with position information
//! - Pretty-printing with configurable indentation
//! - Python bindings via PyO3 (optional `python` feature)
//!
//! # Limitations
//!
//! - Surrogate-pair escapes (two consecutive `\u` escapes encoding one
//!   non-BMP character, as used for emoji) are not combined into a single code
//!   point; each half is rejected as
//!   [`InvalidUnicode`](JsonError::InvalidUnicode), so characters outside the
//!   Basic Multilingual Plane cannot be expressed via `\u` escapes.
//! - Only the first complete top-level value is validated. Any content after it
//!   is silently ignored rather than reported as an error.
//! - Object key order is not preserved. Objects are stored in an
//!   [`FxHashMap`](rustc_hash::FxHashMap), so iteration and serialization order
//!   are unspecified.
//!
//! # Quick Start
//!
//! ```rust
//! use rust_json_parser::{JsonParser, JsonValue};
//!
//! let value = JsonParser::new(r#"{"name": "Jochen", "age": 96}"#).parse().unwrap();
//!
//! assert_eq!(value.get("name"), Some(&JsonValue::String("Jochen".to_string().into())));
//! assert_eq!(value.get("age"), Some(&JsonValue::Number(96.0)));
//! ```

#![warn(missing_docs)]

mod error;
mod parser;
mod tokenizer;
mod value;

pub use error::JsonError;
pub use parser::JsonParser;
pub use tokenizer::{Token, Tokenizer};
pub use value::JsonValue;

/// Convenience alias for `std::result::Result<T, JsonError>`.
pub type Result<T> = std::result::Result<T, JsonError>;

#[cfg(feature = "python")]
mod python_bindings;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() -> Result<()> {
        // Test the full parsing pipeline
        assert_eq!(JsonParser::new("42").parse()?, JsonValue::Number(42.0));
        assert_eq!(JsonParser::new("true").parse()?, JsonValue::Boolean(true));
        assert_eq!(JsonParser::new("null").parse()?, JsonValue::Null);
        assert_eq!(
            JsonParser::new(r#""hello""#).parse()?,
            JsonValue::String("hello".to_string().into())
        );
        Ok(())
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate properly with correct details
        let result = JsonParser::new("@invalid@").parse();
        assert!(result.is_err());

        // Validate error details through pattern matching
        match result {
            Err(JsonError::UnexpectedToken {
                expected,
                found,
                position,
            }) => {
                assert_eq!(expected, "valid JSON token");
                assert_eq!(found, "@");
                assert_eq!(position, 0);
            }
            _ => panic!("Expected UnexpectedToken error"),
        }
    }
}
