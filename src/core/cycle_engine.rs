use super::component::{Event, MemoryComponent, ProbeComponent, ProcessingComponent};
use super::component_registry::ComponentRegistry;
use super::connection_manager::ConnectionManager;
use super::execution_order::ExecutionOrderBuilder;
use super::memory_proxy::CentralMemoryProxy;
use super::types::ComponentId;
use std::collections::HashMap;

pub struct CycleEngine {
    registry: ComponentRegistry,
    connection_manager: ConnectionManager,

    // Store component outputs from previous cycle for current cycle inputs
    pub previous_cycle_outputs: HashMap<(ComponentId, String), Event>,

    // Topologically sorted execution order for processing components
    execution_order: Vec<ComponentId>,

    current_cycle: u64,
}


impl CycleEngine {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            connection_manager: ConnectionManager::new(),
            previous_cycle_outputs: HashMap::new(),
            execution_order: Vec::new(),
            current_cycle: 0,
        }
    }

    pub fn register_processing(&mut self, component: Box<dyn ProcessingComponent>) {
        self.registry.register_processing(component);
    }

    pub fn register_memory(&mut self, component: Box<dyn MemoryComponent>) {
        self.registry.register_memory(component);
    }

    pub fn register_probe(&mut self, component: Box<dyn ProbeComponent>) {
        self.registry.register_probe(component);
    }

    pub fn connect_memory(
        &mut self,
        proc_id: ComponentId,
        port: String,
        mem_id: ComponentId,
    ) -> Result<(), String> {
        self.connection_manager.connect_memory(&self.registry, proc_id, port, mem_id)
    }

    pub fn connect(
        &mut self,
        source: (ComponentId, String),
        target: (ComponentId, String),
    ) -> Result<(), String> {
        self.connection_manager.connect(&self.registry, source, target)
    }

    pub fn connect_probe(
        &mut self,
        source_port: (ComponentId, String),
        probe_id: ComponentId,
    ) -> Result<(), String> {
        self.connection_manager.connect_probe(&self.registry, source_port, probe_id)
    }

    /// Analyzes the graph of processing components to build a topologically sorted execution order.
    /// Uses Kahn's algorithm to detect cycles and ensure deterministic execution.
    pub fn build_execution_order(&mut self) -> Result<(), String> {
        self.execution_order = ExecutionOrderBuilder::build_execution_order(
            &self.registry,
            self.connection_manager.connections(),
        )?;
        Ok(())
    }

    pub fn run_cycle(&mut self) {
        // 1. Collect current cycle outputs
        let mut current_cycle_outputs: HashMap<(ComponentId, String), Event> = HashMap::new();

        // 2. Execute all processing components in topological order
        for comp_id in &self.execution_order {
            // Gather inputs for this component from PREVIOUS cycle outputs
            let mut inputs = HashMap::new();

            if let Some(component) = self.registry.processing_components().get(comp_id) {
                for input_port in component.input_ports() {
                    // Look for connections to this input port
                    for ((source_id, source_port), targets) in self.connection_manager.connections() {
                        for (target_id, target_port) in targets {
                            if target_id == comp_id && target_port == input_port {
                                // Use previous_cycle_outputs for current cycle inputs
                                if let Some(event) = self
                                    .previous_cycle_outputs
                                    .get(&(source_id.clone(), source_port.clone()))
                                {
                                    inputs.insert(input_port.to_string(), event.clone());
                                }
                            }
                        }
                    }
                }

                // Create proxy for this component evaluation only
                let mut proxy = CentralMemoryProxy::new(
                    self.registry.memory_components(),
                    self.connection_manager.memory_connections(),
                );

                // Evaluate component with memory proxy access
                let outputs = component.evaluate(&inputs, &mut proxy);

                // Store outputs for NEXT cycle
                for output_port in component.output_ports() {
                    if let Some(event) = outputs.get(output_port) {
                        current_cycle_outputs
                            .insert((comp_id.clone(), output_port.to_string()), event.clone());
                    }
                }
                // Proxy is dropped here, releasing the borrow
            }
        }

        // 3. Trigger probes for current cycle outputs (only connected probes)
        for ((source_id, source_port), event) in &current_cycle_outputs {
            if let Some(probe_ids) = self.connection_manager.probes().get(&(source_id.clone(), source_port.clone())) {
                for probe_id in probe_ids {
                    if let Some(probe) = self.registry.probe_components().get_mut(probe_id) {
                        probe.probe(source_id, source_port, event);
                    }
                }
            }
        }

        // 4. Update previous cycle outputs for next cycle
        self.previous_cycle_outputs = current_cycle_outputs;

        // 5. End cycle on all memory components (create next snapshot)
        for memory_ref in self.registry.memory_components().values() {
            memory_ref.borrow_mut().end_cycle();
        }

        self.current_cycle += 1;
    }

    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }

    /// Returns the current topological execution order for debugging/inspection
    pub fn execution_order(&self) -> &[ComponentId] {
        &self.execution_order
    }
}

