use std::collections::{HashMap, HashSet};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo;
use log::{debug, info};

use super::component::BaseComponent;
use super::types::ComponentId;

/// A directed graph representing dependencies between components.
/// An edge A -> B exists if component A can emit an event that component B subscribes to.
pub struct DependencyGraph {
    /// The underlying directed graph.
    /// Nodes are components, edges represent potential event flow.
    graph: DiGraph<ComponentId, u64>,
    
    /// Maps component IDs to their node indices in the graph.
    component_to_node: HashMap<ComponentId, NodeIndex>,
    
    /// Maps node indices in the graph to their component IDs.
    node_to_component: HashMap<NodeIndex, ComponentId>,
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    pub fn new() -> Self {
        DependencyGraph {
            graph: DiGraph::new(),
            component_to_node: HashMap::new(),
            node_to_component: HashMap::new(),
        }
    }
    
    /// Builds a dependency graph from a collection of components.
    pub fn build<'a, I>(components: I) -> Self 
    where
        I: IntoIterator<Item = &'a dyn BaseComponent>,
    {
        let mut graph = DependencyGraph::new();
        
        // First, add all components as nodes in the graph
        for component in components.into_iter() {
            let component_id = component.component_id().clone();
            let node_idx = graph.graph.add_node(component_id.clone());
            graph.component_to_node.insert(component_id.clone(), node_idx);
            graph.node_to_component.insert(node_idx, component_id);
        }
        
        // Next, build a map from event types to components that subscribe to them
        let mut subscription_map: HashMap<&'static str, Vec<ComponentId>> = HashMap::new();
        for component in components.into_iter() {
            let component_id = component.component_id().clone();
            for event_type in component.subscriptions() {
                subscription_map.entry(*event_type)
                    .or_default()
                    .push(component_id.clone());
            }
        }
        
        // Now, add edges based on event emission and subscription
        for component in components.into_iter() {
            let source_id = component.component_id().clone();
            let source_node = *graph.component_to_node.get(&source_id).unwrap();
            
            // For each event type that this component can emit
            for emitted_event_type in component.emitted_events() {
                // Find all components that subscribe to this event type
                if let Some(subscribers) = subscription_map.get(emitted_event_type) {
                    for target_id in subscribers {
                        // Don't add self-loops
                        if &source_id != target_id {
                            let target_node = *graph.component_to_node.get(target_id).unwrap();
                            // Add an edge with an initial weight of 0
                            // The weight will be updated during profiling
                            graph.graph.add_edge(source_node, target_node, 0);
                        }
                    }
                }
            }
        }
        
        info!("Built dependency graph with {} nodes and {} edges", 
              graph.graph.node_count(), graph.graph.edge_count());
        
        graph
    }
    
    /// Updates the weight of an edge between two components.
    pub fn update_edge_weight(&mut self, source: &ComponentId, target: &ComponentId, new_weight: u64) -> bool {
        if let (Some(&source_node), Some(&target_node)) = (
            self.component_to_node.get(source),
            self.component_to_node.get(target),
        ) {
            if let Some(edge_idx) = self.graph.find_edge(source_node, target_node) {
                *self.graph.edge_weight_mut(edge_idx).unwrap() = new_weight;
                debug!("Updated edge weight from {:?} to {:?}: {}", source, target, new_weight);
                return true;
            }
        }
        false
    }
    
    /// Returns a reference to the internal graph.
    pub fn graph(&self) -> &DiGraph<ComponentId, u64> {
        &self.graph
    }
    
    /// Returns a component ID for a given node index.
    pub fn get_component(&self, node: NodeIndex) -> Option<&ComponentId> {
        self.node_to_component.get(&node)
    }
    
    /// Returns a node index for a given component ID.
    pub fn get_node(&self, component: &ComponentId) -> Option<&NodeIndex> {
        self.component_to_node.get(component)
    }
}

#[cfg(test)]
mod tests {
    use super::super::component::BaseComponent;
    use super::super::event::Event;
    use super::super::types::ComponentId;
    use super::DependencyGraph;
    use std::fmt;
    
    // Simple mock event for testing
    struct MockEvent {
        source_id: String,
        target_ids: Option<Vec<String>>,
        event_type: String,
    }
    
    impl Event for MockEvent {
        fn source_id(&self) -> &str {
            &self.source_id
        }
        
        fn target_ids(&self) -> Option<Vec<&ComponentId>> {
            self.target_ids.as_ref().map(|ids| ids.iter().map(|id| id as &ComponentId).collect())
        }
        
        fn event_type(&self) -> &str {
            &self.event_type
        }
        
        fn clone_event(&self) -> Box<dyn Event> {
            Box::new(MockEvent {
                source_id: self.source_id.clone(),
                target_ids: self.target_ids.clone(),
                event_type: self.event_type.clone(),
            })
        }
    }
    
    impl fmt::Debug for MockEvent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "MockEvent({}->{})", self.source_id, self.event_type)
        }
    }
    
    // Mock component for testing
    struct MockComponent {
        id: String,
        subscriptions: Vec<&'static str>,
        emitted_events: Vec<&'static str>,
    }
    
    impl BaseComponent for MockComponent {
        fn component_id(&self) -> &ComponentId {
            &self.id
        }
        
        fn subscriptions(&self) -> &[&'static str] {
            &self.subscriptions
        }
        
        fn emitted_events(&self) -> &[&'static str] {
            &self.emitted_events
        }
        
        fn react_atomic(&mut self, _events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
            vec![]
        }
    }
    
    #[test]
    fn test_dependency_graph_construction() {
        // Create mock components with different subscriptions and emissions
        let component_a = MockComponent {
            id: "A".to_string(),
            subscriptions: vec!["event1", "event2"],
            emitted_events: vec!["event3", "event4"],
        };
        
        let component_b = MockComponent {
            id: "B".to_string(),
            subscriptions: vec!["event3"], // B subscribes to event3 which A emits
            emitted_events: vec!["event5"],
        };
        
        let component_c = MockComponent {
            id: "C".to_string(),
            subscriptions: vec!["event4", "event5"], // C subscribes to events A and B emit
            emitted_events: vec!["event1"], // C emits event1 which A subscribes to
        };
        
        let components: Vec<&dyn BaseComponent> = vec![&component_a, &component_b, &component_c];
        
        // Build dependency graph
        let graph = DependencyGraph::build(components);
        
        // Verify the graph structure
        let petgraph = graph.graph();
        
        // We expect 3 nodes (one for each component)
        assert_eq!(petgraph.node_count(), 3);
        
        // We expect the following edges:
        // A -> B (A emits event3, B subscribes to event3)
        // A -> C (A emits event4, C subscribes to event4)
        // B -> C (B emits event5, C subscribes to event5)
        // C -> A (C emits event1, A subscribes to event1)
        assert_eq!(petgraph.edge_count(), 4);
        
        // Verify specific edges by checking component connections
        let a_node = graph.get_node(&component_a.id).unwrap();
        let b_node = graph.get_node(&component_b.id).unwrap();
        let c_node = graph.get_node(&component_c.id).unwrap();
        
        // Verify A -> B edge exists
        assert!(petgraph.contains_edge(*a_node, *b_node));
        
        // Verify A -> C edge exists
        assert!(petgraph.contains_edge(*a_node, *c_node));
        
        // Verify B -> C edge exists  
        assert!(petgraph.contains_edge(*b_node, *c_node));
        
        // Verify C -> A edge exists
        assert!(petgraph.contains_edge(*c_node, *a_node));
    }
} 