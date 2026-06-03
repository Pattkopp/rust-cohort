use std::borrow::Cow;

use crate::error::JsonError;

/// A single token produced by the [`Tokenizer`].
///
/// Tokens represent the structural and literal elements of a JSON document.
/// Most users do not interact with tokens directly — [`crate::JsonParser`]
/// handles tokenization internally.
#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// A JSON string literal, with escape sequences resolved.
    String(Cow<'a, str>),
    /// A JSON number, stored as `f64` per the [JSON spec](https://www.json.org/json-en.html).
    Number(f64),
    /// `true` or `false`.
    Boolean(bool),
    /// `null`.
    Null,
}

/// A streaming tokenizer that turns a JSON input string into [`Token`]s.
///
/// `Tokenizer` implements [`Iterator`], yielding one `Result<Token, JsonError>`
/// at a time as it scans the input — it never builds a full token list. It
/// handles whitespace, string escapes (including `\uXXXX`), numbers, booleans,
/// and null. [`crate::JsonParser`] drives it internally via
/// [`Peekable`](std::iter::Peekable).
///
/// # Examples
///
/// ```rust
/// use rust_json_parser::Tokenizer;
///
/// // Pull tokens lazily, or collect them all (short-circuiting on first error).
/// let tokens: Vec<_> = Tokenizer::new("[1, true]").collect::<Result<_, _>>().unwrap();
/// assert_eq!(tokens.len(), 5);
/// ```
pub struct Tokenizer<'a> {
    input: &'a str,
    position: usize,
}

// Iterator trait
impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, JsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

