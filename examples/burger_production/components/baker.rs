use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::ComponentId;
use uuid::Uuid;
use log::{info, debug, warn};

use crate::events::{
    StartBakingEvent, BreadReadyEvent,
    START_BAKING_EVENT, BREAD_READY_EVENT,
    BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT
};
use crate::ProductionStrategy;

/// Baker component that processes raw bread into cooked buns
#[derive(Debug)]
pub struct Baker {
    pub component_id: ComponentId,
    pub target_buffer_id: ComponentId,
    pub baking_delay: u64,
    pub is_production_stopped: bool,
    pub items_in_process: u32,
    pub max_concurrent_items: u32,
    pub production_strategy: ProductionStrategy,
}

impl Baker {
    pub fn new(
        component_id: ComponentId, 
        target_buffer_id: ComponentId, 
        baking_delay: u64,
        max_concurrent_items: u32,
        production_strategy: ProductionStrategy
    ) -> Self {
        Self {
            component_id,
            target_buffer_id,
            baking_delay,
            is_production_stopped: false,
            items_in_process: 0,
            max_concurrent_items,
            production_strategy,
        }
    }

    /// Creates a self-scheduled StartBakingEvent for continuous production
    fn create_start_baking_event(&self) -> Box<dyn Event> {
        Box::new(StartBakingEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
        })
    }

    /// Creates a BreadReadyEvent to send to the buffer
    fn create_bread_ready_event(&self, bread_id: String) -> Box<dyn Event> {
        Box::new(BreadReadyEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.target_buffer_id.clone(),
            bread_id,
        })
    }

    /// Handles starting the baking process
    fn handle_start_baking(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        
        // First check if this is a continuation after completing a previous item
        // If so, we need to decrement the items_in_process counter
        // (This is a workaround since we don't receive our own bread_ready events)
        if self.items_in_process >= self.max_concurrent_items {
            self.items_in_process = 0; // Reset since we're at capacity and starting fresh
            debug!("[Baker:{}] Reset items in process counter for continuous production", self.component_id);
        }

        // Only start baking if production is not stopped and we have capacity
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            // Start baking a new bread bun
            self.items_in_process += 1;
            let bread_id = Uuid::new_v4().to_string();
            
            info!("[Baker:{}] Started baking bread {} (items in process: {}/{})", 
                  self.component_id, bread_id, self.items_in_process, self.max_concurrent_items);

            // Schedule the bread to be ready after baking delay
            let bread_ready_event = self.create_bread_ready_event(bread_id.clone());
            output_events.push((bread_ready_event, self.baking_delay));
            
            debug!("[Baker:{}] Scheduled bread {} to be ready in {} cycles", 
                   self.component_id, bread_id, self.baking_delay);
            
        } else {
            debug!("[Baker:{}] Cannot start baking - stopped: {}, capacity: {}/{}", 
                   self.component_id, self.is_production_stopped, self.items_in_process, self.max_concurrent_items);
        }

        output_events
    }

    /// Handles when the downstream buffer becomes full
    fn handle_buffer_full(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Stop production when downstream buffer is full (backpressure)
        warn!("[Baker:{}] Downstream buffer full - stopping production", self.component_id);
        self.is_production_stopped = true;
        Vec::new()
    }

    /// Handles when the downstream buffer has space available
    fn handle_buffer_space_available(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Resume production when space becomes available
        if self.is_production_stopped {
            info!("[Baker:{}] Downstream buffer has space - resuming production", self.component_id);
            self.is_production_stopped = false;
            
            // Resume baking if we have capacity
            if self.items_in_process < self.max_concurrent_items {
                let start_baking_event = self.create_start_baking_event();
                output_events.push((start_baking_event, 1)); // Resume quickly
                debug!("[Baker:{}] Scheduled resumption baking cycle in 1 cycle", self.component_id);
            }
        }

        output_events
    }

    /// Handles bread ready events (decrements items in process counter)
    fn handle_bread_ready(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Decrement items in process when bread is ready and sent to buffer
        if self.items_in_process > 0 {
            self.items_in_process -= 1;
            info!("[Baker:{}] Bread completed and sent to buffer (items in process: {}/{})", 
                  self.component_id, self.items_in_process, self.max_concurrent_items);
        }

        // Only schedule next item for BufferBased strategy (continuous production)
        let mut output_events = Vec::new();
        if matches!(self.production_strategy, ProductionStrategy::BufferBased) 
           && !self.is_production_stopped 
           && self.items_in_process < self.max_concurrent_items {
            let start_baking_event = self.create_start_baking_event();
            output_events.push((start_baking_event, 1));
            debug!("[Baker:{}] BufferBased strategy - scheduled next baking cycle", self.component_id);
        } else if matches!(self.production_strategy, ProductionStrategy::OrderBased) {
            debug!("[Baker:{}] OrderBased strategy - waiting for production request", self.component_id);
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

        for event in events {
            match event.event_type() {
                START_BAKING_EVENT => {
                    let mut new_events = self.handle_start_baking();
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
                BREAD_READY_EVENT => {
                    // Handle our own bread ready events for flow control
                    if event.source_id() == &self.component_id {
                        debug!("[Baker:{}] Received own bread ready event", self.component_id);
                        let mut new_events = self.handle_bread_ready();
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