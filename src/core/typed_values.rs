use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::types::ComponentValue;

/// Type-erased but type-safe container for component values
#[derive(Debug)]
pub struct TypedValue {
    data: Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    type_id: TypeId,
}

impl TypedValue {
    /// Create a new typed value
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            data: Box::new(value),
        }
    }
    
    /// Get a reference to the contained value
    pub fn get<T: 'static>(&self) -> Result<&T, String> {
        if TypeId::of::<T>() != self.type_id {
            return Err(format!(
                "Type mismatch: expected {}, found {}", 
                std::any::type_name::<T>(), 
                self.type_name
            ));
        }
        
        self.data.downcast_ref::<T>()
            .ok_or_else(|| format!("Failed to downcast to {}", std::any::type_name::<T>()))
    }
    
    /// Consume the typed value and return the contained value
    pub fn into_inner<T: 'static>(self) -> Result<T, String> {
        if TypeId::of::<T>() != self.type_id {
            return Err(format!(
                "Type mismatch: expected {}, found {}", 
                std::any::type_name::<T>(), 
                self.type_name
            ));
        }
        
        self.data.downcast::<T>()
            .map(|boxed| *boxed)
            .map_err(|_| format!("Failed to downcast to {}", std::any::type_name::<T>()))
    }
    
    /// Get the type name of the contained value
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
    
    /// Get the type ID of the contained value
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    /// Check if the contained value is of type T
    pub fn is_type<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
    
    /// Convert TypedValue to Event (ComponentValue) for probes and external interfaces
    pub fn to_event(&self) -> Option<ComponentValue> {
        if self.is_type::<i64>() {
            self.get::<i64>().ok().map(|v| ComponentValue::Int(*v))
        } else if self.is_type::<f64>() {
            self.get::<f64>().ok().map(|v| ComponentValue::Float(*v))
        } else if self.is_type::<String>() {
            self.get::<String>().ok().map(|v| ComponentValue::String(v.clone()))
        } else if self.is_type::<bool>() {
            self.get::<bool>().ok().map(|v| ComponentValue::Bool(*v))
        } else {
            None
        }
    }
    
    /// Create TypedValue from Event (ComponentValue)
    pub fn from_event(event: ComponentValue) -> Self {
        match event {
            ComponentValue::Int(v) => TypedValue::new(v),
            ComponentValue::Float(v) => TypedValue::new(v),
            ComponentValue::String(v) => TypedValue::new(v),
            ComponentValue::Bool(v) => TypedValue::new(v),
        }
    }
}

impl Clone for TypedValue {
    fn clone(&self) -> Self {
        // Clone by converting to ComponentValue and back (for common types)
        if let Some(event) = self.to_event() {
            TypedValue::from_event(event)
        } else {
            // For custom types that don't convert to ComponentValue, we can't clone
            panic!("TypedValue::clone() is not supported for type {}. Custom types must implement manual cloning.", self.type_name);
        }
    }
}

/// Trait for accessing typed inputs in components
pub trait TypedInputs {
    /// Get typed input value
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String>;
    
    /// Check if input exists
    fn has_input(&self, port: &str) -> bool;
    
    /// Get all available input port names
    fn input_ports(&self) -> Vec<&str>;
    
    /// Get the number of inputs
    fn len(&self) -> usize;
    
    /// Check if there are no inputs
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Trait for collecting typed outputs from components
pub trait TypedOutputs {
    /// Set typed output value
    fn set<T: Send + Sync + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
    
    /// Check if an output port is valid
    fn is_valid_port(&self, port: &str) -> bool;
    
    /// Get all expected output port names
    fn expected_ports(&self) -> Vec<&str>;
    
    /// Get the collected outputs (consumes self)
    fn into_map(self) -> HashMap<String, TypedValue>;
}

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
    pub fn insert<T: Send + Sync + 'static>(&mut self, port: String, value: T) {
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
    fn set<T: Send + Sync + 'static>(&mut self, port: &str, value: T) -> Result<(), String> {
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

/// Helper trait for types that can be used in the typed system
pub trait TypedData: Send + Sync + Clone + 'static {}

// Implement for common types
impl TypedData for i64 {}
impl TypedData for f64 {}
impl TypedData for String {}
impl TypedData for bool {}
impl TypedData for u64 {}
impl TypedData for i32 {}
impl TypedData for f32 {}
impl TypedData for u32 {}
impl<T: TypedData> TypedData for Vec<T> {}
impl<T: TypedData> TypedData for Option<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_value_basic() {
        let value = TypedValue::new(42i64);
        assert_eq!(value.get::<i64>().unwrap(), &42);
        assert!(value.is_type::<i64>());
        assert!(!value.is_type::<String>());
    }
    
    #[test]
    fn test_typed_value_type_mismatch() {
        let value = TypedValue::new(42i64);
        let result = value.get::<String>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }
    
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