// inherent impl block
impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    // === Helper Functions ===
    fn next_token(&mut self) -> Option<Result<Token<'a>, JsonError>> {
        while let Some(ch) = self.advance() {
            let token_start = self.position - 1;
            let token = match ch {
                b' ' | b'\t' | b'\n' | b'\r' => continue,
                b'{' => Ok(Token::LeftBrace),
                b'}' => Ok(Token::RightBrace),
                b'[' => Ok(Token::LeftBracket),
                b']' => Ok(Token::RightBracket),
                b',' => Ok(Token::Comma),
                b':' => Ok(Token::Colon),
                ch if ch.is_ascii_alphabetic() => self.read_keyword(token_start),
                ch if ch.is_ascii_digit() || ch == b'-' => self.read_digit(token_start),
                b'"' => self.read_string(token_start),
                _ => Err(JsonError::UnexpectedToken {
                    expected: "valid JSON token".to_string(),
                    found: (ch as char).to_string(),
                    position: token_start,
                }),
            };
            return Some(token);
        }
        None
    }

    // look at current char without advancing
    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.position).copied()
    }

    // return current char, then move forward
    fn advance(&mut self) -> Option<u8> {
        let token = self.input.as_bytes().get(self.position).copied();
        self.position += 1;
        token
    }

    // combined method that peeks and advances in one step to address the PR feedback
    fn advance_if(&mut self, predicate: impl Fn(u8) -> bool) -> Option<u8> {
        if self.peek().is_some_and(&predicate) {
            self.advance()
        } else {
            None
        }
    }

    // === tokenizer helper functions ===
    fn read_keyword(&mut self, token_start: usize) -> Result<Token<'a>, JsonError> {
        while self.advance_if(|ch| ch.is_ascii_alphabetic()).is_some() {}
        let keyword = &self.input[token_start..self.position];
        match keyword {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Err(JsonError::UnexpectedToken {
                expected: "valid keyword".to_string(),
                found: keyword.to_string(),
                position: token_start,
            }),
        }
    }

    fn read_digit(&mut self, token_start: usize) -> Result<Token<'a>, JsonError> {
        while self
            .advance_if(|ch| matches!(ch, b'0'..=b'9' | b'.' | b'E' | b'e' | b'+' | b'-'))
            .is_some()
        {}

        let number = &self.input[token_start..self.position];
        match number.parse::<f64>() {
            Ok(num_parsed) => Ok(Token::Number(num_parsed)),
            Err(_) => Err(JsonError::InvalidNumber {
                value: number.to_string(),
                position: token_start,
            }),
        }
    }

    fn read_string(&mut self, token_start: usize) -> Result<Token<'a>, JsonError> {
        let start = self.position;
        if let Some(special_offset) = self.input[start..].find(['\\', '"']) {
            let special_index = start + special_offset;
            let special_byte = self.input.as_bytes()[special_index];
            if special_byte == b'"' {
                self.position = special_index + 1;
                return Ok(Token::String(Cow::Borrowed(
                    &self.input[start..special_index],
                )));
            }
        }
        let mut content: String = String::with_capacity(64);
        loop {
            match self.advance() {
                Some(b'"') => break,
                Some(b'\\') => match self.advance() {
                    Some(ch) => match ch {
                        b'"' | b'\\' | b'/' => content.push(ch as char),
                        b'b' => content.push('\x08'),
                        b'f' => content.push('\x0C'),
                        b'n' => content.push('\n'),
                        b'r' => content.push('\r'),
                        b't' => content.push('\t'),
                        b'u' => match self.input.get(self.position..(self.position + 4)) {
                            Some(hex_chars) => {
                                // turn hex_chars into String
                                let hex_str: String = String::from(hex_chars);
                                // convert String to u32, returns a Result with wrong error type
                                // transform ParseIntError to JsonError, ? then propagates it
                                let code_point =
                                    u32::from_str_radix(&hex_str, 16).map_err(|_| {
                                        JsonError::InvalidUnicode {
                                            sequence: hex_str.clone(),   // need to clone because I use hex_str below
                                            position: self.position - 2, // includes the \ in the position
                                        }
                                    })?;
                                // create char from Unicode, returns Option
                                // ok_or is map_err equivalent for Option, creates Some to Ok and None to Err
                                let ch = char::from_u32(code_point).ok_or(
                                    JsonError::InvalidUnicode {
                                        sequence: hex_str,
                                        position: self.position - 2,
                                    },
                                )?;
                                self.position += 4;
                                content.push(ch)
                            }
                            None => {
                                let remaining: String = self.input[self.position..]
                                    .chars()
                                    .take_while(|ch| ch.is_ascii_hexdigit()) // only take the hex digits
                                    .collect();
                                return Err(JsonError::InvalidUnicode {
                                    sequence: remaining,
                                    position: self.position - 2,
                                });
                            }
                        },
                        ch => {
                            return Err(JsonError::InvalidEscape {
                                char: ch,
                                position: self.position - 2,
                            });
                        }
                    },
                    None => {
                        return Err(JsonError::UnexpectedEndOfInput {
                            expected: "valid escape char".to_string(),
                            position: token_start,
                        });
                    }
                },
                Some(_) => {
                    let literal_start = self.position - 1;
                    let literal_end = match self.input[literal_start..].find(['\\', '"']) {
                        Some(special_offset) => literal_start + special_offset,
                        None => self.input.len(),
                    };
                    content.push_str(&self.input[literal_start..literal_end]);
                    self.position = literal_end;
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "closing quote".to_string(),
                        position: token_start,
                    });
                }
            }
        }
        Ok(Token::String(Cow::Owned(content)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Struct Usage Tests ===

    #[test]
    fn test_tokenizer_struct_creation() {
        let _tokenizer = Tokenizer::new(r#""hello""#);
        // Tokenizer should be created without error
        // Internal state is private, so we exercise it via the iterator
    }

    #[test]
    fn test_tokenizer_multiple_tokens() {
        // Collecting the iterator yields every token in order.
        let tokens = Tokenizer::new("123 456")
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens.len(), 2);
    }

    // === Basic Token Tests (from Week 1 - ensure they still pass) ===

    #[test]
    fn test_tokenize_number() {
        let tokens = Tokenizer::new("42").collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_tokenize_negative_number() {
        let tokens = Tokenizer::new("-9.6")
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::Number(-9.6)]);
    }

    #[test]
    fn test_tokenize_literals() {
        assert_eq!(
            Tokenizer::new("true")
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            vec![Token::Boolean(true)]
        );

        assert_eq!(
            Tokenizer::new("false")
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            vec![Token::Boolean(false)]
        );

        assert_eq!(
            Tokenizer::new("null")
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            vec![Token::Null]
        );
    }

    #[test]
    fn test_tokenize_simple_string() {
        let tokens = Tokenizer::new(r#""hello""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string().into())]);
    }

    // === Escape Sequence Tests ===

    #[test]
    fn test_escape_newline() {
        let tokens = Tokenizer::new(r#""hello\nworld""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("hello\nworld".to_string().into())]
        );
    }

    #[test]
    fn test_escape_tab() {
        let tokens = Tokenizer::new(r#""col1\tcol2""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string().into())]);
    }

    #[test]
    fn test_escape_quote() {
        let tokens = Tokenizer::new(r#""say \"hello\"""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("say \"hello\"".to_string().into())]
        );
    }

    #[test]
    fn test_escape_backslash() {
        let tokens = Tokenizer::new(r#""path\\to\\file""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("path\\to\\file".to_string().into())]
        );
    }

    #[test]
    fn test_escape_forward_slash() {
        let tokens = Tokenizer::new(r#""a\/b""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("a/b".to_string().into())]);
    }

    #[test]
    fn test_escape_carriage_return() {
        let tokens = Tokenizer::new(r#""line\r\n""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("line\r\n".to_string().into())]);
    }

    #[test]
    fn test_escape_backspace_formfeed() {
        let tokens = Tokenizer::new(r#""\b\f""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("\u{0008}\u{000C}".to_string().into())]
        );
    }

    #[test]
    fn test_multiple_escapes() {
        let tokens = Tokenizer::new(r#""a\nb\tc\"""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string().into())]);
    }

    // === Unicode Escape Tests ===

    #[test]
    fn test_unicode_escape_basic() {
        // \u0041 is 'A'
        let tokens = Tokenizer::new(r#""\u0041""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("A".to_string().into())]);
    }

    #[test]
    fn test_unicode_escape_multiple() {
        // \u0048\u0069 is "Hi"
        let tokens = Tokenizer::new(r#""\u0048\u0069""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("Hi".to_string().into())]);
    }

    #[test]
    fn test_unicode_escape_mixed() {
        // Mix of regular chars and unicode escapes
        let tokens = Tokenizer::new(r#""Hello \u0057orld""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("Hello World".to_string().into())]
        );
    }

    #[test]
    fn test_unicode_escape_lowercase() {
        // Lowercase hex digits should work too
        let tokens = Tokenizer::new(r#""\u004a""#)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tokens, vec![Token::String("J".to_string().into())]);
    }

    // === Error Tests ===

    #[test]
    fn test_invalid_escape_sequence() {
        let result = Tokenizer::new(r#""\q""#).collect::<Result<Vec<Token>, _>>();
        assert!(matches!(result, Err(JsonError::InvalidEscape { .. })));
    }

    #[test]
    fn test_invalid_unicode_too_short() {
        let result = Tokenizer::new(r#""\u004""#).collect::<Result<Vec<Token>, _>>();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_invalid_unicode_bad_hex() {
        let result = Tokenizer::new(r#""\u00GG""#).collect::<Result<Vec<Token>, _>>();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let result = Tokenizer::new(r#""hello\n"#).collect::<Result<Vec<Token>, _>>();
        assert!(result.is_err());
    }
}
