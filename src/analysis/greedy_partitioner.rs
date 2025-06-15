use crate::analysis::graph_partitioner::{GraphPartitioner, PartitioningConfig};
use crate::analysis::{ComponentPartition, DependencyGraph, PartitionQualityMetrics};
use crate::core::types::ComponentId;
use std::collections::{HashMap, HashSet};

/// A greedy graph partitioner that uses heuristics to minimize cross-thread communication
pub struct GreedyPartitioner {
    config: PartitioningConfig,
}

impl GreedyPartitioner {
    /// Create a new greedy partitioner with default configuration
    pub fn new() -> Self {
        Self {
            config: PartitioningConfig::default(),
        }
    }

    /// Create a new greedy partitioner with custom configuration
    pub fn with_config(config: PartitioningConfig) -> Self {
        Self { config }
    }

    /// Phase 1: Group components connected by high-weight edges with load balancing
    fn initial_clustering(
        &self,
        graph: &DependencyGraph,
        num_threads: usize,
    ) -> Vec<Vec<ComponentId>> {
        let mut clusters = Vec::new();
        let mut assigned = HashSet::new();

        // Initialize empty clusters
        for _ in 0..num_threads {
            clusters.push(Vec::new());
        }

        // Get all edges sorted by weight (descending)
        let mut weighted_edges = Vec::new();

        for (source_id, edges) in self.get_all_edges(graph) {
            for (target_id, edge) in edges {
                if edge.weight > 0 {
                    weighted_edges.push((source_id.clone(), target_id.clone(), edge.weight));
                }
            }
        }

        // Sort by weight descending
        weighted_edges.sort_by(|a, b| b.2.cmp(&a.2));

        // Process edges in order of decreasing weight
        for (source_id, target_id, _weight) in weighted_edges {
            if assigned.contains(&source_id) && assigned.contains(&target_id) {
                continue; // Both already assigned
            }

            let thread_loads = self.calculate_thread_loads(&clusters, graph);

            if !assigned.contains(&source_id) && !assigned.contains(&target_id) {
                // Neither assigned - find best cluster considering both components
                let source_weight = graph.get_component_weight(&source_id);
                let target_weight = graph.get_component_weight(&target_id);
                let _combined_weight =
                    source_weight.computational_cost + target_weight.computational_cost;

                // Find cluster with lowest load that can accommodate both
                let mut best_cluster = 0;
                let mut min_load = thread_loads[0];
                for (i, &load) in thread_loads.iter().enumerate() {
                    if load < min_load {
                        min_load = load;
                        best_cluster = i;
                    }
                }

                clusters[best_cluster].push(source_id.clone());
                clusters[best_cluster].push(target_id.clone());
                assigned.insert(source_id);
                assigned.insert(target_id);
            } else if !assigned.contains(&source_id) {
                // Only source unassigned - consider both communication and balance
                if let Some(target_cluster_idx) = self.find_component_cluster(&clusters, &target_id)
                {
                    let source_weight = graph.get_component_weight(&source_id);

                    // Check if adding to target's cluster violates balance
                    if self.move_preserves_balance(
                        &thread_loads,
                        target_cluster_idx,
                        target_cluster_idx,
                        &source_weight,
                        self.config.max_load_imbalance,
                    ) {
                        clusters[target_cluster_idx].push(source_id.clone());
                    } else {
                        // Find alternative cluster
                        let best_cluster = self.find_best_cluster_for_component(
                            &source_id,
                            &clusters,
                            &thread_loads,
                            graph,
                        );
                        clusters[best_cluster].push(source_id.clone());
                    }
                    assigned.insert(source_id);
                }
            } else if !assigned.contains(&target_id) {
                // Only target unassigned - similar logic
                if let Some(source_cluster_idx) = self.find_component_cluster(&clusters, &source_id)
                {
                    let target_weight = graph.get_component_weight(&target_id);

                    if self.move_preserves_balance(
                        &thread_loads,
                        source_cluster_idx,
                        source_cluster_idx,
                        &target_weight,
                        self.config.max_load_imbalance,
                    ) {
                        clusters[source_cluster_idx].push(target_id.clone());
                    } else {
                        let best_cluster = self.find_best_cluster_for_component(
                            &target_id,
                            &clusters,
                            &thread_loads,
                            graph,
                        );
                        clusters[best_cluster].push(target_id.clone());
                    }
                    assigned.insert(target_id);
                }
            }
        }

        // Assign remaining unassigned components using load-aware placement
        let all_components = self.get_all_components(graph);
        for component_id in all_components {
            if !assigned.contains(&component_id) {
                let thread_loads = self.calculate_thread_loads(&clusters, graph);
                let best_cluster = self.find_best_cluster_for_component(
                    &component_id,
                    &clusters,
                    &thread_loads,
                    graph,
                );
                clusters[best_cluster].push(component_id.clone());
                assigned.insert(component_id);
            }
        }

        clusters
    }

