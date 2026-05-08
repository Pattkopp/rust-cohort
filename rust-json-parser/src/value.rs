use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        if let JsonValue::Number(f) = self {
            Some(*f)
        } else {
            None
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let JsonValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        if let JsonValue::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        if let Some(arr) = self.as_array() {
            arr.get(index)
        } else {
            None
        }
    }
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        if let JsonValue::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if let Some(map) = self.as_object() {
            map.get(key)
        } else {
            None
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
        let str_val = JsonValue::String("hello".to_string());

        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string());
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
            JsonValue::String("test".to_string()),
            JsonValue::String("test".to_string())
        );

        assert_ne!(JsonValue::Null, JsonValue::Boolean(false));
        assert_ne!(JsonValue::Number(1.0), JsonValue::Number(2.0));
    }

    mod display_tests {

        #[test]
        fn test_display_primitives() {
            assert_eq!(JsonValue::Null.to_string(), "null");
            assert_eq!(JsonValue::Boolean(true).to_string(), "true");
            assert_eq!(JsonValue::Boolean(false).to_string(), "false");
            assert_eq!(JsonValue::Number(42.0).to_string(), "42");
            assert_eq!(JsonValue::Number(3.14).to_string(), "3.14");
            assert_eq!(
                JsonValue::String("hello".to_string()).to_string(),
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
            assert_eq!(JsonValue::Object(HashMap::new()).to_string(), "{}");
        }

        #[test]
        fn test_display_escape_string() {
            let value = JsonValue::String("hello\nworld".to_string());
            assert_eq!(value.to_string(), "\"hello\\nworld\"");
        }

        #[test]
        fn test_display_escape_quotes() {
            let value = JsonValue::String("say \"hi\"".to_string());
            assert_eq!(value.to_string(), "\"say \\\"hi\\\"\"");
        }

        #[test]
        fn test_display_nested() {
            let value = parse_json(r#"{"arr": [1, 2]}"#).unwrap();
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
