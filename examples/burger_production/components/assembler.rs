use rsim::core::component::BaseComponent;
use rsim::core::event::Event;
use rsim::core::types::{ComponentId, ComponentValue};
use uuid::Uuid;
use log::{info, debug, warn};

use crate::events::{
    StartAssemblyEvent, BurgerReadyEvent, RequestItemEvent,
    START_ASSEMBLY_EVENT, BURGER_READY_EVENT,
    ITEM_ADDED_EVENT, BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT
};

/// Assembler component that combines fried meat and cooked bread to create complete burgers
#[derive(Debug)]
pub struct Assembler {
    pub component_id: ComponentId,
    pub meat_buffer_id: ComponentId,
    pub bread_buffer_id: ComponentId,
    pub target_buffer_id: ComponentId,
    pub assembly_delay: u64,
    pub is_production_stopped: bool,
    pub items_in_process: u32,
    pub max_concurrent_items: u32,
    pub pending_meat_requests: u32,
    pub pending_bread_requests: u32,
    pub available_meat_count: u32,
    pub available_bread_count: u32,
}

impl Assembler {
    pub fn new(
        component_id: ComponentId,
        meat_buffer_id: ComponentId,
        bread_buffer_id: ComponentId,
        target_buffer_id: ComponentId,
        assembly_delay: u64,
        max_concurrent_items: u32
    ) -> Self {
        Self {
            component_id,
            meat_buffer_id,
            bread_buffer_id,
            target_buffer_id,
            assembly_delay,
            is_production_stopped: false,
            items_in_process: 0,
            max_concurrent_items,
            pending_meat_requests: 0,
            pending_bread_requests: 0,
            available_meat_count: 0,
            available_bread_count: 0,
        }
    }

