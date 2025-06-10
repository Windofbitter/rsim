use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::ComponentId;
use uuid::Uuid;
use log::{info, debug, warn};

use crate::events::{
    StartFryingEvent, MeatReadyEvent,
    START_FRYING_EVENT, MEAT_READY_EVENT,
    BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT
};
use crate::ProductionStrategy;

/// Fryer component that processes raw meat into fried meat patties
#[derive(Debug)]
pub struct Fryer {
    pub component_id: ComponentId,
    pub target_buffer_id: ComponentId,
    pub frying_delay: u64,
    pub is_production_stopped: bool,
    pub items_in_process: u32,
    pub max_concurrent_items: u32,
    pub production_strategy: ProductionStrategy,
}

impl Fryer {
    pub fn new(
        component_id: ComponentId, 
        target_buffer_id: ComponentId, 
        frying_delay: u64,
        max_concurrent_items: u32,
        production_strategy: ProductionStrategy
    ) -> Self {
        Self {
            component_id,
            target_buffer_id,
            frying_delay,
            is_production_stopped: false,
            items_in_process: 0,
            max_concurrent_items,
            production_strategy,
        }
    }

    /// Creates a self-scheduled StartFryingEvent for continuous production
    fn create_start_frying_event(&self) -> Box<dyn Event> {
        Box::new(StartFryingEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
        })
    }

    /// Creates a MeatReadyEvent to send to the buffer and self for flow control
    fn create_meat_ready_event(&self, meat_id: String) -> Box<dyn Event> {
        // Create an event with no target_ids so it broadcasts to all subscribers
        Box::new(MeatReadyEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.target_buffer_id.clone(), // Still target buffer primarily
            meat_id,
        })
    }

    /// Handles starting the frying process
    fn handle_start_frying(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        
        // Check if we have capacity for new production
        if self.items_in_process >= self.max_concurrent_items {
            debug!("[Fryer:{}] Already at capacity ({}/{}), cannot start new item", 
                   self.component_id, self.items_in_process, self.max_concurrent_items);
            return output_events;
        }

        // Only start frying if production is not stopped and we have capacity
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            // Start frying a new meat patty
            self.items_in_process += 1;
            let meat_id = Uuid::new_v4().to_string();
            
            info!("[Fryer:{}] Started frying meat {} (items in process: {}/{})", 
                  self.component_id, meat_id, self.items_in_process, self.max_concurrent_items);

            // Schedule the meat to be ready after frying delay
            let meat_ready_event = self.create_meat_ready_event(meat_id.clone());
            output_events.push((meat_ready_event, self.frying_delay));
            
            debug!("[Fryer:{}] Scheduled meat {} to be ready in {} cycles", 
                   self.component_id, meat_id, self.frying_delay);
            
        } else {
            debug!("[Fryer:{}] Cannot start frying - stopped: {}, capacity: {}/{}", 
                   self.component_id, self.is_production_stopped, self.items_in_process, self.max_concurrent_items);
        }

        output_events
    }

    /// Handles when the downstream buffer becomes full
    fn handle_buffer_full(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Stop production when downstream buffer is full (backpressure)
        warn!("[Fryer:{}] Downstream buffer full - stopping production", self.component_id);
        self.is_production_stopped = true;
        Vec::new()
    }

    /// Handles when the downstream buffer has space available
    fn handle_buffer_space_available(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Resume production when space becomes available
        if self.is_production_stopped {
            info!("[Fryer:{}] Downstream buffer has space - resuming production", self.component_id);
            self.is_production_stopped = false;
            
            // Resume frying if we have capacity
            if self.items_in_process < self.max_concurrent_items {
                let start_frying_event = self.create_start_frying_event();
                output_events.push((start_frying_event, 1)); // Resume quickly
                debug!("[Fryer:{}] Scheduled resumption frying cycle in 1 cycle", self.component_id);
            }
        }

        output_events
    }

    /// Handles meat ready events (decrements items in process counter)
    fn handle_meat_ready(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Decrement items in process when meat is ready and sent to buffer
        if self.items_in_process > 0 {
            self.items_in_process -= 1;
            info!("[Fryer:{}] Meat completed and sent to buffer (items in process: {}/{})", 
                  self.component_id, self.items_in_process, self.max_concurrent_items);
        }

        // Only schedule next item for BufferBased strategy (continuous production)
        let mut output_events = Vec::new();
        if matches!(self.production_strategy, ProductionStrategy::BufferBased) 
           && !self.is_production_stopped 
           && self.items_in_process < self.max_concurrent_items {
            let start_frying_event = self.create_start_frying_event();
            output_events.push((start_frying_event, 1));
            debug!("[Fryer:{}] BufferBased strategy - scheduled next frying cycle", self.component_id);
        } else if matches!(self.production_strategy, ProductionStrategy::OrderBased) {
            debug!("[Fryer:{}] OrderBased strategy - waiting for production request", self.component_id);
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

        for event in events {
            match event.event_type() {
                START_FRYING_EVENT => {
                    let mut new_events = self.handle_start_frying();
                    output_events.append(&mut new_events);
                }
                BUFFER_FULL_EVENT => {
                    // Only handle if this event is targeted at us
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_full();
                            output_events.append(&mut new_events);
                        }
                    }
                }
                BUFFER_SPACE_AVAILABLE_EVENT => {
                    // Only handle if this event is targeted at us
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_space_available();
                            output_events.append(&mut new_events);
                        }
                    }
                }
                MEAT_READY_EVENT => {
                    // Handle our own meat ready events for flow control
                    if event.source_id() == &self.component_id {
                        let mut new_events = self.handle_meat_ready();
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