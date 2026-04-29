use crate::error::JsonError;

// TODO: Define your Token enum here
// Hint: You need variants for:
// LeftBrace, RightBrace, LeftBracket, RightBracket, Comma, Colon
// String(String), Number(f64), Boolean(bool), Null
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

// TODO: Implement your tokenize function here
pub fn tokenize(input: &str) -> Result<Vec<Token>, JsonError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();
    'main_loop: while let Some(c) = iter.next() {
        let token = match c {
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ch if ch.is_ascii_alphabetic() => {
                let mut word = String::from(ch);
                while let Some(next_ch) = iter.next_if(|&next_ch| next_ch.is_ascii_alphabetic()) {
                    word.push(next_ch);
                }
                match word.as_str() {
                    "true" => Token::Boolean(true),
                    "false" => Token::Boolean(false),
                    "null" => Token::Null,
                    // TODO: Once `tokenize` returns `Result<Vec<Token>, TokenizeError>`, replace
                    // this log-and-skip with `Err(TokenizeError::UnknownKeyword(other.into()))`.
                    other => {
                        eprintln!("Invalid JSON! Unknown keyword: {}", other);
                        continue;
                    }
                }
            }
            ch if ch.is_ascii_digit() || ch == '-' => {
                let mut num_str = String::from(ch);
                while let Some(next_ch) = iter
                    .next_if(|&next_ch| matches!(next_ch, '0'..='9' | '.' | 'E' | 'e' | '+' | '-'))
                {
                    num_str.push(next_ch);
                }
                // TODO: Once `tokenize` returns `Result<Vec<Token>, TokenizeError>`, replace
                // this log-and-skip with `Err(TokenizeError::InvalidNumber(...))` carrying
                // the bad string and the parse error.
                match num_str.parse::<f64>() {
                    Ok(num_parsed) => Token::Number(num_parsed),
                    Err(err) => {
                        eprintln!("could not parse number {} as f64: {}", num_str, err);
                        continue;
                    }
                }
            }
            '"' => {
                let mut content = String::new();
                loop {
                    // TODO: Once `tokenize` returns `Result<Vec<Token>, TokenizeError>`, replace
                    // this log-and-skip with `Err(TokenizeError::InvalidString(...))`.
                    match iter.next() {
                        Some(ch) => match ch {
                            '"' => break,
                            ch => content.push(ch),
                        },
                        None => {
                            eprintln!("unterminated string");
                            break 'main_loop; // EOF inside a string: abandon the partial content and stop tokenizing.
                        }
                    }
                }
                Token::String(content)
            }
            // TODO: Once `tokenize` returns `Result<Vec<Token>, TokenizeError>`, replace
            // this log-and-skip with `Err(TokenizeError::UnexpectedChar(ch))`.
            ch => {
                eprintln!("character skipped: {}", ch);
                continue;
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