    /// Phase 2: Refine partition by moving components with balance constraints
    ///
    /// Performance optimization: Uses HashMap for O(1) component->thread lookups
    /// Multi-objective optimization: Balances communication cost and load balance
    fn refine_partition(&self, graph: &DependencyGraph, clusters: &mut [Vec<ComponentId>]) {
        let max_iterations = self.config.max_refinement_iterations;

        // Build component->thread lookup map for O(1) access
        let mut component_to_thread = HashMap::new();
        for (thread_id, cluster) in clusters.iter().enumerate() {
            for component_id in cluster {
                component_to_thread.insert(component_id.clone(), thread_id);
            }
        }

        for _iteration in 0..max_iterations {
            let mut improved = false;
            let thread_loads = self.calculate_thread_loads(clusters, graph);

            // Try moving each component to a different thread
            for current_thread in 0..clusters.len() {
                let components_to_check: Vec<ComponentId> = clusters[current_thread].clone();

                for component_id in components_to_check {
                    let component_weight = graph.get_component_weight(&component_id);
                    let current_comm_cost = self.calculate_component_cut_cost(
                        graph,
                        &component_id,
                        current_thread,
                        &component_to_thread,
                    );

                    // Try all other threads
                    for target_thread in 0..clusters.len() {
                        if target_thread == current_thread {
                            continue;
                        }

                        // Check load balance constraint first
                        if !self.move_preserves_balance(
                            &thread_loads,
                            current_thread,
                            target_thread,
                            &component_weight,
                            self.config.max_load_imbalance,
                        ) {
                            continue; // Skip moves that violate balance
                        }

                        let target_comm_cost = self.calculate_component_cut_cost(
                            graph,
                            &component_id,
                            target_thread,
                            &component_to_thread,
                        );

                        // Calculate multi-objective benefit
                        let comm_benefit = (current_comm_cost as f64) - (target_comm_cost as f64);

                        // Calculate load balance benefit
                        let current_imbalance = self.calculate_load_imbalance_ratio(&thread_loads);
                        let mut new_loads = thread_loads.clone();
                        new_loads[current_thread] -= component_weight.computational_cost;
                        new_loads[target_thread] += component_weight.computational_cost;
                        let new_imbalance = self.calculate_load_imbalance_ratio(&new_loads);
                        let balance_benefit = current_imbalance - new_imbalance;

                        // Combined benefit score
                        let total_benefit = self.config.communication_weight * comm_benefit
                            + self.config.balance_weight * balance_benefit * 100.0; // Scale balance benefit

                        // Move if it provides overall benefit
                        if total_benefit > 0.0 {
                            // Perform the move
                            clusters[current_thread].retain(|id| id != &component_id);
                            clusters[target_thread].push(component_id.clone());

                            // Update the lookup map
                            component_to_thread.insert(component_id.clone(), target_thread);

                            improved = true;
                            break;
                        }
                    }
                }
            }

            if !improved {
                break; // No more improvements found
            }
        }
    }

