use std::collections::HashMap;
use super::typed_value::TypedValue;
use super::events::Event;

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