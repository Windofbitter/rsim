use std::collections::HashMap;
use crate::core::values::events::Event;
use crate::core::values::traits::EventInputs;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}