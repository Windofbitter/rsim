use super::super::values::events::Event;
use super::super::values::implementations::{EventInputMap, EventOutputMap};
use super::super::values::traits::EventOutputs;
use super::super::components::manager::ComponentInstance;
use super::super::components::module::{ComponentModule, EvaluationContext};
use super::super::components::registry::ComponentRegistry;
use super::super::connections::manager::ConnectionManager;
use super::execution_order::ExecutionOrderBuilder;
use super::super::memory::proxy::TypeSafeCentralMemoryProxy;
use super::super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    /// Component registry for managing all components
    component_registry: ComponentRegistry,
    /// Connection manager for handling all connections
    connection_manager: ConnectionManager,
    /// Store event outputs from current cycle for next cycle inputs
    current_event_outputs: HashMap<(ComponentId, String), Event>,
    /// Execution order for processing components (topologically sorted)
    execution_order: Vec<ComponentId>,
    /// Current cycle counter
    current_cycle: u64,
}

impl CycleEngine {
    pub fn new() -> Self {
        Self {
            component_registry: ComponentRegistry::new(),
            connection_manager: ConnectionManager::new(),
            current_event_outputs: HashMap::new(),
            execution_order: Vec::new(),
            current_cycle: 0,
        }
    }

    /// Register a component instance
    pub fn register_component(&mut self, instance: ComponentInstance) -> Result<(), String> {
        self.component_registry.register_component(instance)
    }

    /// Connect two component ports
    pub fn connect(
        &mut self,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        self.connection_manager.connect(&self.component_registry, source, target)
    }

    /// Connect component to memory
    pub fn connect_memory(
        &mut self,
        component_id: ComponentId,
        port: String,
        memory_id: ComponentId,
    ) -> Result<(), String> {
        self.connection_manager.connect_memory(&self.component_registry, component_id, port, memory_id)
    }


    /// Build execution order using topological sort
    pub fn build_execution_order(&mut self) -> Result<(), String> {
        // Use the well-designed Kahn's algorithm implementation
        let order = ExecutionOrderBuilder::build_execution_order(
            &self.component_registry,
            self.connection_manager.connections(),
        )?;
        
        self.execution_order = order;
        Ok(())
    }

    /// Run a single simulation cycle
    pub fn run_cycle(&mut self) {
        let mut new_event_outputs: HashMap<(ComponentId, String), Event> = HashMap::new();

        // Execute processing components in topological order
        let execution_order = self.execution_order.clone();
        for comp_id in &execution_order {
            // Gather event inputs from previous cycle outputs
            let mut event_inputs = EventInputMap::new();
            
            // Get the processing module info without holding a borrow
            let (input_ports, _output_ports, evaluate_fn) = {
                if let Some(instance) = self.component_registry.get_component(comp_id) {
                    if let ComponentModule::Processing(proc_module) = &instance.module {
                        (
                            proc_module.input_ports.clone(),
                            proc_module.output_ports.clone(),
                            proc_module.evaluate_fn
                        )
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            };

            for input_port in &input_ports {
                // Look for connections to this input port
                for ((source_id, source_port), targets) in self.connection_manager.connections() {
                    for (target_id, target_port) in targets {
                        if target_id == comp_id && target_port == &input_port.name {
                            // Look for event from previous cycle
                            if let Some(event) = self.current_event_outputs.get(&(source_id.clone(), source_port.clone())) {
                                event_inputs.insert_event(input_port.name.clone(), event.clone());
                            }
                        }
                    }
                }
            }

            // Create type-safe memory proxy - now safe with Rc<RefCell<...>>
            let mut memory_proxy = TypeSafeCentralMemoryProxy::new(
                self.component_registry.memory_modules(),
                self.connection_manager.memory_connections(),
                comp_id.clone(),
            );

            // Get mutable access to the component for state
            if let Some(instance) = self.component_registry.get_component_mut(comp_id) {
                // Create event outputs collector (flexible to allow any type)
                let mut event_outputs = EventOutputMap::new_flexible(self.current_cycle);
                
                // Create evaluation context
                let eval_context = EvaluationContext {
                    inputs: &event_inputs,
                    memory: &mut memory_proxy,
                    state: instance.state_mut(),
                    component_id: comp_id,
                };

                // Evaluate the component
                match evaluate_fn(&eval_context, &mut event_outputs) {
                    Ok(()) => {
                        // Store event outputs for next cycle
                        let output_map = event_outputs.into_event_map();
                        for (port_name, event) in output_map {
                            new_event_outputs.insert(
                                (comp_id.clone(), port_name),
                                event,
                            );
                        }
                    },
                    Err(error) => {
                        eprintln!("Error evaluating component '{}': {}", comp_id, error);
                    }
                }
            }
        }

        // Update event outputs for next cycle
        self.current_event_outputs = new_event_outputs;

        // End cycle on all memory modules
        for memory_module_ref in self.component_registry.memory_modules().values() {
            memory_module_ref.borrow_mut().create_snapshot();
        }

        self.current_cycle += 1;
    }


    /// Get current cycle number
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }

    /// Get execution order
    pub fn execution_order(&self) -> &[ComponentId] {
        &self.execution_order
    }

    /// Get component by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<&ComponentInstance> {
        self.component_registry.get_component(id)
    }

    /// Get mutable component by ID
    pub fn get_component_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentInstance> {
        self.component_registry.get_component_mut(id)
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        self.component_registry.component_ids()
    }

}

impl Default for CycleEngine {
    fn default() -> Self {
        Self::new()
    }
}