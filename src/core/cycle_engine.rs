use super::component::Component;
use super::connection_manager::ConnectionManager;

pub struct CycleEngine {
    pub connection_manager: ConnectionManager,
    pub current_cycle: u64,
}

impl CycleEngine {
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
            current_cycle: 0,
        }
    }

    pub fn run_cycle(&mut self) {
        // Clear previous cycle's signals
        self.connection_manager.current_signals.clear();
        
        // Phase 1: Combinational Propagation
        // Evaluate all combinational components in the pre-calculated topological order.
        for comp_id in &self.connection_manager.combinational_order.clone() {
            if let Some(Component::Combinational(comp)) = 
                self.connection_manager.components.get(comp_id) {
                
                // Gather inputs using the reverse mapping
                let inputs = self.connection_manager.gather_inputs(
                    comp_id, 
                    &comp.input_ports()
                );
                
                // Evaluate component
                if let Some(output_signal) = comp.evaluate(&inputs) {
                    // Publish signal (stores + triggers probes)
                    let output_port = (comp_id.clone(), comp.output_port().to_string());
                    self.connection_manager.publish_signal(output_port, output_signal);
                }
            }
        }

        // Phase 2: Sequential State Preparation
        // All sequential components read their inputs and prepare their next state.
        let sequential_ids = self.connection_manager.sequential_ids.clone();
        for comp_id in &sequential_ids {
            // First gather inputs
            let inputs = if let Some(Component::Sequential(comp)) = 
                self.connection_manager.components.get(comp_id) {
                self.connection_manager.gather_inputs(
                    comp_id, 
                    &comp.input_ports()
                )
            } else {
                continue;
            };
            
            // Then prepare next state
            if let Some(Component::Sequential(comp)) = 
                self.connection_manager.components.get_mut(comp_id) {
                comp.prepare_next_state(&inputs);
            }
        }

        // Phase 3: Sequential State Commit + Output
        // All sequential components atomically update their state for the next cycle.
        for comp_id in &self.connection_manager.sequential_ids.clone() {
            if let Some(Component::Sequential(comp)) = 
                self.connection_manager.components.get_mut(comp_id) {
                
                comp.commit_state_change();
                
                // Publish sequential component's output
                if let Some(output_signal) = comp.current_output() {
                    let output_port = (comp_id.clone(), comp.output_port().to_string());
                    self.connection_manager.publish_signal(output_port, output_signal);
                }
            }
        }

        self.current_cycle += 1;
    }
}