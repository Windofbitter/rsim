#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::component::BaseComponent;
    use crate::core::types::{ComponentId, ComponentValue};
    use crate::core::event::Event;
    use std::collections::HashMap;
    
    #[derive(Clone, Debug)]
    struct TestEvent {
        id: String,
        event_type: String,
        source_id: ComponentId,
        target_ids: Option<Vec<ComponentId>>,
        data: HashMap<String, ComponentValue>,
    }
    
    impl Event for TestEvent {
        fn id(&self) -> &String {
            &self.id
        }
        
        fn event_type(&self) -> &str {
            &self.event_type
        }
        
        fn source_id(&self) -> &ComponentId {
            &self.source_id
        }
        
        fn target_ids(&self) -> Option<Vec<ComponentId>> {
            self.target_ids.clone()
        }
        
        fn data(&self) -> HashMap<String, ComponentValue> {
            self.data.clone()
        }
        
        fn clone_event(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
    }
    
    struct TestComponent {
        id: ComponentId,
        subscriptions: Vec<&'static str>,
    }
    
    impl TestComponent {
        fn new(id: &str, subscriptions: Vec<&'static str>) -> Self {
            Self {
                id: id.to_string(),
                subscriptions,
            }
        }
    }
    
    impl BaseComponent for TestComponent {
        fn component_id(&self) -> &ComponentId {
            &self.id
        }
        
        fn subscriptions(&self) -> &[&'static str] {
            &self.subscriptions
        }
        
        fn react_atomic(&mut self, _events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
            Vec::new()
        }
    }
    
    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.get_component_count(), 0);
        assert_eq!(graph.get_edge_count(), 0);
    }
    
    #[test]
    fn test_add_single_component() {
        let mut graph = DependencyGraph::new();
        let id = "comp1".to_string();
        let subs = vec!["event1".to_string(), "event2".to_string()];
        
        graph.add_component(id.clone(), subs.clone());
        
        assert_eq!(graph.get_component_count(), 1);
        assert_eq!(graph.get_edge_count(), 0);
    }
    
    #[test]
    fn test_manual_edge_creation() {
        let mut graph = DependencyGraph::new();
        
        let id1 = "producer".to_string();
        let id2 = "consumer".to_string();
        
        graph.add_component(id1.clone(), vec![]);
        graph.add_component(id2.clone(), vec!["data_ready".to_string()]);
        
        graph.add_edge(&id1, &id2, "data_ready".to_string());
        
        assert_eq!(graph.get_component_count(), 2);
        assert_eq!(graph.get_edge_count(), 1);
        
        let outgoing = graph.get_outgoing_edges(&id1);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].0, &id2);
        assert_eq!(outgoing[0].1.event_type, "data_ready");
        assert_eq!(outgoing[0].1.weight, 0);
        
        let incoming = graph.get_incoming_edges(&id2);
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].0, &id1);
        assert_eq!(incoming[0].1.event_type, "data_ready");
    }
    
    #[test]
    fn test_edge_weight_update() {
        let mut graph = DependencyGraph::new();
        
        let id1 = "source".to_string();
        let id2 = "target".to_string();
        let event_type = "test_event".to_string();
        
        graph.add_component(id1.clone(), vec![]);
        graph.add_component(id2.clone(), vec![event_type.clone()]);
        graph.add_edge(&id1, &id2, event_type.clone());
        
        graph.update_edge_weight(&id1, &id2, &event_type, 100);
        
        let edges = graph.get_outgoing_edges(&id1);
        assert_eq!(edges[0].1.weight, 100);
    }
    
    #[test]
    fn test_communication_analysis() {
        let mut graph = DependencyGraph::new();
        
        let comp1 = "comp1".to_string();
        let comp2 = "comp2".to_string();
        let comp3 = "comp3".to_string();
        
        graph.add_component(comp1.clone(), vec![]);
        graph.add_component(comp2.clone(), vec!["event1".to_string()]);
        graph.add_component(comp3.clone(), vec!["event2".to_string()]);
        
        graph.add_edge(&comp1, &comp2, "event1".to_string());
        graph.add_edge(&comp2, &comp3, "event2".to_string());
        
        graph.update_edge_weight(&comp1, &comp2, &"event1".to_string(), 50);
        graph.update_edge_weight(&comp2, &comp3, &"event2".to_string(), 75);
        
        let analysis = graph.analyze_communication_patterns();
        
        assert_eq!(analysis.component_count, 3);
        assert_eq!(analysis.edge_count, 2);
        assert_eq!(analysis.total_weight, 125);
        assert_eq!(analysis.max_weight, 75);
        assert_eq!(analysis.heaviest_edges.len(), 2);
        assert_eq!(analysis.heaviest_edges[0].3, 75);
        assert_eq!(analysis.heaviest_edges[1].3, 50);
    }
    
    #[test]
    fn test_dot_output() {
        let mut graph = DependencyGraph::new();
        
        graph.add_component("A".to_string(), vec![]);
        graph.add_component("B".to_string(), vec!["event1".to_string()]);
        graph.add_edge(&"A".to_string(), &"B".to_string(), "event1".to_string());
        
        let dot = graph.to_dot();
        
        assert!(dot.contains("digraph DependencyGraph"));
        assert!(dot.contains("\"A\" [label=\"A\"]"));
        assert!(dot.contains("\"B\" [label=\"B\"]"));
        assert!(dot.contains("\"A\" -> \"B\" [label=\"event1\"]"));
    }
    
    #[test]
    fn test_mermaid_output() {
        let mut graph = DependencyGraph::new();
        
        graph.add_component("X".to_string(), vec![]);
        graph.add_component("Y".to_string(), vec!["data".to_string()]);
        graph.add_edge(&"X".to_string(), &"Y".to_string(), "data".to_string());
        graph.update_edge_weight(&"X".to_string(), &"Y".to_string(), &"data".to_string(), 42);
        
        let mermaid = graph.to_mermaid();
        
        assert!(mermaid.contains("graph LR"));
        assert!(mermaid.contains("X[X]"));
        assert!(mermaid.contains("Y[Y]"));
        assert!(mermaid.contains("X -->|data (42)| Y"));
    }
}