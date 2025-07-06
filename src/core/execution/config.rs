/// Configuration for RSim simulation execution
/// 
/// This module provides configuration types for controlling simulation execution behavior,
/// including concurrency settings and thread pool management.

/// Enumeration of supported concurrency modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyMode {
    /// Sequential execution mode - components are executed in order within a single thread
    Sequential,
    /// Parallel execution mode using Rayon - components can be executed concurrently
    Rayon,
}

impl Default for ConcurrencyMode {
    fn default() -> Self {
        ConcurrencyMode::Sequential
    }
}

/// Configuration for simulation execution
/// 
/// This struct holds configuration options that control how the simulation is executed,
/// including concurrency settings and resource management.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// The concurrency mode to use for execution
    pub concurrency_mode: ConcurrencyMode,
    /// The size of the thread pool for parallel execution
    /// Only relevant when concurrency_mode is Rayon
    pub thread_pool_size: Option<usize>,
}

impl SimulationConfig {
    /// Create a new simulation configuration with default values
    /// 
    /// Default configuration uses Sequential mode with no thread pool
    pub fn new() -> Self {
        Self {
            concurrency_mode: ConcurrencyMode::default(),
            thread_pool_size: None,
        }
    }
    
    /// Set the concurrency mode for the simulation
    /// 
    /// # Arguments
    /// * `mode` - The concurrency mode to use
    /// 
    /// # Returns
    /// A new configuration with the specified concurrency mode
    pub fn with_concurrency(mut self, mode: ConcurrencyMode) -> Self {
        self.concurrency_mode = mode;
        self
    }
    
    /// Set the thread pool size for parallel execution
    /// 
    /// # Arguments
    /// * `size` - The number of threads to use in the thread pool
    /// 
    /// # Returns
    /// A new configuration with the specified thread pool size
    /// 
    /// # Note
    /// This setting only affects execution when concurrency_mode is Rayon
    pub fn with_thread_pool_size(mut self, size: usize) -> Self {
        self.thread_pool_size = Some(size);
        self
    }
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SimulationConfig::default();
        assert_eq!(config.concurrency_mode, ConcurrencyMode::Sequential);
        assert_eq!(config.thread_pool_size, None);
    }

    #[test]
    fn test_config_builder() {
        let config = SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon)
            .with_thread_pool_size(4);
        
        assert_eq!(config.concurrency_mode, ConcurrencyMode::Rayon);
        assert_eq!(config.thread_pool_size, Some(4));
    }

    #[test]
    fn test_concurrency_mode_default() {
        let mode = ConcurrencyMode::default();
        assert_eq!(mode, ConcurrencyMode::Sequential);
    }

    #[test]
    fn test_concurrency_mode_equality() {
        assert_eq!(ConcurrencyMode::Sequential, ConcurrencyMode::Sequential);
        assert_eq!(ConcurrencyMode::Rayon, ConcurrencyMode::Rayon);
        assert_ne!(ConcurrencyMode::Sequential, ConcurrencyMode::Rayon);
    }
}