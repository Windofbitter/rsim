use std::collections::HashMap;
use crate::core::values::typed_value::TypedValue;
use crate::core::values::events::Event;
use crate::core::components::types::{Inputs, Outputs};

/// Unified input implementation that merges TypedInputs and EventInputs
/// 
/// This implementation provides progressive disclosure - components can access
/// simple typed values or detailed event information as needed.
pub struct UnifiedInputMap {
    events: HashMap<String, Event>,
}

impl UnifiedInputMap {
    /// Create a new empty input map
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }
    
    /// Create from a map of events
    pub fn from_events(events: HashMap<String, Event>) -> Self {
        Self { events }
    }
    
    /// Create from a map of typed values (convenience method)
    pub fn from_typed_values(values: HashMap<String, TypedValue>) -> Self {
        let events = values
            .into_iter()
            .map(|(name, value)| {
                let event = Event::from_typed_value(0, value); // Default timestamp of 0
                (name, event)
            })
            .collect();
        Self { events }
    }
    
    /// Insert an event directly
    pub fn insert_event(&mut self, port: String, event: Event) {
        self.events.insert(port, event);
    }
    
    /// Insert a typed value as an event with timestamp 0
    pub fn insert<T: Send + Sync + Clone + 'static>(&mut self, port: String, value: T) {
        let event = Event::new(0, value);
        self.events.insert(port, event);
    }
    
    /// Insert a typed value with a specific timestamp
    pub fn insert_with_timestamp<T: Send + Sync + Clone + 'static>(&mut self, port: String, value: T, timestamp: u64) {
        let event = Event::new(timestamp, value);
        self.events.insert(port, event);
    }
    
    /// Get the raw event for advanced use cases
    pub fn get_event(&self, port: &str) -> Option<&Event> {
        self.events.get(port)
    }
}

impl Inputs for UnifiedInputMap {
    fn get_typed_value(&self, port: &str) -> Result<TypedValue, String> {
        let event = self.events.get(port)
            .ok_or_else(|| format!("Input port '{}' not found", port))?;
        
        Ok(event.payload.clone())
    }
    
    fn get_timestamp(&self, port: &str) -> Result<u64, String> {
        let event = self.events.get(port)
            .ok_or_else(|| format!("Input port '{}' not found", port))?;
        
        Ok(event.timestamp)
    }
    
    fn has_input(&self, port: &str) -> bool {
        self.events.contains_key(port)
    }
    
    fn input_ports(&self) -> Vec<&str> {
        self.events.keys().map(|s| s.as_str()).collect()
    }
    
    fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for UnifiedInputMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified output implementation that merges TypedOutputs and EventOutputs
/// 
/// This implementation collects outputs as events and provides both simple
/// value setting and advanced timestamp control.
pub struct UnifiedOutputMap {
    events: HashMap<String, Event>,
    expected_ports: Vec<String>,
}

impl UnifiedOutputMap {
    /// Create a new output map with expected ports
    pub fn new(expected_ports: Vec<String>) -> Self {
        Self {
            events: HashMap::new(),
            expected_ports,
        }
    }
    
    /// Create an output map without port validation (accepts any port)
    pub fn new_flexible() -> Self {
        Self {
            events: HashMap::new(),
            expected_ports: Vec::new(),
        }
    }
    
    /// Get the collected events (consumes self)
    pub fn into_events(self) -> HashMap<String, Event> {
        self.events
    }
    
    /// Get the collected typed values (consumes self)
    pub fn into_typed_values(self) -> HashMap<String, TypedValue> {
        self.events
            .into_iter()
            .map(|(name, event)| (name, event.payload))
            .collect()
    }
    
    /// Set an event directly (advanced use case)
    pub fn set_event(&mut self, port: &str, event: Event) -> Result<(), String> {
        if !self.expected_ports.is_empty() && !self.expected_ports.contains(&port.to_string()) {
            return Err(format!(
                "Unknown output port '{}'. Expected ports: {:?}",
                port, self.expected_ports
            ));
        }
        
        self.events.insert(port.to_string(), event);
        Ok(())
    }
}

impl Outputs for UnifiedOutputMap {
    fn set_typed_value(&mut self, port: &str, value: TypedValue) -> Result<(), String> {
        if !self.expected_ports.is_empty() && !self.expected_ports.contains(&port.to_string()) {
            return Err(format!(
                "Unknown output port '{}'. Expected ports: {:?}",
                port, self.expected_ports
            ));
        }
        
        let event = Event::from_typed_value(0, value); // Default timestamp
        self.events.insert(port.to_string(), event);
        Ok(())
    }
    
    fn set_typed_value_with_timestamp(
        &mut self, 
        port: &str, 
        value: TypedValue, 
        timestamp: u64
    ) -> Result<(), String> {
        if !self.expected_ports.is_empty() && !self.expected_ports.contains(&port.to_string()) {
            return Err(format!(
                "Unknown output port '{}'. Expected ports: {:?}",
                port, self.expected_ports
            ));
        }
        
        let event = Event::from_typed_value(timestamp, value);
        self.events.insert(port.to_string(), event);
        Ok(())
    }
    
    fn is_valid_port(&self, port: &str) -> bool {
        self.expected_ports.is_empty() || self.expected_ports.contains(&port.to_string())
    }
    
    fn expected_ports(&self) -> Vec<&str> {
        self.expected_ports.iter().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::components::types::{InputsExt, OutputsExt};

    #[test]
    fn test_unified_input_map() {
        let mut inputs = UnifiedInputMap::new();
        inputs.insert("test_port".to_string(), 42i32);
        
        assert!(inputs.has_input("test_port"));
        assert_eq!(inputs.get::<i32>("test_port").unwrap(), 42);
        assert_eq!(inputs.get_timestamp("test_port").unwrap(), 0);
    }

    #[test]
    fn test_unified_output_map() {
        let mut outputs = UnifiedOutputMap::new(vec!["result".to_string()]);
        
        outputs.set("result", 100i32).unwrap();
        assert!(outputs.is_valid_port("result"));
        assert!(!outputs.is_valid_port("invalid"));
        
        let events = outputs.into_events();
        assert!(events.contains_key("result"));
    }

    #[test]
    fn test_timestamp_handling() {
        let mut inputs = UnifiedInputMap::new();
        inputs.insert_with_timestamp("timed_port".to_string(), "test".to_string(), 12345);
        
        assert_eq!(inputs.get::<String>("timed_port").unwrap(), "test");
        assert_eq!(inputs.get_timestamp("timed_port").unwrap(), 12345);
    }
}