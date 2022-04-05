use regex::Regex;

use crate::{CedError, CedResult};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    Number(isize),
    Text(String),
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::Text(_) => ValueType::Text,
        }
    }
    pub fn from_str(src: &str, value_type: ValueType) -> CedResult<Self> {
        Ok(match value_type {
            ValueType::Number => {
                let src_number = src.parse::<isize>().map_err(|_| {
                    CedError::InvalidValueType(format!("\"{}\" is not a valid number", src))
                })?;
                Value::Number(src_number)
            }
            ValueType::Text => Value::Text(src.to_string()),
        })
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::Number(num) => num.to_string(),
            Self::Text(txt) => txt.to_string(),
        };
        write!(f, "{}", out)
    }
}

// This struct should not expose value directly
// because some limiters are mutually exclusive.
#[derive(Default, Clone, Debug)]
pub struct ValueLimiter {
    // Allowed variant
    value_type: ValueType,
    default: Option<Value>,
    variant: Option<Vec<Value>>,
    pattern: Option<Regex>, // -> This better be a regex
}

impl ValueLimiter {
    pub fn qualify(&self, value: &Value) -> bool {
        // Only when value typ matches limiter's type
        if self.value_type != value.get_type() {
            return false;
        }
        match value {
            Value::Number(num) => {
                if let Some(variant) = self.variant.as_ref() {
                    variant.contains(value)
                } else if let Some(pattern) = self.pattern.as_ref() {
                    pattern.is_match(&num.to_string())
                } else {
                    true
                }
            }
            Value::Text(text) => {
                if let Some(variant) = self.variant.as_ref() {
                    variant.contains(value)
                } else if let Some(pattern) = self.pattern.as_ref() {
                    pattern.is_match(text)
                } else {
                    true
                }
            }
        }
    }

    pub fn from_line(flags: &Vec<String>) -> Self {
        // TODO
        Self::default()
    }

    pub fn get_type(&self) -> ValueType {
        self.value_type
    }

    pub fn set_type(&mut self, column_type: ValueType) {
        self.value_type = column_type;
    }

    pub fn get_default(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    pub fn get_variant(&self) -> Option<&Vec<Value>> {
        self.variant.as_ref()
    }

    pub fn set_variant(&mut self, default: Value, variants: &Vec<Value>) -> CedResult<()> {
        if !variants.contains(&default) {
            return Err(CedError::InvalidLimiter(format!(
                "Default value should be among one of variants"
            )));
        }
        self.default.replace(default);
        self.variant.replace(variants.to_vec());
        Ok(())
    }

    pub fn get_pattern(&self) -> Option<&Regex> {
        self.pattern.as_ref()
    }

    pub fn set_pattern(&mut self, default: Value, pattern: Regex) -> CedResult<()> {
        if !pattern.is_match(&default.to_string()) {
            return Err(CedError::InvalidLimiter(format!(
                "Default value should match pattern"
            )));
        }
        self.default.replace(default);
        self.pattern.replace(pattern);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ValueType {
    Number,
    Text,
}

impl ValueType {
    pub fn from_str(src: &str) -> Self {
        if src.to_lowercase().as_str() == "number" {
            Self::Number
        } else {
            Self::Text
        }
    }
}

impl Default for ValueType {
    fn default() -> Self {
        Self::Text
    }
}
