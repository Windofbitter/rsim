use std::collections::{HashMap, HashSet};
use super::component::BaseComponent;
use super::types::ComponentId;
use super::event::{Event, EventType};

pub struct EventManager {
    components: HashMap<ComponentId, Box<dyn BaseComponent>>,
    subscriptions: HashMap<EventType, HashSet<ComponentId>>,
}

impl EventManager {
    /// Create a new EventManager
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Register a component and process its subscriptions
    pub fn register_component(&mut self, component: Box<dyn BaseComponent>) -> Result<(), String> {
        let component_id = component.component_id().clone();
        
        // Check for duplicate registration
        if self.components.contains_key(&component_id) {
            return Err(format!("Component with ID '{}' is already registered", component_id));
        }
        
        // Process component's subscriptions
        for event_type in component.subscriptions() {
            self.subscriptions
                .entry(event_type.clone())
                .or_insert_with(HashSet::new)
                .insert(component_id.clone());
        }
        
        // Store the component
        self.components.insert(component_id, component);
        
        Ok(())
    }


    /// Route event to appropriate subscribers, returning target component IDs
    pub fn route_event(&self, event: &Event) -> Vec<ComponentId> {
        match &event.target_ids {
            // If specific targets are provided, filter to only registered components
            Some(targets) => {
                targets.iter()
                    .filter(|id| self.components.contains_key(*id))
                    .cloned()
                    .collect()
            }
            // If no specific targets, find all subscribers to this event type
            None => {
                self.subscriptions
                    .get(&event.event_type)
                    .map(|subscribers| subscribers.iter().cloned().collect())
                    .unwrap_or_else(Vec::new)
            }
        }
    }

}