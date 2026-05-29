//! A JSON parser library built from scratch in Rust.
//!
//! `rust_json_parser` provides a complete JSON parser that tokenizes and parses
//! JSON strings into a structured [`JsonValue`] type. It supports all JSON data types
//! including nested objects and arrays, with detailed error reporting via [`JsonError`].
//!
//! # Features
//!
//! - Full JSON spec support: objects, arrays, strings, numbers, booleans, and null
//! - Unicode escape sequences (`\uXXXX`)
//! - Detailed error messages with position information
//! - Pretty-printing with configurable indentation
//! - Python bindings via PyO3 (optional `python` feature)
//!
//! # Quick Start
//!
//! ```rust
//! use rust_json_parser::{JsonParser, JsonValue};
//!
//! let mut parser = JsonParser::new();
//! let value = parser.parse(r#"{"name": "Jochen", "age": 96}"#).unwrap();
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
        let mut parser = JsonParser::new();
        assert_eq!(parser.parse("42")?, JsonValue::Number(42.0));
        assert_eq!(parser.parse("true")?, JsonValue::Boolean(true));
        assert_eq!(parser.parse("null")?, JsonValue::Null);
        assert_eq!(
            parser.parse(r#""hello""#)?,
            JsonValue::String("hello".to_string().into())
        );
        Ok(())
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate properly with correct details
        let mut parser = JsonParser::new();
        let result = parser.parse("@invalid@");
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
