//! The assembler component for the burger production simulation.

use rsim::core::{
    component::BaseComponent,
    event::{Event, EventId, EventType},
    types::{ComponentId, ComponentValue},
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::events::{
    BurgerReadyEvent, TriggerProductionEvent, ItemAddedEvent, RequestItemEvent,
    ItemDispatchedEvent, ItemDroppedEvent, BufferFullEvent, BufferSpaceAvailableEvent,
};
use crate::ProductionMode;

/// Simplified assembly state
#[derive(Debug, Clone, PartialEq)]
enum AssemblerState {
    Idle,
    WaitingForIngredients,
    Assembling,
}

/// The Assembler component, responsible for combining meat and bread into complete burgers.
pub struct Assembler {
    id: ComponentId,
    production_mode: ProductionMode,
    processing_time: u64,
    is_production_stopped: bool,
    subscriptions: Vec<&'static str>,
    // Assembly state management
    state: AssemblerState,
    // Ingredient tracking
    meat_item: Option<String>,
    bread_item: Option<String>,
    // Backpressure handling
    held_item: Option<BurgerReadyEvent>,
}

impl Assembler {
    /// Creates a new `Assembler`.
    pub fn new(
        id: ComponentId,
        production_mode: ProductionMode,
        processing_time: u64,
    ) -> Self {
        Self {
            id,
            production_mode,
            processing_time,
            is_production_stopped: false,
            subscriptions: vec![
                "TriggerProductionEvent",
                "ItemAddedEvent",
                "ItemDispatchedEvent",
                "ItemDroppedEvent",
                "BufferFullEvent",
                "BufferSpaceAvailableEvent",
                "BurgerReadyEvent",
            ],
            state: AssemblerState::Idle,
            meat_item: None,
            bread_item: None,
            held_item: None,
        }
    }

    fn can_start_assembly(&self) -> bool {
        !self.is_production_stopped && self.state == AssemblerState::Idle
    }

    fn handle_trigger_production(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();

        match self.state {
            AssemblerState::Idle => {
                if !self.can_start_assembly() {
                    return new_events;
                }

                log::info!("[Assembler {}] Starting ingredient acquisition for new burger.", self.id);

                // Request both ingredients via broadcast
                let meat_request = RequestItemEvent::new(
                    self.id.clone(),
                    None, // Broadcast to all buffers
                    self.id.clone(),
                    "meat".to_string(),
                );
                let bread_request = RequestItemEvent::new(
                    self.id.clone(),
                    None, // Broadcast to all buffers
                    self.id.clone(),
                    "bread".to_string(),
                );

                new_events.push((Box::new(meat_request), 0));
                new_events.push((Box::new(bread_request), 0));

                // Update state to waiting for ingredients
                self.state = AssemblerState::WaitingForIngredients;
            }
            AssemblerState::WaitingForIngredients => {
                if self.is_production_stopped {
                    return new_events;
                }

                log::info!("[Assembler {}] Retrying ingredient acquisition - requesting missing ingredients.", self.id);

                // Only request missing ingredients
                if self.meat_item.is_none() {
                    let meat_request = RequestItemEvent::new(
                        self.id.clone(),
                        None, // Broadcast to all buffers
                        self.id.clone(),
                        "meat".to_string(),
                    );
                    new_events.push((Box::new(meat_request), 0));
                }

                if self.bread_item.is_none() {
                    let bread_request = RequestItemEvent::new(
                        self.id.clone(),
                        None, // Broadcast to all buffers
                        self.id.clone(),
                        "bread".to_string(),
                    );
                    new_events.push((Box::new(bread_request), 0));
                }
            }
            AssemblerState::Assembling => {
                // Already assembling, ignore trigger
            }
        }

        new_events
    }

    fn handle_item_dispatched(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();
        let data = event.data();

        let item_type = data.get("item_type")
            .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("unknown");
        let item_id = data.get("item_id")
            .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
            .unwrap_or("unknown");
        let success = data.get("success")
            .and_then(|v| if let ComponentValue::Bool(b) = v { Some(*b) } else { None })
            .unwrap_or(false);

        if self.state == AssemblerState::WaitingForIngredients {
            if success {
                match item_type {
                    "meat" => {
                        log::info!("[Assembler {}] Received meat ingredient: {}", self.id, item_id);
                        self.meat_item = Some(item_id.to_string());
                    }
                    "bread" => {
                        log::info!("[Assembler {}] Received bread ingredient: {}", self.id, item_id);
                        self.bread_item = Some(item_id.to_string());
                    }
                    _ => {
                        log::warn!("[Assembler {}] Received unexpected item type: {}", self.id, item_type);
                    }
                }

                // Check if we have both ingredients
                if let (Some(meat), Some(bread)) = (&self.meat_item, &self.bread_item) {
                    log::info!("[Assembler {}] Both ingredients acquired. Starting assembly.", self.id);
                    
                    let burger_id = format!("burger_{}", Uuid::new_v4());
                    let burger_event = BurgerReadyEvent::new(
                        self.id.clone(),
                        None, // Broadcast to all interested components
                        burger_id,
                    );

                    new_events.push((Box::new(burger_event), self.processing_time));

                    self.state = AssemblerState::Assembling;

                    // In BufferBased mode, immediately trigger next production cycle
                    if self.production_mode == ProductionMode::BufferBased && !self.is_production_stopped {
                        let trigger_event = TriggerProductionEvent::new(
                            self.id.clone(),
                            Some(vec![self.id.clone()])
                        );
                        new_events.push((Box::new(trigger_event), 1));
                    }
                }
            } else {
                // Ingredient request failed - keep acquired ingredients, don't reset to idle
                log::info!("[Assembler {}] Ingredient request failed for {}. Keeping acquired ingredients.", self.id, item_type);
            }
        }

        new_events
    }

    fn handle_item_added(&mut self, event: &dyn Event) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();

        // Only trigger production in BufferBased mode when idle
        if self.production_mode == ProductionMode::BufferBased && self.can_start_assembly() {
            let data = event.data();
            let buffer_type = data.get("buffer_type")
                .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("unknown");

            // Check if this is from an ingredient buffer (input buffers only)
            // Ignore events from assembly buffer (our output)
            if buffer_type == "fried_meat" || buffer_type == "cooked_bread" {
                log::info!("[Assembler {}] Ingredient available in {}. Triggering production.", self.id, buffer_type);
                let trigger_event = TriggerProductionEvent::new(
                    self.id.clone(),
                    Some(vec![self.id.clone()])
                );
                new_events.push((Box::new(trigger_event), 0));
            }
        }

        new_events
    }
}

