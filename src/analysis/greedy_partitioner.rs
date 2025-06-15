use crate::analysis::{DependencyGraph, ComponentPartition, PartitionQualityMetrics};
use crate::analysis::graph_partitioner::{GraphPartitioner, PartitioningConfig};
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
    
    /// Phase 1: Group components connected by high-weight edges
    fn initial_clustering(&self, graph: &DependencyGraph, num_threads: usize) -> Vec<Vec<ComponentId>> {
        let mut clusters = Vec::new();
        let mut assigned = HashSet::new();
        
        // Get all edges sorted by weight (descending)
        let mut weighted_edges = Vec::new();
        
        // Access the internal graph structure
        // We'll need to add methods to DependencyGraph to expose this functionality
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
            
            if !assigned.contains(&source_id) && !assigned.contains(&target_id) {
                // Neither assigned - create new cluster
                if clusters.len() < num_threads {
                    let new_cluster = vec![source_id.clone(), target_id.clone()];
                    assigned.insert(source_id);
                    assigned.insert(target_id);
                    clusters.push(new_cluster);
                } else {
                    // No more threads available, add to smallest cluster
                    let mut smallest_idx = 0;
                    let mut smallest_size = clusters[0].len();
                    for (i, cluster) in clusters.iter().enumerate() {
                        if cluster.len() < smallest_size {
                            smallest_size = cluster.len();
                            smallest_idx = i;
                        }
                    }
                    clusters[smallest_idx].push(source_id.clone());
                    clusters[smallest_idx].push(target_id.clone());
                    assigned.insert(source_id);
                    assigned.insert(target_id);
                }
            } else if !assigned.contains(&source_id) {
                // Only source unassigned - add to target's cluster
                if let Some(target_cluster_idx) = self.find_component_cluster(&clusters, &target_id) {
                    clusters[target_cluster_idx].push(source_id.clone());
                    assigned.insert(source_id);
                }
            } else if !assigned.contains(&target_id) {
                // Only target unassigned - add to source's cluster
                if let Some(source_cluster_idx) = self.find_component_cluster(&clusters, &source_id) {
                    clusters[source_cluster_idx].push(target_id.clone());
                    assigned.insert(target_id);
                }
            }
        }
        
        // Assign remaining unassigned components
        let all_components = self.get_all_components(graph);
        for component_id in all_components {
            if !assigned.contains(&component_id) {
                // Add to smallest cluster, or create new one if under thread limit
                if clusters.len() < num_threads {
                    clusters.push(vec![component_id.clone()]);
                } else {
                    let mut smallest_idx = 0;
                    let mut smallest_size = clusters[0].len();
                    for (i, cluster) in clusters.iter().enumerate() {
                        if cluster.len() < smallest_size {
                            smallest_size = cluster.len();
                            smallest_idx = i;
                        }
                    }
                    clusters[smallest_idx].push(component_id.clone());
                }
                assigned.insert(component_id);
            }
        }
        
        // Ensure we have exactly num_threads clusters
        while clusters.len() < num_threads {
            clusters.push(Vec::new());
        }
        
        clusters
    }
    
    /// Phase 2: Refine partition by moving components to reduce cut weight
    fn refine_partition(&self, graph: &DependencyGraph, clusters: &mut [Vec<ComponentId>]) {
        let max_iterations = self.config.max_refinement_iterations;
        
        for _iteration in 0..max_iterations {
            let mut improved = false;
            
            // Try moving each component to a different thread
            for current_thread in 0..clusters.len() {
                let components_to_check: Vec<ComponentId> = clusters[current_thread].clone();
                
                for component_id in components_to_check {
                    let current_cost = self.calculate_component_cut_cost(graph, &component_id, current_thread, clusters);
                    
                    // Try all other threads
                    for target_thread in 0..clusters.len() {
                        if target_thread == current_thread {
                            continue;
                        }
                        
                        // Check if move would violate balance constraints
                        let imbalance_after_move = self.calculate_imbalance_after_move(
                            clusters, current_thread, target_thread
                        );
                        
                        if imbalance_after_move > self.config.max_imbalance_ratio {
                            continue;
                        }
                        
                        let target_cost = self.calculate_component_cut_cost(graph, &component_id, target_thread, clusters);
                        
                        // Move if it reduces cost
                        if target_cost < current_cost {
                            // Perform the move
                            clusters[current_thread].retain(|id| id != &component_id);
                            clusters[target_thread].push(component_id.clone());
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
    fn calculate_component_cut_cost(&self, graph: &DependencyGraph, component_id: &ComponentId, thread_id: usize, clusters: &[Vec<ComponentId>]) -> u64 {
        let mut cut_cost = 0u64;
        
        // Check outgoing edges
        let outgoing = graph.get_outgoing_edges(component_id);
        for (target_id, edge) in outgoing {
            let target_thread = self.find_component_cluster(clusters, target_id);
            if let Some(target_thread_id) = target_thread {
                if target_thread_id != thread_id {
                    cut_cost += edge.weight;
                }
            }
        }
        
        // Check incoming edges  
        let incoming = graph.get_incoming_edges(component_id);
        for (source_id, edge) in incoming {
            let source_thread = self.find_component_cluster(clusters, source_id);
            if let Some(source_thread_id) = source_thread {
                if source_thread_id != thread_id {
                    cut_cost += edge.weight;
                }
            }
        }
        
        cut_cost
    }
    
    /// Calculate imbalance ratio after moving a component between threads
    fn calculate_imbalance_after_move(&self, clusters: &[Vec<ComponentId>], from_thread: usize, to_thread: usize) -> f64 {
        let mut sizes: Vec<usize> = clusters.iter().map(|cluster| cluster.len()).collect();
        
        if sizes[from_thread] > 0 {
            sizes[from_thread] -= 1;
        }
        sizes[to_thread] += 1;
        
        let min_size = *sizes.iter().min().unwrap_or(&0);
        let max_size = *sizes.iter().max().unwrap_or(&0);
        
        if min_size == 0 {
            f64::INFINITY
        } else {
            max_size as f64 / min_size as f64
        }
    }
    
    /// Find which cluster contains a component
    fn find_component_cluster(&self, clusters: &[Vec<ComponentId>], component_id: &ComponentId) -> Option<usize> {
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
    fn calculate_quality_metrics(&self, graph: &DependencyGraph, partition: &ComponentPartition) -> PartitionQualityMetrics {
        let cut_weight = self.calculate_cut_weight(graph, partition);
        let modularity = self.calculate_modularity(graph, partition);
        let load_balance_score = self.calculate_load_balance_score(partition);
        
        PartitionQualityMetrics::new(cut_weight, modularity, load_balance_score)
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
                modularity += (internal_weight as f64 / total_weight as f64) - expected / total_weight as f64;
            }
        }
        
        modularity
    }
    
    /// Calculate load balance score (1.0 = perfect balance)
    fn calculate_load_balance_score(&self, partition: &ComponentPartition) -> f64 {
        let stats = partition.load_balance_stats();
        if stats.imbalance_ratio == f64::INFINITY || stats.max_components == 0 {
            return 0.0;
        }
        
        // Score is inversely related to imbalance ratio
        // Perfect balance (ratio = 1.0) gives score = 1.0
        1.0 / stats.imbalance_ratio
    }
    
    /// Get total weight of all edges in the graph
    fn get_total_edge_weight(&self, graph: &DependencyGraph) -> u64 {
        let analysis = graph.analyze_communication_patterns();
        analysis.total_weight
    }
    
    /// Helper to get all edges in the graph
    fn get_all_edges(&self, graph: &DependencyGraph) -> HashMap<ComponentId, Vec<(ComponentId, crate::analysis::dependency_graph::CommunicationEdge)>> {
        graph.get_all_edges()
    }
    
    /// Helper to get all component IDs
    fn get_all_components(&self, graph: &DependencyGraph) -> Vec<ComponentId> {
        graph.get_all_component_ids()
    }
}

impl GraphPartitioner for GreedyPartitioner {
    fn partition(&self, graph: &DependencyGraph, num_threads: usize) -> Result<ComponentPartition, String> {
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
        
        let partitioner_with_config = GreedyPartitioner::with_config(PartitioningConfig::balance_load());
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
    fn test_calculate_imbalance_after_move() {
        let partitioner = GreedyPartitioner::new();
        let clusters = vec![
            vec!["comp1".to_string(), "comp2".to_string()], // 2 components
            vec!["comp3".to_string()],                       // 1 component  
            vec![]                                           // 0 components
        ];
        
        // Move from thread 0 to thread 1: [1, 2, 0]
        let imbalance = partitioner.calculate_imbalance_after_move(&clusters, 0, 1);
        assert_eq!(imbalance, f64::INFINITY); // 2/0 = infinity
        
        // Move from thread 0 to thread 2: [1, 1, 1] 
        let imbalance = partitioner.calculate_imbalance_after_move(&clusters, 0, 2);
        assert!((imbalance - 1.0).abs() < 0.001);
    }
}