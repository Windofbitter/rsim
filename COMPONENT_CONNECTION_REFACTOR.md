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
    
    // NEW: Define input port names (e.g., ["A", "B"] for NAND gate)
    fn input_ports(&self) -> Vec<&'static str>;
    
    // NEW: Single output port name (e.g., "Y" for NAND gate)
    fn output_port(&self) -> &'static str;
    
    // NEW: Specify compatible component types for validation
    fn compatible_component_types(&self) -> Vec<&'static str>;
    
    // NEW: The single event type this component outputs
    fn output_event_type(&self) -> &'static str;
    
    // MODIFIED: Receives events by port name, returns single event or None
    fn react_atomic(&mut self, port_events: HashMap<String, Box<dyn Event>>) 
        -> Option<Box<dyn Event>>;
    
    // REMOVED: subscriptions() method
    // REMOVED: connected_components() method (handled by ConnectionManager)
}
```

### 2. Replace EventManager with ConnectionManager
```rust
pub struct ConnectionManager {
    components: HashMap<ComponentId, Box<dyn BaseComponent>>,
    // (source_comp, source_port) -> Vec<(target_comp, target_port)>
    connections: HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
}
```

Key methods:
- `register_component(component)` - Add component
- `connect_ports(source_id, source_port, target_id, target_port)` - Create port-to-port connection
- `get_port_connections(source_id, source_port)` - Get targets for specific output port
- `route_event_to_ports(event, source_id, source_port)` - Route event through connections

### 3. Replace SimulationEngine with Per-Cycle Evaluation
- Remove EventScheduler entirely - no priority queue needed
- Evaluate all components every cycle synchronously
- Components maintain internal state between cycles
- Events flow to connected components for next cycle

### 4. Simplify Event Structure
- Remove `target_ids` field (no longer needed)
- Events always flow through connections

## Implementation Steps

1. **Create ConnectionManager**
   - New module `src/core/connection_manager.rs`
   - Implement component registration and connection methods

2. **Update BaseComponent trait**
   - Add `input_ports()` method to define input port names
   - Add `output_port()` method to define single output port name
   - Add `compatible_component_types()` method
   - Add `output_event_type()` method
   - Remove `subscriptions()` method
   - Update `react_atomic()` to receive port-based events

3. **Replace SimulationEngine with CycleEngine**
   - Remove EventScheduler dependency
   - Implement per-cycle evaluation loop
   - Use ConnectionManager for event routing

4. **Update Event trait**
   - Remove `target_ids()` method
   - Simplify event structure

5. **Create example components**
   - Implement basic logic gates with internal state
   - Show how per-cycle evaluation works in practice

## Benefits
- Clearer component relationships
- More intuitive for hardware modeling (synchronous circuits)
- Enables true parallelization - all components evaluated simultaneously
- Eliminates complex event scheduling and priority queues
- Simpler event routing through direct connections
- Matches real digital circuit behavior (clock-driven)

## Example Usage
```rust
// Create components
let nand1 = Box::new(NandGate::new("nand1")); // input_ports: ["A", "B"], output_port: "Y"
let nand2 = Box::new(NandGate::new("nand2")); // input_ports: ["A", "B"], output_port: "Y"

// Register components
connection_manager.register_component(nand1);
connection_manager.register_component(nand2);

// Connect ports: nand1's output Y to nand2's input A
connection_manager.connect_ports("nand1", "Y", "nand2", "A");

// Per-cycle evaluation loop
for cycle in 0..max_cycles {
    // All components evaluate simultaneously with port-specific inputs
    let events: Vec<_> = components.iter_mut()
        .filter_map(|comp| comp.react_atomic(port_events_for_component))
        .collect();
    
    // Route events through port connections for next cycle
    route_events_through_port_connections(events);
}
```