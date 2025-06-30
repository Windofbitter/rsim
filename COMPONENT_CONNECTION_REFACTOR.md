# Component-to-Component Connection Refactoring Plan

## Overview
Refactor the simulation from event-subscription model to direct component-to-component connections, similar to how chips are wired.

## Current Architecture
- Components subscribe to event types
- EventManager routes events based on subscriptions
- Events can be broadcast or targeted

## New Architecture
- Components connect directly to other components
- Events broadcast to all connected components
- No event type subscriptions needed

## Key Changes

### 1. Update BaseComponent Trait
```rust
pub trait BaseComponent {
    fn component_id(&self) -> &ComponentId;
    
    // NEW: Get list of connected component IDs
    fn connected_components(&self) -> &[ComponentId];
    
    // NEW: Specify compatible component types for validation
    fn compatible_component_types(&self) -> Vec<&'static str>;
    
    // NEW: The single event type this component outputs
    fn output_event_type(&self) -> &'static str;
    
    // MODIFIED: Returns single event or None
    fn react_atomic(&mut self, events: Vec<Box<dyn Event>>) -> Option<(Box<dyn Event>, u64)>;
    
    // REMOVED: subscriptions() method
}
```

### 2. Replace EventManager with ConnectionManager
```rust
pub struct ConnectionManager {
    components: HashMap<ComponentId, Box<dyn BaseComponent>>,
    connections: HashMap<ComponentId, Vec<ComponentId>>, // source -> targets
}
```

Key methods:
- `register_component(component)` - Add component
- `connect(source_id, target_id)` - Create connection (validates compatibility)
- `get_connected_components(source_id)` - Get targets for routing

### 3. Update Event Routing in SimulationEngine
- Remove subscription-based routing
- Components output single event or None per reaction
- Event is cloned and sent to all connected components

### 4. Simplify Event Structure
- Remove `target_ids` field (no longer needed)
- Events always flow through connections

## Implementation Steps

1. **Create ConnectionManager**
   - New module `src/core/connection_manager.rs`
   - Implement component registration and connection methods

2. **Update BaseComponent trait**
   - Add `connected_components()` method
   - Remove `subscriptions()` method

3. **Modify SimulationEngine**
   - Replace EventManager with ConnectionManager
   - Update event routing logic to use connections

4. **Update Event trait**
   - Remove `target_ids()` method
   - Simplify event structure

5. **Create example components**
   - Implement basic logic gates (NAND, NOR, etc.)
   - Show how connections work in practice

## Benefits
- Clearer component relationships
- More intuitive for hardware modeling
- Simpler event routing
- No need to manage subscriptions

## Example Usage
```rust
// Create components
let nand1 = Box::new(NandGate::new("nand1"));
let nand2 = Box::new(NandGate::new("nand2"));

// Register components
connection_manager.register_component(nand1);
connection_manager.register_component(nand2);

// Connect them
connection_manager.connect("nand1", "nand2");

// Events from nand1 automatically flow to nand2
```