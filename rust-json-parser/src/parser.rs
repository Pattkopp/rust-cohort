use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

// Result type alias for convenience
type Result<T> = std::result::Result<T, JsonError>;

#[derive(Debug, PartialEq)]
pub struct JsonParser {
    tokens: Vec<Token>,
    current: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Result<Self> {
        let tokens = Tokenizer::new(input).tokenize()?;
        Ok(Self { tokens, current: 0 })
    }
    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current).cloned();
        self.current += 1;
        token
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    pub fn parse(&mut self) -> Result<JsonValue> {
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
        let parser = JsonParser::new("42");
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parser_creation_tokenize_error() {
        let parser = JsonParser::new(r#""\q""#); // Invalid escape
        assert!(parser.is_err());
    }

    // === Primitive Parsing Tests ===

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Number(42.0));
    }

    #[test]
    fn test_parse_negative_number() {
        let mut parser = JsonParser::new("-3.14").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_boolean_true() {
        let mut parser = JsonParser::new("true").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        let mut parser = JsonParser::new("false").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null").unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::Null);
    }

    #[test]
    fn test_parse_simple_string() {
        let mut parser = JsonParser::new(r#""hello""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("hello".to_string()));
    }

    // === Escape Sequence Integration Tests ===

    #[test]
    fn test_parse_string_with_newline() {
        let mut parser = JsonParser::new(r#""hello\nworld""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_string_with_tab() {
        let mut parser = JsonParser::new(r#""col1\tcol2""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
    }

    #[test]
    fn test_parse_string_with_quotes() {
        let mut parser = JsonParser::new(r#""say \"hi\"""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(value, JsonValue::String("Hello".to_string()));
    }

    #[test]
    fn test_parse_complex_escapes() {
        let mut parser = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#).unwrap();
        let value = parser.parse().unwrap();
        assert_eq!(
            value,
            JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
        );
    }

    // === Error Tests ===

    #[test]
    fn test_parse_empty_input() {
        let parser = JsonParser::new("");
        // Could fail at tokenization (no tokens) or parsing (empty token list)
        // Either is acceptable - just verify it's an error
        assert!(parser.is_err() || parser.unwrap().parse().is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let parser = JsonParser::new("   ");
        assert!(parser.is_err() || parser.unwrap().parse().is_err());
    }

    mod array_tests {
        use super::*;

        #[test]
        fn test_parse_empty_array() {
            let value = parse_json("[]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![]));
        }

        #[test]
        fn test_parse_array_single() {
            let value = parse_json("[1]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
        }

        #[test]
        fn test_parse_array_multiple() {
            let value = parse_json("[1, 2, 3]").unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_array_mixed_types() {
            let value = parse_json(r#"[1, "two", true, null]"#).unwrap();
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
            let value = parse_json("[[1, 2], [3, 4]]").unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
                JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_deeply_nested() {
            let value = parse_json("[[[1]]]").unwrap();
            let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
                JsonValue::Number(1.0),
            ])])]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_array_accessor() {
            let value = parse_json("[1, 2, 3]").unwrap();
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 3);
        }

        #[test]
        fn test_array_get_index() {
            let value = parse_json("[10, 20, 30]").unwrap();
            assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
            assert_eq!(value.get_index(5), None);
        }
    }

    mod object_tests {

        #[test]
        fn test_parse_empty_object() {
            let value = parse_json("{}").unwrap();
            assert_eq!(value, JsonValue::Object(HashMap::new()));
        }

        #[test]
        fn test_parse_object_single_key() {
            let value = parse_json(r#"{"key": "value"}"#).unwrap();
            let mut expected = HashMap::new();
            expected.insert("key".to_string(), JsonValue::String("value".to_string()));
            assert_eq!(value, JsonValue::Object(expected));
        }

        #[test]
        fn test_parse_object_multiple_keys() {
            let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
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
            let value = parse_json(r#"{"outer": {"inner": 1}}"#).unwrap();
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
            let value = parse_json(r#"{"items": [1, 2, 3]}"#).unwrap();
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
            let value = parse_json(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
            if let JsonValue::Array(arr) = value {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected array");
            }
        }

        #[test]
        fn test_object_accessor() {
            let value = parse_json(r#"{"name": "test"}"#).unwrap();
            let obj = value.as_object().unwrap();
            assert_eq!(obj.len(), 1);
        }

        #[test]
        fn test_object_get() {
            let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
            assert_eq!(
                value.get("name"),
                Some(&JsonValue::String("Alice".to_string()))
            );
            assert_eq!(value.get("missing"), None);
        }
    }
    mod error_tests {

        #[test]
        fn test_error_unclosed_array() {
            let result = parse_json("[1, 2");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_unclosed_object() {
            let result = parse_json(r#"{"key": 1"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_array() {
            let result = parse_json("[1, 2,]");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_object() {
            let result = parse_json(r#"{"a": 1,}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_colon() {
            let result = parse_json(r#"{"key" 1}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_invalid_key() {
            let result = parse_json(r#"{123: "value"}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_array() {
            let result = parse_json("[1 2 3]");
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_object() {
            let result = parse_json(r#"{"a": 1 "b": 2}"#);
            assert!(result.is_err());
        }
    }
}
