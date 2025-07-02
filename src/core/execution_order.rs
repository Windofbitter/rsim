use super::component_registry::{ComponentRegistry, ComponentType};
use super::types::ComponentId;
use std::collections::{HashMap, VecDeque};

/// Manages topological sorting and execution order calculation for processing components
pub struct ExecutionOrderBuilder;

impl ExecutionOrderBuilder {
    /// Analyzes the graph of processing components to build a topologically sorted execution order.
    /// Uses Kahn's algorithm to detect cycles and ensure deterministic execution.
    pub fn build_execution_order(
        registry: &ComponentRegistry,
        connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    ) -> Result<Vec<ComponentId>, String> {
        let mut adj_list: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
        let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();

        // Initialize graph data structures for all processing components
        for comp_id in registry.components_by_type(ComponentType::Processing).keys() {
            in_degree.insert(comp_id.clone(), 0);
            adj_list.insert(comp_id.clone(), Vec::new());
        }

        // Build adjacency list and in-degrees from connections
        for ((source_id, _source_port), targets) in connections {
            // Only consider connections between processing components
            if !registry.has_component_of_type(source_id, ComponentType::Processing) {
                continue;
            }

            for (target_id, _target_port) in targets {
                if !registry.has_component_of_type(target_id, ComponentType::Processing) {
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
        if sorted_order.len() == registry.components_by_type(ComponentType::Processing).len() {
            Ok(sorted_order)
        } else {
            Err("Cycle detected in processing component dependencies".to_string())
        }
    }
}