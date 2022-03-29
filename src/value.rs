use regex::Regex;

#[derive(Clone)]
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
#[derive(Default)]
pub struct ValueLimiter {
    // Allowed variant
    default:  Option<Value>,
    variant:  Option<Vec<Value>>,
    prefix :  Option<String>,
    postfix:  Option<String>,
    pattern:  Option<Regex>, // -> This better be a regex
}

impl ValueLimiter {
    pub fn get_default(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    pub fn get_variant(&self) -> Option<&Vec<Value>> {
        self.variant.as_ref()
    }

    pub fn get_prefix(&self) -> Option<&String> {
        self.prefix.as_ref()
    }

    pub fn get_postfix(&self) -> Option<&String> {
        self.postfix.as_ref()
    }

    pub fn get_pattern(&self) -> Option<&Regex> {
        self.pattern.as_ref()
    }
}

#[derive(Clone, Copy)]
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
