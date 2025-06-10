# Event-Based Simulator Architecture

## Overview
Event-based discrete time simulator where components subscribe to events, maintain state, and generate new events in response.

## Core Classes

### BaseComponent
**Responsibility**: Define interface for all simulation components

**Properties:**
- `component_id`: Unique identifier
- `state`: Internal component state (dictionary)
- `subscriptions`: List of event types this component listens to

**Methods:**
- `react_atomic(events)`: Process list of events, return list of (event, delay_cycles) tuples

### Event (Trait-Based Design)
**Responsibility**: Define interface for all event types with type-safe, strongly-typed implementations

**Event Trait Interface:**
- `event_id()`: Unique identifier for this event instance
- `source_id()`: Originating component ID
- `target_ids()`: Optional specific target components
- `event_type()`: String classification for routing/subscriptions

**Concrete Event Types:**
Each event type implements the Event trait with its own strongly-typed fields:
- `MemoryReadEvent`: address, size
- `ClockTickEvent`: cycle
- `ProcessorInstruction`: opcode, operands
- Custom events as needed

**Benefits:**
- Type safety: Compiler enforces correct event structure
- Extensibility: Easy to add new event types without touching core framework
- Performance: No runtime type casting of generic data payloads

### EventManager
**Responsibility**: Manage component registration and event routing

**Properties:**
- `components`: Dict of component_id → component instance
- `subscriptions`: Dict of event_type → set of component_ids

**Methods:**
- `register_component(component)`: Add component and process its subscriptions
- `route_event(event)`: Determine target components based on event.target_ids or subscriptions

### EventScheduler
**Responsibility**: Maintain priority queue of future events based on relative delays

**Properties:**
- `event_queue`: Min-heap priority queue [(delay_cycles, sequence_num, event, targets)]
- `sequence_counter`: Ensure FIFO for same-delay events

**Methods:**
- `schedule_event(event, targets, delay_cycles)`: Add event to queue with delay
- `get_next_time_events()`: Return all events with minimum delay
- `has_events()`: Check if queue is empty
- `peek_next_delay()`: Get minimum delay without removing events
- `advance_time(cycles)`: Reduce all event delays by specified cycles

### SimulationEngine
**Responsibility**: Orchestrate simulation execution and coordinate between managers

**Properties:**
- `event_manager`: Handles component registration and event routing
- `scheduler`: Handles event timing and priority queue
- `current_cycle`: Tracks current simulation time
- `max_cycles`: Optional simulation limit

**Methods:**
- `new(max_cycles)`: Create new engine with optional cycle limit
- `register_component(component)`: Register component with EventManager
- `schedule_initial_event(event, targets, delay)`: Schedule initial events
- `run()`: Main simulation loop, returns final cycle count
- `step()`: Process one time step, returns true if events remain
- `current_cycle()`: Get current simulation time
- `has_pending_events()`: Check if simulation can continue

## Execution Flow

1. **Initialization**
   - Register components with EventManager
   - Schedule initial events with EventScheduler

2. **Main Loop** (in SimulationEngine)
   ```
   while scheduler.has_events() and current_cycle < max_cycles:
       // Get next event delay and advance simulation time
       next_delay = scheduler.peek_next_delay()
       current_cycle += next_delay
       scheduler.advance_time(next_delay)
       
       // Extract all events for current time
       events = scheduler.get_next_time_events()
       
       // Group events by target component for batch processing
       grouped_events = group_by_component(events)
       
       // Process events and handle component reactions
       for component_id, event_list in grouped_events:
           new_events = component.react_atomic(event_list)  // Returns Vec<(Event, u64)>
           
           // Route and schedule each emitted event
           for (event, delay_cycles) in new_events:
               targets = event_manager.route_event(&event)
               scheduler.schedule_event(event, targets, delay_cycles)
   ```

3. **Event Processing Flow**
   - **Time Advancement**: SimulationEngine advances current_cycle by next event delay
   - **Event Extraction**: All events at current time are pulled from scheduler
   - **Event Grouping**: Events grouped by target component for efficient batch processing
   - **Component Reactions**: Components process their events via `react_atomic()` 
   - **Event Emission**: Components return new events with delay cycles: `Vec<(Event, u64)>`
   - **Event Routing**: EventManager determines targets for each emitted event
   - **Event Scheduling**: New events scheduled in EventScheduler with their delays

## Key Design Decisions

- **Separation of Concerns**: Event routing (EventManager) vs timing (EventScheduler) vs execution (SimulationEngine)
- **Deterministic Ordering**: Priority queue + sequence numbers ensure reproducible execution
- **Batch Processing**: All events at same delay processed together for efficiency
- **Component Event Emission**: Components return `Vec<(Event, u64)>` tuples for delayed event generation
- **Centralized Orchestration**: SimulationEngine coordinates time advancement, event routing, and scheduling
- **Static Subscriptions**: Declared at component creation (can extend to dynamic later)
- **Delay-Based Scheduling**: Events use relative delays instead of absolute times for simpler time management
- **Time Tracking**: SimulationEngine maintains current_cycle and advances by minimum event delays