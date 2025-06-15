use crate::analysis::{ComponentPartition, DependencyGraph};

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
    fn partition(
        &self,
        graph: &DependencyGraph,
        num_threads: usize,
    ) -> Result<ComponentPartition, String>;

    /// Get the name of this partitioning algorithm
    fn algorithm_name(&self) -> &'static str;

    /// Get a description of this partitioning algorithm
    fn algorithm_description(&self) -> &'static str {
        "No description available"
    }
}

/// Configuration options for graph partitioning algorithms focused on communication minimization
#[derive(Debug, Clone)]
pub struct PartitioningConfig {
    /// Random seed for reproducible results (if applicable)
    pub random_seed: Option<u64>,
    /// Maximum number of refinement iterations
    pub max_refinement_iterations: usize,
    /// Weight for communication minimization objective (0.0 to 1.0)
    pub communication_weight: f64,
    /// Weight for load balancing objective (0.0 to 1.0)
    pub balance_weight: f64,
    /// Maximum allowed load imbalance ratio (e.g., 1.2 = 20% imbalance allowed)
    pub max_load_imbalance: f64,
}

impl Default for PartitioningConfig {
    fn default() -> Self {
        Self {
            random_seed: Some(42),
            max_refinement_iterations: 10,
            communication_weight: 0.7,
            balance_weight: 0.3,
            max_load_imbalance: 1.2,
        }
    }
}

impl PartitioningConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with custom refinement iterations
    pub fn with_refinement_iterations(iterations: usize) -> Self {
        Self {
            max_refinement_iterations: iterations,
            ..Default::default()
        }
    }

    /// Create a configuration with custom random seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            random_seed: Some(seed),
            ..Default::default()
        }
    }

    /// Create a configuration optimized for communication minimization
    pub fn communication_optimized() -> Self {
        Self {
            communication_weight: 0.9,
            balance_weight: 0.1,
            max_load_imbalance: 1.5,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for load balancing
    pub fn balance_optimized() -> Self {
        Self {
            communication_weight: 0.3,
            balance_weight: 0.7,
            max_load_imbalance: 1.1,
            ..Default::default()
        }
    }

    /// Create a configuration with custom weights
    pub fn with_weights(
        communication_weight: f64,
        balance_weight: f64,
        max_load_imbalance: f64,
    ) -> Self {
        Self {
            communication_weight,
            balance_weight,
            max_load_imbalance,
            ..Default::default()
        }
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
        assert_eq!(config.random_seed, Some(42));
        assert_eq!(config.max_refinement_iterations, 10);
    }

    #[test]
    fn test_partitioning_config_creation() {
        let config = PartitioningConfig::new();
        assert_eq!(config.random_seed, Some(42));
        assert_eq!(config.max_refinement_iterations, 10);

        let config_with_iterations = PartitioningConfig::with_refinement_iterations(20);
        assert_eq!(config_with_iterations.max_refinement_iterations, 20);
        assert_eq!(config_with_iterations.random_seed, Some(42));

        let config_with_seed = PartitioningConfig::with_seed(123);
        assert_eq!(config_with_seed.random_seed, Some(123));
        assert_eq!(config_with_seed.max_refinement_iterations, 10);
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
