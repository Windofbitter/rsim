use crate::core::builder::simulation_builder::ComponentInstance;
use crate::core::types::ComponentId;
use crate::core::execution::execution_order::ExecutionOrderBuilder;
use crate::core::components::module::{EvaluationContext, MemoryModuleTrait};
use crate::core::values::implementations::{EventInputMap, EventOutputMap};
use crate::core::values::events::Event;
use crate::core::values::traits::EventOutputs;
use crate::core::memory::proxy::MemoryProxy;
use std::collections::HashMap;

/// Simplified cycle engine for the new direct API
/// 
/// This engine manages the execution of component evaluation functions
/// in a deterministic order for each simulation cycle.
pub struct CycleEngine {
    /// All component instances
    components: HashMap<ComponentId, ComponentInstance>,
    /// Port connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Current cycle number
    current_cycle: u64,
    /// Execution order for processing components (topologically sorted)
    execution_order: Vec<ComponentId>,
    /// Output buffer for current cycle
    output_buffer: HashMap<(ComponentId, String), Event>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
}

impl CycleEngine {
    /// Create a new cycle engine
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: HashMap::new(),
            current_cycle: 0,
            execution_order: Vec::new(),
            output_buffer: HashMap::new(),
            memory_connections: HashMap::new(),
        }
    }

    /// Register a component instance
    pub fn register_component_instance(&mut self, instance: ComponentInstance) -> Result<(), String> {
        let id = instance.id.clone();
        self.components.insert(id, instance);
        Ok(())
    }

    /// Add a connection between components
    pub fn connect(
        &mut self,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        // Validate that both components exist
        if !self.components.contains_key(&source.0) {
            return Err(format!("Source component '{}' not found", source.0));
        }
        if !self.components.contains_key(&target.0) {
            return Err(format!("Target component '{}' not found", target.0));
        }

        // Add the connection
        self.connections
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);

        Ok(())
    }

    /// Add a memory connection between a component port and a memory module
    pub fn connect_memory(
        &mut self,
        component_port: (ComponentId, String),
        memory_id: ComponentId,
    ) -> Result<(), String> {
        // Validate that both components exist
        if !self.components.contains_key(&component_port.0) {
            return Err(format!("Component '{}' not found", component_port.0));
        }
        if !self.components.contains_key(&memory_id) {
            return Err(format!("Memory component '{}' not found", memory_id));
        }

        // Add the memory connection
        self.memory_connections.insert(component_port, memory_id);

        Ok(())
    }

    /// Execute one simulation cycle
    pub fn cycle(&mut self) -> Result<(), String> {
        self.current_cycle += 1;

        // Collect all component IDs and their types first to avoid borrowing conflicts
        let memory_components: Vec<ComponentId> = self.components
            .iter()
            .filter_map(|(id, comp)| {
                if comp.module.is_memory() {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        // Execute processing components in topological order
        for component_id in &self.execution_order.clone() {
            self.execute_processing_component(component_id)?;
        }

        // Update memory components
        for component_id in memory_components {
            self.execute_memory_component(&component_id)?;
        }

        Ok(())
    }

    /// Execute a processing component
    fn execute_processing_component(&mut self, component_id: &ComponentId) -> Result<(), String> {
        // First, collect inputs from connected outputs
        let inputs = self.collect_inputs(component_id)?;
        
        // Extract processor info and evaluate function
        let processor = {
            let component = self.components.get(component_id)
                .ok_or_else(|| format!("Component '{}' not found", component_id))?;
            component.module.as_processing()
                .ok_or_else(|| format!("Component '{}' is not a processing component", component_id))?
                .clone()
        };
        
        // Create memory proxy with references to actual memory modules
        let memory_modules: HashMap<ComponentId, &mut dyn MemoryModuleTrait> = self.components
            .iter_mut()
            .filter_map(|(id, comp)| {
                if let Some(memory_module) = comp.module.as_memory_mut() {
                    Some((id.clone(), memory_module))
                } else {
                    None
                }
            })
            .collect();
        
        let mut memory_proxy = MemoryProxy::new(
            self.memory_connections.clone(),
            component_id.clone(),
            memory_modules,
        );
        
        // Create evaluation context
        let mut context = EvaluationContext {
            inputs: &inputs,
            memory: &mut memory_proxy,
            state: None, // Processing components have no state
            component_id,
        };
        
        // Create output map for this component
        let mut outputs = EventOutputMap::new_flexible(self.current_cycle);
        
        // Execute the component's evaluation function
        (processor.evaluate_fn)(&mut context, &mut outputs)?;
        
        // Store outputs in buffer for next cycle
        for (port, event) in outputs.into_event_map() {
            self.output_buffer.insert((component_id.clone(), port), event);
        }
        
        Ok(())
    }

    /// Execute a memory component
    fn execute_memory_component(&mut self, component_id: &ComponentId) -> Result<(), String> {
        let component = self.components.get_mut(component_id)
            .ok_or_else(|| format!("Component '{}' not found", component_id))?;
        
        if let Some(memory_module) = component.module.as_memory_mut() {
            // Update memory state: current → snapshot for next cycle
            memory_module.create_snapshot();
        }
        
        Ok(())
    }

    /// Get the current cycle number
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        self.components.keys().collect()
    }

    /// Check if a component exists
    pub fn has_component(&self, id: &ComponentId) -> bool {
        self.components.contains_key(id)
    }

    /// Build execution order for deterministic simulation
    pub fn build_execution_order(&mut self) -> Result<(), String> {
        // Get all processing component IDs
        let processing_components: Vec<ComponentId> = self.components
            .iter()
            .filter_map(|(id, comp)| {
                if comp.module.is_processing() {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Build topologically sorted execution order
        self.execution_order = ExecutionOrderBuilder::build_execution_order(
            &processing_components,
            &self.connections,
        )?;
        
        Ok(())
    }

    /// Run a single simulation cycle (alias for cycle method)
    pub fn run_cycle(&mut self) -> Result<(), String> {
        self.cycle()
    }
    
    /// Collect inputs for a component from connected outputs
    fn collect_inputs(&self, component_id: &ComponentId) -> Result<EventInputMap, String> {
        let mut inputs = EventInputMap::new();
        
        // Find all connections that target this component
        for ((source_id, source_port), targets) in &self.connections {
            for (target_id, target_port) in targets {
                if target_id == component_id {
                    // Get the output event from the buffer
                    if let Some(event) = self.output_buffer.get(&(source_id.clone(), source_port.clone())) {
                        inputs.insert_event(target_port.clone(), event.clone());
                    }
                }
            }
        }
        
        Ok(inputs)
    }
}