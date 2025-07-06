use crate::core::types::ComponentId;
use std::collections::HashMap;

/// Manages topological sorting and execution order calculation for processing components
pub struct ExecutionOrderBuilder;

impl ExecutionOrderBuilder {
    /// Analyzes the graph of processing components to build a topologically sorted execution order
    /// organized into stages for concurrent execution.
    /// Uses modified Kahn's algorithm to detect cycles and ensure deterministic execution.
    pub fn build_execution_order_stages(
        component_ids: &[ComponentId],
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    ) -> Result<Vec<Vec<ComponentId>>, String> {
        let mut adj_list: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
        let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();

        // Initialize graph data structures for all components
        for comp_id in component_ids {
            in_degree.insert(comp_id.clone(), 0);
            adj_list.insert(comp_id.clone(), Vec::new());
        }

        // Build adjacency list and in-degrees from connections
        for ((source_id, _source_port), targets) in connections {
            // Only consider connections between components we're tracking
            if !in_degree.contains_key(source_id) {
                continue;
            }

            for (target_id, _target_port) in targets {
                if !in_degree.contains_key(target_id) {
                    continue;
                }

                // Add edge from source to target
                adj_list.get_mut(source_id).unwrap().push(target_id.clone());
                *in_degree.get_mut(target_id).unwrap() += 1;
            }
        }

        // Modified Kahn's algorithm for stage-based topological sort
        let mut stages = Vec::new();
        let mut processed_count = 0;

        while processed_count < component_ids.len() {
            // Find all components with zero in-degree (current stage)
            let mut current_stage: Vec<ComponentId> = in_degree
                .iter()
                .filter(|(_, &degree)| degree == 0)
                .map(|(id, _)| id.clone())
                .collect();

            if current_stage.is_empty() {
                return Err("Cycle detected in component dependencies".to_string());
            }

            // Sort stage for deterministic results
            current_stage.sort();

            // Process all components in current stage
            for comp_id in &current_stage {
                // Remove from in_degree map (mark as processed)
                in_degree.remove(comp_id);
                processed_count += 1;

                // Decrease in-degree of all neighbors
                if let Some(neighbors) = adj_list.get(comp_id) {
                    for neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            *degree -= 1;
                        }
                    }
                }
            }

            stages.push(current_stage);
        }

        Ok(stages)
    }

    /// Analyzes the graph of processing components to build a topologically sorted execution order.
    /// Uses Kahn's algorithm to detect cycles and ensure deterministic execution.
    /// This method provides backwards compatibility by flattening the stages.
    pub fn build_execution_order(
        component_ids: &[ComponentId],
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    ) -> Result<Vec<ComponentId>, String> {
        // Use the stages method and flatten the result for backwards compatibility
        let stages = Self::build_execution_order_stages(component_ids, connections)?;
        Ok(stages.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ComponentId;

    fn create_test_component(id: &str) -> ComponentId {
        ComponentId::new(id.to_string(), "test".to_string())
    }

    #[test]
    fn test_build_execution_order_stages_simple() {
        // Test a simple linear dependency: A -> B -> C
        let components = vec![
            create_test_component("A"),
            create_test_component("B"),
            create_test_component("C"),
        ];
        
        let mut connections = HashMap::new();
        connections.insert(
            (create_test_component("A"), "out".to_string()),
            vec![(create_test_component("B"), "in".to_string())],
        );
        connections.insert(
            (create_test_component("B"), "out".to_string()),
            vec![(create_test_component("C"), "in".to_string())],
        );
        
        let stages = ExecutionOrderBuilder::build_execution_order_stages(&components, &connections)
            .expect("Should build execution order");
        
        // Should have 3 stages with one component each
        assert_eq!(stages.len(), 3);
        assert_eq!(stages[0], vec![create_test_component("A")]);
        assert_eq!(stages[1], vec![create_test_component("B")]);
        assert_eq!(stages[2], vec![create_test_component("C")]);
    }

    #[test]
    fn test_build_execution_order_stages_parallel() {
        // Test parallel components: A -> B, A -> C, B -> D, C -> D
        let components = vec![
            create_test_component("A"),
            create_test_component("B"),
            create_test_component("C"),
            create_test_component("D"),
        ];
        
        let mut connections = HashMap::new();
        connections.insert(
            (create_test_component("A"), "out".to_string()),
            vec![
                (create_test_component("B"), "in".to_string()),
                (create_test_component("C"), "in".to_string()),
            ],
        );
        connections.insert(
            (create_test_component("B"), "out".to_string()),
            vec![(create_test_component("D"), "in1".to_string())],
        );
        connections.insert(
            (create_test_component("C"), "out".to_string()),
            vec![(create_test_component("D"), "in2".to_string())],
        );
        
        let stages = ExecutionOrderBuilder::build_execution_order_stages(&components, &connections)
            .expect("Should build execution order");
        
        // Should have 3 stages: [A], [B,C], [D]
        assert_eq!(stages.len(), 3);
        assert_eq!(stages[0], vec![create_test_component("A")]);
        
        // Stage 1 should have B and C (sorted alphabetically)
        let mut stage1 = stages[1].clone();
        stage1.sort();
        assert_eq!(stage1, vec![create_test_component("B"), create_test_component("C")]);
        
        assert_eq!(stages[2], vec![create_test_component("D")]);
    }

    #[test]
    fn test_build_execution_order_stages_cycle_detection() {
        // Test cycle detection: A -> B -> A
        let components = vec![
            create_test_component("A"),
            create_test_component("B"),
        ];
        
        let mut connections = HashMap::new();
        connections.insert(
            (create_test_component("A"), "out".to_string()),
            vec![(create_test_component("B"), "in".to_string())],
        );
        connections.insert(
            (create_test_component("B"), "out".to_string()),
            vec![(create_test_component("A"), "in".to_string())],
        );
        
        let result = ExecutionOrderBuilder::build_execution_order_stages(&components, &connections);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cycle detected"));
    }

    #[test]
    fn test_build_execution_order_backwards_compatibility() {
        // Test that the flattened method produces the same result as the staged method
        let components = vec![
            create_test_component("A"),
            create_test_component("B"),
            create_test_component("C"),
        ];
        
        let mut connections = HashMap::new();
        connections.insert(
            (create_test_component("A"), "out".to_string()),
            vec![(create_test_component("B"), "in".to_string())],
        );
        connections.insert(
            (create_test_component("B"), "out".to_string()),
            vec![(create_test_component("C"), "in".to_string())],
        );
        
        let stages = ExecutionOrderBuilder::build_execution_order_stages(&components, &connections)
            .expect("Should build execution order");
        let flattened = ExecutionOrderBuilder::build_execution_order(&components, &connections)
            .expect("Should build execution order");
        
        let expected_flat: Vec<ComponentId> = stages.into_iter().flatten().collect();
        assert_eq!(flattened, expected_flat);
    }
}