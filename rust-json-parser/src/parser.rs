use std::collections::HashMap;

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

    // return the current token and advance
    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current).cloned();
        self.current += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        let mut arr = Vec::new();
        if matches!(self.peek(), Some(Token::RightBracket)) {
            self.advance();
            return Ok(JsonValue::Array(arr));
        }
        loop {
            arr.push(self.parse_value()?);
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                    match self.peek() {
                        Some(t @ Token::RightBracket) => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "next array item".to_string(),
                                found: format!("{:?}", t),
                                position: self.current,
                            });
                        }
                        _ => continue,
                    }
                }
                Some(Token::RightBracket) => {
                    self.advance();
                    break;
                }
                Some(t) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "comma or closing bracket".to_string(),
                        found: format!("{:?}", t),
                        position: self.current,
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "comma or closing bracket".to_string(),
                        position: self.current,
                    });
                }
            }
        }
        Ok(JsonValue::Array(arr))
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        let mut obj = HashMap::new();
        if matches!(self.peek(), Some(Token::RightBrace)) {
            self.advance();
            return Ok(JsonValue::Object(obj));
        }
        loop {
            let key = self.expect_string_key()?;
            self.expect_colon()?;
            let value = self.parse_value()?;
            obj.insert(key, value);
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                    match self.peek() {
                        Some(t @ Token::RightBrace) => {
                            return Err(JsonError::UnexpectedToken {
                                expected: "next object item".to_string(),
                                found: format!("{:?}", t),
                                position: self.current,
                            });
                        }
                        _ => continue,
                    }
                }
                Some(Token::RightBrace) => {
                    self.advance();
                    break;
                }
                Some(t) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "comma or closing brace".to_string(),
                        found: format!("{:?}", t),
                        position: self.current,
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "comma or closing brace".to_string(),
                        position: self.current,
                    });
                }
            }
        }
        Ok(JsonValue::Object(obj))
    }

    pub fn parse(&mut self, input: &str) -> Result<JsonValue> {
        self.tokens = Tokenizer::new(input).tokenize()?;
        self.current = 0;
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
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
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::LeftBrace) => self.parse_object(),
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

    fn expect_string_key(&mut self) -> Result<String> {
        match self.advance() {
            Some(Token::String(key)) => Ok(key),
            Some(t) => Err(JsonError::UnexpectedToken {
                expected: "string as key for an object".to_string(),
                found: format!("{:?}", t),
                position: self.current,
            }),
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "string as key for an object".to_string(),
                position: self.current,
            }),
        }
    }

    fn expect_colon(&mut self) -> Result<()> {
        match self.advance() {
            Some(Token::Colon) => Ok(()),
            Some(t) => Err(JsonError::UnexpectedToken {
                expected: "colon as separator for an object".to_string(),
                found: format!("{:?}", t),
                position: self.current,
            }),
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "colon as separator for an object".to_string(),
                position: self.current,
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

    mod array_tests {
        use super::*;

        #[test]
        fn test_parse_empty_array() {
            let value = JsonParser::new().parse("[]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![]));
        }

        #[test]
        fn test_parse_array_single() {
            let value = JsonParser::new().parse("[1]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
        }

        #[test]
        fn test_parse_array_multiple() {
            let value = JsonParser::new().parse("[1, 2, 3]").unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_array_mixed_types() {
            let value = JsonParser::new()
                .parse(r#"[1, "two", true, null]"#)
                .unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::String("two".to_string()),
                JsonValue::Boolean(true),
                JsonValue::Null,
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_nested_arrays() {
            let value = JsonParser::new().parse("[[1, 2], [3, 4]]").unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
                JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_deeply_nested() {
            let value = JsonParser::new().parse("[[[1]]]").unwrap();
            let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
                JsonValue::Number(1.0),
            ])])]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_array_accessor() {
            let value = JsonParser::new().parse("[1, 2, 3]").unwrap();
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 3);
        }
    }

    mod object_tests {
        use super::*;
        use std::collections::HashMap;

        #[test]
        fn test_parse_empty_object() {
            let value = JsonParser::new().parse("{}").unwrap();
            assert_eq!(value, JsonValue::Object(HashMap::new()));
        }

        #[test]
        fn test_parse_object_single_key() {
            let value = JsonParser::new().parse(r#"{"key": "value"}"#).unwrap();
            let mut expected = HashMap::new();
            expected.insert("key".to_string(), JsonValue::String("value".to_string()));
            assert_eq!(value, JsonValue::Object(expected));
        }

        #[test]
        fn test_parse_object_multiple_keys() {
            let value = JsonParser::new()
                .parse(r#"{"name": "Alice", "age": 30}"#)
                .unwrap();
            if let JsonValue::Object(obj) = value {
                assert_eq!(
                    obj.get("name"),
                    Some(&JsonValue::String("Alice".to_string()))
                );
                assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_nested_object() {
            let value = JsonParser::new()
                .parse(r#"{"outer": {"inner": 1}}"#)
                .unwrap();
            if let JsonValue::Object(outer) = value {
                if let Some(JsonValue::Object(inner)) = outer.get("outer") {
                    assert_eq!(inner.get("inner"), Some(&JsonValue::Number(1.0)));
                } else {
                    panic!("Expected nested object");
                }
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_array_in_object() {
            let value = JsonParser::new().parse(r#"{"items": [1, 2, 3]}"#).unwrap();
            if let JsonValue::Object(obj) = value {
                if let Some(JsonValue::Array(arr)) = obj.get("items") {
                    assert_eq!(arr.len(), 3);
                } else {
                    panic!("Expected array");
                }
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_object_in_array() {
            let value = JsonParser::new().parse(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
            if let JsonValue::Array(arr) = value {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected array");
            }
        }

        #[test]
        fn test_object_accessor() {
            let value = JsonParser::new().parse(r#"{"name": "test"}"#).unwrap();
            let obj = value.as_object().unwrap();
            assert_eq!(obj.len(), 1);
        }

        #[test]
        fn test_object_get() {
            let value = JsonParser::new()
                .parse(r#"{"name": "Alice", "age": 30}"#)
                .unwrap();
            assert_eq!(
                value.get("name"),
                Some(&JsonValue::String("Alice".to_string()))
            );
            assert_eq!(value.get("missing"), None);
        }
    }
    mod error_tests {
        use super::*;

        #[test]
        fn test_error_unclosed_array() {
            let result = JsonParser::new().parse("[1, 2");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_unclosed_object() {
            let result = JsonParser::new().parse(r#"{"key": 1"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_array() {
            let result = JsonParser::new().parse("[1, 2,]");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_object() {
            let result = JsonParser::new().parse(r#"{"a": 1,}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_colon() {
            let result = JsonParser::new().parse(r#"{"key" 1}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_invalid_key() {
            let result = JsonParser::new().parse(r#"{123: "value"}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_array() {
            let result = JsonParser::new().parse("[1 2 3]");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_object() {
            let result = JsonParser::new().parse(r#"{"a": 1 "b": 2}"#);
            assert!(result.is_err());
        }
    }
}
