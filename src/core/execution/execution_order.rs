use crate::core::types::ComponentId;
use std::collections::HashMap;

/// Represents a sub-level within a stage, containing components that can execute in parallel
#[derive(Debug, Clone)]
pub struct SubLevel {
    pub components: Vec<ComponentId>,
}

/// Represents a stage with sub-levels for more granular dependency management
#[derive(Debug, Clone)]
pub struct Stage {
    pub sub_levels: Vec<SubLevel>,
}

/// Manages topological sorting and execution order calculation for processing components
pub struct ExecutionOrderBuilder;

impl ExecutionOrderBuilder {
    /// Analyzes the graph of processing components to build a topologically sorted execution order
    /// organized into stages with sub-levels for more granular dependency management.
    /// This method subdivides each stage by exact dependency depth to ensure proper ordering.
    pub fn build_execution_order_with_sub_levels(
        component_ids: &[ComponentId],
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    ) -> Result<Vec<Stage>, String> {
        // First, get the regular stage-based execution order
        let regular_stages = Self::build_execution_order_stages(component_ids, connections)?;
        
        // Now, for each stage, further subdivide into sub-levels based on detailed dependency analysis
        let mut enhanced_stages = Vec::new();
        
        for stage_components in regular_stages {
            if stage_components.is_empty() {
                continue;
            }
            
            // If stage has only one component, it becomes a single sub-level
            if stage_components.len() == 1 {
                let sub_level = SubLevel {
                    components: stage_components,
                };
                let stage = Stage {
                    sub_levels: vec![sub_level],
                };
                enhanced_stages.push(stage);
                continue;
            }
            
            // For stages with multiple components, analyze internal dependencies
            let sub_levels = Self::subdivide_stage_into_sub_levels(&stage_components, connections)?;
            let stage = Stage { sub_levels };
            enhanced_stages.push(stage);
        }
        
        Ok(enhanced_stages)
    }
    
    /// Subdivides a stage into sub-levels based on internal dependencies
    /// Components that have no dependencies within the stage can run in parallel
    /// Components that depend on other components in the stage must run sequentially
    fn subdivide_stage_into_sub_levels(
        stage_components: &[ComponentId],
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    ) -> Result<Vec<SubLevel>, String> {
        // Build internal dependency graph for this stage
        let mut internal_adj_list: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
        let mut internal_in_degree: HashMap<ComponentId, usize> = HashMap::new();
        
        // Initialize for all components in this stage
        for comp_id in stage_components {
            internal_adj_list.insert(comp_id.clone(), Vec::new());
            internal_in_degree.insert(comp_id.clone(), 0);
        }
        
        // Build internal adjacency list (only dependencies within this stage)
        for ((source_id, _source_port), targets) in connections {
            if !internal_adj_list.contains_key(source_id) {
                continue;
            }
            
            for (target_id, _target_port) in targets {
                if !internal_adj_list.contains_key(target_id) {
                    continue;
                }
                
                // Add internal edge
                internal_adj_list.get_mut(source_id).unwrap().push(target_id.clone());
                *internal_in_degree.get_mut(target_id).unwrap() += 1;
            }
        }
        
        // Use modified Kahn's algorithm to create sub-levels
        let mut sub_levels = Vec::new();
        let mut remaining_components = stage_components.len();
        
        while remaining_components > 0 {
            // Find all components with zero internal in-degree (can run in parallel)
            let mut current_sub_level: Vec<ComponentId> = internal_in_degree
                .iter()
                .filter(|(_, &degree)| degree == 0)
                .map(|(id, _)| id.clone())
                .collect();
            
            if current_sub_level.is_empty() {
                // This should not happen if the original topological sort was correct,
                // but if it does, fall back to sequential execution
                for comp_id in stage_components {
                    if internal_in_degree.contains_key(comp_id) {
                        current_sub_level.push(comp_id.clone());
                        break;
                    }
                }
            }
            
            if current_sub_level.is_empty() {
                break;
            }
            
            // Remove processed components and update in-degrees
            for comp_id in &current_sub_level {
                internal_in_degree.remove(comp_id);
                remaining_components -= 1;
                
                // Decrease in-degree of all internal neighbors
                if let Some(neighbors) = internal_adj_list.get(comp_id) {
                    for neighbor in neighbors {
                        if let Some(degree) = internal_in_degree.get_mut(neighbor) {
                            *degree -= 1;
                        }
                    }
                }
            }
            
            // Sort for deterministic results
            current_sub_level.sort();
            
            let sub_level = SubLevel {
                components: current_sub_level,
            };
            sub_levels.push(sub_level);
        }
        
        // If no sub-levels were created, fall back to single sub-level
        if sub_levels.is_empty() {
            let mut all_components = stage_components.to_vec();
            all_components.sort();
            sub_levels.push(SubLevel {
                components: all_components,
            });
        }
        
        Ok(sub_levels)
    }

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