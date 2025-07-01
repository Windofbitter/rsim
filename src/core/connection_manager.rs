use super::component::{Component, Signal};
use super::types::ComponentId;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct ConnectionManager {
    pub components: HashMap<ComponentId, Component>,
    // Active data-flow connections: (Source Component, Source Port) -> Vec<(Target Component, Target Port)>
    pub connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    // Passive monitoring probes: (Source Component, Source Port) -> Vec<ProbeComponentId>
    pub probes: HashMap<(ComponentId, String), Vec<ComponentId>>,
    // The calculated, safe execution order for combinational components.
    pub combinational_order: Vec<ComponentId>,
    // The list of all sequential components.
    pub sequential_ids: Vec<ComponentId>,
    
    // NEW: Reverse mapping for efficient input gathering
    // Maps (target_component, input_port) -> (source_component, source_port)
    input_sources: HashMap<(ComponentId, String), (ComponentId, String)>,
    
    // NEW: Signal storage for current cycle
    pub current_signals: HashMap<(ComponentId, String), Signal>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            connections: HashMap::new(),
            probes: HashMap::new(),
            combinational_order: Vec::new(),
            sequential_ids: Vec::new(),
            input_sources: HashMap::new(),
            current_signals: HashMap::new(),
        }
    }

    pub fn register_component(&mut self, component: Component) {
        let id = component.as_base().component_id().clone();
        if let Component::Sequential(_) = &component {
            self.sequential_ids.push(id.clone());
        }
        self.components.insert(id, component);
    }

    pub fn connect(&mut self, source: (ComponentId, String), target: (ComponentId, String)) -> Result<(), String> {
        // Validate that input port doesn't already have a source
        if self.input_sources.contains_key(&target) {
            return Err(format!("Input port {:?} already connected to a source", target));
        }
        
        // Validate that both components exist
        if !self.components.contains_key(&source.0) {
            return Err(format!("Source component {} does not exist", source.0));
        }
        if !self.components.contains_key(&target.0) {
            return Err(format!("Target component {} does not exist", target.0));
        }
        
        self.connections.entry(source.clone()).or_default().push(target.clone());
        self.input_sources.insert(target, source);
        Ok(())
    }

    pub fn add_probe(&mut self, source_port: (ComponentId, String), probe_id: ComponentId) -> Result<(), String> {
        // Validate that source component exists
        if !self.components.contains_key(&source_port.0) {
            return Err(format!("Source component {} does not exist", source_port.0));
        }
        
        // Validate that probe component exists and is a probe
        match self.components.get(&probe_id) {
            Some(Component::Probe(_)) => {},
            Some(_) => return Err(format!("Component {} is not a probe component", probe_id)),
            None => return Err(format!("Probe component {} does not exist", probe_id)),
        }
        
        self.probes.entry(source_port).or_default().push(probe_id);
        Ok(())
    }

    /// Analyzes the graph of combinational components to find a safe execution order.
    /// This is a topological sort. It will return an error if a cycle is detected.
    pub fn build_evaluation_order(&mut self) -> Result<(), String> {
        let mut adj_list = HashMap::new();
        let mut in_degree = HashMap::new();
        let mut combinational_ids = HashSet::new();

        // Initialize graph data structures
        for (id, comp) in &self.components {
            if let Component::Combinational(_) = comp {
                combinational_ids.insert(id.clone());
                in_degree.entry(id.clone()).or_insert(0);
                adj_list.entry(id.clone()).or_insert_with(Vec::new);
            }
        }

        // Build adjacency list and in-degrees from connections
        for (source, targets) in &self.connections {
            let (source_id, _) = source;
            if !combinational_ids.contains(source_id) { continue; }

            for (target_id, _) in targets {
                if !combinational_ids.contains(target_id) { continue; }
                adj_list.get_mut(source_id).unwrap().push(target_id.clone());
                *in_degree.entry(target_id.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<ComponentId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut sorted_order = Vec::new();
        while let Some(u) = queue.pop_front() {
            sorted_order.push(u.clone());
            if let Some(neighbors) = adj_list.get(&u) {
                for v in neighbors {
                    if let Some(degree) = in_degree.get_mut(v) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(v.clone());
                        }
                    }
                }
            }
        }

        if sorted_order.len() == combinational_ids.len() {
            self.combinational_order = sorted_order;
            Ok(())
        } else {
            Err("Combinational cycle detected in component graph.".to_string())
        }
    }
    
    /// Builds reverse mapping for efficient input signal gathering
    /// Note: This method is only needed if connections were made before the mapping was built
    /// Normally, input_sources is maintained by the connect() method
    pub fn build_input_mapping(&mut self) {
        self.input_sources.clear();
        
        for ((source_id, source_port), targets) in &self.connections {
            for (target_id, target_port) in targets {
                // Check for duplicate input mappings (should not happen with new validation)
                if let Some(existing) = self.input_sources.get(&(target_id.clone(), target_port.clone())) {
                    panic!("Duplicate input mapping found: ({}, {}) already connected to {:?}, cannot also connect to ({}, {})", 
                           target_id, target_port, existing, source_id, source_port);
                }
                
                self.input_sources.insert(
                    (target_id.clone(), target_port.clone()),
                    (source_id.clone(), source_port.clone())
                );
            }
        }
    }
    
    /// Gathers input signals for a component based on its input ports
    pub fn gather_inputs(&self, component_id: &ComponentId, input_ports: &[&str]) 
        -> HashMap<String, Signal> {
        let mut inputs = HashMap::new();
        
        for port in input_ports {
            if let Some((source_id, source_port)) = 
                self.input_sources.get(&(component_id.clone(), port.to_string())) {
                
                if let Some(signal) = self.current_signals.get(&(source_id.clone(), source_port.clone())) {
                    // Clone the signal for this component
                    inputs.insert(port.to_string(), signal.clone());
                }
            }
        }
        
        inputs
    }
    
    /// Publishes a signal and triggers all associated probes
    pub fn publish_signal(&mut self, source: (ComponentId, String), signal: Signal) {
        // Store signal for input gathering
        self.current_signals.insert(source.clone(), signal.clone());
        
        // Trigger all probes for this output port
        if let Some(probe_ids) = self.probes.get(&source) {
            for probe_id in probe_ids {
                if let Some(Component::Probe(probe)) = self.components.get_mut(probe_id) {
                    probe.probe(&signal);
                }
            }
        }
    }
}