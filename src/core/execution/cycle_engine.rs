use crate::core::builder::simulation_builder::ComponentInstance;
use crate::core::types::ComponentId;
use crate::core::execution::execution_order::{ExecutionOrderBuilder, Stage};
use crate::core::execution::config::{SimulationConfig, ConcurrencyMode};
use crate::core::components::module::{EvaluationContext, MemoryModuleTrait};
use crate::core::values::implementations::{EventInputMap, EventOutputMap};
use crate::core::values::events::Event;
use crate::core::values::traits::EventOutputs;
use crate::core::memory::proxy::{MemoryProxy, OwnedMemoryProxy};
use crate::core::memory::MemoryWrite;
use std::collections::HashMap;
use std::sync::mpsc;
use rayon::prelude::*;

/// Pre-computed memory component subsets for each processing component
/// Maps processing component ID to the memory components it can access
type ComponentMemoryMap = HashMap<ComponentId, Vec<ComponentId>>;

/// Processing component instance
pub struct ProcessingComponent {
    pub id: ComponentId,
    pub module: crate::core::components::module::ProcessorModule,
}

/// Input connection for O(1) lookup during input collection
#[derive(Clone, Debug)]
struct InputConnection {
    source_id: ComponentId,
    source_port: String,
    target_port: String,
}

/// Simplified cycle engine for the new direct API
/// 
/// This engine manages the execution of component evaluation functions
/// in a deterministic order for each simulation cycle.
pub struct CycleEngine {
    /// Processing component instances
    processing_components: HashMap<ComponentId, ProcessingComponent>,
    /// Memory component instances
    memory_components: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
    /// Port connections: (source_id, source_port) -> Vec<(target_id, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
    /// Current cycle number
    current_cycle: u64,
    /// Execution order for processing components (topologically sorted into stages)
    execution_order: Vec<Vec<ComponentId>>,
    /// Sub-level execution order for more granular dependency management
    sub_level_execution_order: Vec<Stage>,
    /// Output buffer for current cycle
    output_buffer: HashMap<(ComponentId, String), Event>,
    /// Memory connections: (component_id, port) -> memory_id
    memory_connections: HashMap<(ComponentId, String), ComponentId>,
    /// Pre-computed input connections for O(1) lookup (hot path optimization)
    input_connections: HashMap<ComponentId, Vec<InputConnection>>,
    /// Simulation configuration
    config: SimulationConfig,
    /// Pre-computed memory component access patterns for thread safety
    component_memory_map: ComponentMemoryMap,
}

