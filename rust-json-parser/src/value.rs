use std::{borrow::Cow, fmt};

use rustc_hash::FxHashMap;

/// Represents a JSON value.
///
/// Each variant corresponds to one of the six JSON data types defined in
/// [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259).
/// Use [`crate::JsonParser`] to produce a `JsonValue` from a JSON string,
/// then inspect or extract data using the accessor methods below.
///
/// # Examples
///
/// ```rust
/// use rust_json_parser::{JsonParser, JsonValue};
///
/// let mut parser = JsonParser::new();
///
/// // Each JSON type maps to a JsonValue variant
/// assert_eq!(parser.parse("null").unwrap(), JsonValue::Null);
/// assert_eq!(parser.parse("true").unwrap(), JsonValue::Boolean(true));
/// assert_eq!(parser.parse("42").unwrap(), JsonValue::Number(42.0));
/// assert_eq!(parser.parse(r#""hello""#).unwrap(), JsonValue::String("hello".to_string().into()));
/// assert_eq!(parser.parse("[1, 2]").unwrap(), JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue<'a> {
    /// JSON `null`.
    Null,
    /// JSON boolean (`true` or `false`).
    Boolean(bool),
    /// JSON number, stored as an `f64` per the JSON specification.
    Number(f64),
    /// JSON string, with escape sequences already resolved.
    String(Cow<'a, str>),
    /// JSON array — an ordered sequence of [`JsonValue`]s.
    Array(Vec<JsonValue<'a>>),
    /// JSON object — a map of string keys to [`JsonValue`]s.
    Object(FxHashMap<Cow<'a, str>, JsonValue<'a>>),
}

impl<'a> JsonValue<'a> {
    /// Returns `true` if this value is `JsonValue::Null`.
    ///
    /// ```rust
    /// use rust_json_parser::JsonValue;
    ///
    /// assert!(JsonValue::Null.is_null());
    /// assert!(!JsonValue::Boolean(false).is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// If this is a `String`, returns the inner `&str`. Otherwise returns `None`.
    ///
    /// ```rust
    /// use rust_json_parser::JsonValue;
    ///
    /// let val = JsonValue::String("hello".to_string().into());
    /// assert_eq!(val.as_str(), Some("hello"));
    /// assert_eq!(JsonValue::Null.as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self {
            Some(s.as_ref())
        } else {
            None
        }
    }

    /// If this is a `Number`, returns the inner `f64`. Otherwise returns `None`.
    ///
    /// ```rust
    /// use rust_json_parser::JsonValue;
    ///
    /// assert_eq!(JsonValue::Number(3.14).as_f64(), Some(3.14));
    /// assert_eq!(JsonValue::Null.as_f64(), None);
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        if let JsonValue::Number(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    /// If this is a `Boolean`, returns the inner `bool`. Otherwise returns `None`.
    ///
    /// ```rust
    /// use rust_json_parser::JsonValue;
    ///
    /// assert_eq!(JsonValue::Boolean(true).as_bool(), Some(true));
    /// assert_eq!(JsonValue::Null.as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        if let JsonValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// If this is an `Array`, returns a slice of the elements. Otherwise returns `None`.
    ///
    /// ```rust
    /// use rust_json_parser::{JsonParser, JsonValue};
    ///
    /// let mut parser = JsonParser::new();
    /// let value = parser.parse("[1, 2, 3]").unwrap();
    ///
    /// assert_eq!(value.as_array().unwrap().len(), 3);
    /// assert_eq!(JsonValue::Null.as_array(), None);
    /// ```
    pub fn as_array(&self) -> Option<&[JsonValue<'_>]> {
        if let JsonValue::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    /// If this is an `Object`, returns a reference to the underlying [`FxHashMap`]. Otherwise returns `None`.
    ///
    /// ```rust
    /// use rust_json_parser::{JsonParser, JsonValue};
    ///
    /// let mut parser = JsonParser::new();
    /// let value = parser.parse(r#"{"a": 1}"#).unwrap();
    ///
    /// let obj = value.as_object().unwrap();
    /// assert!(obj.contains_key("a"));
    /// ```
    pub fn as_object(&self) -> Option<&FxHashMap<Cow<'a, str>, JsonValue<'_>>> {
        if let JsonValue::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    /// Looks up a key in a JSON object. Returns `None` if this is not an object
    /// or if the key does not exist.
    ///
    /// ```rust
    /// use rust_json_parser::{JsonParser, JsonValue};
    ///
    /// let mut parser = JsonParser::new();
    /// let value = parser.parse(r#"{"name": "Alice"}"#).unwrap();
    ///
    /// assert_eq!(value.get("name"), Some(&JsonValue::String("Alice".to_string().into())));
    /// assert_eq!(value.get("missing"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&JsonValue<'_>> {
        if let Some(map) = self.as_object() {
            map.get(key)
        } else {
            None
        }
    }

    /// Returns a pretty-printed JSON string with the given indentation width.
    ///
    /// ```rust
    /// use rust_json_parser::{JsonParser, JsonValue};
    ///
    /// let mut parser = JsonParser::new();
    /// let value = parser.parse(r#"{"a": [1, 2]}"#).unwrap();
    ///
    /// let output = value.pretty_print(2);
    /// assert!(output.contains('\n'));
    /// ```
    pub fn pretty_print(&self, indent: usize) -> String {
        const INITIAL_DEPTH: usize = 0;
        self.pretty_print_recursive(indent, INITIAL_DEPTH)
    }
    fn pretty_print_recursive(&self, indent: usize, depth: usize) -> String {
        let current_indent = " ".repeat(indent * depth);
        let child_indent = " ".repeat(indent * (depth + 1));
        match self {
            JsonValue::Null
            | JsonValue::Boolean(_)
            | JsonValue::Number(_)
            | JsonValue::String(_) => self.to_string(),
            JsonValue::Array(json_values) => {
                if json_values.is_empty() {
                    "[]".to_string()
                } else {
                    let elements: Vec<String> = json_values
                        .iter()
                        .map(|val| {
                            format!(
                                "{}{}",
                                child_indent,
                                val.pretty_print_recursive(indent, depth + 1)
                            )
                        })
                        .collect();
                    format!("[\n{}\n{}]", elements.join(",\n"), current_indent)
                }
            }
            JsonValue::Object(hash_map) => {
                if hash_map.is_empty() {
                    "{}".to_string()
                } else {
                    let map: Vec<String> = hash_map
                        .iter()
                        .map(|(key, val)| {
                            format!(
                                "{}{}: {}",
                                child_indent,
                                JsonValue::String(Cow::Owned(key.to_string().into())),
                                val.pretty_print_recursive(indent, depth + 1)
                            )
                        })
                        .collect();
                    format!("{{\n{}\n{}}}", map.join(",\n"), current_indent)
                }
            }
        }
    }
}

impl fmt::Display for JsonValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(b) => write!(f, "{}", b),
            JsonValue::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JsonValue::String(s) => {
                write!(f, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '\n' => write!(f, "\\n")?,
                        '\t' => write!(f, "\\t")?,
                        '\r' => write!(f, "\\r")?,
                        '\\' => write!(f, "\\\\")?,
                        '\"' => write!(f, "\\\"")?,
                        ch => write!(f, "{}", ch)?,
                    }
                }
                write!(f, "\"")
            }
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, element) in arr.iter().enumerate() {
                    if i != 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i != 0 {
                        write!(f, ",")?;
                    }
                    write!(
                        f,
                        "{}:{}",
                        JsonValue::String(Cow::Owned(key.to_string().into())),
                        value
                    )?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Boolean(true);
        let num_val = JsonValue::Number(42.5);
        let str_val = JsonValue::String("hello".to_string().into());

        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string().into());
        assert_eq!(value.as_str(), Some("test"));
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_bool(), None);
        assert!(!value.is_null());

        let value = JsonValue::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_str(), None);

        let value = JsonValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));

        let value = JsonValue::Null;
        assert!(value.is_null());
    }

    #[test]
    fn test_json_value_equality() {
        assert_eq!(JsonValue::Null, JsonValue::Null);
        assert_eq!(JsonValue::Boolean(true), JsonValue::Boolean(true));
        assert_eq!(JsonValue::Number(42.0), JsonValue::Number(42.0));
        assert_eq!(
            JsonValue::String("test".to_string().into()),
            JsonValue::String("test".to_string().into())
        );

        assert_ne!(JsonValue::Null, JsonValue::Boolean(false));
        assert_ne!(JsonValue::Number(1.0), JsonValue::Number(2.0));
    }

    mod display_tests {
        use super::*;
        use crate::parser::JsonParser;

        #[test]
        fn test_display_primitives() {
            assert_eq!(JsonValue::Null.to_string(), "null");
            assert_eq!(JsonValue::Boolean(true).to_string(), "true");
            assert_eq!(JsonValue::Boolean(false).to_string(), "false");
            assert_eq!(JsonValue::Number(42.0).to_string(), "42");
            assert_eq!(JsonValue::Number(9.6).to_string(), "9.6");
            assert_eq!(
                JsonValue::String("hello".to_string().into()).to_string(),
                "\"hello\""
            );
        }

        #[test]
        fn test_display_array() {
            let value = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
            assert_eq!(value.to_string(), "[1,2]");
        }

        #[test]
        fn test_display_empty_containers() {
            assert_eq!(JsonValue::Array(vec![]).to_string(), "[]");
            assert_eq!(JsonValue::Object(FxHashMap::default()).to_string(), "{}");
        }

        #[test]
        fn test_display_escape_string() {
            let value = JsonValue::String("hello\nworld".to_string().into());
            assert_eq!(value.to_string(), "\"hello\\nworld\"");
        }

        #[test]
        fn test_display_escape_quotes() {
            let value = JsonValue::String("say \"hi\"".to_string().into());
            assert_eq!(value.to_string(), "\"say \\\"hi\\\"\"");
        }

        #[test]
        fn test_display_nested() {
            let value = JsonParser::new().parse(r#"{"arr": [1, 2]}"#).unwrap();
            let output = value.to_string();
            // Object key order may vary, so check components
            assert!(output.contains("\"arr\""));
            assert!(output.contains("[1,2]"));
        }

        #[test]
        fn test_display_nested_array() {
            let value = JsonValue::Array(vec![JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
            ])]);
            assert_eq!(value.to_string(), "[[1,2]]");
        }
    }
}
