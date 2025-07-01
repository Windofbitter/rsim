use super::component::Component;
use super::connection_manager::ConnectionManager;
use super::types::ComponentId;

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
        // Note: No event clearing needed - double buffering handles this
        
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
                if let Some(output_event) = comp.evaluate(&inputs) {
                    // Publish event (stores + triggers probes)
                    let output_port = (comp_id.clone(), comp.output_port().to_string());
                    self.connection_manager.publish_event(output_port, output_event);
                }
            }
        }

        // Phase 2: Sequential State Preparation
        // Process each sequential component individually to avoid borrow conflicts
        let sequential_ids = self.connection_manager.sequential_ids.clone();
        for comp_id in &sequential_ids {
            // For each component, gather inputs and immediately call prepare_next_state
            // This pattern avoids holding references across mutable borrows
            self.prepare_sequential_component(comp_id);
        }

        // Phase 3: Sequential State Commit + Output
        // All sequential components atomically update their state for the next cycle.
        for comp_id in &self.connection_manager.sequential_ids.clone() {
            if let Some(Component::Sequential(comp)) = 
                self.connection_manager.components.get_mut(comp_id) {
                
                comp.commit_state_change();
                
                // Publish sequential component's output
                if let Some(output_event) = comp.current_output() {
                    let output_port = (comp_id.clone(), comp.output_port().to_string());
                    self.connection_manager.publish_event(output_port, output_event);
                }
            }
        }

        // End of cycle: Atomic buffer swap makes new events available for next cycle
        self.connection_manager.swap_event_buffers();
        
        self.current_cycle += 1;
    }
    
    /// Prepares a sequential component by gathering its inputs and calling prepare_next_state
    /// This method handles the borrow checker constraints by doing the work in sequence
    fn prepare_sequential_component(&mut self, comp_id: &ComponentId) {
        // Step 1: Get the input ports for this component
        let input_ports = if let Some(Component::Sequential(comp)) = 
            self.connection_manager.components.get(comp_id) {
            comp.input_ports()
        } else {
            return; // Component not found or not sequential
        };
        
        // Step 2: Gather inputs using the input ports
        let inputs = self.connection_manager.gather_inputs(comp_id, &input_ports);
        
        // Step 3: Call prepare_next_state with the gathered inputs
        if let Some(Component::Sequential(comp)) = 
            self.connection_manager.components.get_mut(comp_id) {
            comp.prepare_next_state(&inputs);
        }
    }
}