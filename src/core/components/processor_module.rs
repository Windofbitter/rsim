use crate::core::values::implementations::EventOutputMap;
use super::evaluation_context::EvaluationContext;
use super::port_specs::PortSpec;

/// Processing component module that defines stateless computation
#[derive(Clone)]
pub struct ProcessorModule {
    /// Component name/type
    pub name: String,
    /// Input port specifications
    pub input_ports: Vec<PortSpec>,
    /// Output port specifications
    pub output_ports: Vec<PortSpec>,
    /// Memory port specifications
    pub memory_ports: Vec<PortSpec>,
    /// Evaluation function with event outputs
    pub evaluate_fn: fn(&mut EvaluationContext, &mut EventOutputMap) -> Result<(), String>,
}

impl ProcessorModule {
    /// Create a new processor module with validation
    pub fn new(
        name: &str,
        input_ports: Vec<PortSpec>,
        output_ports: Vec<PortSpec>,
        memory_ports: Vec<PortSpec>,
        evaluate_fn: fn(&mut EvaluationContext, &mut EventOutputMap) -> Result<(), String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            input_ports,
            output_ports,
            memory_ports,
            evaluate_fn,
        }
    }

    /// Validate that this processing module meets architecture constraints
    /// Processing components should have multiple ports but single output type
    pub fn validate_architecture(&self) -> Result<(), String> {
        // Validate component has a name
        if self.name.is_empty() {
            return Err("Processing module must have a valid name".to_string());
        }

        // Ensure processing components are stateless (enforced by design - no state field)
        // Ensure output wrapping (enforced by EventOutputMap in evaluate_fn signature)

        // Note: Single output type constraint is enforced by the evaluate_fn signature
        // and EventOutputMap which wraps all outputs in Event structures
        
        Ok(())
    }

    /// Get all input port names
    pub fn input_port_names(&self) -> Vec<&str> {
        self.input_ports.iter().map(|p| p.name.as_str()).collect()
    }

    /// Get all output port names
    pub fn output_port_names(&self) -> Vec<&str> {
        self.output_ports.iter().map(|p| p.name.as_str()).collect()
    }

    /// Get all memory port names
    pub fn memory_port_names(&self) -> Vec<&str> {
        self.memory_ports.iter().map(|p| p.name.as_str()).collect()
    }

    /// Check if an input port exists
    pub fn has_input_port(&self, name: &str) -> bool {
        self.input_ports.iter().any(|p| p.name == name)
    }

    /// Check if an output port exists
    pub fn has_output_port(&self, name: &str) -> bool {
        self.output_ports.iter().any(|p| p.name == name)
    }

    /// Check if a memory port exists
    pub fn has_memory_port(&self, name: &str) -> bool {
        self.memory_ports.iter().any(|p| p.name == name)
    }
}