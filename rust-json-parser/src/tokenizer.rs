use crate::error::JsonError;

/// A single token produced by the [`Tokenizer`].
///
/// Tokens represent the structural and literal elements of a JSON document.
/// Most users do not interact with tokens directly — [`crate::JsonParser`]
/// handles tokenization internally.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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
    String(String),
    /// A JSON number, stored as `f64` per the [JSON spec](https://www.json.org/json-en.html).
    Number(f64),
    /// `true` or `false`.
    Boolean(bool),
    /// `null`.
    Null,
}

/// Converts a JSON input string into a sequence of [`Token`]s.
///
/// The tokenizer is used internally by [`crate::JsonParser`]. It handles
/// whitespace, string escapes (including `\uXXXX`), numbers, booleans, and null.
pub struct Tokenizer {
    input: Vec<char>,
    current: usize,
}

impl Tokenizer {
    /// Creates a new tokenizer for the given input string.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
        }
    }

    // === Helper Functions ===
    // look at current char without advancing
    fn peek(&self) -> Option<char> {
        self.input.get(self.current).copied()
    }

    // return current char, then move forward
    fn advance(&mut self) -> Option<char> {
        let token = self.input.get(self.current).copied();
        self.current += 1;
        token
    }

    // combined method that peeks and advances in one step to address the PR feedback
    fn advance_if(&mut self, predicate: impl Fn(char) -> bool) -> Option<char> {
        if self.peek().is_some_and(&predicate) {
            self.advance()
        } else {
            None
        }
    }

    // === tokenizer helper functions ===
    fn read_keyword(&mut self, ch: char, token_start: usize) -> Result<Token, JsonError> {
        let mut word = String::from(ch);

        while let Some(ch) = self.advance_if(|ch| ch.is_ascii_alphabetic()) {
            word.push(ch);
        }
        match word.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Err(JsonError::UnexpectedToken {
                expected: "valid keyword".to_string(),
                found: word,
                position: token_start,
            }),
        }
    }

    fn read_digit(&mut self, ch: char, token_start: usize) -> Result<Token, JsonError> {
        let mut num_str = String::with_capacity(20);
        num_str.push(ch);

        while let Some(ch) =
            self.advance_if(|ch| matches!(ch, '0'..='9' | '.' | 'E' | 'e' | '+' | '-'))
        {
            num_str.push(ch);
        }
        match num_str.parse::<f64>() {
            Ok(num_parsed) => Ok(Token::Number(num_parsed)),
            Err(_) => Err(JsonError::InvalidNumber {
                value: num_str,
                position: token_start,
            }),
        }
    }

    fn read_string(&mut self, token_start: usize) -> Result<Token, JsonError> {
        let mut content = String::new();
        loop {
            match self.advance() {
                Some('"') => break,
                Some('\\') => match self.advance() {
                    Some(ch) => match ch {
                        '"' | '\\' | '/' => content.push(ch),
                        'b' => content.push('\u{0008}'),
                        'f' => content.push('\u{000C}'),
                        'n' => content.push('\n'),
                        'r' => content.push('\r'),
                        't' => content.push('\t'),
                        'u' => match self.input.get(self.current..(self.current + 4)) {
                            Some(hex_chars) => {
                                // turn hex_chars into String
                                let hex_str: String = hex_chars.iter().collect();
                                // convert String to u32, returns a Result with wrong error type
                                // transform ParseIntError to JsonError, ? then propagates it
                                let code_point =
                                    u32::from_str_radix(&hex_str, 16).map_err(|_| {
                                        JsonError::InvalidUnicode {
                                            sequence: hex_str.clone(),  // need to clone because I use hex_str below
                                            position: self.current - 2, // includes the \ in the position
                                        }
                                    })?;
                                // create char from Unicode, returns Option
                                // ok_or is map_err equivalent for Option, creates Some to Ok and None to Err
                                let ch = char::from_u32(code_point).ok_or(
                                    JsonError::InvalidUnicode {
                                        sequence: hex_str,
                                        position: self.current - 2,
                                    },
                                )?;
                                self.current += 4;
                                content.push(ch);
                            }
                            None => {
                                let remaining: String = self.input[self.current..]
                                    .iter()
                                    .take_while(|ch| ch.is_ascii_hexdigit()) // only take the hex digits
                                    .collect();
                                return Err(JsonError::InvalidUnicode {
                                    sequence: remaining,
                                    position: self.current - 2,
                                });
                            }
                        },
                        ch => {
                            return Err(JsonError::InvalidEscape {
                                char: ch,
                                position: self.current - 2,
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
                Some(ch) => content.push(ch),
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "closing quote".to_string(),
                        position: token_start,
                    });
                }
            }
        }
        Ok(Token::String(content))
    }

    /// Consumes the input and returns the full list of tokens.
    ///
    /// # Errors
    ///
    /// Returns a [`JsonError`] if the input contains invalid tokens,
    /// bad escape sequences, or malformed numbers.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, JsonError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.advance() {
            let token_start = self.current - 1; // counter has already advanced but token is still the one before
            let token = match ch {
                ' ' | '\t' | '\n' | '\r' => continue,
                '{' => Ok(Token::LeftBrace),
                '}' => Ok(Token::RightBrace),
                '[' => Ok(Token::LeftBracket),
                ']' => Ok(Token::RightBracket),
                ',' => Ok(Token::Comma),
                ':' => Ok(Token::Colon),
                ch if ch.is_ascii_alphabetic() => self.read_keyword(ch, token_start),
                ch if ch.is_ascii_digit() || ch == '-' => self.read_digit(ch, token_start),
                '"' => self.read_string(token_start),
                _ => Err(JsonError::UnexpectedToken {
                    expected: "valid JSON token".to_string(),
                    found: ch.to_string(),
                    position: token_start,
                }),
            };
            tokens.push(token?);
        }
        Ok(tokens)
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
        // Internal state is private, so we test via tokenize()
    }

    #[test]
    fn test_tokenizer_multiple_tokens() {
        // Tests that a single tokenize() call handles multiple tokens
        // Note: Unlike Python iterators, calling tokenize() again on the same
        // instance would return empty - the input has been consumed.
        // Create a new Tokenizer instance if you need to parse new input.
        let mut tokenizer = Tokenizer::new("123 456");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
    }

    // === Basic Token Tests (from Week 1 - ensure they still pass) ===

    #[test]
    fn test_tokenize_number() {
        let mut tokenizer = Tokenizer::new("42");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_tokenize_negative_number() {
        let mut tokenizer = Tokenizer::new("-9.6");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(-9.6)]);
    }

    #[test]
    fn test_tokenize_literals() {
        let mut t1 = Tokenizer::new("true");
        assert_eq!(t1.tokenize().unwrap(), vec![Token::Boolean(true)]);

        let mut t2 = Tokenizer::new("false");
        assert_eq!(t2.tokenize().unwrap(), vec![Token::Boolean(false)]);

        let mut t3 = Tokenizer::new("null");
        assert_eq!(t3.tokenize().unwrap(), vec![Token::Null]);
    }

    #[test]
    fn test_tokenize_simple_string() {
        let mut tokenizer = Tokenizer::new(r#""hello""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string())]);
    }

    // === Escape Sequence Tests ===

    #[test]
    fn test_escape_newline() {
        let mut tokenizer = Tokenizer::new(r#""hello\nworld""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello\nworld".to_string())]);
    }

    #[test]
    fn test_escape_tab() {
        let mut tokenizer = Tokenizer::new(r#""col1\tcol2""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string())]);
    }

    #[test]
    fn test_escape_quote() {
        let mut tokenizer = Tokenizer::new(r#""say \"hello\"""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("say \"hello\"".to_string())]);
    }

    #[test]
    fn test_escape_backslash() {
        let mut tokenizer = Tokenizer::new(r#""path\\to\\file""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
    }

    #[test]
    fn test_escape_forward_slash() {
        let mut tokenizer = Tokenizer::new(r#""a\/b""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("a/b".to_string())]);
    }

    #[test]
    fn test_escape_carriage_return() {
        let mut tokenizer = Tokenizer::new(r#""line\r\n""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("line\r\n".to_string())]);
    }

    #[test]
    fn test_escape_backspace_formfeed() {
        let mut tokenizer = Tokenizer::new(r#""\b\f""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("\u{0008}\u{000C}".to_string())]);
    }

    #[test]
    fn test_multiple_escapes() {
        let mut tokenizer = Tokenizer::new(r#""a\nb\tc\"""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string())]);
    }

    // === Unicode Escape Tests ===

    #[test]
    fn test_unicode_escape_basic() {
        // \u0041 is 'A'
        let mut tokenizer = Tokenizer::new(r#""\u0041""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("A".to_string())]);
    }

    #[test]
    fn test_unicode_escape_multiple() {
        // \u0048\u0069 is "Hi"
        let mut tokenizer = Tokenizer::new(r#""\u0048\u0069""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("Hi".to_string())]);
    }

    #[test]
    fn test_unicode_escape_mixed() {
        // Mix of regular chars and unicode escapes
        let mut tokenizer = Tokenizer::new(r#""Hello \u0057orld""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("Hello World".to_string())]);
    }

    #[test]
    fn test_unicode_escape_lowercase() {
        // Lowercase hex digits should work too
        let mut tokenizer = Tokenizer::new(r#""\u004a""#);
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("J".to_string())]);
    }

    // === Error Tests ===

    #[test]
    fn test_invalid_escape_sequence() {
        let mut tokenizer = Tokenizer::new(r#""\q""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidEscape { .. })));
    }

    #[test]
    fn test_invalid_unicode_too_short() {
        let mut tokenizer = Tokenizer::new(r#""\u004""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_invalid_unicode_bad_hex() {
        let mut tokenizer = Tokenizer::new(r#""\u00GG""#);
        let result = tokenizer.tokenize();
        assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
    }

    #[test]
    fn test_unterminated_string_with_escape() {
        let mut tokenizer = Tokenizer::new(r#""hello\n"#);
        let result = tokenizer.tokenize();
        assert!(result.is_err());
    }
}
