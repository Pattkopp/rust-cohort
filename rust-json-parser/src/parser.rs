use std::borrow::Cow;
use std::iter::Peekable;
use std::str;

use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::error::JsonError;
use crate::tokenizer::{Token, Tokenizer};
use crate::value::JsonValue;

// Result type alias for convenience
type Result<T> = std::result::Result<T, JsonError>;
type TokenResult<'a> = Result<Token<'a>>; // = std::result::Result<Token<'a>, JsonError>

/// A recursive descent JSON parser.
///
/// `JsonParser` reads tokens on demand from a [`Tokenizer`] and parses them
/// into a [`JsonValue`] tree in a single pass — no intermediate token
/// collection is built. Create one with [`JsonParser::new()`], passing the
/// input, then call [`parse()`](JsonParser::parse) once to consume the parser
/// and produce the value.
///
/// # Examples
///
/// ```rust
/// use rust_json_parser::{JsonParser, JsonValue};
///
/// let value = JsonParser::new(r#"[1, "two", true]"#).parse().unwrap();
///
/// assert_eq!(value.as_array().unwrap().len(), 3);
/// ```
pub struct JsonParser<'a> {
    tokens: Peekable<Tokenizer<'a>>,
    position: usize,
}

impl<'a> JsonParser<'a> {
    /// Creates a parser ready to read the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: Tokenizer::new(input).peekable(),
            position: 0,
        }
    }

    fn next_token(&mut self) -> Result<Token<'a>> {
        match self.advance() {
            Some(result) => result, // Result<Token, JsonError> — return as-is
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "a JSON token".to_string(),
                position: self.position,
            }),
        }
    }

    // return the current token and advance
    fn advance(&mut self) -> Option<TokenResult<'a>> {
        self.tokens.next().inspect(|_| self.position += 1)
    }

    fn peek(&mut self) -> Option<&TokenResult<'a>> {
        self.tokens.peek()
    }

    fn parse_array(&mut self) -> Result<JsonValue<'a>> {
        let mut arr = Vec::new();
        if matches!(self.peek(), Some(Ok(Token::RightBracket))) {
            self.advance();
            return Ok(JsonValue::Array(arr));
        }
        loop {
            arr.push(self.parse_value()?);
            match self.next_token()? {
                Token::Comma => continue,
                Token::RightBracket => break,
                t => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "comma or closing bracket".to_string(),
                        found: format!("{:?}", t),
                        position: self.position,
                    });
                }
            }
        }
        Ok(JsonValue::Array(arr))
    }

    fn parse_object(&mut self) -> Result<JsonValue<'a>> {
        let mut obj: FxHashMap<Cow<'a, str>, JsonValue> =
            FxHashMap::with_capacity_and_hasher(16, FxBuildHasher);
        if matches!(self.peek(), Some(Ok(Token::RightBrace))) {
            self.advance();
            return Ok(JsonValue::Object(obj));
        }
        loop {
            let key = self.expect_string_key()?;
            self.expect_colon()?;
            let value = self.parse_value()?;
            obj.insert(key, value);
            match self.next_token()? {
                Token::Comma => continue,
                Token::RightBrace => break,
                t => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "comma or closing brace".to_string(),
                        found: format!("{:?}", t),
                        position: self.position,
                    });
                }
            }
        }
        Ok(JsonValue::Object(obj))
    }

    /// Parses the input this parser was created with into a [`JsonValue`].
    ///
    /// This consumes the parser. Build a fresh [`JsonParser`] with
    /// [`JsonParser::new`] for each input, then call `parse` once.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_json_parser::{JsonParser, JsonValue};
    ///
    /// assert_eq!(JsonParser::new("42").parse().unwrap(), JsonValue::Number(42.0));
    /// assert_eq!(JsonParser::new("null").parse().unwrap(), JsonValue::Null);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`JsonError`](crate::JsonError) if the input is not valid JSON:
    ///
    /// - [`UnexpectedToken`](crate::JsonError::UnexpectedToken) — invalid or misplaced token
    /// - [`UnexpectedEndOfInput`](crate::JsonError::UnexpectedEndOfInput) — input ended too early
    /// - [`InvalidNumber`](crate::JsonError::InvalidNumber) — malformed numeric literal
    /// - [`InvalidEscape`](crate::JsonError::InvalidEscape) — unrecognized escape sequence
    /// - [`InvalidUnicode`](crate::JsonError::InvalidUnicode) — bad `\uXXXX` sequence
    pub fn parse(mut self) -> Result<JsonValue<'a>> {
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue<'a>> {
        let token = self.next_token()?;
        match token {
            Token::Null => Ok(JsonValue::Null),
            Token::Boolean(b) => Ok(JsonValue::Boolean(b)),
            Token::Number(f) => Ok(JsonValue::Number(f)),
            Token::String(s) => Ok(JsonValue::String(s)),
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_object(),
            t => Err(JsonError::UnexpectedToken {
                expected: "primitive JSON value".to_string(),
                found: format!("{:?}", t),
                position: self.position - 1,
            }),
        }
    }

    fn expect_string_key(&mut self) -> Result<Cow<'a, str>> {
        let token = self.next_token()?;
        match token {
            Token::String(key) => Ok(key),
            t => Err(JsonError::UnexpectedToken {
                expected: "string as key for an object".to_string(),
                found: format!("{:?}", t),
                position: self.position,
            }),
        }
    }

    fn expect_colon(&mut self) -> Result<()> {
        let token = self.next_token()?;
        match token {
            Token::Colon => Ok(()),
            t => Err(JsonError::UnexpectedToken {
                expected: "colon as separator for an object".to_string(),
                found: format!("{:?}", t),
                position: self.position,
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
        let result = JsonParser::new("42").parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_parse_tokenize_error() {
        let result = JsonParser::new(r#""\q""#).parse(); // Invalid escape
        assert!(result.is_err());
    }

    // === Primitive Parsing Tests ===

    #[test]
    fn test_parse_number() {
        let value = JsonParser::new("42").parse().unwrap();
        assert_eq!(value, JsonValue::Number(42.0));
    }

    #[test]
    fn test_parse_negative_number() {
        let value = JsonParser::new("-9.6").parse().unwrap();
        assert_eq!(value, JsonValue::Number(-9.6));
    }

    #[test]
    fn test_parse_boolean_true() {
        let value = JsonParser::new("true").parse().unwrap();
        assert_eq!(value, JsonValue::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        let value = JsonParser::new("false").parse().unwrap();
        assert_eq!(value, JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let value = JsonParser::new("null").parse().unwrap();
        assert_eq!(value, JsonValue::Null);
    }

    #[test]
    fn test_parse_simple_string() {
        let value = JsonParser::new(r#""hello""#).parse().unwrap();
        assert_eq!(value, JsonValue::String("hello".to_string().into()));
    }

    // === Escape Sequence Integration Tests ===

    #[test]
    fn test_parse_string_with_newline() {
        let value = JsonParser::new(r#""hello\nworld""#).parse().unwrap();
        assert_eq!(value, JsonValue::String("hello\nworld".to_string().into()));
    }

    #[test]
    fn test_parse_string_with_tab() {
        let value = JsonParser::new(r#""col1\tcol2""#).parse().unwrap();
        assert_eq!(value, JsonValue::String("col1\tcol2".to_string().into()));
    }

    #[test]
    fn test_parse_string_with_quotes() {
        let value = JsonParser::new(r#""say \"hi\"""#).parse().unwrap();
        assert_eq!(value, JsonValue::String("say \"hi\"".to_string().into()));
    }

    #[test]
    fn test_parse_string_with_unicode() {
        let value = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#).parse().unwrap();
        assert_eq!(value, JsonValue::String("Hello".to_string().into()));
    }

    #[test]
    fn test_parse_complex_escapes() {
        let value = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#).parse().unwrap();
        assert_eq!(
            value,
            JsonValue::String("line1\nline2\t\"quoted\"!".to_string().into())
        );
    }

    // === Error Tests ===

    #[test]
    fn test_parse_empty_input() {
        let result = JsonParser::new("").parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = JsonParser::new("   ").parse();
        assert!(result.is_err());
    }

    mod array_tests {
        use super::*;

        #[test]
        fn test_parse_empty_array() {
            let value = JsonParser::new("[]").parse().unwrap();
            assert_eq!(value, JsonValue::Array(vec![]));
        }

        #[test]
        fn test_parse_array_single() {
            let value = JsonParser::new("[1]").parse().unwrap();
            assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
        }

        #[test]
        fn test_parse_array_multiple() {
            let value = JsonParser::new("[1, 2, 3]").parse().unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_array_mixed_types() {
            let value = JsonParser::new(r#"[1, "two", true, null]"#).parse()
                .unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::String("two".to_string().into()),
                JsonValue::Boolean(true),
                JsonValue::Null,
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_nested_arrays() {
            let value = JsonParser::new("[[1, 2], [3, 4]]").parse().unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
                JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
            ]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_parse_deeply_nested() {
            let value = JsonParser::new("[[[1]]]").parse().unwrap();
            let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
                JsonValue::Number(1.0),
            ])])]);
            assert_eq!(value, expected);
        }

        #[test]
        fn test_array_accessor() {
            let value = JsonParser::new("[1, 2, 3]").parse().unwrap();
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 3);
        }
    }

    mod object_tests {
        use super::*;
        use rustc_hash::FxHashMap;

        #[test]
        fn test_parse_empty_object() {
            let value = JsonParser::new("{}").parse().unwrap();
            assert_eq!(value, JsonValue::Object(FxHashMap::default()));
        }

        #[test]
        fn test_parse_object_single_key() {
            let value = JsonParser::new(r#"{"key": "value"}"#).parse().unwrap();
            let mut expected = FxHashMap::default();
            expected.insert(
                Cow::Borrowed("key"),
                JsonValue::String("value".to_string().into()),
            );
            assert_eq!(value, JsonValue::Object(expected));
        }

        #[test]
        fn test_parse_object_multiple_keys() {
            let value = JsonParser::new(r#"{"name": "Alice", "age": 30}"#).parse()
                .unwrap();
            if let JsonValue::Object(obj) = value {
                assert_eq!(
                    obj.get("name"),
                    Some(&JsonValue::String("Alice".to_string().into()))
                );
                assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
            } else {
                panic!("Expected object");
            }
        }

        #[test]
        fn test_parse_nested_object() {
            let value = JsonParser::new(r#"{"outer": {"inner": 1}}"#).parse()
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
            let value = JsonParser::new(r#"{"items": [1, 2, 3]}"#).parse().unwrap();
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
            let value = JsonParser::new(r#"[{"a": 1}, {"b": 2}]"#).parse().unwrap();
            if let JsonValue::Array(arr) = value {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected array");
            }
        }

        #[test]
        fn test_object_accessor() {
            let value = JsonParser::new(r#"{"name": "test"}"#).parse().unwrap();
            let obj = value.as_object().unwrap();
            assert_eq!(obj.len(), 1);
        }

        #[test]
        fn test_object_get() {
            let value = JsonParser::new(r#"{"name": "Alice", "age": 30}"#).parse()
                .unwrap();
            assert_eq!(
                value.get("name"),
                Some(&JsonValue::String("Alice".to_string().into()))
            );
            assert_eq!(value.get("missing"), None);
        }
    }
    mod error_tests {
        use super::*;

        #[test]
        fn test_error_unclosed_array() {
            let result = JsonParser::new("[1, 2").parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_unclosed_object() {
            let result = JsonParser::new(r#"{"key": 1"#).parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_array() {
            let result = JsonParser::new("[1, 2,]").parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_trailing_comma_object() {
            let result = JsonParser::new(r#"{"a": 1,}"#).parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_colon() {
            let result = JsonParser::new(r#"{"key" 1}"#).parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_invalid_key() {
            let result = JsonParser::new(r#"{123: "value"}"#).parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_array() {
            let result = JsonParser::new("[1 2 3]").parse();
            assert!(result.is_err());
        }

        #[test]
        fn test_error_missing_comma_object() {
            let result = JsonParser::new(r#"{"a": 1 "b": 2}"#).parse();
            assert!(result.is_err());
        }
    }
}
