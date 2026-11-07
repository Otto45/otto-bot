use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;

/// A value an exception's context may hold: primitives only, so a model, a row or a bound parameter's value cannot
/// be put in a context at all.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ContextValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl From<&str> for ContextValue {
    fn from(value: &str) -> Self {
        ContextValue::String(value.to_string())
    }
}

impl From<String> for ContextValue {
    fn from(value: String) -> Self {
        ContextValue::String(value)
    }
}

impl From<i64> for ContextValue {
    fn from(value: i64) -> Self {
        ContextValue::Integer(value)
    }
}

impl From<u16> for ContextValue {
    fn from(value: u16) -> Self {
        ContextValue::Integer(i64::from(value))
    }
}

impl From<f64> for ContextValue {
    fn from(value: f64) -> Self {
        ContextValue::Float(value)
    }
}

impl From<bool> for ContextValue {
    fn from(value: bool) -> Self {
        ContextValue::Boolean(value)
    }
}

impl fmt::Display for ContextValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextValue::String(value) => formatter.write_str(value),
            ContextValue::Integer(value) => write!(formatter, "{value}"),
            ContextValue::Float(value) => write!(formatter, "{value}"),
            ContextValue::Boolean(value) => write!(formatter, "{value}"),
        }
    }
}

/// What an exception says about where it happened, under snake_case keys, for the engineers who read the logs. It
/// never holds anything sensitive: no bound parameter values, no credentials.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ContextualData {
    values: BTreeMap<String, ContextValue>,
}

impl ContextualData {
    pub fn new() -> Self {
        ContextualData {
            values: BTreeMap::new(),
        }
    }

    /// The same context with one more value, so a context is built in one expression:
    /// `ContextualData::new().with("customer_id", customer_id)`.
    pub fn with(mut self, key: &str, value: impl Into<ContextValue>) -> Self {
        self.insert(key, value);
        self
    }

    pub fn insert(&mut self, key: &str, value: impl Into<ContextValue>) {
        self.values.insert(key.to_string(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        self.values.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ContextValue)> {
        self.values.iter()
    }
}

impl fmt::Display for ContextualData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("{")?;
        for (index, (key, value)) in self.values.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{key}: {value}")?;
        }
        formatter.write_str("}")
    }
}
