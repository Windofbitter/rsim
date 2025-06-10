use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, SimulationTime};
use uuid::Uuid;

use crate::events::{
    StartBakingEvent, BreadReadyEvent,
    START_BAKING_EVENT, BREAD_READY_EVENT,
    BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT
};

/// Baker component that processes raw bread into cooked buns
#[derive(Debug)]
pub struct Baker {
    pub component_id: ComponentId,
    pub target_buffer_id: ComponentId,
    pub baking_delay: u64,
    pub is_production_stopped: bool,
    pub items_in_process: u32,
    pub max_concurrent_items: u32,
}

impl Baker {
    pub fn new(
        component_id: ComponentId, 
        target_buffer_id: ComponentId, 
        baking_delay: u64,
        max_concurrent_items: u32
    ) -> Self {
        Self {
            component_id,
            target_buffer_id,
            baking_delay,
            is_production_stopped: false,
            items_in_process: 0,
            max_concurrent_items,
        }
    }

    /// Creates a self-scheduled StartBakingEvent for continuous production
    fn create_start_baking_event(&self, timestamp: SimulationTime) -> Box<dyn Event> {
        Box::new(StartBakingEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            timestamp,
        })
    }

    /// Creates a BreadReadyEvent to send to the buffer
    fn create_bread_ready_event(&self, timestamp: SimulationTime, bread_id: String) -> Box<dyn Event> {
        Box::new(BreadReadyEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.target_buffer_id.clone(),
            timestamp,
            bread_id,
        })
    }

    /// Handles starting the baking process
    fn handle_start_baking(&mut self, current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Only start baking if production is not stopped and we have capacity
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            // Start baking a new bread bun
            self.items_in_process += 1;
            let bread_id = Uuid::new_v4().to_string();

            // Schedule the bread to be ready after baking delay
            let bread_ready_event = self.create_bread_ready_event(current_time, bread_id);
            output_events.push((bread_ready_event, self.baking_delay));

            // Schedule next baking cycle if not stopped and still have capacity
            if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
                let next_baking_event = self.create_start_baking_event(current_time);
                output_events.push((next_baking_event, 1)); // Start next item quickly
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
            
            // Resume baking if we have capacity
            if self.items_in_process < self.max_concurrent_items {
                let start_baking_event = self.create_start_baking_event(current_time);
                output_events.push((start_baking_event, 1)); // Resume quickly
            }
        }

        output_events
    }

    /// Handles bread ready events (decrements items in process counter)
    fn handle_bread_ready(&mut self, _current_time: SimulationTime) -> Vec<(Box<dyn Event>, u64)> {
        // Decrement items in process when bread is ready and sent to buffer
        if self.items_in_process > 0 {
            self.items_in_process -= 1;
        }

        // If production is not stopped and we now have capacity, start next item
        let mut output_events = Vec::new();
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            let start_baking_event = self.create_start_baking_event(0); // Use current time
            output_events.push((start_baking_event, 1));
        }

        output_events
    }
}

impl BaseComponent for Baker {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            START_BAKING_EVENT,
            BUFFER_FULL_EVENT,
            BUFFER_SPACE_AVAILABLE_EVENT,
            BREAD_READY_EVENT, // Subscribe to own bread ready events for flow control
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let current_time = 0; // In a real simulation, this would come from the engine

        for event in events {
            match event.event_type() {
                START_BAKING_EVENT => {
                    let mut new_events = self.handle_start_baking(current_time);
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
                BREAD_READY_EVENT => {
                    // Handle our own bread ready events for flow control
                    if event.source_id() == &self.component_id {
                        let mut new_events = self.handle_bread_ready(current_time);
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