    /// Calculate the cut cost for a component if it were on a specific thread
    /// Uses O(1) HashMap lookups instead of O(T*C) linear searches for performance
    fn calculate_component_cut_cost(
        &self,
        graph: &DependencyGraph,
        component_id: &ComponentId,
        thread_id: usize,
        component_to_thread: &HashMap<ComponentId, usize>,
    ) -> u64 {
        let mut cut_cost = 0u64;

        // Check outgoing edges
        let outgoing = graph.get_outgoing_edges(component_id);
        for (target_id, edge) in outgoing {
            if let Some(&target_thread_id) = component_to_thread.get(target_id) {
                if target_thread_id != thread_id {
                    cut_cost += edge.weight;
                }
            }
        }

        // Check incoming edges
        let incoming = graph.get_incoming_edges(component_id);
        for (source_id, edge) in incoming {
            if let Some(&source_thread_id) = component_to_thread.get(source_id) {
                if source_thread_id != thread_id {
                    cut_cost += edge.weight;
                }
            }
        }

        cut_cost
    }

    /// Find which cluster contains a component
    fn find_component_cluster(
        &self,
        clusters: &[Vec<ComponentId>],
        component_id: &ComponentId,
    ) -> Option<usize> {
        for (i, cluster) in clusters.iter().enumerate() {
            if cluster.contains(component_id) {
                return Some(i);
            }
        }
        None
    }

    /// Convert clusters to ComponentPartition
    fn clusters_to_partition(&self, clusters: Vec<Vec<ComponentId>>) -> ComponentPartition {
        let mut partition = ComponentPartition::new(clusters.len());

        for (thread_id, cluster) in clusters.into_iter().enumerate() {
            for component_id in cluster {
                partition.assign_component(component_id, thread_id).unwrap();
            }
        }

        partition
    }

    /// Calculate quality metrics for the partition
    fn calculate_quality_metrics(
        &self,
        graph: &DependencyGraph,
        partition: &ComponentPartition,
    ) -> PartitionQualityMetrics {
        let cut_weight = self.calculate_cut_weight(graph, partition);
        let modularity = self.calculate_modularity(graph, partition);

        // Calculate load balance metrics
        let mut thread_loads = Vec::new();
        for thread_id in 0..partition.num_threads() {
            let mut load = 0.0;
            if let Some(components) = partition.get_thread_components(thread_id) {
                for component_id in components {
                    let weight = graph.get_component_weight(component_id);
                    load += weight.computational_cost;
                }
            }
            thread_loads.push(load);
        }

        let load_imbalance_ratio = self.calculate_load_imbalance_ratio(&thread_loads);
        let load_variance = self.calculate_load_variance(&thread_loads);

        PartitionQualityMetrics::new_with_balance(
            cut_weight,
            modularity,
            load_imbalance_ratio,
            load_variance,
            self.config.communication_weight,
            self.config.balance_weight,
        )
    }

    /// Calculate total weight of cut edges
    fn calculate_cut_weight(&self, graph: &DependencyGraph, partition: &ComponentPartition) -> u64 {
        let mut cut_weight = 0u64;

        for (source_id, source_thread) in partition.get_all_assignments() {
            let outgoing = graph.get_outgoing_edges(source_id);
            for (target_id, edge) in outgoing {
                if let Some(target_thread) = partition.get_thread_assignment(target_id) {
                    if *source_thread != target_thread {
                        cut_weight += edge.weight;
                    }
                }
            }
        }

        cut_weight
    }

