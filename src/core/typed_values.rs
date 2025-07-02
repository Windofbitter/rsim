use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global event ID counter for unique event identification
static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique event ID
fn next_event_id() -> u64 {
    EVENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Event wrapper containing timestamp, unique ID, and typed payload
#[derive(Debug, Clone)]
pub struct Event {
    pub event_id: u64,
    pub timestamp: u64,
    pub payload: TypedValue,
}

impl Event {
    /// Create a new event with auto-generated timestamp and ID
    pub fn new<T: Send + Sync + Clone + 'static>(timestamp: u64, payload: T) -> Self {
        Self {
            event_id: next_event_id(),
            timestamp,
            payload: TypedValue::new(payload),
        }
    }
    
    /// Create event from existing TypedValue
    pub fn from_typed_value(timestamp: u64, payload: TypedValue) -> Self {
        Self {
            event_id: next_event_id(),
            timestamp,
            payload,
        }
    }
    
    /// Get the payload as a specific type
    pub fn get_payload<T: 'static>(&self) -> Result<&T, String> {
        self.payload.get::<T>()
    }
    
    /// Extract the payload as a specific type
    pub fn into_payload<T: 'static>(self) -> Result<T, String> {
        self.payload.into_inner::<T>()
    }
}

/// Type-erased but type-safe container for component values
#[derive(Debug)]
pub struct TypedValue {
    data: Box<dyn Any + Send + Sync>,
    clone_fn: fn(&dyn Any) -> Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    type_id: TypeId,
}

impl TypedValue {
    /// Create a new typed value
    pub fn new<T: Send + Sync + Clone + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            data: Box::new(value),
            clone_fn: |any| {
                let typed = any.downcast_ref::<T>().expect("Type mismatch in clone_fn");
                Box::new(typed.clone())
            },
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
    
}

impl Clone for TypedValue {
    fn clone(&self) -> Self {
        Self {
            data: (self.clone_fn)(self.data.as_ref()),
            clone_fn: self.clone_fn,
            type_name: self.type_name,
            type_id: self.type_id,
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
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
    
    /// Check if an output port is valid
    fn is_valid_port(&self, port: &str) -> bool;
    
    /// Get all expected output port names
    fn expected_ports(&self) -> Vec<&str>;
    
    /// Get the collected outputs (consumes self)
    fn into_map(self) -> HashMap<String, TypedValue>;
}

/// Trait for accessing event inputs with progressive disclosure
pub trait EventInputs {
    /// Get typed input value (convenience method)
    fn get<T: 'static + Clone>(&self, port: &str) -> Result<T, String>;
    
    /// Get full event for a port
    fn get_event(&self, port: &str) -> Result<&Event, String>;
    
    /// Get timestamp for a port (convenience method)
    fn get_timestamp(&self, port: &str) -> Result<u64, String>;
    
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

/// Trait for collecting event outputs with progressive disclosure
pub trait EventOutputs {
    /// Set typed output value (convenience method)
    fn set<T: Send + Sync + Clone + 'static>(&mut self, port: &str, value: T) -> Result<(), String>;
    
    /// Emit event directly
    fn emit_event(&mut self, port: &str, event: Event) -> Result<(), String>;
    
    /// Check if an output port is valid
    fn is_valid_port(&self, port: &str) -> bool;
    
    /// Get all expected output port names
    fn expected_ports(&self) -> Vec<&str>;
    
    /// Get the collected events (consumes self)
    fn into_event_map(self) -> HashMap<String, Event>;
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
    
    #[test]
    fn test_event_creation() {
        let event = Event::new(123, 42i64);
        assert_eq!(event.timestamp, 123);
        assert_eq!(event.get_payload::<i64>().unwrap(), &42);
        assert!(event.event_id > 0);
    }
    
    #[test]
    fn test_event_unique_ids() {
        let event1 = Event::new(1, 42i64);
        let event2 = Event::new(1, 42i64);
        assert_ne!(event1.event_id, event2.event_id);
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