mod error;
mod parser;
mod tokenizer;
mod value;

pub use error::JsonError;
pub use parser::JsonParser;
pub use tokenizer::{Token, Tokenizer};
pub use value::JsonValue;

// Type alias for convenience
// Users can write Result<JsonValue> instead of std::result::Result<JsonValue, JsonError>
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
            JsonValue::String("hello".to_string())
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
