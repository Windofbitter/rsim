use std::collections::HashMap;
use crate::core::values::typed_value::TypedValue;
use crate::core::values::traits::TypedOutputs;

/// Implementation of TypedOutputs backed by a HashMap with port validation
pub struct TypedOutputMap {
    outputs: HashMap<String, TypedValue>,
    expected_ports: HashMap<String, &'static str>, // port_name -> type_name
}

impl TypedOutputMap {
    /// Create a new output map with expected ports
    pub fn new(expected_ports: Vec<(&str, &'static str)>) -> Self {
        let expected_ports = expected_ports.into_iter()
            .map(|(port, type_name)| (port.to_string(), type_name))
            .collect();
            
        Self {
            outputs: HashMap::new(),
            expected_ports,
        }
    }
    
    /// Create an output map without port validation (accepts any port)
    pub fn new_flexible() -> Self {
        Self {
            outputs: HashMap::new(),
            expected_ports: HashMap::new(),
        }
    }
}

impl TypedOutputs for TypedOutputMap {
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String> {
        // If we have expected ports defined, validate the port exists
        if !self.expected_ports.is_empty() && !self.expected_ports.contains_key(port) {
            return Err(format!(
                "Unknown output port '{}'. Expected ports: {:?}", 
                port, 
                self.expected_ports.keys().collect::<Vec<_>>()
            ));
        }
        
        // If we have type expectations, validate the type
        if let Some(&expected_type) = self.expected_ports.get(port) {
            let actual_type = std::any::type_name::<T>();
            if expected_type != actual_type {
                return Err(format!(
                    "Type mismatch for output port '{}': expected {}, got {}", 
                    port, expected_type, actual_type
                ));
            }
        }
        
        self.outputs.insert(port.to_string(), TypedValue::new(value));
        Ok(())
    }
    
    fn is_valid_port(&self, port: &str) -> bool {
        self.expected_ports.is_empty() || self.expected_ports.contains_key(port)
    }
    
    fn expected_ports(&self) -> Vec<&str> {
        self.expected_ports.keys().map(|s| s.as_str()).collect()
    }
    
    fn into_map(self) -> HashMap<String, TypedValue> {
        self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_outputs() {
        let mut outputs = TypedOutputMap::new(vec![
            ("out1", "i64"),
            ("out2", "alloc::string::String"),
        ]);
        
        outputs.set("out1", 42i64).unwrap();
        outputs.set("out2", "hello".to_string()).unwrap();
        
        // Invalid port should fail
        assert!(outputs.set("invalid", 42i64).is_err());
        
        let map = outputs.into_map();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("out1"));
        assert!(map.contains_key("out2"));
    }
    
    #[test]
    fn test_flexible_outputs() {
        let mut outputs = TypedOutputMap::new_flexible();
        
        // Should accept any port when flexible
        outputs.set("any_port", 42i64).unwrap();
        outputs.set("another_port", "hello".to_string()).unwrap();
        
        let map = outputs.into_map();
        assert_eq!(map.len(), 2);
    }
}