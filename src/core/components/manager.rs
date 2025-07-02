use crate::core::components::module::ComponentModule;
use crate::core::components::state::ComponentState;
use crate::core::types::ComponentId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Manages component module templates and creates component instances
pub struct ComponentManager {
    /// Registry of component module templates by name
    module_templates: HashMap<String, ComponentModule>,
    /// Counter for automatic ID generation
    id_counter: AtomicU64,
}

/// Represents an instantiated component created from a module template
pub struct ComponentInstance {
    /// Unique component ID
    pub id: ComponentId,
    /// The component module (cloned from template)
    pub module: ComponentModule,
    /// Optional component state
    pub state: Option<Box<dyn ComponentState>>,
}

impl ComponentManager {
    /// Create a new component manager
    pub fn new() -> Self {
        Self {
            module_templates: HashMap::new(),
            id_counter: AtomicU64::new(0),
        }
    }

    /// Register a component module template
    pub fn register_module(&mut self, name: &str, module: ComponentModule) -> Result<(), String> {
        if self.module_templates.contains_key(name) {
            return Err(format!("Component module '{}' is already registered", name));
        }
        
        self.module_templates.insert(name.to_string(), module);
        Ok(())
    }

    /// Create a component instance with a specific ID
    pub fn create_component(&self, module_name: &str, id_string: String) -> Result<ComponentInstance, String> {
        let template = self.module_templates.get(module_name)
            .ok_or_else(|| format!("Component module '{}' not found", module_name))?;

        let module = self.clone_module(template)?;
        
        // Components don't currently support state factories
        let state = None;
        
        let component_id = ComponentId::new(id_string, module_name.to_string());

        Ok(ComponentInstance {
            id: component_id,
            module,
            state,
        })
    }

    /// Create a component instance with an automatically generated ID
    pub fn create_component_auto_id(&self, module_name: &str) -> Result<ComponentInstance, String> {
        let id = self.generate_unique_id(module_name);
        self.create_component(module_name, id.id().to_string())
    }

    /// Generate a unique component ID
    pub fn generate_unique_id(&self, module_name: &str) -> ComponentId {
        let counter = self.id_counter.fetch_add(1, Ordering::SeqCst);
        let id_string = format!("{}_{}", module_name, counter);
        ComponentId::new(id_string, module_name.to_string())
    }

    /// Check if a module template exists
    pub fn has_module(&self, name: &str) -> bool {
        self.module_templates.contains_key(name)
    }

    /// Get the names of all registered modules
    pub fn module_names(&self) -> Vec<String> {
        self.module_templates.keys().cloned().collect()
    }

    /// Get a reference to a module template
    pub fn get_module(&self, name: &str) -> Option<&ComponentModule> {
        self.module_templates.get(name)
    }

    /// Clone a component module (handles different module types appropriately)
    fn clone_module(&self, template: &ComponentModule) -> Result<ComponentModule, String> {
        match template {
            ComponentModule::Processing(proc_module) => {
                Ok(ComponentModule::Processing(proc_module.clone()))
            },
            ComponentModule::Memory(mem_module) => {
                Ok(ComponentModule::Memory(mem_module.clone_module()))
            },
        }
    }

    /// Create multiple components of the same type with auto-generated IDs
    pub fn create_components(&self, module_name: &str, count: usize) -> Result<Vec<ComponentInstance>, String> {
        let mut components = Vec::new();
        for _ in 0..count {
            components.push(self.create_component_auto_id(module_name)?);
        }
        Ok(components)
    }

    /// Create components with custom ID prefixes
    pub fn create_components_with_prefix(&self, module_name: &str, prefix: &str, count: usize) -> Result<Vec<ComponentInstance>, String> {
        let mut components = Vec::new();
        for i in 0..count {
            let id_string = format!("{}_{}", prefix, i);
            components.push(self.create_component(module_name, id_string)?);
        }
        Ok(components)
    }

    /// Validate that all required ports are satisfied for a component type
    pub fn validate_component_ports(&self, module_name: &str, connected_ports: &[String]) -> Result<(), String> {
        let template = self.module_templates.get(module_name)
            .ok_or_else(|| format!("Component module '{}' not found", module_name))?;

        match template {
            ComponentModule::Processing(proc_module) => {
                // Check that all required input ports are connected
                for port_spec in &proc_module.input_ports {
                    if port_spec.required && !connected_ports.contains(&port_spec.name) {
                        return Err(format!(
                            "Required input port '{}' is not connected for component type '{}'",
                            port_spec.name, module_name
                        ));
                    }
                }
            },
            ComponentModule::Memory(_) => {
                // Memory components typically don't have required connection validation
            },
        }

        Ok(())
    }

    /// Get statistics about registered modules
    pub fn get_module_stats(&self) -> ComponentManagerStats {
        let mut processing_count = 0;
        let mut memory_count = 0;

        for module in self.module_templates.values() {
            match module {
                ComponentModule::Processing(_) => processing_count += 1,
                ComponentModule::Memory(_) => memory_count += 1,
            }
        }

        ComponentManagerStats {
            total_modules: self.module_templates.len(),
            processing_modules: processing_count,
            memory_modules: memory_count,
            components_created: self.id_counter.load(Ordering::SeqCst),
        }
    }
}

/// Statistics about the component manager state
#[derive(Debug, Clone)]
pub struct ComponentManagerStats {
    pub total_modules: usize,
    pub processing_modules: usize,
    pub memory_modules: usize,
    pub components_created: u64,
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentInstance {
    /// Get the component ID
    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    /// Get the module name
    pub fn module_name(&self) -> &str {
        self.module.name()
    }

    /// Check if this is a processing component
    pub fn is_processing(&self) -> bool {
        self.module.is_processing()
    }

    /// Check if this is a memory component
    pub fn is_memory(&self) -> bool {
        self.module.is_memory()
    }


    /// Get mutable access to the component state
    pub fn state_mut(&mut self) -> Option<&mut dyn ComponentState> {
        match self.state.as_mut() {
            Some(s) => Some(s.as_mut()),
            None => None,
        }
    }

    /// Get immutable access to the component state
    pub fn state(&self) -> Option<&dyn ComponentState> {
        self.state.as_ref().map(|s| s.as_ref())
    }
}