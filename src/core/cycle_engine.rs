use super::typed_values::{TypedValue, TypedInputMap, TypedOutputMap, TypedOutputs};
use super::component_manager::ComponentInstance;
use super::component_module::{ComponentModule, EvaluationContext, MemoryModuleTrait};
use super::memory_proxy::TypeSafeCentralMemoryProxy;
use super::types::ComponentId;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct CycleEngine {
    /// Module-based components
    components: HashMap<ComponentId, ComponentInstance>,
    /// Memory modules
    memory_modules: HashMap<ComponentId, RefCell<Box<dyn MemoryModuleTrait>>>,
    /// Component connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
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
            components: HashMap::new(),
            memory_modules: HashMap::new(),
            connections: HashMap::new(),
            memory_connections: HashMap::new(),
            current_typed_outputs: HashMap::new(),
            execution_order: Vec::new(),
            current_cycle: 0,
        }
    }

    /// Register a component instance
    pub fn register_component(&mut self, instance: ComponentInstance) -> Result<(), String> {
        let id = instance.id().clone();
        
        // Separate memory modules from other components
        if instance.is_memory() {
            if let ComponentModule::Memory(memory_module) = &instance.module {
                self.memory_modules.insert(id.clone(), RefCell::new(memory_module.clone_module()));
            }
        }
        
        self.components.insert(id, instance);
        Ok(())
    }

    /// Connect two component ports
    pub fn connect(
        &mut self,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        // Validate components exist
        if !self.components.contains_key(&source.0) {
            return Err(format!("Source component '{}' not found", source.0));
        }
        if !self.components.contains_key(&target.0) {
            return Err(format!("Target component '{}' not found", target.0));
        }

        // Validate ports
        self.validate_source_port(&source.0, &source.1)?;
        self.validate_target_port(&target.0, &target.1)?;

        // Check for input port collision
        for targets in self.connections.values() {
            for existing_target in targets {
                if existing_target == &target {
                    return Err(format!(
                        "Input port '{}' on component '{}' already connected",
                        target.1, target.0
                    ));
                }
            }
        }

        self.connections.entry(source).or_default().push(target);
        Ok(())
    }

    /// Connect component to memory
    pub fn connect_memory(
        &mut self,
        component_id: ComponentId,
        port: String,
        memory_id: ComponentId,
    ) -> Result<(), String> {
        // Validate components exist
        if !self.components.contains_key(&component_id) {
            return Err(format!("Component '{}' not found", component_id));
        }
        if !self.memory_modules.contains_key(&memory_id) && !self.components.contains_key(&memory_id) {
            return Err(format!("Memory component '{}' not found", memory_id));
        }

        // Validate memory port exists
        self.validate_memory_port(&component_id, &port)?;

        let port_key = (component_id, port);
        if self.memory_connections.contains_key(&port_key) {
            return Err(format!(
                "Memory port already connected for component '{}'",
                port_key.0
            ));
        }

        self.memory_connections.insert(port_key, memory_id);
        Ok(())
    }


    /// Build execution order using topological sort
    pub fn build_execution_order(&mut self) -> Result<(), String> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        // Get all processing components
        let processing_components: Vec<ComponentId> = self.components
            .iter()
            .filter(|(_, instance)| instance.is_processing())
            .map(|(id, _)| id.clone())
            .collect();

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
        for ((source_id, _), targets) in &self.connections {
            for (target_id, _) in targets {
                if target_id == component_id && source_id != component_id {
                    if let Some(source_instance) = self.components.get(source_id) {
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
                if let Some(instance) = self.components.get(comp_id) {
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
                for ((source_id, source_port), targets) in &self.connections {
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

            // Create type-safe memory proxy
            let mut memory_proxy = TypeSafeCentralMemoryProxy::new(
                &self.memory_modules,
                &self.memory_connections,
                comp_id.clone(),
            );

            // Get mutable access to the component for state
            if let Some(instance) = self.components.get_mut(comp_id) {
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
        for memory_module_ref in self.memory_modules.values() {
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
        self.components.get(id)
    }

    /// Get mutable component by ID
    pub fn get_component_mut(&mut self, id: &ComponentId) -> Option<&mut ComponentInstance> {
        self.components.get_mut(id)
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        self.components.keys().collect()
    }

    /// Helper method to validate source port exists
    fn validate_source_port(&self, component_id: &str, port: &str) -> Result<(), String> {
        let instance = &self.components[component_id];
        
        match &instance.module {
            ComponentModule::Processing(proc_module) => {
                if !proc_module.has_output_port(port) {
                    return Err(format!(
                        "Output port '{}' not found on processing component '{}'. Valid ports: {:?}",
                        port, component_id, proc_module.output_port_names()
                    ));
                }
            },
            ComponentModule::Memory(_) => {
                if port != "out" {
                    return Err(format!(
                        "Output port '{}' not found on memory component '{}'. Valid port: 'out'",
                        port, component_id
                    ));
                }
            },
        }
        
        Ok(())
    }

    /// Helper method to validate target port exists
    fn validate_target_port(&self, component_id: &str, port: &str) -> Result<(), String> {
        let instance = &self.components[component_id];
        
        match &instance.module {
            ComponentModule::Processing(proc_module) => {
                if !proc_module.has_input_port(port) {
                    return Err(format!(
                        "Input port '{}' not found on processing component '{}'. Valid ports: {:?}",
                        port, component_id, proc_module.input_port_names()
                    ));
                }
            },
            ComponentModule::Memory(_) => {
                if port != "in" {
                    return Err(format!(
                        "Input port '{}' not found on memory component '{}'. Valid port: 'in'",
                        port, component_id
                    ));
                }
            },
        }
        
        Ok(())
    }

    /// Helper method to validate memory port exists
    fn validate_memory_port(&self, component_id: &str, port: &str) -> Result<(), String> {
        let instance = &self.components[component_id];
        
        if let ComponentModule::Processing(proc_module) = &instance.module {
            if !proc_module.has_memory_port(port) {
                return Err(format!(
                    "Memory port '{}' not found on component '{}'. Valid ports: {:?}",
                    port, component_id, proc_module.memory_port_names()
                ));
            }
        } else {
            return Err(format!(
                "Component '{}' is not a processing component and cannot have memory ports",
                component_id
            ));
        }
        
        Ok(())
    }
}

impl Default for CycleEngine {
    fn default() -> Self {
        Self::new()
    }
}