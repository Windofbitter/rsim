use std::collections::HashMap;
use crate::core::values::typed_value::TypedValue;
use crate::core::values::events::Event;
use crate::core::values::traits::{TypedInputs, TypedOutputs, EventInputs, EventOutputs};

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

/// Implementation of EventInputs backed by a HashMap of Events
pub struct EventInputMap {
    inputs: HashMap<String, Event>,
}

impl EventInputMap {
    /// Create a new empty event input map
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
        }
    }
    
    /// Create event input map from a HashMap of Events
    pub fn from_map(inputs: HashMap<String, Event>) -> Self {
        Self { inputs }
    }
    
    /// Insert an event
    pub fn insert_event(&mut self, port: String, event: Event) {
        self.inputs.insert(port, event);
    }
    
    /// Insert a typed value with timestamp
    pub fn insert<T: Send + Sync + Clone + 'static>(&mut self, port: String, timestamp: u64, value: T) {
        self.inputs.insert(port, Event::new(timestamp, value));
    }
}

impl EventInputs for EventInputMap {
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String> {
        let event = self.inputs.get(port)
            .ok_or_else(|| format!("Input port '{}' not found", port))?;
        
        let value_ref = event.payload.get::<T>()?;
        Ok(value_ref.clone())
    }
    
    fn get_event(&self, port: &str) -> Result<&Event, String> {
        self.inputs.get(port)
            .ok_or_else(|| format!("Input port '{}' not found", port))
    }
    
    fn get_timestamp(&self, port: &str) -> Result<u64, String> {
        let event = self.get_event(port)?;
        Ok(event.timestamp)
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

impl Default for EventInputMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of EventOutputs backed by a HashMap with port validation
pub struct EventOutputMap {
    outputs: HashMap<String, Event>,
    expected_ports: HashMap<String, &'static str>, // port_name -> type_name
    timestamp: u64,
}

impl EventOutputMap {
    /// Create a new event output map with expected ports
    pub fn new(expected_ports: Vec<(&str, &'static str)>, timestamp: u64) -> Self {
        let expected_ports = expected_ports.into_iter()
            .map(|(port, type_name)| (port.to_string(), type_name))
            .collect();
            
        Self {
            outputs: HashMap::new(),
            expected_ports,
            timestamp,
        }
    }
    
    /// Create an event output map without port validation (accepts any port)
    pub fn new_flexible(timestamp: u64) -> Self {
        Self {
            outputs: HashMap::new(),
            expected_ports: HashMap::new(),
            timestamp,
        }
    }
}

impl EventOutputs for EventOutputMap {
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String> {
        // Port validation (same as TypedOutputMap)
        if !self.expected_ports.is_empty() && !self.expected_ports.contains_key(port) {
            return Err(format!(
                "Unknown output port '{}'. Expected ports: {:?}", 
                port, 
                self.expected_ports.keys().collect::<Vec<_>>()
            ));
        }
        
        // Type validation
        if let Some(&expected_type) = self.expected_ports.get(port) {
            let actual_type = std::any::type_name::<T>();
            if expected_type != actual_type {
                return Err(format!(
                    "Type mismatch for output port '{}': expected {}, got {}", 
                    port, expected_type, actual_type
                ));
            }
        }
        
        let event = Event::new(self.timestamp, value);
        self.outputs.insert(port.to_string(), event);
        Ok(())
    }
    
    fn emit_event(&mut self, port: &str, event: Event) -> Result<(), String> {
        // Port validation
        if !self.expected_ports.is_empty() && !self.expected_ports.contains_key(port) {
            return Err(format!(
                "Unknown output port '{}'. Expected ports: {:?}", 
                port, 
                self.expected_ports.keys().collect::<Vec<_>>()
            ));
        }
        
        self.outputs.insert(port.to_string(), event);
        Ok(())
    }
    
    fn is_valid_port(&self, port: &str) -> bool {
        self.expected_ports.is_empty() || self.expected_ports.contains_key(port)
    }
    
    fn expected_ports(&self) -> Vec<&str> {
        self.expected_ports.keys().map(|s| s.as_str()).collect()
    }
    
    fn into_event_map(self) -> HashMap<String, Event> {
        self.outputs
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
    
    #[test]
    fn test_event_inputs() {
        let mut inputs = EventInputMap::new();
        inputs.insert("port1".to_string(), 100, 42i64);
        inputs.insert("port2".to_string(), 101, "hello".to_string());
        
        // Test convenience methods
        assert_eq!(inputs.get::<i64>("port1").unwrap(), 42);
        assert_eq!(inputs.get::<String>("port2").unwrap(), "hello");
        
        // Test timestamp access
        assert_eq!(inputs.get_timestamp("port1").unwrap(), 100);
        assert_eq!(inputs.get_timestamp("port2").unwrap(), 101);
        
        // Test event access
        let event = inputs.get_event("port1").unwrap();
        assert_eq!(event.timestamp, 100);
        assert_eq!(event.get_payload::<i64>().unwrap(), &42);
        
        assert!(inputs.has_input("port1"));
        assert!(!inputs.has_input("port3"));
        assert_eq!(inputs.len(), 2);
    }
    
    #[test]
    fn test_event_outputs() {
        let mut outputs = EventOutputMap::new_flexible(200);
        
        // Test convenience method
        outputs.set("out1", 42i64).unwrap();
        outputs.set("out2", "hello".to_string()).unwrap();
        
        let event_map = outputs.into_event_map();
        assert_eq!(event_map.len(), 2);
        
        let event1 = &event_map["out1"];
        assert_eq!(event1.timestamp, 200);
        assert_eq!(event1.get_payload::<i64>().unwrap(), &42);
        
        let event2 = &event_map["out2"];
        assert_eq!(event2.timestamp, 200);
        assert_eq!(event2.get_payload::<String>().unwrap(), "hello");
    }
}