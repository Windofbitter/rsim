use super::component::{ProcessingComponent, MemoryComponent, ProbeComponent, EngineMemoryProxy, Event};
use super::types::ComponentId;
use std::collections::{HashMap, VecDeque};
use std::cell::RefCell;

pub struct CycleEngine {
    processing_components: HashMap<ComponentId, Box<dyn ProcessingComponent>>,
    memory_components: HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>>,
    probe_components: HashMap<ComponentId, Box<dyn ProbeComponent>>,
    
    // Memory connections: (component_id, port) -> memory_component_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    
    // Port connections: (source_id, port) -> Vec<(target_id, port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    
    // Store component outputs from previous cycle for current cycle inputs
    pub previous_cycle_outputs: HashMap<(ComponentId, String), Event>,
    
    // Topologically sorted execution order for processing components
    execution_order: Vec<ComponentId>,
    
    current_cycle: u64,
}

// Engine's centralized memory proxy
pub struct CentralMemoryProxy<'a> {
    memory_components: &'a HashMap<ComponentId, RefCell<Box<dyn MemoryComponent>>>,
    memory_connections: &'a HashMap<(ComponentId, String), ComponentId>,
}

impl<'a> EngineMemoryProxy for CentralMemoryProxy<'a> {
    fn read(&self, component_id: &ComponentId, port: &str, address: &str) -> Option<Event> {
        let mem_id = self.memory_connections.get(&(component_id.clone(), port.to_string()))?;
        let memory_ref = self.memory_components.get(mem_id)?;
        memory_ref.borrow().read_snapshot(address)
    }
    
