use super::typed_values::{TypedValue, TypedInputMap, TypedOutputMap, TypedOutputs};
use super::component_manager::ComponentInstance;
use super::component_module::{ComponentModule, EvaluationContext};
use super::component_registry::{ComponentRegistry, ComponentType};
use super::connection_manager::ConnectionManager;
use super::memory_proxy::TypeSafeCentralMemoryProxy;
use super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    /// Component registry for managing all components
    component_registry: ComponentRegistry,
    /// Connection manager for handling all connections
    connection_manager: ConnectionManager,
    /// Store typed outputs from current cycle for next cycle inputs
    current_typed_outputs: HashMap<(ComponentId, String), TypedValue>,
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
            current_typed_outputs: HashMap::new(),
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
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        // Get all processing components
        let processing_components: Vec<ComponentId> = self.component_registry.component_ids_by_type(ComponentType::Processing);

        // Perform DFS-based topological sort
        for component_id in &processing_components {
            if !visited.contains(component_id) {
                self.topological_sort_visit(
                    component_id,
                    &mut visited,
                    &mut temp_visited,
                    &mut order,
                )?;
            }
        }

        order.reverse(); // Reverse to get correct topological order
        self.execution_order = order;
        Ok(())
    }

    fn topological_sort_visit(
        &self,
        component_id: &ComponentId,
        visited: &mut std::collections::HashSet<ComponentId>,
        temp_visited: &mut std::collections::HashSet<ComponentId>,
        order: &mut Vec<ComponentId>,
    ) -> Result<(), String> {
        if temp_visited.contains(component_id) {
            return Err(format!("Cycle detected involving component '{}'", component_id));
        }
        if visited.contains(component_id) {
            return Ok(());
        }

        temp_visited.insert(component_id.clone());

        // Visit all components that this component depends on (provides input to this component)
        for ((source_id, _), targets) in self.connection_manager.connections() {
            for (target_id, _) in targets {
                if target_id == component_id && source_id != component_id {
                    if let Some(source_instance) = self.component_registry.get_component(source_id) {
                        if source_instance.is_processing() {
                            self.topological_sort_visit(source_id, visited, temp_visited, order)?;
                        }
                    }
                }
            }
        }

        temp_visited.remove(component_id);
        visited.insert(component_id.clone());
        order.push(component_id.clone());

        Ok(())
    }

    /// Run a single simulation cycle
    pub fn run_cycle(&mut self) {
        let mut new_typed_outputs: HashMap<(ComponentId, String), TypedValue> = HashMap::new();

        // Execute processing components in topological order
        let execution_order = self.execution_order.clone();
        for comp_id in &execution_order {
            // Gather typed inputs from previous cycle outputs
            let mut typed_inputs = TypedInputMap::new();
            
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
                            // Look for typed value from previous cycle
                            if let Some(typed_value) = self.current_typed_outputs.get(&(source_id.clone(), source_port.clone())) {
                                typed_inputs.insert_typed(input_port.name.clone(), typed_value.clone());
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
                // Create typed outputs collector (flexible to allow any type)
                let mut typed_outputs = TypedOutputMap::new_flexible();
                
                // Create evaluation context
                let eval_context = EvaluationContext {
                    inputs: &typed_inputs,
                    memory: &mut memory_proxy,
                    state: instance.state_mut(),
                    component_id: comp_id,
                };

                // Evaluate the component
                match evaluate_fn(&eval_context, &mut typed_outputs) {
                    Ok(()) => {
                        // Store typed outputs for next cycle
                        let output_map = typed_outputs.into_map();
                        for (port_name, typed_value) in output_map {
                            new_typed_outputs.insert(
                                (comp_id.clone(), port_name),
                                typed_value,
                            );
                        }
                    },
                    Err(error) => {
                        eprintln!("Error evaluating component '{}': {}", comp_id, error);
                    }
                }
            }
        }

        // Update typed outputs for next cycle
        self.current_typed_outputs = new_typed_outputs;

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