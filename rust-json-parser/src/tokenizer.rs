use crate::error::JsonError;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64), // https://www.json.org/json-en.html
    Boolean(bool),
    Null,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.char_indices().peekable();
    while let Some((pos, c)) = iter.next() {
        let token = match c {
            ' ' | '\t' | '\n' | '\r' => continue,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            c if c.is_ascii_alphabetic() => {
                let mut word = String::from(c);
                while let Some((_, next_ch)) =
                    iter.next_if(|&(_, next_ch)| next_ch.is_ascii_alphabetic())
                {
                    word.push(next_ch);
                }
                match word.as_str() {
                    "true" => Token::Boolean(true),
                    "false" => Token::Boolean(false),
                    "null" => Token::Null,
                    _ => {
                        return Err(JsonError::UnexpectedToken {
                            expected: "valid keyword".to_string(),
                            found: word,
                            position: pos,
                        });
                    }
                }
            }
            c if c.is_ascii_digit() || c == '-' => {
                let mut num_str = String::from(c);
                while let Some((_, next_ch)) = iter.next_if(|&(_, next_ch)| {
                    matches!(next_ch, '0'..='9' | '.' | 'E' | 'e' | '+' | '-')
                }) {
                    num_str.push(next_ch);
                }
                match num_str.parse::<f64>() {
                    Ok(num_parsed) => Token::Number(num_parsed),
                    Err(_) => {
                        return Err(JsonError::InvalidNumber {
                            value: num_str,
                            position: pos,
                        });
                    }
                }
            }
            '"' => {
                let mut content = String::new();
                loop {
                    match iter.next() {
                        Some((_, '"')) => break,
                        Some((_, c)) => content.push(c),
                        None => {
                            return Err(JsonError::UnexpectedEndOfInput {
                                expected: "closing quote".to_string(),
                                position: pos,
                            });
                        }
                    }
                }
                Token::String(content)
            }

            _ => {
                return Err(JsonError::UnexpectedToken {
                    expected: "valid JSON token".to_string(),
                    found: c.to_string(),
                    position: pos,
                });
            }
        };
        tokens.push(token);
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::JsonError;

    // Result type alias for cleaner test signatures
    type Result<T> = std::result::Result<T, JsonError>;

    // String boundary tests - verify inner vs outer quote handling

    #[test]
    fn test_empty_string() -> Result<()> {
        // Outer boundary: adjacent quotes with no inner content
        let tokens = tokenize(r#""""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_containing_json_special_chars() -> Result<()> {
        // Inner handling: JSON delimiters inside strings don't break tokenization
        let tokens = tokenize(r#""{key: value}""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_keyword_like_content() -> Result<()> {
        // Inner handling: "true", "false", "null" inside strings stay as string content
        let tokens = tokenize(r#""not true or false""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
        Ok(())
    }

    #[test]
    fn test_string_with_number_like_content() -> Result<()> {
        // Inner handling: numeric content inside strings doesn't become Number tokens
        let tokens = tokenize(r#""phone: 555-1234""#)?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
        Ok(())
    }

    // Number parsing tests

    #[test]
    fn test_negative_number() -> Result<()> {
        let tokens = tokenize("-42")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
        Ok(())
    }

    #[test]
    fn test_decimal_number() -> Result<()> {
        let tokens = tokenize("0.5")?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
        Ok(())
    }

    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let err = tokenize(".5").unwrap_err();
        assert!(matches!(
            err,
            JsonError::UnexpectedToken { position: 0, .. }
        ));
    }

    // Error position tests

    #[test]
    fn test_invalid_keyword_error_position_points_to_start() {
        let input = "   xyz";
        let result = tokenize(input);
        assert!(result.is_err());
        if let Err(JsonError::UnexpectedToken { position, .. }) = result {
            assert_eq!(
                position, 3,
                "error position should point to the start of 'xyz' (index 3), not past it"
            );
        } else {
            panic!("expected UnexpectedToken error");
        }
    }

    #[test]
    fn test_unterminated_string() {
        let err = tokenize(r#""missing end quote"#).unwrap_err();
        assert!(matches!(
            err,
            JsonError::UnexpectedEndOfInput { position: 0, .. }
        ));
    }
}
