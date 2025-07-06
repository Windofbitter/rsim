use std::collections::HashMap;
use crate::core::values::typed_value::TypedValue;
use crate::core::values::traits::TypedInputs;

/// Implementation of TypedInputs backed by a HashMap
pub struct TypedInputMap {
    inputs: HashMap<String, TypedValue>,
}

impl TypedInputMap {
    /// Create a new empty input map
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
        }
    }
    
    /// Create input map from a HashMap of TypedValues
    pub fn from_map(inputs: HashMap<String, TypedValue>) -> Self {
        Self { inputs }
    }
    
    /// Insert a typed value
    pub fn insert<T: Send + Sync + Clone + 'static>(&mut self, port: String, value: T) {
        self.inputs.insert(port, TypedValue::new(value));
    }
    
    /// Insert a TypedValue directly
    pub fn insert_typed(&mut self, port: String, value: TypedValue) {
        self.inputs.insert(port, value);
    }
}

impl TypedInputs for TypedInputMap {
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String> {
        let typed_value = self.inputs.get(port)
            .ok_or_else(|| format!("Input port '{}' not found", port))?;
        
        let value_ref = typed_value.get::<T>()?;
        Ok(value_ref.clone())
    }
    
    fn has_input(&self, port: &str) -> bool {
        self.inputs.contains_key(port)
    }
    
    fn input_ports(&self) -> Vec<&str> {
        self.inputs.keys().map(|s| s.as_str()).collect()
    }
    
    fn len(&self) -> usize {
        self.inputs.len()
    }
}

impl Default for TypedInputMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_inputs() {
        let mut inputs = TypedInputMap::new();
        inputs.insert("port1".to_string(), 42i64);
        inputs.insert("port2".to_string(), "hello".to_string());
        
        assert_eq!(inputs.get::<i64>("port1").unwrap(), 42);
        assert_eq!(inputs.get::<String>("port2").unwrap(), "hello");
        assert!(inputs.has_input("port1"));
        assert!(!inputs.has_input("port3"));
        assert_eq!(inputs.len(), 2);
    }
}