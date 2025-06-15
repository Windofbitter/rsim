use crate::analysis::ComponentPartition;
use crate::core::component::BaseComponent;
use crate::core::event::Event;
use crate::core::types::ComponentId;
use crate::parallel::cross_thread_router::CrossThreadEventRouter;
use crate::parallel::thread_local_engine::{ThreadLocalEngine, ThreadStatistics};
use std::collections::HashMap;
use std::sync::{Arc, Barrier};
use std::thread;

/// Main parallel simulation engine that coordinates multiple thread-local engines
pub struct ParallelSimulationEngine {
    thread_engines: Vec<ThreadLocalEngine>,
    cross_thread_router: CrossThreadEventRouter,
    partition: ComponentPartition,
    synchronization_barrier: Arc<Barrier>,
    num_threads: usize,
    current_cycle: u64,
    max_cycles: Option<u64>,
}

impl ParallelSimulationEngine {
    /// Create a new parallel simulation engine with the given partition
    pub fn new(partition: ComponentPartition, max_cycles: Option<u64>) -> Self {
        let num_threads = partition.num_threads();
        let synchronization_barrier = Arc::new(Barrier::new(num_threads));
        
        // Create thread-local engines
        let mut thread_engines = Vec::with_capacity(num_threads);
        for thread_id in 0..num_threads {
            thread_engines.push(ThreadLocalEngine::new(thread_id));
        }
        
        // Create cross-thread router
        let (cross_thread_router, _receivers) = CrossThreadEventRouter::new(partition.clone(), num_threads);
        
        Self {
            thread_engines,
            cross_thread_router,
            partition,
            synchronization_barrier,
            num_threads,
            current_cycle: 0,
            max_cycles,
        }
    }
    
    /// Register a component with the appropriate thread-local engine
    pub fn register_component(&mut self, component: Box<dyn BaseComponent>) -> Result<(), String> {
        let component_id = component.component_id();
        
        // Find which thread this component belongs to
        let thread_id = self.partition.get_thread_assignment(component_id)
            .ok_or_else(|| format!("Component {} not found in partition", component_id))?;
        
        // Register with the appropriate thread-local engine
        if thread_id >= self.num_threads {
            return Err(format!("Thread ID {} exceeds number of threads {}", thread_id, self.num_threads));
        }
        
        self.thread_engines[thread_id].register_component(component)?;
        Ok(())
    }
    
    /// Schedule an initial event
    pub fn schedule_initial_event(&mut self, event: Box<dyn Event + Send + Sync>, delay_cycles: u64) {
        // Route initial events through the cross-thread router
        let target_threads = self.cross_thread_router.route_event(event.as_ref());
        
        for thread_id in target_threads {
            if thread_id < self.num_threads {
                self.thread_engines[thread_id].schedule_event(event.clone_event(), delay_cycles);
            }
        }
    }
    
    /// Run the parallel simulation
    pub fn run(self) -> Result<u64, String> {
        // Create thread handles
        let mut handles = vec![];
        
        // Split components for parallel execution
        let thread_engines = self.thread_engines;
        let barrier = self.synchronization_barrier.clone();
        let max_cycles = self.max_cycles;
        
        // Create router and receivers
        let (router, receivers) = CrossThreadEventRouter::new(self.partition.clone(), self.num_threads);
        let router = Arc::new(router);
        
        // Spawn threads
        for (mut engine, receiver) in thread_engines.into_iter().zip(receivers.into_iter()) {
            let barrier_clone = barrier.clone();
            let router_clone = router.clone();
            
            let handle = thread::spawn(move || {
                engine.run_with_synchronization(barrier_clone, router_clone, receiver, max_cycles)
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        let mut final_cycles = vec![];
        for handle in handles {
            match handle.join() {
                Ok(result) => match result {
                    Ok(cycle) => final_cycles.push(cycle),
                    Err(e) => return Err(format!("Thread execution error: {}", e)),
                },
                Err(_) => return Err("Thread panic occurred".to_string()),
            }
        }
        
        // Return the maximum cycle reached by any thread
        let final_cycle = *final_cycles.iter().max().unwrap_or(&0);
        Ok(final_cycle)
    }
    
    /// Get current simulation cycle
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }
    
    /// Check if simulation has pending events (across all threads)
    /// Note: This may not be accurate after run() is called since engines are moved
    pub fn has_pending_events(&self) -> bool {
        self.thread_engines.iter().any(|engine| engine.has_pending_events())
    }
}

/// Result of a parallel simulation run
pub struct ParallelSimulationResult {
    pub final_cycle: u64,
    pub partition: ComponentPartition,
    pub thread_statistics: Vec<ThreadStatistics>,
}


impl ParallelSimulationResult {
    /// Calculate speedup compared to single-threaded execution
    pub fn speedup(&self, single_thread_time: f64, parallel_time: f64) -> f64 {
        single_thread_time / parallel_time
    }
    
    /// Calculate efficiency (speedup / num_threads)
    pub fn efficiency(&self, speedup: f64) -> f64 {
        speedup / self.partition.num_threads() as f64
    }
    
    /// Print summary of results
    pub fn print_summary(&self) {
        println!("=== Parallel Simulation Summary ===");
        println!("Final cycle: {}", self.final_cycle);
        println!("Number of threads: {}", self.partition.num_threads());
        
        for stats in &self.thread_statistics {
            println!("\nThread {}: ", stats.thread_id);
            println!("  Components: {}", stats.components_count);
            println!("  Events processed: {}", stats.events_processed);
            println!("  Cross-thread events sent: {}", stats.cross_thread_events_sent);
            println!("  Cross-thread events received: {}", stats.cross_thread_events_received);
        }
    }
}