use crate::core::types::ComponentId;
use std::collections::HashMap;

pub type ThreadId = usize;

/// Represents the assignment of components to threads for parallel execution
#[derive(Debug, Clone)]
pub struct ComponentPartition {
    /// Maps each component to its assigned thread
    thread_assignments: HashMap<ComponentId, ThreadId>,
    /// Maps each thread to its list of assigned components
    thread_components: HashMap<ThreadId, Vec<ComponentId>>,
    /// Quality metrics for this partition
    quality_metrics: PartitionQualityMetrics,
    /// Number of threads in this partition
    num_threads: usize,
}

impl ComponentPartition {
    /// Create a new empty partition for the specified number of threads
    pub fn new(num_threads: usize) -> Self {
        let mut thread_components = HashMap::new();
        for thread_id in 0..num_threads {
            thread_components.insert(thread_id, Vec::new());
        }
        
        Self {
            thread_assignments: HashMap::new(),
            thread_components,
            quality_metrics: PartitionQualityMetrics::default(),
            num_threads,
        }
    }
    
    /// Assign a component to a specific thread
    pub fn assign_component(&mut self, component_id: ComponentId, thread_id: ThreadId) -> Result<(), String> {
        if thread_id >= self.num_threads {
            return Err(format!("Thread ID {} exceeds number of threads {}", thread_id, self.num_threads));
        }
        
        // Remove from previous thread if already assigned
        if let Some(old_thread_id) = self.thread_assignments.get(&component_id) {
            if let Some(components) = self.thread_components.get_mut(old_thread_id) {
                components.retain(|id| id != &component_id);
            }
        }
        
        // Assign to new thread
        self.thread_assignments.insert(component_id.clone(), thread_id);
        self.thread_components.get_mut(&thread_id)
            .unwrap()
            .push(component_id);
        
        Ok(())
    }
    
    /// Get the thread ID assigned to a component
    pub fn get_thread_assignment(&self, component_id: &ComponentId) -> Option<ThreadId> {
        self.thread_assignments.get(component_id).copied()
    }
    
    /// Get all components assigned to a specific thread
    pub fn get_thread_components(&self, thread_id: ThreadId) -> Option<&Vec<ComponentId>> {
        self.thread_components.get(&thread_id)
    }
    
    /// Get all thread assignments as a map
    pub fn get_all_assignments(&self) -> &HashMap<ComponentId, ThreadId> {
        &self.thread_assignments
    }
    
    /// Get the number of threads in this partition
    pub fn num_threads(&self) -> usize {
        self.num_threads
    }
    
    /// Get the number of components assigned
    pub fn num_components(&self) -> usize {
        self.thread_assignments.len()
    }
    
    /// Update the quality metrics for this partition
    pub fn set_quality_metrics(&mut self, metrics: PartitionQualityMetrics) {
        self.quality_metrics = metrics;
    }
    
    /// Get the quality metrics for this partition
    pub fn quality_metrics(&self) -> &PartitionQualityMetrics {
        &self.quality_metrics
    }
    
    /// Check if all components are assigned to exactly one thread
    pub fn validate(&self) -> Result<(), String> {
        // Check that all threads have valid IDs
        for thread_id in self.thread_assignments.values() {
            if *thread_id >= self.num_threads {
                return Err(format!("Invalid thread ID {} (max: {})", thread_id, self.num_threads - 1));
            }
        }
        
        // Check consistency between the two maps
        let mut component_count_per_thread = vec![0; self.num_threads];
        for thread_id in self.thread_assignments.values() {
            component_count_per_thread[*thread_id] += 1;
        }
        
        for (thread_id, components) in &self.thread_components {
            if component_count_per_thread[*thread_id] != components.len() {
                return Err(format!("Inconsistent component count for thread {}", thread_id));
            }
        }
        
        Ok(())
    }
    
    /// Get load balance statistics
    pub fn load_balance_stats(&self) -> LoadBalanceStats {
        let component_counts: Vec<usize> = (0..self.num_threads)
            .map(|thread_id| self.thread_components.get(&thread_id).map_or(0, |v| v.len()))
            .collect();
        
        let total_components = component_counts.iter().sum::<usize>();
        let min_components = *component_counts.iter().min().unwrap_or(&0);
        let max_components = *component_counts.iter().max().unwrap_or(&0);
        let avg_components = if self.num_threads > 0 {
            total_components as f64 / self.num_threads as f64
        } else {
            0.0
        };
        
        // Calculate variance
        let variance = if self.num_threads > 0 {
            component_counts.iter()
                .map(|&count| (count as f64 - avg_components).powi(2))
                .sum::<f64>() / self.num_threads as f64
        } else {
            0.0
        };
        
        LoadBalanceStats {
            min_components,
            max_components,
            avg_components,
            variance,
            imbalance_ratio: if min_components > 0 { max_components as f64 / min_components as f64 } else { f64::INFINITY },
        }
    }
    
