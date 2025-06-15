use crate::core::component::BaseComponent;
use crate::core::types::ComponentId;
use crate::core::event::EventType;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub id: ComponentId,
    pub subscriptions: Vec<EventType>,
}

#[derive(Debug, Clone)]
pub struct CommunicationEdge {
    pub event_type: EventType,
    pub weight: u64,
}

pub struct DependencyGraph {
    graph: DiGraph<ComponentNode, CommunicationEdge>,
    node_map: HashMap<ComponentId, NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, id: ComponentId, subscriptions: Vec<EventType>) -> NodeIndex {
        let node = ComponentNode {
            id: id.clone(),
            subscriptions,
        };
        let index = self.graph.add_node(node);
        self.node_map.insert(id, index);
        index
    }

    pub fn build_from_components<'a, I>(&mut self, components: I)
    where
        I: Iterator<Item = &'a Box<dyn BaseComponent>>,
    {
        let components_vec: Vec<_> = components.collect();
        
        for component in &components_vec {
            self.add_component(
                component.component_id().clone(),
                component.subscriptions().iter().map(|s| s.to_string()).collect(),
            );
        }
        
        for source_component in &components_vec {
            let source_id = source_component.component_id();
            let source_idx = self.node_map[source_id];
            
            for target_component in &components_vec {
                if source_component.component_id() == target_component.component_id() {
                    continue;
                }
                
                let target_subscriptions = target_component.subscriptions();
                let emitted_events = Self::infer_emitted_events(source_component.as_ref());
                
                for event_type in &emitted_events {
                    if target_subscriptions.iter().any(|sub| sub == event_type) {
                        let target_idx = self.node_map[target_component.component_id()];
                        
                        let edge = CommunicationEdge {
                            event_type: event_type.clone(),
                            weight: 0,
                        };
                        
                        self.graph.add_edge(source_idx, target_idx, edge);
                    }
                }
            }
        }
    }
    
    fn infer_emitted_events(component: &dyn BaseComponent) -> Vec<EventType> {
        component.emitted_events().iter().map(|s| s.to_string()).collect()
    }
    
    pub fn add_edge(&mut self, source: &ComponentId, target: &ComponentId, event_type: EventType) {
        if let (Some(&source_idx), Some(&target_idx)) = 
            (self.node_map.get(source), self.node_map.get(target)) {
            let edge = CommunicationEdge {
                event_type,
                weight: 0,
            };
            self.graph.add_edge(source_idx, target_idx, edge);
        }
    }
    
    pub fn update_edge_weight(&mut self, source: &ComponentId, target: &ComponentId, event_type: &EventType, weight: u64) {
        if let (Some(&source_idx), Some(&target_idx)) = 
            (self.node_map.get(source), self.node_map.get(target)) {
            
            // Find all outgoing edges from source that target the specified target node
            let edge_ids: Vec<_> = self.graph.edges_directed(source_idx, Direction::Outgoing)
                .filter(|edge_ref| edge_ref.target() == target_idx)
                .map(|edge_ref| edge_ref.id())
                .collect();
            
            for edge_id in edge_ids {
                if let Some(edge) = self.graph.edge_weight_mut(edge_id) {
                    if edge.event_type == *event_type {
                        edge.weight = weight;
                        return; // Found and updated the correct edge
                    }
                }
            }
        }
    }
    
    pub fn get_component_count(&self) -> usize {
        self.graph.node_count()
    }
    
    pub fn get_edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    
    pub fn get_outgoing_edges(&self, component_id: &ComponentId) -> Vec<(&ComponentId, &CommunicationEdge)> {
        if let Some(&node_idx) = self.node_map.get(component_id) {
            self.graph
                .edges_directed(node_idx, Direction::Outgoing)
                .map(|edge| {
                    let target_node = &self.graph[edge.target()];
                    (&target_node.id, edge.weight())
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_incoming_edges(&self, component_id: &ComponentId) -> Vec<(&ComponentId, &CommunicationEdge)> {
        if let Some(&node_idx) = self.node_map.get(component_id) {
            self.graph
                .edges_directed(node_idx, Direction::Incoming)
                .map(|edge| {
                    let source_node = &self.graph[edge.source()];
                    (&source_node.id, edge.weight())
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn to_dot(&self) -> String {
        use std::fmt::Write;
        
        let mut output = String::new();
        writeln!(&mut output, "digraph DependencyGraph {{").unwrap();
        writeln!(&mut output, "    rankdir=LR;").unwrap();
        writeln!(&mut output, "    node [shape=box, style=rounded];").unwrap();
        
        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            writeln!(&mut output, "    \"{}\" [label=\"{}\"];", node.id, node.id).unwrap();
        }
        
        for edge_ref in self.graph.edge_references() {
            let source = &self.graph[edge_ref.source()].id;
            let target = &self.graph[edge_ref.target()].id;
            let edge = edge_ref.weight();
            
            let label = if edge.weight > 0 {
                format!("{} ({})", edge.event_type, edge.weight)
            } else {
                edge.event_type.clone()
            };
            
            writeln!(
                &mut output,
                "    \"{}\" -> \"{}\" [label=\"{}\"];",
                source, target, label
            ).unwrap();
        }
        
        writeln!(&mut output, "}}").unwrap();
        output
    }
    
    pub fn to_mermaid(&self) -> String {
        use std::fmt::Write;
        
        let mut output = String::new();
        writeln!(&mut output, "graph LR").unwrap();
        
        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            writeln!(&mut output, "    {}[{}]", node.id, node.id).unwrap();
        }
        
        for edge_ref in self.graph.edge_references() {
            let source = &self.graph[edge_ref.source()].id;
            let target = &self.graph[edge_ref.target()].id;
            let edge = edge_ref.weight();
            
            let label = if edge.weight > 0 {
                format!("{} ({})", edge.event_type, edge.weight)
            } else {
                edge.event_type.clone()
            };
            
            writeln!(
                &mut output,
                "    {} -->|{}| {}",
                source, label, target
            ).unwrap();
        }
        
        output
    }
    
    pub fn analyze_communication_patterns(&self) -> CommunicationAnalysis {
        let mut total_weight = 0u64;
        let mut max_weight = 0u64;
        let mut edge_weights = Vec::new();
        
        for edge in self.graph.edge_weights() {
            total_weight += edge.weight;
            max_weight = max_weight.max(edge.weight);
            edge_weights.push(edge.weight);
        }
        
        let mut heaviest_edges = Vec::new();
        for edge_ref in self.graph.edge_references() {
            if edge_ref.weight().weight > 0 {
                let source = &self.graph[edge_ref.source()].id;
                let target = &self.graph[edge_ref.target()].id;
                heaviest_edges.push((
                    source.clone(),
                    target.clone(),
                    edge_ref.weight().event_type.clone(),
                    edge_ref.weight().weight,
                ));
            }
        }
        
        heaviest_edges.sort_by(|a, b| b.3.cmp(&a.3));
        heaviest_edges.truncate(10);
        
        CommunicationAnalysis {
            total_weight,
            max_weight,
            edge_count: self.graph.edge_count(),
            component_count: self.graph.node_count(),
            heaviest_edges,
        }
    }
}

#[derive(Debug)]
pub struct CommunicationAnalysis {
    pub total_weight: u64,
    pub max_weight: u64,
    pub edge_count: usize,
    pub component_count: usize,
    pub heaviest_edges: Vec<(ComponentId, ComponentId, EventType, u64)>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "dependency_graph_tests.rs"]
mod tests;