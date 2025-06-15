use crate::core::event::EventType;
use crate::core::types::ComponentId;
use crate::analysis::dependency_graph::DependencyGraph;
use std::collections::HashMap;

/// Tracks event flows during simulation for dependency graph weighting
#[derive(Debug)]
pub struct ProfilingCollector {
    /// Counts events flowing from source to target components
    /// Key: (source_id, target_id, event_type)
    /// Value: count of events
    event_counts: HashMap<(ComponentId, ComponentId, EventType), u64>,
    
    /// Whether profiling is currently active
    enabled: bool,
}

impl ProfilingCollector {
    /// Create a new profiling collector
    pub fn new() -> Self {
        Self {
            event_counts: HashMap::new(),
            enabled: false,
        }
    }
    
    /// Enable profiling collection
    pub fn enable(&mut self) {
        self.enabled = true;
        self.event_counts.clear();
    }
    
    /// Disable profiling collection
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Record an event flow from source to target
    pub fn record_event_flow(
        &mut self, 
        source_id: &ComponentId,
        target_id: &ComponentId, 
        event_type: &str
    ) {
        if !self.enabled {
            return;
        }
        
        let key = (source_id.clone(), target_id.clone(), event_type.to_string());
        *self.event_counts.entry(key).or_insert(0) += 1;
    }
    
    /// Apply collected profiling data to weight a dependency graph
    pub fn apply_weights_to_graph(&self, graph: &mut DependencyGraph) {
        for ((source_id, target_id, event_type), count) in &self.event_counts {
            graph.update_edge_weight(source_id, target_id, event_type, *count);
        }
    }
    
    /// Get total number of events recorded
    pub fn total_events(&self) -> u64 {
        self.event_counts.values().sum()
    }
    
    /// Get counts for a specific source component
    pub fn get_source_counts(&self, source_id: &ComponentId) -> Vec<(&ComponentId, &EventType, u64)> {
        self.event_counts
            .iter()
            .filter(|((src, _, _), _)| src == source_id)
            .map(|((_, target, event_type), count)| (target, event_type, *count))
            .collect()
    }
    
    /// Get counts for a specific target component
    pub fn get_target_counts(&self, target_id: &ComponentId) -> Vec<(&ComponentId, &EventType, u64)> {
        self.event_counts
            .iter()
            .filter(|((_, tgt, _), _)| tgt == target_id)
            .map(|((source, event_type, _), count)| (source, event_type, *count))
            .collect()
    }
    
    /// Get the heaviest communication paths
    pub fn get_heaviest_paths(&self, limit: usize) -> Vec<(ComponentId, ComponentId, EventType, u64)> {
        let mut paths: Vec<_> = self.event_counts
            .iter()
            .map(|((src, tgt, evt), count)| (src.clone(), tgt.clone(), evt.clone(), *count))
            .collect();
        
        paths.sort_by(|a, b| b.3.cmp(&a.3));
        paths.truncate(limit);
        paths
    }
    
    /// Generate a summary report of the profiling data
    pub fn generate_report(&self) -> ProfilingReport {
        let total_events = self.total_events();
        let unique_flows = self.event_counts.len();
        let heaviest_paths = self.get_heaviest_paths(10);
        
        let mut component_activity = HashMap::new();
        for ((source, target, _), count) in &self.event_counts {
            *component_activity.entry(source.clone()).or_insert(0u64) += count;
            *component_activity.entry(target.clone()).or_insert(0u64) += count;
        }
        
        let mut busiest_components: Vec<_> = component_activity.into_iter().collect();
        busiest_components.sort_by(|a, b| b.1.cmp(&a.1));
        busiest_components.truncate(10);
        
        ProfilingReport {
            total_events,
            unique_flows,
            heaviest_paths,
            busiest_components,
        }
    }
}

impl Default for ProfilingCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary report of profiling data
#[derive(Debug)]
pub struct ProfilingReport {
    pub total_events: u64,
    pub unique_flows: usize,
    pub heaviest_paths: Vec<(ComponentId, ComponentId, EventType, u64)>,
    pub busiest_components: Vec<(ComponentId, u64)>,
}

impl ProfilingReport {
    /// Print a formatted report to stdout
    pub fn print(&self) {
        println!("=== Profiling Report ===");
        println!("Total events recorded: {}", self.total_events);
        println!("Unique communication flows: {}", self.unique_flows);
        
        println!("\nHeaviest communication paths:");
        for (i, (source, target, event_type, count)) in self.heaviest_paths.iter().enumerate() {
            println!("  {}. {} -> {} [{}]: {} events", 
                i + 1, source, target, event_type, count);
        }
        
        println!("\nBusiest components:");
        for (i, (component, activity)) in self.busiest_components.iter().enumerate() {
            println!("  {}. {}: {} total events", i + 1, component, activity);
        }
    }
}