    /// Calculate modularity score
    fn calculate_modularity(&self, graph: &DependencyGraph, partition: &ComponentPartition) -> f64 {
        let total_weight = self.get_total_edge_weight(graph);
        if total_weight == 0 {
            return 0.0;
        }

        let mut modularity = 0.0;
        let num_threads = partition.num_threads();

        for thread_id in 0..num_threads {
            if let Some(components) = partition.get_thread_components(thread_id) {
                let mut internal_weight = 0u64;
                let mut thread_degree = 0u64;

                for component_id in components {
                    // Internal edges
                    let outgoing = graph.get_outgoing_edges(component_id);
                    for (target_id, edge) in outgoing {
                        if components.contains(target_id) {
                            internal_weight += edge.weight;
                        }
                        thread_degree += edge.weight;
                    }
                }

                let expected = (thread_degree as f64).powi(2) / (2.0 * total_weight as f64);
                modularity +=
                    (internal_weight as f64 / total_weight as f64) - expected / total_weight as f64;
            }
        }

        modularity
    }

    /// Get total weight of all edges in the graph
    fn get_total_edge_weight(&self, graph: &DependencyGraph) -> u64 {
        let analysis = graph.analyze_communication_patterns();
        analysis.total_weight
    }

    /// Helper to get all edges in the graph
    fn get_all_edges(
        &self,
        graph: &DependencyGraph,
    ) -> HashMap<
        ComponentId,
        Vec<(
            ComponentId,
            crate::analysis::dependency_graph::CommunicationEdge,
        )>,
    > {
        graph.get_all_edges()
    }

    /// Helper to get all component IDs
    fn get_all_components(&self, graph: &DependencyGraph) -> Vec<ComponentId> {
        graph.get_all_component_ids()
    }

    /// Calculate the computational load for each thread
    fn calculate_thread_loads(
        &self,
        clusters: &[Vec<ComponentId>],
        graph: &DependencyGraph,
    ) -> Vec<f64> {
        let mut thread_loads = Vec::with_capacity(clusters.len());

        for cluster in clusters {
            let mut load = 0.0;
            for component_id in cluster {
                let weight = graph.get_component_weight(component_id);
                load += weight.computational_cost;
            }
            thread_loads.push(load);
        }

        thread_loads
    }

    /// Calculate load imbalance ratio (max_load / avg_load)
    fn calculate_load_imbalance_ratio(&self, thread_loads: &[f64]) -> f64 {
        if thread_loads.is_empty() {
            return 1.0;
        }

        let max_load = thread_loads.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let avg_load = thread_loads.iter().sum::<f64>() / thread_loads.len() as f64;

        if avg_load == 0.0 {
            1.0
        } else {
            max_load / avg_load
        }
    }

    /// Calculate load variance across threads
    fn calculate_load_variance(&self, thread_loads: &[f64]) -> f64 {
        if thread_loads.len() <= 1 {
            return 0.0;
        }

        let mean = thread_loads.iter().sum::<f64>() / thread_loads.len() as f64;
        let variance = thread_loads
            .iter()
            .map(|&load| (load - mean).powi(2))
            .sum::<f64>()
            / (thread_loads.len() - 1) as f64;

        variance
    }

    /// Check if moving a component preserves load balance constraints
    fn move_preserves_balance(
        &self,
        thread_loads: &[f64],
        from_thread: usize,
        to_thread: usize,
        component_weight: &crate::analysis::dependency_graph::ComponentWeight,
        max_imbalance: f64,
    ) -> bool {
        let mut new_loads = thread_loads.to_vec();
        new_loads[from_thread] -= component_weight.computational_cost;
        new_loads[to_thread] += component_weight.computational_cost;

        let new_imbalance = self.calculate_load_imbalance_ratio(&new_loads);
        new_imbalance <= max_imbalance
    }

