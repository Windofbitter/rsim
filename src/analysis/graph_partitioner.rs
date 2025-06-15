use crate::analysis::{DependencyGraph, ComponentPartition};

/// Trait for graph partitioning algorithms that assign components to threads
pub trait GraphPartitioner {
    /// Partition the given dependency graph into the specified number of threads
    /// 
    /// # Arguments
    /// * `graph` - The weighted dependency graph to partition
    /// * `num_threads` - Target number of threads (must be > 0)
    /// 
    /// # Returns
    /// * `Ok(ComponentPartition)` - Successful partitioning with quality metrics
    /// * `Err(String)` - Error message if partitioning fails
    fn partition(&self, graph: &DependencyGraph, num_threads: usize) -> Result<ComponentPartition, String>;
    
    /// Get the name of this partitioning algorithm
    fn algorithm_name(&self) -> &'static str;
    
    /// Get a description of this partitioning algorithm
    fn algorithm_description(&self) -> &'static str {
        "No description available"
    }
}

/// Configuration options for graph partitioning algorithms
#[derive(Debug, Clone)]
pub struct PartitioningConfig {
    /// Weight given to minimizing cut edges (0.0 to 1.0)
    pub cut_weight_importance: f64,
    /// Weight given to load balancing (0.0 to 1.0) 
    pub load_balance_importance: f64,
    /// Maximum allowed imbalance ratio (max_load / min_load)
    pub max_imbalance_ratio: f64,
    /// Random seed for reproducible results (if applicable)
    pub random_seed: Option<u64>,
    /// Maximum number of refinement iterations
    pub max_refinement_iterations: usize,
}

impl Default for PartitioningConfig {
    fn default() -> Self {
        Self {
            cut_weight_importance: 0.7,
            load_balance_importance: 0.3,
            max_imbalance_ratio: 2.0,
            random_seed: Some(42),
            max_refinement_iterations: 10,
        }
    }
}

impl PartitioningConfig {
    /// Create a configuration that prioritizes minimizing communication
    pub fn minimize_communication() -> Self {
        Self {
            cut_weight_importance: 0.9,
            load_balance_importance: 0.1,
            max_imbalance_ratio: 3.0,
            ..Default::default()
        }
    }
    
    /// Create a configuration that prioritizes load balancing
    pub fn balance_load() -> Self {
        Self {
            cut_weight_importance: 0.3,
            load_balance_importance: 0.7,
            max_imbalance_ratio: 1.2,
            ..Default::default()
        }
    }
    
    /// Create a balanced configuration
    pub fn balanced() -> Self {
        Self::default()
    }
}

/// Factory for creating partitioner instances
pub struct PartitionerFactory;

impl PartitionerFactory {
    /// Create a greedy partitioner with default configuration
    pub fn create_greedy() -> Box<dyn GraphPartitioner> {
        Box::new(crate::analysis::greedy_partitioner::GreedyPartitioner::new())
    }
    
    /// Create a greedy partitioner with custom configuration
    pub fn create_greedy_with_config(config: PartitioningConfig) -> Box<dyn GraphPartitioner> {
        Box::new(crate::analysis::greedy_partitioner::GreedyPartitioner::with_config(config))
    }
    
    /// Get a list of available partitioning algorithms
    pub fn available_algorithms() -> Vec<&'static str> {
        vec!["greedy"]
    }
    
    /// Create a partitioner by name
    pub fn create_by_name(algorithm: &str) -> Result<Box<dyn GraphPartitioner>, String> {
        match algorithm.to_lowercase().as_str() {
            "greedy" => Ok(Self::create_greedy()),
            _ => Err(format!("Unknown partitioning algorithm: {}", algorithm)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_partitioning_config_defaults() {
        let config = PartitioningConfig::default();
        assert!((config.cut_weight_importance - 0.7).abs() < 0.001);
        assert!((config.load_balance_importance - 0.3).abs() < 0.001);
        assert!((config.max_imbalance_ratio - 2.0).abs() < 0.001);
        assert_eq!(config.random_seed, Some(42));
        assert_eq!(config.max_refinement_iterations, 10);
    }
    
    #[test]
    fn test_specialized_configs() {
        let comm_config = PartitioningConfig::minimize_communication();
        assert!((comm_config.cut_weight_importance - 0.9).abs() < 0.001);
        assert!((comm_config.load_balance_importance - 0.1).abs() < 0.001);
        
        let balance_config = PartitioningConfig::balance_load();
        assert!((balance_config.cut_weight_importance - 0.3).abs() < 0.001);
        assert!((balance_config.load_balance_importance - 0.7).abs() < 0.001);
        assert!((balance_config.max_imbalance_ratio - 1.2).abs() < 0.001);
    }
    
    #[test] 
    fn test_factory_available_algorithms() {
        let algorithms = PartitionerFactory::available_algorithms();
        assert!(algorithms.contains(&"greedy"));
        assert!(!algorithms.is_empty());
    }
    
    #[test]
    fn test_factory_create_by_name() {
        // Valid algorithm
        let result = PartitionerFactory::create_by_name("greedy");
        assert!(result.is_ok());
        
        // Invalid algorithm  
        match PartitionerFactory::create_by_name("nonexistent") {
            Ok(_) => panic!("Should have failed for unknown algorithm"),
            Err(error_msg) => {
                assert!(error_msg.contains("Unknown partitioning algorithm"));
            }
        }
    }
}