//! The fryer component for the burger production simulation.

use rsim::core::{
    component::BaseComponent,
    event::{Event, EventId, EventType},
    types::{ComponentId, ComponentValue},
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::events::{
    MeatReadyEvent, TriggerProductionEvent, BUFFER_FULL_EVENT, BUFFER_SPACE_AVAILABLE_EVENT,
    TRIGGER_PRODUCTION_EVENT,
};
use crate::{ProductionMode, MEAT_READY_EVENT};

/// The Fryer component, responsible for converting raw meat into fried patties.
pub struct Fryer {
    id: ComponentId,
    production_mode: ProductionMode,
    output_buffer: ComponentId,
    processing_time: u64,
    is_production_stopped: bool,
    subscriptions: Vec<&'static str>,
}

impl Fryer {
    /// Creates a new `Fryer`.
    pub fn new(
        id: ComponentId,
        production_mode: ProductionMode,
        output_buffer: ComponentId,
        processing_time: u64,
    ) -> Self {
        Self {
            id,
            production_mode,
            output_buffer,
            processing_time,
            is_production_stopped: false,
            subscriptions: vec![
                TRIGGER_PRODUCTION_EVENT,
                BUFFER_FULL_EVENT,
                BUFFER_SPACE_AVAILABLE_EVENT,
            ],
        }
    }

    fn handle_trigger_production(&mut self) -> Option<MeatReadyEvent> {
        if self.is_production_stopped {
            return None;
        }

        // In a real implementation, we would check for raw materials.
        // For now, we assume they are always available.
        log::info!("[Fryer {}] Starting to fry a new patty.", self.id);

        Some(MeatReadyEvent::new(
            self.id.clone(),
            Some(vec![self.output_buffer.clone()]),
        ))
    }
}

impl BaseComponent for Fryer {
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
                TRIGGER_PRODUCTION_EVENT => {
                    if let Some(meat_ready_event) = self.handle_trigger_production() {
                        new_events.push((Box::new(meat_ready_event), self.processing_time));

                        // In buffer-based mode, schedule the next production cycle.
                        if self.production_mode == ProductionMode::BufferBased
                            && !self.is_production_stopped
                        {
                            let trigger_event =
                                TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                            new_events.push((Box::new(trigger_event), 1));
                        }
                    }
                }
                BUFFER_FULL_EVENT => {
                    log::info!(
                        "[Fryer {}] received BufferFullEvent. Pausing production.",
                        self.id
                    );
                    self.is_production_stopped = true;
                }
                BUFFER_SPACE_AVAILABLE_EVENT => {
                    log::info!(
                        "[Fryer {}] received BufferSpaceAvailableEvent. Resuming production.",
                        self.id
                    );
                    let was_stopped = self.is_production_stopped;
                    self.is_production_stopped = false;

                    // If in buffer-based mode, trigger production immediately if it was stopped.
                    if self.production_mode == ProductionMode::BufferBased && was_stopped {
                        let trigger_event =
                            TriggerProductionEvent::new(self.id.clone(), Some(vec![self.id.clone()]));
                        new_events.push((Box::new(trigger_event), 1));
                    }
                }
                _ => {
                    log::warn!(
                        "[Fryer {}] received unhandled event type: {}",
                        self.id,
                        event.event_type()
                    );
                }
            }
        }
        new_events
    }
} 