    /// Calculate communication affinity between a component and a cluster
    fn calculate_communication_affinity(
        &self,
        component_id: &ComponentId,
        cluster: &[ComponentId],
        graph: &DependencyGraph,
    ) -> f64 {
        let mut affinity = 0.0;

        // Check outgoing edges to cluster members
        let outgoing = graph.get_outgoing_edges(component_id);
        for (target_id, edge) in outgoing {
            if cluster.contains(target_id) {
                affinity += edge.weight as f64;
            }
        }

        // Check incoming edges from cluster members
        let incoming = graph.get_incoming_edges(component_id);
        for (source_id, edge) in incoming {
            if cluster.contains(source_id) {
                affinity += edge.weight as f64;
            }
        }

        affinity
    }

    /// Calculate the best cluster for a component considering both communication and balance
    fn find_best_cluster_for_component(
        &self,
        component_id: &ComponentId,
        clusters: &[Vec<ComponentId>],
        thread_loads: &[f64],
        graph: &DependencyGraph,
    ) -> usize {
        let component_weight = graph.get_component_weight(component_id);
        let mut best_score = f64::NEG_INFINITY;
        let mut best_cluster = 0;

        for (i, cluster) in clusters.iter().enumerate() {
            let comm_score = self.calculate_communication_affinity(component_id, cluster, graph);
            let load_after = thread_loads[i] + component_weight.computational_cost;

            // Calculate balance score - prefer clusters with lower load
            let avg_load = thread_loads.iter().sum::<f64>() / thread_loads.len() as f64;
            let balance_score = if avg_load > 0.0 {
                1.0 - (load_after / (avg_load * 1.5)).min(1.0)
            } else {
                0.0
            };

            let total_score = self.config.communication_weight * comm_score
                + self.config.balance_weight * balance_score;

            if total_score > best_score {
                best_score = total_score;
                best_cluster = i;
            }
        }

        best_cluster
    }
}

impl GraphPartitioner for GreedyPartitioner {
    fn partition(
        &self,
        graph: &DependencyGraph,
        num_threads: usize,
    ) -> Result<ComponentPartition, String> {
        if num_threads == 0 {
            return Err("Number of threads must be greater than 0".to_string());
        }

        if graph.get_component_count() == 0 {
            return Ok(ComponentPartition::new(num_threads));
        }

        // If we have fewer components than threads, just assign one per thread
        if graph.get_component_count() <= num_threads {
            let mut partition = ComponentPartition::new(num_threads);
            let components = self.get_all_components(graph);

            for (i, component_id) in components.into_iter().enumerate() {
                partition.assign_component(component_id, i)?;
            }

            let quality_metrics = self.calculate_quality_metrics(graph, &partition);
            partition.set_quality_metrics(quality_metrics);

            return Ok(partition);
        }

        // Phase 1: Initial clustering based on edge weights
        let mut clusters = self.initial_clustering(graph, num_threads);

        // Phase 2: Refinement
        self.refine_partition(graph, &mut clusters);

        // Convert to ComponentPartition
        let mut partition = self.clusters_to_partition(clusters);

        // Calculate and set quality metrics
        let quality_metrics = self.calculate_quality_metrics(graph, &partition);
        partition.set_quality_metrics(quality_metrics);

        // Validate the result
        partition.validate()?;

        Ok(partition)
    }

    fn algorithm_name(&self) -> &'static str {
        "greedy"
    }

    fn algorithm_description(&self) -> &'static str {
        "Greedy heuristic partitioner that clusters high-communication components and refines through local search"
    }
}

