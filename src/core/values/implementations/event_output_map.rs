use std::collections::HashMap;
use crate::core::values::events::Event;
use crate::core::values::traits::EventOutputs;

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