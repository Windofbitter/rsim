use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, SimulationTime};
use uuid::Uuid;

use crate::events::{
    StartFryingEvent, MeatReadyEvent,
    START_FRYING_EVENT, MEAT_READY_EVENT,
    BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT
};

/// Fryer component that processes raw meat into fried meat patties
#[derive(Debug)]
pub struct Fryer {
    pub component_id: ComponentId,
    pub target_buffer_id: ComponentId,
    pub frying_delay: u64,
    pub is_production_stopped: bool,
    pub items_in_process: u32,
    pub max_concurrent_items: u32,
}

impl Fryer {
    pub fn new(
        component_id: ComponentId, 
        target_buffer_id: ComponentId, 
        frying_delay: u64,
        max_concurrent_items: u32
    ) -> Self {
        Self {
            component_id,
            target_buffer_id,
            frying_delay,
            is_production_stopped: false,
            items_in_process: 0,
            max_concurrent_items,
        }
    }

    /// Creates a self-scheduled StartFryingEvent for continuous production
    fn create_start_frying_event(&self, timestamp: SimulationTime) -> Box<dyn Event> {
        Box::new(StartFryingEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            timestamp,
        })
    }

    /// Creates a MeatReadyEvent to send to the buffer
    fn create_meat_ready_event(&self, timestamp: SimulationTime, meat_id: String) -> Box<dyn Event> {
        Box::new(MeatReadyEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.target_buffer_id.clone(),
            timestamp,
            meat_id,
        })
    }

    /// Handles starting the frying process
    fn handle_start_frying(&mut self, current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Only start frying if production is not stopped and we have capacity
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            // Start frying a new meat patty
            self.items_in_process += 1;
            let meat_id = Uuid::new_v4().to_string();

            // Schedule the meat to be ready after frying delay
            let meat_ready_event = self.create_meat_ready_event(current_time, meat_id);
            output_events.push((meat_ready_event, self.frying_delay));

            // Schedule next frying cycle if not stopped and still have capacity
            if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
                let next_frying_event = self.create_start_frying_event(current_time);
                output_events.push((next_frying_event, 1)); // Start next item quickly
            }
        }

        output_events
    }

    /// Handles when the downstream buffer becomes full
    fn handle_buffer_full(&mut self, _current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        // Stop production when downstream buffer is full (backpressure)
        self.is_production_stopped = true;
        Vec::new()
    }

    /// Handles when the downstream buffer has space available
    fn handle_buffer_space_available(&mut self, current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Resume production when space becomes available
        if self.is_production_stopped {
            self.is_production_stopped = false;
            
            // Resume frying if we have capacity
            if self.items_in_process < self.max_concurrent_items {
                let start_frying_event = self.create_start_frying_event(current_time);
                output_events.push((start_frying_event, 1)); // Resume quickly
            }
        }

        output_events
    }

    /// Handles meat ready events (decrements items in process counter)
    fn handle_meat_ready(&mut self, _current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        // Decrement items in process when meat is ready and sent to buffer
        if self.items_in_process > 0 {
            self.items_in_process -= 1;
        }

        // If production is not stopped and we now have capacity, start next item
        let mut output_events = Vec::new();
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            let start_frying_event = self.create_start_frying_event(0); // Use current time
            output_events.push((start_frying_event, 1));
        }

        output_events
    }
}

impl BaseComponent for Fryer {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            START_FRYING_EVENT,
            BUFFER_FULL_EVENT,
            BUFFER_SPACE_AVAILABLE_EVENT,
            MEAT_READY_EVENT, // Subscribe to own meat ready events for flow control
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let current_time = 0; // In a real simulation, this would come from the engine

        for event in events {
            match event.event_type() {
                START_FRYING_EVENT => {
                    let mut new_events = self.handle_start_frying(current_time);
                    output_events.append(&mut new_events);
                }
                BUFFER_FULL_EVENT => {
                    // Only handle if this event is targeted at us
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_full(current_time);
                            output_events.append(&mut new_events);
                        }
                    }
                }
                BUFFER_SPACE_AVAILABLE_EVENT => {
                    // Only handle if this event is targeted at us
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_space_available(current_time);
                            output_events.append(&mut new_events);
                        }
                    }
                }
                MEAT_READY_EVENT => {
                    // Handle our own meat ready events for flow control
                    if event.source_id() == &self.component_id {
                        let mut new_events = self.handle_meat_ready(current_time);
                        output_events.append(&mut new_events);
                    }
                }
                _ => {
                    // Ignore unknown events
                }
            }
        }

        output_events
    }
}