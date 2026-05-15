use crate::{JsonError, JsonParser, JsonValue};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::types::{PyDict, PyList};
use pyo3::{IntoPyObjectExt, prelude::*};
use std::collections::HashMap;

// Type conversion: Rust to Python
impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            JsonValue::Null => Ok(py.None().into_bound(py)),
            JsonValue::Boolean(b) => Ok(b.into_bound_py_any(py)?), // https://docs.rs/pyo3/latest/pyo3/conversion/trait.IntoPyObjectExt.html
            JsonValue::Number(n) => Ok(n.into_bound_py_any(py)?),
            JsonValue::String(s) => Ok(s.into_bound_py_any(py)?),
            JsonValue::Array(arr) => Ok(arr.into_bound_py_any(py)?),
            JsonValue::Object(obj) => Ok(obj.into_bound_py_any(py)?),
        }
    }
}

// Type conversion: Rust errors to Python exceptions
impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> PyErr {
        PyValueError::new_err(err.to_string()) // we only have Value errors and can use the Display trait
    }
}

// Python-callable functions
#[pyfunction]
// changed the signature because of https://github.com/PyO3/pyo3/discussions/4826
fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    JsonParser::new().parse(input)?.into_bound_py_any(py)
}

#[pyfunction]
fn parse_json_file<'py>(py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyAny>> {
    parse_json(py, &std::fs::read_to_string(path)?)
}

#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
fn dumps(obj: &Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
    let obj = py_to_json_value(obj)?;
    match indent {
        Some(n) => Ok(obj.pretty_print(n)),
        _ => Ok(obj.to_string()),
    }
}

// Helper (not exposed to Python)
fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
    // 1. Check None first
    if obj.is_none() {
        return Ok(JsonValue::Null);
    }

    // 2. Check bool before numbers (critical!)
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(JsonValue::Boolean(b));
    }

    // 3. Check numbers
    if let Ok(n) = obj.extract::<f64>() {
        return Ok(JsonValue::Number(n));
    }

    // 4. Check string
    if let Ok(s) = obj.extract::<String>() {
        return Ok(JsonValue::String(s));
    }

    // 5. Check list (recurse on elements)
    if let Ok(list) = obj.cast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_json_value(&item)?); // Recursive call
        }
        return Ok(JsonValue::Array(arr));
    }

    // 6. Check dict (recurse on values)
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = HashMap::new();
        for (k, v) in dict.iter() {
            let k = k.extract::<String>()?;
            let v = py_to_json_value(&v)?;
            map.insert(k, v);
        }
        return Ok(JsonValue::Object(map));
    }

    // 7. Unsupported type
    Err(PyValueError::new_err(
        "Unsupported type for JSON conversion",
    ))
}

// Module registration
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    Ok(())
}
