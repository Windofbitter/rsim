use super::types::ComponentId;
use std::any::Any;
use std::collections::HashMap;

// The universal message type passed between components.
pub type Signal = Box<dyn Any + Send>;

// The foundational trait for all components.
pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
}

// Trait for components that are part of the primary, active data-flow graph.
pub trait ActiveComponent: BaseComponent {
    fn input_ports(&self) -> Vec<&'static str>;
    fn output_port(&self) -> &'static str;
}

// Trait for stateless components that produce an output based on their current inputs.
pub trait CombinationalComponent: ActiveComponent {
    fn evaluate(&self, port_signals: &HashMap<String, Signal>) -> Option<Signal>;
}

// Trait for stateful components that have a clocked behavior.
pub trait SequentialComponent: ActiveComponent {
    fn current_output(&self) -> Option<Signal>;
    fn prepare_next_state(&mut self, port_signals: &HashMap<String, Signal>);
    fn commit_state_change(&mut self);
}

// Trait for passive monitoring components (e.g., metrics collectors, loggers).
pub trait ProbeComponent: BaseComponent {
    fn probe(&mut self, signal: &Signal);
}

// An enum to hold any type of component in the ConnectionManager.
pub enum Component {
    Combinational(Box<dyn CombinationalComponent>),
    Sequential(Box<dyn SequentialComponent>),
    Probe(Box<dyn ProbeComponent>),
}

// Add helper methods for easy access.
impl Component {
    pub fn as_base(&self) -> &dyn BaseComponent {
        match self {
            Component::Combinational(c) => c.as_ref(),
            Component::Sequential(c) => c.as_ref(),
            Component::Probe(c) => c.as_ref(),
        }
    }
}