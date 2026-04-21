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
pub fn tokenize(input: &str) -> Vec<Token> {
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
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_braces() {
        let tokens = tokenize("{}");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
    }

    #[test]
    fn test_simple_string() {
        let tokens = tokenize(r#""hello""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize("42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
    }

    #[test]
    fn test_tokenize_string() {
        let tokens = tokenize(r#""hello world""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_boolean_and_null() {
        let tokens = tokenize("true false null");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
    }

    #[test]
    fn test_simple_object() {
        let tokens = tokenize(r#"{"name": "Alice"}"#);
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
    }

    #[test]
    fn test_multiple_values() {
        let tokens = tokenize(r#"{"age": 30, "active": true}"#);

        // Verify we have the right tokens
        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
    }

    // String boundary tests - verify inner vs outer quote handling

    #[test]
    fn test_empty_string() {
        // Outer boundary: adjacent quotes with no inner content
        let tokens = tokenize(r#""""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
    }

    #[test]
    fn test_string_containing_json_special_chars() {
        // Inner handling: JSON delimiters inside strings don't break tokenization
        let tokens = tokenize(r#""{key: value}""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("{key: value}".to_string()));
    }

    #[test]
    fn test_string_with_keyword_like_content() {
        // Inner handling: "true", "false", "null" inside strings stay as string content
        let tokens = tokenize(r#""not true or false""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("not true or false".to_string()));
    }

    #[test]
    fn test_string_with_number_like_content() {
        // Inner handling: numeric content inside strings doesn't become Number tokens
        let tokens = tokenize(r#""phone: 555-1234""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("phone: 555-1234".to_string()));
    }

    // Number parsing tests

    #[test]
    fn test_negative_number() {
        let tokens = tokenize("-42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-42.0));
    }

    #[test]
    fn test_decimal_number() {
        let tokens = tokenize("0.5");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.5));
    }

    #[test]
    fn test_leading_decimal_not_a_number() {
        // .5 is invalid JSON - numbers must have leading digit (0.5 is valid)
        let tokens = tokenize(".5");
        // Should NOT be interpreted as 0.5
        assert!(!tokens.contains(&Token::Number(0.5)));
    }
}
