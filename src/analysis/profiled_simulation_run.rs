use crate::core::simulation_engine::SimulationEngine;
use crate::core::component::BaseComponent;
use crate::core::event::Event;
use crate::analysis::{DependencyGraph, ProfilingReport, ComponentPartition, GraphPartitioner, PartitionerFactory};

/// Orchestrates a profiling run to weight the dependency graph
pub struct ProfiledSimulationRun {
    profiling_cycles: Option<u64>,
}

impl ProfiledSimulationRun {
    /// Create a new profiled simulation run with default settings
    pub fn new() -> Self {
        Self {
            profiling_cycles: Some(1000), // Default to 1000 cycles for profiling
        }
    }
    
    /// Set the number of cycles to run for profiling (None = unlimited)
    pub fn with_profiling_cycles(mut self, cycles: Option<u64>) -> Self {
        self.profiling_cycles = cycles;
        self
    }
    
    /// Run the complete profiling pipeline and return a weighted dependency graph
    /// 
    /// The component_factory function should create a fresh set of components for profiling.
    /// This is necessary because profiling needs its own component instances.
    pub fn run_profiling_pipeline<F>(
        &self,
        component_factory: F,
        initial_events: Vec<(Box<dyn Event>, u64)>,
    ) -> Result<ProfiledSimulationResult, String>
    where
        F: Fn() -> Vec<Box<dyn BaseComponent>>,
    {
        // Step 1: Create components for dependency graph analysis
        let analysis_components = component_factory();
        let mut dependency_graph = DependencyGraph::new();
        dependency_graph.build_from_components(analysis_components.iter());
        
        println!("Built initial dependency graph: {} components, {} edges", 
            dependency_graph.get_component_count(), 
            dependency_graph.get_edge_count());
        
        // Step 2: Create fresh components for profiling simulation
        let profiling_components = component_factory();
        let mut engine = SimulationEngine::with_profiling(self.profiling_cycles);
        
        // Register all components with the engine
        for component in profiling_components {
            engine.register_component(component)?;
        }
        
        // Schedule initial events
        for (event, delay) in initial_events {
            engine.schedule_initial_event(event, delay);
        }
        
        println!("Starting profiling run...");
        let final_cycle = engine.run()?;
        println!("Profiling run completed at cycle {}", final_cycle);
        
        // Step 3: Extract profiling data and weight the graph
        // Since we used with_profiling(), profiler is guaranteed to exist
        let profiler = engine.profiler().unwrap();
        let profiling_report = Some(profiler.generate_report());
        profiler.apply_weights_to_graph(&mut dependency_graph);
        
        println!("Applied profiling weights to dependency graph");
        
        Ok(ProfiledSimulationResult {
            weighted_graph: dependency_graph,
            profiling_report,
            final_cycle,
            partition: None,
        })
    }
    
    /// Quick profiling run that just returns the profiling report without building a full graph
    pub fn quick_profile<F>(
        &self,
        component_factory: F,
        initial_events: Vec<(Box<dyn Event>, u64)>,
    ) -> Result<ProfilingReport, String>
    where
        F: Fn() -> Vec<Box<dyn BaseComponent>>,
    {
        let mut engine = SimulationEngine::with_profiling(self.profiling_cycles);
        
        // Register components
        let components = component_factory();
        for component in components {
            engine.register_component(component)?;
        }
        
        // Schedule initial events
        for (event, delay) in initial_events {
            engine.schedule_initial_event(event, delay);
        }
        
        println!("Starting quick profiling run...");
        engine.run()?;
        
        engine.profiler()
            .map(|profiler| profiler.generate_report())
            .ok_or_else(|| "Profiling was not enabled".to_string())
    }
}

