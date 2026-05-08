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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration() -> Result<()> {
        // Test the full parsing pipeline
        assert_eq!(JsonParser::new("42")?.parse()?, JsonValue::Number(42.0));
        assert_eq!(JsonParser::new("true")?.parse()?, JsonValue::Boolean(true));
        assert_eq!(JsonParser::new("null")?.parse()?, JsonValue::Null);
        assert_eq!(
            JsonParser::new(r#""hello""#)?.parse()?,
            JsonValue::String("hello".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_error_propagation() {
        // Test that errors propagate properly with correct details
        let result = JsonParser::new("@invalid@");
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