impl BaseComponent for Assembler {
    fn component_id(&self) -> &ComponentId {
        &self.id
    }

    fn subscriptions(&self) -> &[&'static str] {
        &self.subscriptions
    }

    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();

        for event in events {
            match event.event_type() {
                "TriggerProductionEvent" => {
                    // Check if we have a held item to retry first
                    if let Some(held_item) = self.held_item.take() {
                        log::info!("[Assembler {}] Retrying held burger after space became available.", self.id);
                        new_events.push((Box::new(held_item), 0));
                        self.state = AssemblerState::Idle;
                    } else {
                        // Allow trigger production in any state - handle_trigger_production will decide what to do
                        let mut production_events = self.handle_trigger_production();
                        new_events.append(&mut production_events);
                    }
                }
                "ItemAddedEvent" => {
                    let mut item_added_events = self.handle_item_added(event.as_ref());
                    new_events.append(&mut item_added_events);
                }
                "ItemDispatchedEvent" => {
                    let mut dispatched_events = self.handle_item_dispatched(event.as_ref());
                    new_events.append(&mut dispatched_events);
                }
                "ItemDroppedEvent" => {
                    // Check if this component is the intended recipient
                    if let Some(target_ids) = event.target_ids() {
                        if !target_ids.contains(&self.id) {
                            // This event is not for us, ignore it
                            continue;
                        }
                    }
                    
                    let data = event.data();
                    let item_type = data.get("item_type")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    // Only handle drops for burger items (our output)
                    if item_type != "burger" {
                        continue;
                    }
                    
                    let item_id = data.get("item_id")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    let reason = data.get("reason")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    log::info!(
                        "[Assembler {}] Burger {} was dropped. Reason: {}. Will retry when space available.",
                        self.id,
                        item_id,
                        reason
                    );
                    
                    // Store the burger to retry later
                    self.held_item = Some(BurgerReadyEvent::new(
                        self.id.clone(),
                        None, // Broadcast to all interested components
                        item_id.to_string(),
                    ));
                    self.is_production_stopped = true;
                }
                "BufferFullEvent" => {
                    let data = event.data();
                    let buffer_type = data.get("buffer_type")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    // Only pause production if this is from our output buffer (assembly buffer)
                    if buffer_type == "assembly" {
                        log::info!(
                            "[Assembler {}] Assembly buffer is full. Pausing production.",
                            self.id
                        );
                        self.is_production_stopped = true;
                    }
                }
                "BufferSpaceAvailableEvent" => {
                    let data = event.data();
                    let buffer_type = data.get("buffer_type")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    // Only resume production if this is from our output buffer (assembly buffer)
                    if buffer_type == "assembly" {
                        log::info!(
                            "[Assembler {}] Assembly buffer has space available. Resuming production.",
                            self.id
                        );
                        let was_stopped = self.is_production_stopped;
                        self.is_production_stopped = false;

                        // If we have a held item, retry it immediately
                        if self.held_item.is_some() {
                            let trigger_event = TriggerProductionEvent::new(
                                self.id.clone(),
                                Some(vec![self.id.clone()])
                            );
                            new_events.push((Box::new(trigger_event), 0));
                        }
                        // Otherwise, if in buffer-based mode, trigger production if it was stopped
                        else if self.production_mode == ProductionMode::BufferBased && was_stopped && self.state == AssemblerState::Idle {
                            let trigger_event = TriggerProductionEvent::new(
                                self.id.clone(),
                                Some(vec![self.id.clone()])
                            );
                            new_events.push((Box::new(trigger_event), 1));
                        }
                    }
                }
                "BurgerReadyEvent" => {
                    let data = event.data();
                    let burger_id = data.get("item_id")
                        .and_then(|v| if let ComponentValue::String(s) = v { Some(s.as_str()) } else { None })
                        .unwrap_or("unknown");
                    
                    log::info!("[Assembler {}] Burger {} assembly completed. Transitioning to Idle.", self.id, burger_id);
                    
                    // Assembly processing is complete, reset to idle and clear ingredients
                    self.state = AssemblerState::Idle;
                    self.meat_item = None;
                    self.bread_item = None;
                    
                    // No need to forward the event - it already broadcasted to all interested components
                }
                _ => {
                    log::warn!(
                        "[Assembler {}] received unhandled event type: {}",
                        self.id,
                        event.event_type()
                    );
                }
            }
        }


        new_events
    }
}