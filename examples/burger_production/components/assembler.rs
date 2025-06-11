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

/// Assembly state tracking ingredient acquisition
#[derive(Debug, Clone, PartialEq)]
enum AssemblyState {
    Idle,
    WaitingForIngredients {
        meat_requested: bool,
        bread_requested: bool,
        meat_item: Option<String>,
        bread_item: Option<String>,
    },
    Assembling {
        meat_item: String,
        bread_item: String,
    },
}

/// The Assembler component, responsible for combining meat and bread into complete burgers.
pub struct Assembler {
    id: ComponentId,
    production_mode: ProductionMode,
    meat_buffer: ComponentId,
    bread_buffer: ComponentId,
    output_buffer: ComponentId,
    processing_time: u64,
    is_production_stopped: bool,
    subscriptions: Vec<&'static str>,
    // Assembly state management
    assembly_state: AssemblyState,
    // Backpressure handling
    held_item: Option<BurgerReadyEvent>,
}

impl Assembler {
    /// Creates a new `Assembler`.
    pub fn new(
        id: ComponentId,
        production_mode: ProductionMode,
        meat_buffer: ComponentId,
        bread_buffer: ComponentId,
        output_buffer: ComponentId,
        processing_time: u64,
    ) -> Self {
        Self {
            id,
            production_mode,
            meat_buffer,
            bread_buffer,
            output_buffer,
            processing_time,
            is_production_stopped: false,
            subscriptions: vec![
                "TriggerProductionEvent",
                "ItemAddedEvent",
                "ItemDispatchedEvent",
                "ItemDroppedEvent",
                "BufferFullEvent",
                "BufferSpaceAvailableEvent",
            ],
            assembly_state: AssemblyState::Idle,
            held_item: None,
        }
    }

    fn can_start_assembly(&self) -> bool {
        !self.is_production_stopped && self.assembly_state == AssemblyState::Idle
    }

    fn handle_trigger_production(&mut self) -> Vec<(Box<dyn Event>, u64)> {
        let mut new_events = Vec::new();

        if !self.can_start_assembly() {
            return new_events;
        }

        log::info!("[Assembler {}] Starting ingredient acquisition for new burger.", self.id);

        // Request both ingredients
        let meat_request = RequestItemEvent::new(
            self.id.clone(),
            Some(vec![self.meat_buffer.clone()]),
            self.id.clone(),
        );
        let bread_request = RequestItemEvent::new(
            self.id.clone(),
            Some(vec![self.bread_buffer.clone()]),
            self.id.clone(),
        );

        new_events.push((Box::new(meat_request), 0));
        new_events.push((Box::new(bread_request), 0));

        // Update state to track ingredient requests
        self.assembly_state = AssemblyState::WaitingForIngredients {
            meat_requested: true,
            bread_requested: true,
            meat_item: None,
            bread_item: None,
        };

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

        if let AssemblyState::WaitingForIngredients { 
            meat_requested, bread_requested, mut meat_item, mut bread_item 
        } = self.assembly_state.clone() {
            
            if success {
                match item_type {
                    "meat" => {
                        log::info!("[Assembler {}] Received meat ingredient: {}", self.id, item_id);
                        meat_item = Some(item_id.to_string());
                    }
                    "bread" => {
                        log::info!("[Assembler {}] Received bread ingredient: {}", self.id, item_id);
                        bread_item = Some(item_id.to_string());
                    }
                    _ => {
                        log::warn!("[Assembler {}] Received unexpected item type: {}", self.id, item_type);
                    }
                }

                // Check if we have both ingredients
                if let (Some(meat), Some(bread)) = (&meat_item, &bread_item) {
                    log::info!("[Assembler {}] Both ingredients acquired. Starting assembly.", self.id);
                    
                    let burger_id = format!("burger_{}", Uuid::new_v4());
                    let burger_event = BurgerReadyEvent::new(
                        self.id.clone(),
                        Some(vec![self.output_buffer.clone()]),
                        burger_id,
                    );

                    new_events.push((Box::new(burger_event), self.processing_time));

                    self.assembly_state = AssemblyState::Assembling {
                        meat_item: meat.clone(),
                        bread_item: bread.clone(),
                    };

                    // In BufferBased mode, immediately trigger next production cycle
                    if self.production_mode == ProductionMode::BufferBased && !self.is_production_stopped {
                        let trigger_event = TriggerProductionEvent::new(
                            self.id.clone(),
                            Some(vec![self.id.clone()])
                        );
                        new_events.push((Box::new(trigger_event), 1));
                    }
                } else {
                    // Update state with partial ingredients
                    self.assembly_state = AssemblyState::WaitingForIngredients {
                        meat_requested,
                        bread_requested,
                        meat_item,
                        bread_item,
                    };
                }
            } else {
                // Ingredient request failed, reset to idle
                log::info!("[Assembler {}] Ingredient request failed for {}. Resetting to idle.", self.id, item_type);
                self.assembly_state = AssemblyState::Idle;
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

            // Check if this is from one of our ingredient buffers
            if buffer_type == "meat" || buffer_type == "bread" {
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
                        self.assembly_state = AssemblyState::Idle;
                    } else {
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
                    let data = event.data();
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
                        Some(vec![self.output_buffer.clone()]),
                        item_id.to_string(),
                    ));
                    self.is_production_stopped = true;
                }
                "BufferFullEvent" => {
                    log::info!(
                        "[Assembler {}] received BufferFullEvent. Pausing production.",
                        self.id
                    );
                    self.is_production_stopped = true;
                }
                "BufferSpaceAvailableEvent" => {
                    log::info!(
                        "[Assembler {}] received BufferSpaceAvailableEvent. Resuming production.",
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
                    else if self.production_mode == ProductionMode::BufferBased && was_stopped && self.assembly_state == AssemblyState::Idle {
                        let trigger_event = TriggerProductionEvent::new(
                            self.id.clone(),
                            Some(vec![self.id.clone()])
                        );
                        new_events.push((Box::new(trigger_event), 1));
                    }
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

        // Reset assembly state when burger is completed
        if let AssemblyState::Assembling { .. } = self.assembly_state {
            // Check if we just scheduled a BurgerReadyEvent
            if new_events.iter().any(|(event, _)| event.event_type() == "BurgerReadyEvent") {
                self.assembly_state = AssemblyState::Idle;
            }
        }

        new_events
    }
}