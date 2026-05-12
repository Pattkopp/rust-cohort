use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

// Result type alias for convenience
type Result<T> = std::result::Result<T, JsonError>;

#[derive(Debug, PartialEq, Default)]
pub struct JsonParser {
    tokens: Vec<Token>,
    current: usize,
}

impl JsonParser {
    pub fn new() -> Self {
        Self::default()
    }
    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current).cloned();
        self.current += 1;
        token
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    pub fn parse(&mut self, input: &str) -> Result<JsonValue> {
        self.tokens = Tokenizer::new(input).tokenize()?;
        self.current = 0;

        if self.is_at_end() {
            return Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.current,
            });
        };
        match self.advance() {
            Some(Token::Null) => Ok(JsonValue::Null),
            Some(Token::Boolean(b)) => Ok(JsonValue::Boolean(b)),
            Some(Token::Number(f)) => Ok(JsonValue::Number(f)),
            Some(Token::String(s)) => Ok(JsonValue::String(s)),
            Some(t) => Err(JsonError::UnexpectedToken {
                expected: "primitive JSON value".to_string(),
                found: format!("{:?}", t),
                position: self.current - 1,
            }),
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.current - 1,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Struct Usage Tests ===

    #[test]
    fn test_parser_creation() {
        let mut parser = JsonParser::new();
        let result = parser.parse("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_parse_tokenize_error() {
        let mut parser = JsonParser::new();
        let result = parser.parse(r#""\q""#); // Invalid escape
        assert!(result.is_err());
    }

    // === Primitive Parsing Tests ===

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new();
        let value = parser.parse("42").unwrap();
        assert_eq!(value, JsonValue::Number(42.0));
    }

    #[test]
    fn test_parse_negative_number() {
        let mut parser = JsonParser::new();
        let value = parser.parse("-3.14").unwrap();
        assert_eq!(value, JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_boolean_true() {
        let mut parser = JsonParser::new();
        let value = parser.parse("true").unwrap();
        assert_eq!(value, JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        let mut parser = JsonParser::new();
        let value = parser.parse("false").unwrap();
        assert_eq!(value, JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new();
        let value = parser.parse("null").unwrap();
        assert_eq!(value, JsonValue::Null);
    }

    #[test]
    fn test_parse_simple_string() {
        let mut parser = JsonParser::new();
        let value = parser.parse(r#""hello""#).unwrap();
        assert_eq!(value, JsonValue::String("hello".to_string()));
    }

    // === Escape Sequence Integration Tests ===

    #[test]
    fn test_parse_string_with_newline() {
        let mut parser = JsonParser::new();
        let value = parser.parse(r#""hello\nworld""#).unwrap();
        assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_string_with_tab() {
        let mut parser = JsonParser::new();
        let value = parser.parse(r#""col1\tcol2""#).unwrap();
        assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
    }

    #[test]
    fn test_parse_string_with_quotes() {
        let mut parser = JsonParser::new();
        let value = parser.parse(r#""say \"hi\"""#).unwrap();
        assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let mut parser = JsonParser::new();
        let value = parser.parse(r#""\u0048\u0065\u006c\u006c\u006f""#).unwrap();
        assert_eq!(value, JsonValue::String("Hello".to_string()));
    }

    #[test]
    fn test_parse_complex_escapes() {
        let mut parser = JsonParser::new();
        let value = parser.parse(r#""line1\nline2\t\"quoted\"\u0021""#).unwrap();
        assert_eq!(
            value,
            JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
        );
    }

    // === Error Tests ===

    #[test]
    fn test_parse_empty_input() {
        let mut parser = JsonParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let mut parser = JsonParser::new();
        let result = parser.parse("   ");
        assert!(result.is_err());
    }
}
