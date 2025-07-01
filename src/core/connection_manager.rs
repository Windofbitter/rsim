use super::component_registry::ComponentRegistry;
use super::types::ComponentId;
use std::collections::HashMap;

/// Manages all connections between components and validates connection rules
pub struct ConnectionManager {
    // Memory connections: (component_id, port) -> memory_component_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,

    // Port connections: (source_id, port) -> Vec<(target_id, port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,

    // Probe connections: (source_id, port) -> Vec<probe_id>
    probes: HashMap<(ComponentId, String), Vec<ComponentId>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            memory_connections: HashMap::new(),
            connections: HashMap::new(),
            probes: HashMap::new(),
        }
    }

    pub fn connect_memory(
        &mut self,
        registry: &ComponentRegistry,
        proc_id: ComponentId,
        port: String,
        mem_id: ComponentId,
    ) -> Result<(), String> {
        // Validate that the processing component exists
        if !registry.has_processing_component(&proc_id) {
            return Err(format!("Processing component '{}' not found", proc_id));
        }

        // Validate that the memory component exists
        if !registry.has_memory_component(&mem_id) {
            return Err(format!("Memory component '{}' not found", mem_id));
        }

        // Validate that the port exists on the processing component
        if let Some(component) = registry.processing_components().get(&proc_id) {
            let valid_ports: Vec<String> = component
                .memory_ports()
                .iter()
                .map(|s| s.to_string())
                .collect();
            if !valid_ports.contains(&port) {
                return Err(format!(
                    "Memory port '{}' not found on component '{}'. Valid ports: {:?}",
                    port, proc_id, valid_ports
                ));
            }
        }

        // Check if this port is already connected to a memory component
        if let Some(existing_mem_id) = self
            .memory_connections
            .get(&(proc_id.clone(), port.clone()))
        {
            return Err(format!(
                "Memory port '{}' on component '{}' is already connected to memory '{}'",
                port, proc_id, existing_mem_id
            ));
        }

        self.memory_connections.insert((proc_id, port), mem_id);
        Ok(())
    }

    pub fn connect(
        &mut self,
        registry: &ComponentRegistry,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        let (source_id, source_port) = &source;
        let (target_id, target_port) = &target;

        // Validate that source component exists (can be processing, memory, or probe)
        if !registry.has_component(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }

        // Validate that target component exists (can be processing, memory, or probe)
        if !registry.has_component(target_id) {
            return Err(format!("Target component '{}' not found", target_id));
        }

        // Validate source port exists
        if let Some(proc_comp) = registry.processing_components().get(source_id) {
            let valid_outputs: Vec<String> = proc_comp
                .output_ports()
                .iter()
                .map(|s| s.to_string())
                .collect();
            if !valid_outputs.contains(source_port) {
                return Err(format!(
                    "Output port '{}' not found on processing component '{}'. Valid ports: {:?}",
                    source_port, source_id, valid_outputs
                ));
            }
        } else if let Some(mem_comp) = registry.memory_components().get(source_id) {
            let valid_output = mem_comp.borrow().output_port();
            if source_port != valid_output {
                return Err(format!(
                    "Output port '{}' not found on memory component '{}'. Valid port: '{}'",
                    source_port, source_id, valid_output
                ));
            }
        }
        // Note: Probe components don't have output ports, so no validation needed

        // Validate target port exists and check for input collision (Bug 3)
        if let Some(proc_comp) = registry.processing_components().get(target_id) {
            let valid_inputs: Vec<String> = proc_comp
                .input_ports()
                .iter()
                .map(|s| s.to_string())
                .collect();
            if !valid_inputs.contains(target_port) {
                return Err(format!(
                    "Input port '{}' not found on processing component '{}'. Valid ports: {:?}",
                    target_port, target_id, valid_inputs
                ));
            }

            // Check for input port collision (Bug 3 fix)
            for existing_targets in self.connections.values() {
                for (existing_target_id, existing_target_port) in existing_targets {
                    if existing_target_id == target_id && existing_target_port == target_port {
                        return Err(format!("Input port '{}' on component '{}' is already connected. Multiple drivers not allowed.", target_port, target_id));
                    }
                }
            }
        } else if let Some(mem_comp) = registry.memory_components().get(target_id) {
            let valid_input = mem_comp.borrow().input_port();
            if target_port != valid_input {
                return Err(format!(
                    "Input port '{}' not found on memory component '{}'. Valid port: '{}'",
                    target_port, target_id, valid_input
                ));
            }

            // Check for input port collision on memory components too
            for existing_targets in self.connections.values() {
                for (existing_target_id, existing_target_port) in existing_targets {
                    if existing_target_id == target_id && existing_target_port == target_port {
                        return Err(format!("Input port '{}' on memory component '{}' is already connected. Multiple drivers not allowed.", target_port, target_id));
                    }
                }
            }
        }
        // Note: Probe components can accept multiple connections for monitoring

        self.connections.entry(source).or_default().push(target);
        Ok(())
    }

    pub fn connect_probe(
        &mut self,
        registry: &ComponentRegistry,
        source_port: (ComponentId, String),
        probe_id: ComponentId,
    ) -> Result<(), String> {
        let (source_id, source_port_name) = &source_port;

        // Validate that source component exists (can be processing, memory, or probe)
        if !registry.has_component(source_id) {
            return Err(format!("Source component '{}' not found", source_id));
        }

        // Validate that probe component exists
        if !registry.has_probe_component(&probe_id) {
            return Err(format!("Probe component '{}' not found", probe_id));
        }

        // Validate source port exists
        if let Some(proc_comp) = registry.processing_components().get(source_id) {
            let valid_outputs: Vec<String> = proc_comp
                .output_ports()
                .iter()
                .map(|s| s.to_string())
                .collect();
            if !valid_outputs.contains(source_port_name) {
                return Err(format!(
                    "Output port '{}' not found on processing component '{}'. Valid ports: {:?}",
                    source_port_name, source_id, valid_outputs
                ));
            }
        } else if let Some(mem_comp) = registry.memory_components().get(source_id) {
            let valid_output = mem_comp.borrow().output_port();
            if source_port_name != valid_output {
                return Err(format!(
                    "Output port '{}' not found on memory component '{}'. Valid port: '{}'",
                    source_port_name, source_id, valid_output
                ));
            }
        }
        // Note: Probe components can also be sources for other probes

        self.probes.entry(source_port).or_default().push(probe_id);
        Ok(())
    }

    // Getters for connection data
    pub fn memory_connections(&self) -> &HashMap<(ComponentId, String), ComponentId> {
        &self.memory_connections
    }

    pub fn connections(&self) -> &HashMap<(ComponentId, String), Vec<(ComponentId, String)>> {
        &self.connections
    }

    pub fn probes(&self) -> &HashMap<(ComponentId, String), Vec<ComponentId>> {
        &self.probes
    }
}