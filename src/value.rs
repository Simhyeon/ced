use regex::Regex;

#[derive(Clone,PartialEq)]
pub enum Value {
    Number(isize),
    Text(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            Self::Number(num) => num.to_string(),
            Self::Text(txt) => txt.to_string(),
        };
        write!(f,"{}",out)
    }
}

// This struct should not expose value directly
// because some limiters are mutually exclusive.
#[derive(Default, Clone)]
pub struct ValueLimiter {
    // Allowed variant
    value_type : ValueType,
    default    : Option<Value>,
    variant    : Option<Vec<Value>>,
    pattern    : Option<Regex>, // -> This better be a regex
}

impl ValueLimiter {
    pub fn qualify(&self, value: &Value) -> bool {
        match value {
            Value::Number(_) => {
                if self.value_type == ValueType::Number {
                    true
                } else {
                    false
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

    pub fn get_default(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    pub fn get_variant(&self) -> Option<&Vec<Value>> {
        self.variant.as_ref()
    }

    pub fn get_pattern(&self) -> Option<&Regex> {
        self.pattern.as_ref()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ValueType {
    Number,
    Text,
}

impl ValueType {
    pub fn from_str(src : &str) -> Self {
        if src.to_lowercase().as_str() == "number" {
            Self::Number
        } else { Self::Text }
    }
}

impl Default for ValueType {
    fn default() -> Self {
        Self::Text
    }
}
