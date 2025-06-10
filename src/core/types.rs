pub type ComponentId = String;
pub type SimulationTime = u64;

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl ComponentValue {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ComponentValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ComponentValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            ComponentValue::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ComponentValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
}