    fn write(&mut self, component_id: &ComponentId, port: &str, address: &str, data: Event) {
        if let Some(mem_id) = self.memory_connections.get(&(component_id.clone(), port.to_string())) {
            if let Some(memory_ref) = self.memory_components.get(mem_id) {
                memory_ref.borrow_mut().write(address, data);
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
            execution_order: Vec::new(),
            current_cycle: 0,
        }
    }
    
    pub fn register_processing(&mut self, component: Box<dyn ProcessingComponent>) {
        let id = component.component_id().clone();
        self.processing_components.insert(id, component);
    }
    
    pub fn register_memory(&mut self, component: Box<dyn MemoryComponent>) {
        let id = component.component_id().clone();
        self.memory_components.insert(id, RefCell::new(component));
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
    
    /// Analyzes the graph of processing components to build a topologically sorted execution order.
    /// Uses Kahn's algorithm to detect cycles and ensure deterministic execution.
    pub fn build_execution_order(&mut self) -> Result<(), String> {
        let mut adj_list: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
        let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();
        
        // Initialize graph data structures for all processing components
        for comp_id in self.processing_components.keys() {
            in_degree.insert(comp_id.clone(), 0);
            adj_list.insert(comp_id.clone(), Vec::new());
        }
        
        // Build adjacency list and in-degrees from connections
        for ((source_id, _source_port), targets) in &self.connections {
            // Only consider connections between processing components
            if !self.processing_components.contains_key(source_id) {
                continue;
            }
            
            for (target_id, _target_port) in targets {
                if !self.processing_components.contains_key(target_id) {
                    continue;
                }
                
                // Add edge from source to target
                adj_list.get_mut(source_id).unwrap().push(target_id.clone());
                *in_degree.get_mut(target_id).unwrap() += 1;
            }
        }
        
        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<ComponentId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        // Sort queue for deterministic results when multiple components have zero in-degree
        let mut queue_vec: Vec<ComponentId> = queue.into_iter().collect();
        queue_vec.sort();
        queue = queue_vec.into();
        
        let mut sorted_order = Vec::new();
        
        while let Some(u) = queue.pop_front() {
            sorted_order.push(u.clone());
            
            if let Some(neighbors) = adj_list.get(&u) {
                let mut new_zero_degree = Vec::new();
                for v in neighbors {
                    if let Some(degree) = in_degree.get_mut(v) {
                        *degree -= 1;
                        if *degree == 0 {
                            new_zero_degree.push(v.clone());
                        }
                    }
                }
                // Sort before adding to queue for deterministic ordering
                new_zero_degree.sort();
                for v in new_zero_degree {
                    queue.push_back(v);
                }
            }
        }
        
        // Check for cycles
        if sorted_order.len() == self.processing_components.len() {
            self.execution_order = sorted_order;
            Ok(())
        } else {
            Err("Cycle detected in processing component dependencies".to_string())
        }
    }
    
    pub fn run_cycle(&mut self) {
        // 1. Collect current cycle outputs
        let mut current_cycle_outputs: HashMap<(ComponentId, String), Event> = HashMap::new();
        
        // 2. Execute all processing components in topological order
        for comp_id in &self.execution_order {
            // Gather inputs for this component from PREVIOUS cycle outputs
            let mut inputs = HashMap::new();
            
            if let Some(component) = self.processing_components.get(comp_id) {
                for input_port in component.input_ports() {
                    // Look for connections to this input port
                    for ((source_id, source_port), targets) in &self.connections {
                        for (target_id, target_port) in targets {
                            if target_id == comp_id && target_port == input_port {
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
                    memory_components: &self.memory_components,
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
        for memory_ref in self.memory_components.values() {
            memory_ref.borrow_mut().end_cycle();
        }
        
        self.current_cycle += 1;
    }
    
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }
    
    /// Returns the current topological execution order for debugging/inspection
    pub fn execution_order(&self) -> &[ComponentId] {
        &self.execution_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{ProcessingComponent, BaseComponent};
    use std::collections::HashMap;
    
    // Simple test component for topological sorting tests
    struct TestComponent {
        id: ComponentId,
        inputs: Vec<&'static str>,
        outputs: Vec<&'static str>,
    }
    
    impl TestComponent {
        fn new(id: &str, inputs: Vec<&'static str>, outputs: Vec<&'static str>) -> Self {
            Self {
                id: id.to_string(),
                inputs,
                outputs,
            }
        }
    }
    
    impl BaseComponent for TestComponent {
        fn component_id(&self) -> &ComponentId {
            &self.id
        }
    }
    
    impl ProcessingComponent for TestComponent {
        fn input_ports(&self) -> Vec<&'static str> {
            self.inputs.clone()
        }
        
        fn output_ports(&self) -> Vec<&'static str> {
            self.outputs.clone()
        }
        
        fn evaluate(&self, _inputs: &HashMap<String, Event>, _memory_proxy: &mut dyn EngineMemoryProxy) -> HashMap<String, Event> {
            HashMap::new()
        }
    }
    
    #[test]
    fn test_topological_sort_simple_chain() {
        let mut engine = CycleEngine::new();
        
        // Create components: A -> B -> C
        engine.register_processing(Box::new(TestComponent::new("A", vec![], vec!["out"])));
        engine.register_processing(Box::new(TestComponent::new("B", vec!["in"], vec!["out"])));
        engine.register_processing(Box::new(TestComponent::new("C", vec!["in"], vec![])));
        
        // Connect A -> B -> C
        engine.connect(("A".to_string(), "out".to_string()), ("B".to_string(), "in".to_string()));
        engine.connect(("B".to_string(), "out".to_string()), ("C".to_string(), "in".to_string()));
        
        // Build execution order
        let result = engine.build_execution_order();
        assert!(result.is_ok(), "Expected successful topological sort");
        
        let order = engine.execution_order();
        assert_eq!(order.len(), 3);
        
        // A should come before B, B should come before C
        let a_pos = order.iter().position(|x| x == "A").unwrap();
        let b_pos = order.iter().position(|x| x == "B").unwrap();
        let c_pos = order.iter().position(|x| x == "C").unwrap();
        
        assert!(a_pos < b_pos, "A should execute before B");
        assert!(b_pos < c_pos, "B should execute before C");
    }
    
    #[test]
    fn test_topological_sort_cycle_detection() {
        let mut engine = CycleEngine::new();
        
        // Create components with cycle: A -> B -> A
        engine.register_processing(Box::new(TestComponent::new("A", vec!["in"], vec!["out"])));
        engine.register_processing(Box::new(TestComponent::new("B", vec!["in"], vec!["out"])));
        
        // Create cycle
        engine.connect(("A".to_string(), "out".to_string()), ("B".to_string(), "in".to_string()));
        engine.connect(("B".to_string(), "out".to_string()), ("A".to_string(), "in".to_string()));
        
        // Build execution order should fail
        let result = engine.build_execution_order();
        assert!(result.is_err(), "Expected cycle detection error");
        assert!(result.unwrap_err().contains("Cycle detected"));
    }
    
    #[test]
    fn test_topological_sort_no_connections() {
        let mut engine = CycleEngine::new();
        
        // Create isolated components
        engine.register_processing(Box::new(TestComponent::new("A", vec![], vec![])));
        engine.register_processing(Box::new(TestComponent::new("B", vec![], vec![])));
        engine.register_processing(Box::new(TestComponent::new("C", vec![], vec![])));
        
        // Build execution order
        let result = engine.build_execution_order();
        assert!(result.is_ok(), "Expected successful topological sort");
        
        let order = engine.execution_order();
        assert_eq!(order.len(), 3);
        
        // Should be in alphabetical order for determinism
        assert_eq!(order, &["A", "B", "C"]);
    }
}