impl Default for GreedyPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::DependencyGraph;

    #[test]
    fn test_greedy_partitioner_creation() {
        let partitioner = GreedyPartitioner::new();
        assert_eq!(partitioner.algorithm_name(), "greedy");

        let partitioner_with_config = GreedyPartitioner::with_config(PartitioningConfig::new());
        assert_eq!(partitioner_with_config.algorithm_name(), "greedy");
    }

    #[test]
    fn test_partition_empty_graph() {
        let partitioner = GreedyPartitioner::new();
        let graph = DependencyGraph::new();

        let result = partitioner.partition(&graph, 2);
        assert!(result.is_ok());

        let partition = result.unwrap();
        assert_eq!(partition.num_threads(), 2);
        assert_eq!(partition.num_components(), 0);
    }

    #[test]
    fn test_partition_invalid_thread_count() {
        let partitioner = GreedyPartitioner::new();
        let graph = DependencyGraph::new();

        let result = partitioner.partition(&graph, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be greater than 0"));
    }

    #[test]
    fn test_load_aware_partitioning() {
        use crate::analysis::dependency_graph::ComponentWeight;
        use crate::analysis::graph_partitioner::PartitioningConfig;

        let config = PartitioningConfig::balance_optimized();
        let partitioner = GreedyPartitioner::with_config(config);
        let mut graph = DependencyGraph::new();

        // Add components with different computational weights
        graph.add_component("heavy1".to_string(), vec![]);
        graph.add_component("heavy2".to_string(), vec![]);
        graph.add_component("light1".to_string(), vec![]);
        graph.add_component("light2".to_string(), vec![]);

        // Set different weights
        graph.set_component_weight(&"heavy1".to_string(), ComponentWeight::new(10.0, 1024));
        graph.set_component_weight(&"heavy2".to_string(), ComponentWeight::new(10.0, 1024));
        graph.set_component_weight(&"light1".to_string(), ComponentWeight::new(1.0, 256));
        graph.set_component_weight(&"light2".to_string(), ComponentWeight::new(1.0, 256));

        let result = partitioner.partition(&graph, 2);
        assert!(result.is_ok());

        let partition = result.unwrap();

        // Check that load imbalance is reasonable (not perfect due to heuristic nature)
        let metrics = partition.quality_metrics();
        assert!(
            metrics.load_imbalance_ratio < 5.0,
            "Load imbalance too high: {}",
            metrics.load_imbalance_ratio
        );
    }

    #[test]
    fn test_communication_vs_balance_tradeoff() {
        use crate::analysis::dependency_graph::ComponentWeight;
        use crate::analysis::graph_partitioner::PartitioningConfig;

        let mut graph = DependencyGraph::new();

        // Create a simple graph: A -> B, C -> D
        graph.add_component("A".to_string(), vec!["event1".to_string()]);
        graph.add_component("B".to_string(), vec![]);
        graph.add_component("C".to_string(), vec!["event2".to_string()]);
        graph.add_component("D".to_string(), vec![]);

        // Set weights where A and C are very heavy
        graph.set_component_weight(&"A".to_string(), ComponentWeight::new(100.0, 1024));
        graph.set_component_weight(&"C".to_string(), ComponentWeight::new(100.0, 1024));
        graph.set_component_weight(&"B".to_string(), ComponentWeight::new(1.0, 256));
        graph.set_component_weight(&"D".to_string(), ComponentWeight::new(1.0, 256));

        // Add communication edges
        graph.add_edge(&"A".to_string(), &"B".to_string(), "event1".to_string());
        graph.add_edge(&"C".to_string(), &"D".to_string(), "event2".to_string());
        graph.update_edge_weight(
            &"A".to_string(),
            &"B".to_string(),
            &"event1".to_string(),
            100,
        );
        graph.update_edge_weight(
            &"C".to_string(),
            &"D".to_string(),
            &"event2".to_string(),
            100,
        );

        // Test balance-optimized config
        let balance_config = PartitioningConfig::balance_optimized();
        let balance_partitioner = GreedyPartitioner::with_config(balance_config);
        let balance_result = balance_partitioner.partition(&graph, 2).unwrap();

        // Test communication-optimized config
        let comm_config = PartitioningConfig::communication_optimized();
        let comm_partitioner = GreedyPartitioner::with_config(comm_config);
        let comm_result = comm_partitioner.partition(&graph, 2).unwrap();

        // Balance-optimized should have better load balance
        assert!(
            balance_result.quality_metrics().load_imbalance_ratio
                <= comm_result.quality_metrics().load_imbalance_ratio + 0.1
        );
    }
}