    /// Print a summary of this partition
    pub fn print_summary(&self) {
        println!("=== Component Partition Summary ===");
        println!("Number of threads: {}", self.num_threads);
        println!("Number of components: {}", self.num_components());
        
        let load_stats = self.load_balance_stats();
        println!("Load balance - Min: {}, Max: {}, Avg: {:.1}, Variance: {:.2}", 
            load_stats.min_components, load_stats.max_components, 
            load_stats.avg_components, load_stats.variance);
        
        println!("Quality metrics:");
        println!("  Cut weight: {}", self.quality_metrics.cut_weight);
        println!("  Modularity: {:.4}", self.quality_metrics.modularity);
        println!("  Load balance score: {:.4}", self.quality_metrics.load_balance_score);
        
        // Show components per thread
        for thread_id in 0..self.num_threads {
            if let Some(components) = self.thread_components.get(&thread_id) {
                println!("Thread {}: {} components", thread_id, components.len());
                if components.len() <= 5 {
                    println!("  Components: {:?}", components);
                } else {
                    println!("  Components: {:?}... ({} more)", 
                        &components[0..3], components.len() - 3);
                }
            }
        }
    }
}

/// Quality metrics for evaluating partition quality
#[derive(Debug, Clone, Default)]
pub struct PartitionQualityMetrics {
    /// Total weight of edges that cross thread boundaries (lower is better)
    pub cut_weight: u64,
    /// Modularity score measuring how well partitioned the graph is (higher is better, range -1 to 1)
    pub modularity: f64,
    /// Load balance score (higher is better, 1.0 = perfect balance)
    pub load_balance_score: f64,
    /// Combined quality score (higher is better)
    pub overall_score: f64,
}

impl PartitionQualityMetrics {
    /// Create new quality metrics
    pub fn new(cut_weight: u64, modularity: f64, load_balance_score: f64) -> Self {
        // Simple weighted combination - can be tuned
        let overall_score = modularity * 0.6 + load_balance_score * 0.4 - (cut_weight as f64 / 1000.0) * 0.1;
        
        Self {
            cut_weight,
            modularity,
            load_balance_score,
            overall_score,
        }
    }
}

/// Load balance statistics for a partition
#[derive(Debug, Clone)]
pub struct LoadBalanceStats {
    pub min_components: usize,
    pub max_components: usize,
    pub avg_components: f64,
    pub variance: f64,
    pub imbalance_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_partition_creation() {
        let partition = ComponentPartition::new(3);
        assert_eq!(partition.num_threads(), 3);
        assert_eq!(partition.num_components(), 0);
        
        for i in 0..3 {
            assert_eq!(partition.get_thread_components(i).unwrap().len(), 0);
        }
    }
    
    #[test]
    fn test_component_assignment() {
        let mut partition = ComponentPartition::new(2);
        
        // Assign components
        assert!(partition.assign_component("comp1".to_string(), 0).is_ok());
        assert!(partition.assign_component("comp2".to_string(), 1).is_ok());
        assert!(partition.assign_component("comp3".to_string(), 0).is_ok());
        
        // Check assignments
        assert_eq!(partition.get_thread_assignment(&"comp1".to_string()), Some(0));
        assert_eq!(partition.get_thread_assignment(&"comp2".to_string()), Some(1));
        assert_eq!(partition.get_thread_assignment(&"comp3".to_string()), Some(0));
        
        // Check thread components
        assert_eq!(partition.get_thread_components(0).unwrap().len(), 2);
        assert_eq!(partition.get_thread_components(1).unwrap().len(), 1);
        
        // Test invalid thread ID
        assert!(partition.assign_component("comp4".to_string(), 2).is_err());
    }
    
    #[test]
    fn test_component_reassignment() {
        let mut partition = ComponentPartition::new(2);
        
        // Assign component to thread 0
        partition.assign_component("comp1".to_string(), 0).unwrap();
        assert_eq!(partition.get_thread_components(0).unwrap().len(), 1);
        assert_eq!(partition.get_thread_components(1).unwrap().len(), 0);
        
        // Reassign to thread 1
        partition.assign_component("comp1".to_string(), 1).unwrap();
        assert_eq!(partition.get_thread_components(0).unwrap().len(), 0);
        assert_eq!(partition.get_thread_components(1).unwrap().len(), 1);
        assert_eq!(partition.get_thread_assignment(&"comp1".to_string()), Some(1));
    }
    
    #[test]
    fn test_validation() {
        let mut partition = ComponentPartition::new(2);
        
        // Empty partition should be valid
        assert!(partition.validate().is_ok());
        
        // Valid partition
        partition.assign_component("comp1".to_string(), 0).unwrap();
        partition.assign_component("comp2".to_string(), 1).unwrap();
        assert!(partition.validate().is_ok());
    }
    
    #[test]
    fn test_load_balance_stats() {
        let mut partition = ComponentPartition::new(3);
        
        // Unbalanced assignment
        partition.assign_component("comp1".to_string(), 0).unwrap();
        partition.assign_component("comp2".to_string(), 0).unwrap();
        partition.assign_component("comp3".to_string(), 0).unwrap();
        partition.assign_component("comp4".to_string(), 1).unwrap();
        // Thread 2 has no components
        
        let stats = partition.load_balance_stats();
        assert_eq!(stats.min_components, 0);
        assert_eq!(stats.max_components, 3);
        assert!((stats.avg_components - 4.0/3.0).abs() < 0.001);
        assert!(stats.variance > 0.0);
        assert_eq!(stats.imbalance_ratio, f64::INFINITY); // Division by zero case
    }
}