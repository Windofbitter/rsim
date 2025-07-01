use super::component::{ProcessingComponent, MemoryComponent, ProbeComponent, EngineMemoryProxy, Event};
use super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    processing_components: HashMap<ComponentId, Box<dyn ProcessingComponent>>,
    memory_components: HashMap<ComponentId, Box<dyn MemoryComponent>>,
    probe_components: HashMap<ComponentId, Box<dyn ProbeComponent>>,
    
    // Memory connections: (component_id, port) -> memory_component_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    
    // Port connections: (source_id, port) -> Vec<(target_id, port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    
    // Store component outputs from previous cycle for current cycle inputs
    pub previous_cycle_outputs: HashMap<(ComponentId, String), Event>,
    
    current_cycle: u64,
}

// Engine's centralized memory proxy
pub struct CentralMemoryProxy<'a> {
    memory_components: &'a mut HashMap<ComponentId, Box<dyn MemoryComponent>>,
    memory_connections: &'a HashMap<(ComponentId, String), ComponentId>,
}

impl<'a> EngineMemoryProxy for CentralMemoryProxy<'a> {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event> {
        let mem_id = self.memory_connections.get(&(component_id.clone(), port.to_string()))?;
        self.memory_components.get(mem_id)?.read_snapshot(address)
    }
    
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event) {
        if let Some(mem_id) = self.memory_connections.get(&(component_id.clone(), port.to_string())) {
            if let Some(memory) = self.memory_components.get_mut(mem_id) {
                memory.write(address, data);
            }
        }
    }
}

impl CycleEngine {
    pub fn new() -> Self {
        Self {
            processing_components: HashMap::new(),
            memory_components: HashMap::new(),
            probe_components: HashMap::new(),
            memory_connections: HashMap::new(),
            connections: HashMap::new(),
            previous_cycle_outputs: HashMap::new(),
            current_cycle: 0,
        }
    }
    
    pub fn register_processing(&mut self, component: Box<dyn ProcessingComponent>) {
        let id = component.component_id().clone();
        self.processing_components.insert(id, component);
    }
    
    pub fn register_memory(&mut self, component: Box<dyn MemoryComponent>) {
        let id = component.component_id().clone();
        self.memory_components.insert(id, component);
    }
    
    pub fn register_probe(&mut self, component: Box<dyn ProbeComponent>) {
        let id = component.component_id().clone();
        self.probe_components.insert(id, component);
    }
    
    pub fn connect_memory(&mut self, proc_id: ComponentId, port: String, mem_id: ComponentId) {
        self.memory_connections.insert((proc_id, port), mem_id);
    }
    
    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) {
        self.connections.entry(source).or_default().push(target);
    }
    
    pub fn run_cycle(&mut self) {
        // 1. Collect current cycle outputs
        let mut current_cycle_outputs: HashMap<(ComponentId, String), Event> = HashMap::new();
        
        // 2. Execute all processing components (one at a time to avoid borrow conflicts)
        let component_ids: Vec<ComponentId> = self.processing_components.keys().cloned().collect();
        
        for comp_id in component_ids {
            // Gather inputs for this component from PREVIOUS cycle outputs
            let mut inputs = HashMap::new();
            
            if let Some(component) = self.processing_components.get(&comp_id) {
                for input_port in component.input_ports() {
                    // Look for connections to this input port
                    for ((source_id, source_port), targets) in &self.connections {
                        for (target_id, target_port) in targets {
                            if target_id == &comp_id && target_port == input_port {
                                // Use previous_cycle_outputs for current cycle inputs
                                if let Some(event) = self.previous_cycle_outputs.get(&(source_id.clone(), source_port.clone())) {
                                    inputs.insert(input_port.to_string(), event.clone());
                                }
                            }
                        }
                    }
                }
                
                // Create proxy for this component evaluation only
                let mut proxy = CentralMemoryProxy {
                    memory_components: &mut self.memory_components,
                    memory_connections: &self.memory_connections,
                };
                
                // Evaluate component with memory proxy access
                let outputs = component.evaluate(&inputs, &mut proxy);
                
                // Store outputs for NEXT cycle
                for output_port in component.output_ports() {
                    if let Some(event) = outputs.get(output_port) {
                        current_cycle_outputs.insert((comp_id.clone(), output_port.to_string()), event.clone());
                    }
                }
                // Proxy is dropped here, releasing the borrow
            }
        }
        
        // 3. Trigger probes for current cycle outputs
        for ((source_id, source_port), event) in &current_cycle_outputs {
            for (probe_id, probe) in &mut self.probe_components {
                probe.probe(source_id, source_port, event);
            }
        }
        
        // 4. Update previous cycle outputs for next cycle
        self.previous_cycle_outputs = current_cycle_outputs;
        
        // 5. End cycle on all memory components (create next snapshot)
        for memory in self.memory_components.values_mut() {
            memory.end_cycle();
        }
        
        self.current_cycle += 1;
    }
    
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }
}