    /// Creates a StartAssemblyEvent when both ingredients are available
    fn create_start_assembly_event(&self, meat_id: String, bread_id: String) -> Box<dyn Event> {
        Box::new(StartAssemblyEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            meat_id,
            bread_id,
        })
    }

    /// Creates a BurgerReadyEvent to send to the output buffer
    fn create_burger_ready_event(&self, burger_id: String) -> Box<dyn Event> {
        Box::new(BurgerReadyEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: self.target_buffer_id.clone(),
            burger_id,
        })
    }

    /// Creates a RequestItemEvent to request ingredients from buffers
    fn create_request_item_event(&self, target_buffer_id: ComponentId) -> Box<dyn Event> {
        Box::new(RequestItemEvent {
            id: Uuid::new_v4().to_string(),
            source_id: self.component_id.clone(),
            target_id: target_buffer_id,
            quantity: 1,
        })
    }

    /// Attempts to start assembly if both ingredients are available and we have capacity
    fn try_start_assembly(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Only attempt assembly if production is not stopped and we have capacity
        if !self.is_production_stopped && self.items_in_process < self.max_concurrent_items {
            // Check if we have both ingredients available
            if self.available_meat_count > 0 && self.available_bread_count > 0 {
                // Consume ingredients
                self.available_meat_count -= 1;
                self.available_bread_count -= 1;
                self.items_in_process += 1;

                info!("[Assembler:{}] Started assembly with meat and bread (items in process: {}/{}, remaining: meat={}, bread={})", 
                      self.component_id, self.items_in_process, self.max_concurrent_items, 
                      self.available_meat_count, self.available_bread_count);

                // Generate IDs for the ingredients being used
                let meat_id = Uuid::new_v4().to_string();
                let bread_id = Uuid::new_v4().to_string();

                // Create assembly event
                let start_assembly_event = self.create_start_assembly_event(meat_id, bread_id);
                output_events.push((start_assembly_event, 1)); // Start assembly immediately
            } else {
                // Request missing ingredients if we don't have pending requests
                if self.available_meat_count == 0 && self.pending_meat_requests == 0 {
                    debug!("[Assembler:{}] Requesting meat from buffer", self.component_id);
                    let request_meat_event = self.create_request_item_event(self.meat_buffer_id.clone());
                    output_events.push((request_meat_event, 1));
                    self.pending_meat_requests += 1;
                }
                if self.available_bread_count == 0 && self.pending_bread_requests == 0 {
                    debug!("[Assembler:{}] Requesting bread from buffer", self.component_id);
                    let request_bread_event = self.create_request_item_event(self.bread_buffer_id.clone());
                    output_events.push((request_bread_event, 1));
                    self.pending_bread_requests += 1;
                }
            }
        } else {
            debug!("[Assembler:{}] Cannot start assembly - stopped: {}, capacity: {}/{}", 
                   self.component_id, self.is_production_stopped, self.items_in_process, self.max_concurrent_items);
        }

        output_events
    }

    /// Handles starting the assembly process
    fn handle_start_assembly(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Extract meat_id and bread_id from event data
        let data = event.data();
        if let (Some(ComponentValue::String(meat_id)), Some(ComponentValue::String(bread_id))) = 
            (data.get("meat_id"), data.get("bread_id")) {
            
            // Generate burger ID combining meat and bread IDs
            let burger_id = format!("burger_{}_{}", meat_id, bread_id);
            
            info!("[Assembler:{}] Assembling burger {} from meat {} and bread {}", 
                  self.component_id, burger_id, meat_id, bread_id);

            // Schedule the burger to be ready after assembly delay
            let burger_ready_event = self.create_burger_ready_event(burger_id.clone());
            output_events.push((burger_ready_event, self.assembly_delay));
            
            debug!("[Assembler:{}] Scheduled burger {} to be ready in {} cycles", 
                   self.component_id, burger_id, self.assembly_delay);
        }

        output_events
    }

    /// Handles when a burger is ready (decrements items in process)
    fn handle_burger_ready(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Decrement items in process when burger is ready and sent to buffer
        if self.items_in_process > 0 {
            self.items_in_process -= 1;
            info!("[Assembler:{}] Burger completed and sent to buffer (items in process: {}/{})", 
                  self.component_id, self.items_in_process, self.max_concurrent_items);
        }

        // Try to start next assembly if we have capacity and ingredients
        self.try_start_assembly()
    }

    /// Handles when items are added to input buffers
    fn handle_item_added(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();
        let data = event.data();

        if let Some(ComponentValue::String(item_type)) = data.get("item_type") {
            // Update available counts based on item type and source buffer
            if event.source_id() == &self.meat_buffer_id && item_type == "meat" {
                self.available_meat_count += 1;
                if self.pending_meat_requests > 0 {
                    self.pending_meat_requests -= 1;
                }
                debug!("[Assembler:{}] Received meat from buffer (available: {}, pending requests: {})", 
                       self.component_id, self.available_meat_count, self.pending_meat_requests);
            } else if event.source_id() == &self.bread_buffer_id && item_type == "bread" {
                self.available_bread_count += 1;
                if self.pending_bread_requests > 0 {
                    self.pending_bread_requests -= 1;
                }
                debug!("[Assembler:{}] Received bread from buffer (available: {}, pending requests: {})", 
                       self.component_id, self.available_bread_count, self.pending_bread_requests);
            }

            // Try to start assembly now that we have more ingredients
            let mut new_events = self.try_start_assembly();
            output_events.append(&mut new_events);
        }

        output_events
    }

    /// Handles when the downstream buffer becomes full
    fn handle_buffer_full(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        // Stop production when downstream buffer is full (backpressure)
        warn!("[Assembler:{}] Downstream buffer full - stopping production", self.component_id);
        self.is_production_stopped = true;
        Vec::new()
    }

    /// Handles when the downstream buffer has space available
    fn handle_buffer_space_available(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        // Resume production when space becomes available
        if self.is_production_stopped {
            info!("[Assembler:{}] Downstream buffer has space - resuming production", self.component_id);
            self.is_production_stopped = false;
            
            // Try to start assembly if we have capacity and ingredients
            let mut new_events = self.try_start_assembly();
            output_events.append(&mut new_events);
        }

        output_events
    }
}

impl BaseComponent for Assembler {
    fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &[
            START_ASSEMBLY_EVENT,
            BURGER_READY_EVENT,
            ITEM_ADDED_EVENT,
            BUFFER_FULL_EVENT,
            BUFFER_SPACE_AVAILABLE_EVENT,
        ]
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut output_events = Vec::new();

        for event in events {
            match event.event_type() {
                START_ASSEMBLY_EVENT => {
                    let mut new_events = self.handle_start_assembly(event.as_ref());
                    output_events.append(&mut new_events);
                }
                BURGER_READY_EVENT => {
                    // Handle our own burger ready events for flow control
                    if event.source_id() == &self.component_id {
                        let mut new_events = self.handle_burger_ready();
                        output_events.append(&mut new_events);
                    }
                }
                ITEM_ADDED_EVENT => {
                    // Handle item added events from our input buffers
                    if event.source_id() == &self.meat_buffer_id || event.source_id() == &self.bread_buffer_id {
                        let mut new_events = self.handle_item_added(event.as_ref());
                        output_events.append(&mut new_events);
                    }
                }
                BUFFER_FULL_EVENT => {
                    // Only handle if this event is targeted at us (from downstream buffer)
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_full();
                            output_events.append(&mut new_events);
                        }
                    }
                }
                BUFFER_SPACE_AVAILABLE_EVENT => {
                    // Only handle if this event is targeted at us (from downstream buffer)
                    if let Some(target_ids) = event.target_ids() {
                        if target_ids.contains(&self.component_id) {
                            let mut new_events = self.handle_buffer_space_available();
                            output_events.append(&mut new_events);
                        }
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