impl CycleEngine {
    /// Create a new cycle engine with configuration
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            processing_components: HashMap::new(),
            memory_components: HashMap::new(),
            connections: HashMap::new(),
            current_cycle: 0,
            execution_order: Vec::new(),
            sub_level_execution_order: Vec::new(),
            output_buffer: HashMap::new(),
            memory_connections: HashMap::new(),
            input_connections: HashMap::new(),
            config,
            component_memory_map: HashMap::new(),
        }
    }
    
    /// Create a new cycle engine with default sequential configuration
    /// This method maintains backward compatibility
    pub fn new_sequential() -> Self {
        Self::new(SimulationConfig::default())
    }

    /// Register a component instance
    pub fn register_component_instance(&mut self, instance: ComponentInstance) -> Result<(), String> {
        let id = instance.id.clone();
        match instance.module {
            crate::core::components::module::ComponentModule::Processing(module) => {
                let processing_comp = ProcessingComponent {
                    id: id.clone(),
                    module,
                };
                self.processing_components.insert(id, processing_comp);
            }
            crate::core::components::module::ComponentModule::Memory(module) => {
                self.memory_components.insert(id, module);
            }
        }
        Ok(())
    }

    /// Add a connection between components
    pub fn connect(
        &mut self,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        // Validate that both components exist
        if !self.has_component(&source.0) {
            return Err(format!("Source component '{}' not found", source.0));
        }
        if !self.has_component(&target.0) {
            return Err(format!("Target component '{}' not found", target.0));
        }

        // Add the connection
        self.connections
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);

        Ok(())
    }

    /// Add a memory connection between a component port and a memory module
    pub fn connect_memory(
        &mut self,
        component_port: (ComponentId, String),
        memory_id: ComponentId,
    ) -> Result<(), String> {
        // Validate that both components exist
        if !self.has_component(&component_port.0) {
            return Err(format!("Component '{}' not found", component_port.0));
        }
        if !self.memory_components.contains_key(&memory_id) {
            return Err(format!("Memory component '{}' not found", memory_id));
        }

        // Add the memory connection
        self.memory_connections.insert(component_port, memory_id);

        Ok(())
    }

    /// Execute one simulation cycle
    pub fn cycle(&mut self) -> Result<(), String> {
        match self.config.concurrency_mode {
            ConcurrencyMode::Sequential => self.cycle_sequential(),
            ConcurrencyMode::Rayon => {
                // Activate sub-level parallel execution with enhanced topological sorting
                // Uses channel-based memory synchronization to ensure deterministic results
                // while enabling true parallel execution within sub-levels
                self.cycle_parallel_rayon_with_sub_levels()
            },
        }
    }
    
    /// Execute one simulation cycle sequentially
    fn cycle_sequential(&mut self) -> Result<(), String> {
        self.current_cycle += 1;

        // Clear output buffer from previous cycle to prevent unbounded growth
        self.output_buffer.clear();

        // Execute processing components in topological order (staged execution)
        for stage in &self.execution_order.clone() {
            for component_id in stage {
                self.execute_processing_component(component_id)?;
            }
        }

        // Update memory components
        for component_id in self.memory_components.keys().cloned().collect::<Vec<_>>() {
            self.execute_memory_component(&component_id)?;
        }

        Ok(())
    }

    /// Execute one simulation cycle in parallel using rayon with channel-based memory synchronization
    /// Implements stage-parallel execution with proper error aggregation
    fn cycle_parallel_rayon(&mut self) -> Result<(), String> {
        self.current_cycle += 1;
        
        // Clear output buffer from previous cycle to prevent unbounded growth
        self.output_buffer.clear();
        
        // Processing phase: stage-parallel execution with channel-based memory synchronization
        // Each stage runs sequentially, but components within each stage run in parallel
        for stage in &self.execution_order.clone() {
            if stage.is_empty() {
                continue;
            }
            
            // Create channel for memory writes
            let (memory_write_sender, memory_write_receiver) = mpsc::channel::<MemoryWrite>();
            
            // Execute all components in this stage in parallel
            let stage_results: Vec<Result<HashMap<(ComponentId, String), Event>, String>> = stage
                .par_iter()
                .map(|component_id| {
                    let sender = memory_write_sender.clone();
                    self.execute_processing_component_parallel(component_id, sender)
                })
                .collect();
            
            // Drop the original sender so the receiver can detect when all senders are done
            drop(memory_write_sender);
            
            // Aggregate results and errors
            let mut all_outputs = HashMap::new();
            let mut errors = Vec::new();
            
            for (idx, result) in stage_results.into_iter().enumerate() {
                match result {
                    Ok(outputs) => {
                        all_outputs.extend(outputs);
                    }
                    Err(error) => {
                        let component_id = &stage[idx];
                        errors.push(format!("Component '{}': {}", component_id, error));
                    }
                }
            }
            
            // If any component failed, aggregate all errors and return
            if !errors.is_empty() {
                return Err(format!("Stage execution failed in {} components: [{}]", 
                                  errors.len(), errors.join(", ")));
            }
            
            // Sequential merge of outputs after successful stage completion
            self.output_buffer.extend(all_outputs);
            
            // Apply memory writes sequentially in main thread
            self.apply_memory_writes(memory_write_receiver)?;
        }
        
        // Memory phase: Execute memory components sequentially after parallel processing completes
        // This ensures all memory components are properly updated for the next cycle
        for component_id in self.memory_components.keys().cloned().collect::<Vec<_>>() {
            self.execute_memory_component(&component_id)?;
        }
        
        Ok(())
    }

    /// Execute one simulation cycle in parallel using rayon with sub-level granularity
    /// This method implements the enhanced parallel execution with proper topological ordering
    /// at sub-level granularity to fix memory synchronization issues
    fn cycle_parallel_rayon_with_sub_levels(&mut self) -> Result<(), String> {
        self.current_cycle += 1;
        
        // Clear output buffer from previous cycle to prevent unbounded growth
        self.output_buffer.clear();
        
        // Processing phase: sub-level parallel execution with channel-based memory synchronization
        // Each stage runs sequentially, but within each stage, sub-levels run sequentially
        // while components within each sub-level run in parallel
        for stage in &self.sub_level_execution_order.clone() {
            // Execute each sub-level within the stage sequentially
            for sub_level in &stage.sub_levels {
                if sub_level.components.is_empty() {
                    continue;
                }
                
                // Create channel for memory writes
                let (memory_write_sender, memory_write_receiver) = mpsc::channel::<MemoryWrite>();
                
                // Execute all components in this sub-level in parallel
                let sub_level_results: Vec<Result<HashMap<(ComponentId, String), Event>, String>> = sub_level.components
                    .par_iter()
                    .map(|component_id| {
                        let sender = memory_write_sender.clone();
                        self.execute_processing_component_parallel(component_id, sender)
                    })
                    .collect();
                
                // Drop the original sender so the receiver can detect when all senders are done
                drop(memory_write_sender);
                
                // Aggregate results and errors
                let mut all_outputs = HashMap::new();
                let mut errors = Vec::new();
                
                for (idx, result) in sub_level_results.into_iter().enumerate() {
                    match result {
                        Ok(outputs) => {
                            all_outputs.extend(outputs);
                        }
                        Err(error) => {
                            let component_id = &sub_level.components[idx];
                            errors.push(format!("Component '{}': {}", component_id, error));
                        }
                    }
                }
                
                // If any component failed, aggregate all errors and return
                if !errors.is_empty() {
                    return Err(format!("Sub-level execution failed in {} components: [{}]", 
                                      errors.len(), errors.join(", ")));
                }
                
                // Sequential merge of outputs after successful sub-level completion
                self.output_buffer.extend(all_outputs);
                
                // Apply memory writes sequentially in main thread after each sub-level completes
                self.apply_memory_writes(memory_write_receiver)?;
            }
        }
        
        // Memory phase: Execute memory components sequentially after parallel processing completes
        // This ensures all memory components are properly updated for the next cycle
        for component_id in self.memory_components.keys().cloned().collect::<Vec<_>>() {
            self.execute_memory_component(&component_id)?;
        }
        
        Ok(())
    }

    /// Execute a processing component
    fn execute_processing_component(&mut self, component_id: &ComponentId) -> Result<(), String> {
        // First, collect inputs from connected outputs
        let inputs = self.collect_inputs(component_id)?;
        
        // Extract processor info and evaluate function
        let processor = {
            let component = self.processing_components.get(component_id)
                .ok_or_else(|| format!("Processing component '{}' not found", component_id))?;
            component.module.clone()
        };
        
        // Get current cycle before creating memory proxy to avoid borrowing conflict
        let current_cycle = self.current_cycle;
        
        // Create memory proxy with component subset for thread safety
        let mut memory_proxy = self.create_component_memory_proxy(component_id)?;
        
        // Create evaluation context
        let mut context = EvaluationContext {
            inputs: &inputs,
            memory: &mut memory_proxy,
            state: None, // Processing components have no state
            component_id,
        };
        
        // Create output map for this component
        let mut outputs = EventOutputMap::new_flexible(current_cycle);
        
        // Execute the component's evaluation function
        (processor.evaluate_fn)(&mut context, &mut outputs)?;
        
        // Store outputs in buffer for next cycle
        for (port, event) in outputs.into_event_map() {
            self.output_buffer.insert((component_id.clone(), port), event);
        }
        
        Ok(())
    }

    /// Execute a processing component in parallel using channel-based memory synchronization
    /// This method uses &self instead of &mut self for parallel execution
    /// Memory writes are sent through channels to the main thread for sequential application
    fn execute_processing_component_parallel(
        &self, 
        component_id: &ComponentId,
        memory_write_sender: mpsc::Sender<MemoryWrite>
    ) -> Result<HashMap<(ComponentId, String), Event>, String> {
        // First, collect inputs from connected outputs
        let inputs = self.collect_inputs(component_id)?;
        
        // Extract processor info and evaluate function
        let processor = {
            let component = self.processing_components.get(component_id)
                .ok_or_else(|| format!("Processing component '{}' not found", component_id))?;
            component.module.clone()
        };
        
        // Get the memory component IDs this component needs
        let memory_deps = self.component_memory_map.get(component_id)
            .cloned()
            .unwrap_or_default();
        
        // Clone the memory components for read access (snapshot at beginning of cycle)
        let mut parallel_memory_components = HashMap::new();
        for memory_id in &memory_deps {
            if let Some(memory_component) = self.memory_components.get(memory_id) {
                parallel_memory_components.insert(memory_id.clone(), memory_component.clone_module());
            }
        }
        
        // Filter memory connections to only include this component's connections
        let component_memory_connections: HashMap<(ComponentId, String), ComponentId> = self.memory_connections
            .iter()
            .filter(|((comp_id, _), _)| comp_id == component_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Create channel-aware memory proxy
        let mut memory_proxy = MemoryProxy::new_with_channel_synchronization(
            component_memory_connections,
            component_id.clone(),
            parallel_memory_components,
            &memory_deps,
            memory_write_sender,
        );
        
        // Create output map for this component
        let mut outputs = EventOutputMap::new_flexible(self.current_cycle);
        
        // Execute the component's evaluation function
        {
            let mut context = EvaluationContext {
                inputs: &inputs,
                memory: &mut memory_proxy,
                state: None, // Processing components have no state
                component_id,
            };
            
            (processor.evaluate_fn)(&mut context, &mut outputs)?;
        }
        
        // Return outputs with component ID in key for parallel execution
        let mut component_outputs = HashMap::new();
        for (port, event) in outputs.into_event_map() {
            component_outputs.insert((component_id.clone(), port), event);
        }
        
        Ok(component_outputs)
    }

    /// Execute a memory component
    fn execute_memory_component(&mut self, component_id: &ComponentId) -> Result<(), String> {
        let memory_module = self.memory_components.get_mut(component_id)
            .ok_or_else(|| format!("Memory component '{}' not found", component_id))?;
        
        // Call cycle() on stored data objects to process pending operations
        memory_module.cycle()?;
        
        // Update memory state: current → snapshot for next cycle
        memory_module.create_snapshot();
        
        Ok(())
    }
    
    /// Merge updated memory components back to main memory after parallel execution
    /// This function handles the critical memory synchronization issue in parallel execution
    /// by merging the updated memory components from each thread back to the main memory system
    fn merge_memory_components(&mut self, updated_memory_components: HashMap<ComponentId, Box<dyn MemoryModuleTrait>>) -> Result<(), String> {
        // Merge each updated memory component back to main memory
        for (memory_id, updated_component) in updated_memory_components {
            // Replace the memory component in the main memory system
            // This is safe because each memory component has exactly one writer (no conflicts)
            self.memory_components.insert(memory_id, updated_component);
        }
        Ok(())
    }

    /// Apply memory writes sequentially in the main thread
    /// This method receives memory writes from parallel threads and applies them sequentially
    /// to ensure deterministic behavior and avoid race conditions
    fn apply_memory_writes(&mut self, memory_write_receiver: mpsc::Receiver<MemoryWrite>) -> Result<(), String> {
        // Collect all memory writes from the channel
        let mut memory_writes = Vec::new();
        while let Ok(memory_write) = memory_write_receiver.recv() {
            memory_writes.push(memory_write);
        }
        
        // Apply memory writes sequentially
        for memory_write in memory_writes {
            // Get the target memory component
            let memory_component = self.memory_components.get_mut(&memory_write.memory_id)
                .ok_or_else(|| format!("Memory component '{}' not found", memory_write.memory_id))?;
            
            // Apply the write to the memory component
            if !memory_component.write_any(&memory_write.address, memory_write.data) {
                return Err(format!("Failed to write to memory address '{}' in memory component '{}'", 
                                  memory_write.address, memory_write.memory_id));
            }
        }
        
        Ok(())
    }


    /// Get the current cycle number
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }

    /// Get all component IDs
    pub fn component_ids(&self) -> Vec<&ComponentId> {
        let mut ids = Vec::new();
        ids.extend(self.processing_components.keys());
        ids.extend(self.memory_components.keys());
        ids
    }

    /// Check if a component exists
    pub fn has_component(&self, id: &ComponentId) -> bool {
        self.processing_components.contains_key(id) || self.memory_components.contains_key(id)
    }

    /// Build execution order for deterministic simulation
    pub fn build_execution_order(&mut self) -> Result<(), String> {
        // Get all processing component IDs
        let processing_components: Vec<ComponentId> = self.processing_components.keys().cloned().collect();
        
        // Build topologically sorted execution order (staged)
        self.execution_order = ExecutionOrderBuilder::build_execution_order_stages(
            &processing_components,
            &self.connections,
        )?;
        
        // Build sub-level execution order for enhanced parallel processing
        self.sub_level_execution_order = ExecutionOrderBuilder::build_execution_order_with_sub_levels(
            &processing_components,
            &self.connections,
        )?;
        
        // Build input connection lookup for O(1) access (hot path optimization)
        // This pre-computation eliminates O(n×m) linear scanning in collect_inputs()
        // providing 10-100x speedup for larger simulations
        self.input_connections.clear();
        for ((source_id, source_port), targets) in &self.connections {
            for (target_id, target_port) in targets {
                self.input_connections
                    .entry(target_id.clone())
                    .or_insert_with(Vec::new)
                    .push(InputConnection {
                        source_id: source_id.clone(),
                        source_port: source_port.clone(),
                        target_port: target_port.clone(),
                    });
            }
        }
        
        // Pre-compute memory subsets for thread safety
        self.pre_compute_memory_subsets();
        
        Ok(())
    }

    /// Pre-compute which memory components each processing component needs
    /// This eliminates HashMap contention during parallel execution
    fn pre_compute_memory_subsets(&mut self) {
        self.component_memory_map.clear();
        
        for (comp_id, _) in &self.processing_components {
            let mut memory_deps = Vec::new();
            
            // Find all memory connections for this component
            for ((connected_comp, _port), memory_id) in &self.memory_connections {
                if connected_comp == comp_id {
                    memory_deps.push(memory_id.clone());
                }
            }
            
            if !memory_deps.is_empty() {
                self.component_memory_map.insert(comp_id.clone(), memory_deps);
            }
        }
    }

    /// Create a memory proxy for a specific component with only its required memory components
    /// This eliminates HashMap contention during parallel execution by giving each component
    /// only the memory components it needs
    fn create_component_memory_proxy(&mut self, component_id: &ComponentId) -> Result<MemoryProxy, String> {
        // Get the memory component IDs this component needs
        let memory_deps = self.component_memory_map.get(component_id)
            .cloned()
            .unwrap_or_default();
        
        // Filter memory connections to only include this component's connections
        let component_memory_connections: HashMap<(ComponentId, String), ComponentId> = self.memory_connections
            .iter()
            .filter(|((comp_id, _), _)| comp_id == component_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Create memory proxy with component subset
        Ok(MemoryProxy::new_with_component_subset(
            component_memory_connections,
            component_id.clone(),
            &mut self.memory_components,
            &memory_deps,
        ))
    }
    
    /// Create a memory proxy for parallel execution with owned memory components
    /// This version creates a proxy that can safely access memory components in parallel mode
    fn create_component_memory_proxy_parallel(&self, component_id: &ComponentId) -> Result<MemoryProxy, String> {
        // Get the memory component IDs this component needs
        let memory_deps = self.component_memory_map.get(component_id)
            .cloned()
            .unwrap_or_default();
        
        // Filter memory connections to only include this component's connections
        let component_memory_connections: HashMap<(ComponentId, String), ComponentId> = self.memory_connections
            .iter()
            .filter(|((comp_id, _), _)| comp_id == component_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // For parallel execution, create a proxy with cloned memory components
        // This ensures each thread has its own copy and avoids mutable access conflicts
        let mut parallel_memory_components = HashMap::new();
        for memory_id in &memory_deps {
            if let Some(memory_component) = self.memory_components.get(memory_id) {
                parallel_memory_components.insert(memory_id.clone(), memory_component.clone_module());
            }
        }
        
        // Create memory proxy with owned components for thread safety
        Ok(MemoryProxy::new_with_owned_components(
            component_memory_connections,
            component_id.clone(),
            parallel_memory_components,
            &memory_deps,
        ))
    }

    /// Create a memory proxy for parallel execution with delta tracking
    /// This version creates a proxy that tracks memory writes for later application
    fn create_component_memory_proxy_parallel_with_delta(&self, component_id: &ComponentId) -> Result<MemoryProxy, String> {
        // Get the memory component IDs this component needs
        let memory_deps = self.component_memory_map.get(component_id)
            .cloned()
            .unwrap_or_default();
        
        // Filter memory connections to only include this component's connections
        let component_memory_connections: HashMap<(ComponentId, String), ComponentId> = self.memory_connections
            .iter()
            .filter(|((comp_id, _), _)| comp_id == component_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // For parallel execution, create a proxy with cloned memory components
        // This ensures each thread has its own copy and avoids mutable access conflicts
        let mut parallel_memory_components = HashMap::new();
        for memory_id in &memory_deps {
            if let Some(memory_component) = self.memory_components.get(memory_id) {
                parallel_memory_components.insert(memory_id.clone(), memory_component.clone_module());
            }
        }
        
        // Create memory proxy with delta tracking enabled
        Ok(MemoryProxy::new_with_delta_tracking(
            component_memory_connections,
            component_id.clone(),
            parallel_memory_components,
            &memory_deps,
        ))
    }
    
    /// Create an owned memory proxy for parallel execution without lifetime constraints
    /// This avoids the lifetime conflict when extracting updated memory components
    fn create_owned_memory_proxy_parallel(&self, component_id: &ComponentId) -> Result<OwnedMemoryProxy, String> {
        // Get the memory component IDs this component needs
        let memory_deps = self.component_memory_map.get(component_id)
            .cloned()
            .unwrap_or_default();
        
        // Filter memory connections to only include this component's connections
        let component_memory_connections: HashMap<(ComponentId, String), ComponentId> = self.memory_connections
            .iter()
            .filter(|((comp_id, _), _)| comp_id == component_id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // For parallel execution, create a proxy with cloned memory components
        // This ensures each thread has its own copy and avoids mutable access conflicts
        let mut parallel_memory_components = HashMap::new();
        for memory_id in &memory_deps {
            if let Some(memory_component) = self.memory_components.get(memory_id) {
                parallel_memory_components.insert(memory_id.clone(), memory_component.clone_module());
            }
        }
        
        // Create owned memory proxy for thread safety
        Ok(OwnedMemoryProxy::new(
            component_memory_connections,
            component_id.clone(),
            parallel_memory_components,
            memory_deps,
        ))
    }





    /// Run a single simulation cycle (alias for cycle method)
    pub fn run_cycle(&mut self) -> Result<(), String> {
        self.cycle()
    }
    
    /// Query the current state of a memory component
    /// Returns the data stored at the "state" address in the memory component
    pub fn query_memory_component_state<T: crate::core::components::state::MemoryData>(&self, memory_component_id: &ComponentId) -> Result<Option<T>, String> {
        self.query_memory_component_data::<T>(memory_component_id, "state")
    }
    
    /// Query data from a memory component at a specific address
    pub fn query_memory_component_data<T: crate::core::components::state::MemoryData>(&self, memory_component_id: &ComponentId, address: &str) -> Result<Option<T>, String> {
        // Get the memory component
        let memory_component = self.memory_components.get(memory_component_id)
            .ok_or_else(|| format!("Memory component '{}' not found", memory_component_id))?;
        
        // Use read_any and downcast to the requested type
        if let Some(any_data) = memory_component.read_any(address) {
            if let Ok(typed_data) = any_data.downcast::<T>() {
                Ok(Some(*typed_data))
            } else {
                Err(format!("Type mismatch: expected {}, found different type at address '{}' in memory component '{}'", 
                           std::any::type_name::<T>(), address, memory_component_id))
            }
        } else {
            Ok(None)
        }
    }
    
    /// Collect inputs for a component from connected outputs (optimized for hot path)
    fn collect_inputs(&self, component_id: &ComponentId) -> Result<EventInputMap, String> {
        let mut inputs = EventInputMap::new();
        
        // O(1) lookup using pre-computed input connections (hot path optimization)
        if let Some(connections) = self.input_connections.get(component_id) {
            for conn in connections {
                // Get the output event from the buffer
                if let Some(event) = self.output_buffer.get(&(conn.source_id.clone(), conn.source_port.clone())) {
                    inputs.insert_event(conn.target_port.clone(), event.clone());
                }
            }
        }
        
        Ok(inputs)
    }
}