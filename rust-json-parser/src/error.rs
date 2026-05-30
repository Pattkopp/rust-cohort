use std::fmt;

/// Errors produced during JSON tokenization or parsing.
///
/// Every variant carries a `position` field indicating the byte offset in the
/// input where the error was detected. This makes error messages actionable —
/// the caller can point the user to the exact location of the problem.
///
/// `JsonError` implements [`std::fmt::Display`] and [`std::error::Error`],
/// so it integrates with Rust's standard error-handling ecosystem.
///
/// # Examples
///
/// ```rust
/// use rust_json_parser::{JsonParser, JsonError};
///
/// let mut parser = JsonParser::new();
/// let err = parser.parse("@").unwrap_err();
///
/// assert!(matches!(err, JsonError::UnexpectedToken { position: 0, .. }));
/// println!("{err}"); // "unexpected token at position 0: expected valid JSON token, found @"
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// The parser encountered a token it did not expect at this point in the grammar.
    ///
    /// For example, a number where a string key was required, or an invalid
    /// character like `@` at the start of input.
    UnexpectedToken {
        /// What the parser expected at this position.
        expected: String,
        /// What was actually found.
        found: String,
        /// Byte offset in the input.
        position: usize,
    },
    /// The input ended before the parser finished reading a value.
    ///
    /// Common causes: unclosed strings, arrays, or objects.
    UnexpectedEndOfInput {
        /// What the parser was still expecting when input ran out.
        expected: String,
        /// Byte offset where the unexpected end was detected.
        position: usize,
    },
    /// A numeric literal could not be parsed as a valid `f64`.
    ///
    /// Triggered by malformed numbers like `12.34.56` or `--5`.
    InvalidNumber {
        /// The raw text of the invalid number.
        value: String,
        /// Byte offset where the number started.
        position: usize,
    },
    /// An unrecognized escape sequence was found inside a string.
    ///
    /// JSON permits only `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`,
    /// and `\uXXXX`. Anything else (e.g. `\q`) produces this error.
    InvalidEscape {
        /// The byte after the backslash.
        char: u8,
        /// Byte offset of the backslash.
        position: usize,
    },
    /// A `\uXXXX` escape contained invalid or insufficient hex digits.
    ///
    /// Triggered when fewer than four hex digits follow `\u`, or when the
    /// digits do not form a valid Unicode code point.
    InvalidUnicode {
        /// The hex sequence that was found (may be fewer than 4 characters).
        sequence: String,
        /// Byte offset of the backslash.
        position: usize,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "unexpected token at position {}: expected {}, found {}",
                    position, expected, found
                )
            }
            JsonError::UnexpectedEndOfInput { expected, position } => {
                write!(
                    f,
                    "unexpected end of input at position {}: expected {}",
                    position, expected
                )
            }
            JsonError::InvalidNumber { value, position } => {
                write!(f, "invalid number {} at position {}", value, position)
            }
            JsonError::InvalidEscape { char, position } => {
                write!(
                    f,
                    "invalid escape character {} at position {}",
                    *char as char, position
                )
            }
            JsonError::InvalidUnicode { sequence, position } => {
                write!(f, "invalid Unicode {} at position {}", sequence, position)
            }
        }
    }
}

impl std::error::Error for JsonError {}

#[cfg(test)]
mod tests {
    use super::*;

    //  Week 2 Tests

    #[test]
    fn test_error_creation() {
        let error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "@".to_string(),
            position: 5,
        };

        // Error should be Debug-printable
        assert!(format!("{:?}", error).contains("UnexpectedToken"));
    }

    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".to_string(),
            found: "@".to_string(),
            position: 0,
        };

        let message = format!("{}", error);
        assert!(message.contains("position 0"));
        assert!(message.contains("valid JSON"));
        assert!(message.contains("@"));
    }

    #[test]
    fn test_error_variants() {
        let token_error = JsonError::UnexpectedToken {
            expected: "number".to_string(),
            found: "x".to_string(),
            position: 3,
        };

        let eof_error = JsonError::UnexpectedEndOfInput {
            expected: "closing quote".to_string(),
            position: 10,
        };

        let num_error = JsonError::InvalidNumber {
            value: "12.34.56".to_string(),
            position: 0,
        };

        // All variants should be Debug-printable
        assert!(
            format!("{:?}", token_error)
                .contains("expected: \"number\", found: \"x\", position: 3")
        );
        assert!(format!("{:?}", eof_error).contains("closing quote"));
        assert!(format!("{:?}", num_error).contains("12.34.56"));
    }

    // Week 3 Tests

    #[test]
    fn test_invalid_escape_display() {
        let err = JsonError::InvalidEscape {
            char: b'q',
            position: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("escape"));
        assert!(msg.contains("q"));
    }

    #[test]
    fn test_invalid_unicode_display() {
        let err = JsonError::InvalidUnicode {
            sequence: "00GG".to_string(),
            position: 3,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unicode") || msg.contains("Unicode"));
    }

    #[test]
    fn test_error_is_std_error() {
        let err = JsonError::InvalidEscape {
            char: b'x',
            position: 0,
        };
        let _: &dyn std::error::Error = &err; // Must implement Error trait
    }
}
