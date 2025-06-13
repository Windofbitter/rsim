use super::component::BaseComponent;
use super::event::Event;
use super::event_manager::EventManager;
use super::event_scheduler::EventScheduler;
use log::debug;
use std::collections::HashMap;

/// Observer trait for simulation events
pub trait SimulationObserver {
    /// Called when the simulation cycle advances
    fn on_cycle_advance(&mut self, old_cycle: u64, new_cycle: u64);
    
    /// Called when a simulation step completes
    fn on_step_complete(&mut self, cycle: u64, events_processed: usize);
}
pub struct SimulationEngine {
    event_manager: EventManager,
    scheduler: EventScheduler,
    current_cycle: u64,
    max_cycles: Option<u64>,
    observers: Vec<Box<dyn SimulationObserver>>,
}

impl SimulationEngine {
    /// Create a new SimulationEngine with optional cycle limit
    pub fn new(max_cycles: Option<u64>) -> Self {
        Self {
            event_manager: EventManager::new(),
            scheduler: EventScheduler::new(),
            current_cycle: 0,
            max_cycles,
            observers: Vec::new(),
        }
    }

    /// Register a component with the EventManager
    pub fn register_component(&mut self, component: Box<dyn BaseComponent>) -> Result<(), String> {
        self.event_manager.register_component(component)
    }

    /// Add an observer to the simulation
    pub fn add_observer(&mut self, observer: Box<dyn SimulationObserver>) {
        self.observers.push(observer);
    }

    /// Notify all observers of a cycle advance
    fn notify_cycle_advance(&mut self, old_cycle: u64, new_cycle: u64) {
        for observer in &mut self.observers {
            observer.on_cycle_advance(old_cycle, new_cycle);
        }
    }

    /// Notify all observers of step completion
    fn notify_step_complete(&mut self, cycle: u64, events_processed: usize) {
        for observer in &mut self.observers {
            observer.on_step_complete(cycle, events_processed);
        }
    }

    /// Schedule an initial event to start the simulation
    pub fn schedule_initial_event(&mut self, event: Box<dyn Event>, delay_cycles: u64) {
        self.scheduler.schedule_event(event, delay_cycles);
    }

    /// Run the complete simulation, returns final cycle count
    pub fn run(&mut self) -> Result<u64, String> {
        while self.has_pending_events() {
            if let Some(max) = self.max_cycles {
                if self.current_cycle >= max {
                    return Ok(self.current_cycle);
                }
            }

            if !self.step()? {
                break;
            }
        }
        Ok(self.current_cycle)
    }

    /// Process one time step, returns true if events remain
    pub fn step(&mut self) -> Result<bool, String> {
        if !self.has_pending_events() {
            return Ok(false);
        }

        let next_delay = self.scheduler.peek_next_delay().unwrap_or(0);
        self.scheduler.advance_time(next_delay);
        let old_cycle = self.current_cycle;
        self.current_cycle += next_delay;

        // Notify observers of cycle advance
        if old_cycle != self.current_cycle {
            self.notify_cycle_advance(old_cycle, self.current_cycle);
        }

        debug!("=== Simulation Cycle {} ===", self.current_cycle);

        let events = self.scheduler.get_next_time_events();
        let events_count = events.len();

        let mut events_by_component = HashMap::new();

        for event in events {
            let target_ids = self.event_manager.route_event(event.as_ref());

            for target_id in target_ids {
                events_by_component
                    .entry(target_id)
                    .or_insert_with(Vec::new)
                    .push(event.clone_event());
            }
        }

        for (component_id, component_events) in events_by_component {
            if let Some(component) = self.event_manager.get_component_mut(&component_id) {
                let new_events = component.react_atomic(component_events);

                for (new_event, delay) in new_events {
                    self.scheduler.schedule_event(new_event, delay);
                }
            }
        }

        // Notify observers of step completion
        self.notify_step_complete(self.current_cycle, events_count);

        Ok(self.has_pending_events())
    }

    /// Get current simulation time
    pub fn current_cycle(&self) -> u64 {
        self.current_cycle
    }

    /// Check if there are pending events in the scheduler
    pub fn has_pending_events(&self) -> bool {
        self.scheduler.has_events()
    }
}