impl Default for ProfiledSimulationRun {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a complete profiling pipeline run
pub struct ProfiledSimulationResult {
    /// The dependency graph with profiling weights applied
    pub weighted_graph: DependencyGraph,
    /// Detailed report of the profiling data
    pub profiling_report: Option<ProfilingReport>,
    /// Final cycle reached during profiling
    pub final_cycle: u64,
    /// Component partition assignments (if partitioning was performed)
    pub partition: Option<ComponentPartition>,
}

impl ProfiledSimulationResult {
    /// Print a summary of the profiling results
    pub fn print_summary(&self) {
        println!("=== Profiled Simulation Summary ===");
        println!("Final simulation cycle: {}", self.final_cycle);
        println!("Weighted graph: {} components, {} edges", 
            self.weighted_graph.get_component_count(), 
            self.weighted_graph.get_edge_count());
        
        if let Some(ref report) = self.profiling_report {
            report.print();
        }
        
        let analysis = self.weighted_graph.analyze_communication_patterns();
        println!("\n=== Communication Analysis ===");
        println!("Total communication weight: {}", analysis.total_weight);
        println!("Max edge weight: {}", analysis.max_weight);
        println!("Average edge weight: {:.2}", 
            analysis.total_weight as f64 / analysis.edge_count as f64);
        
        if let Some(ref partition) = self.partition {
            println!("\n=== Partition Summary ===");
            println!("Partitioned into {} threads", partition.num_threads());
            println!("Cut weight: {}", partition.quality_metrics().cut_weight);
            println!("Load balance score: {:.3}", partition.quality_metrics().load_balance_score);
            println!("Overall quality score: {:.3}", partition.quality_metrics().overall_score);
        } else {
            println!("\n=== Partition Status ===");
            println!("No partitioning performed yet. Use partition_components() to create thread assignments.");
        }
    }
    
    /// Export the weighted dependency graph to DOT format
    pub fn export_dot(&self) -> String {
        self.weighted_graph.to_dot()
    }
    
    /// Export the weighted dependency graph to Mermaid format
    pub fn export_mermaid(&self) -> String {
        self.weighted_graph.to_mermaid()
    }
    
    /// Partition components into threads using the specified algorithm
    /// 
    /// # Arguments
    /// * `num_threads` - Target number of threads
    /// * `algorithm` - Partitioning algorithm name ("greedy" is supported)
    /// 
    /// # Returns
    /// * `Ok(())` - Partitioning successful, results stored in self.partition
    /// * `Err(String)` - Partitioning failed with error message
    pub fn partition_components(&mut self, num_threads: usize, algorithm: &str) -> Result<(), String> {
        let partitioner = PartitionerFactory::create_by_name(algorithm)?;
        let partition = partitioner.partition(&self.weighted_graph, num_threads)?;
        
        println!("Partitioned {} components into {} threads using {} algorithm",
                 partition.num_components(), partition.num_threads(), algorithm);
        println!("Partition quality - Cut weight: {}, Load balance score: {:.3}",
                 partition.quality_metrics().cut_weight,
                 partition.quality_metrics().load_balance_score);
        
        self.partition = Some(partition);
        Ok(())
    }
    
    /// Partition components using a custom partitioner
    pub fn partition_components_with_partitioner(&mut self, partitioner: Box<dyn GraphPartitioner>, num_threads: usize) -> Result<(), String> {
        let partition = partitioner.partition(&self.weighted_graph, num_threads)?;
        
        println!("Partitioned {} components into {} threads using {} algorithm",
                 partition.num_components(), partition.num_threads(), partitioner.algorithm_name());
        println!("Partition quality - Cut weight: {}, Load balance score: {:.3}",
                 partition.quality_metrics().cut_weight,
                 partition.quality_metrics().load_balance_score);
        
        self.partition = Some(partition);
        Ok(())
    }
    
    /// Get the component partition (if partitioning was performed)
    pub fn get_partition(&self) -> Option<&ComponentPartition> {
        self.partition.as_ref()
    }
    
    /// Print partition summary (if partitioning was performed)
    pub fn print_partition_summary(&self) {
        if let Some(ref partition) = self.partition {
            partition.print_summary();
        } else {
            println!("No partitioning has been performed yet. Call partition_components() first.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profiled_simulation_run_creation() {
        let run = ProfiledSimulationRun::new();
        assert_eq!(run.profiling_cycles, Some(1000));
        
        let run_unlimited = ProfiledSimulationRun::new().with_profiling_cycles(None);
        assert_eq!(run_unlimited.profiling_cycles, None);
    }
}