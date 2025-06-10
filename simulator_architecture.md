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
- `event_manager`: Handles subscriptions
- `scheduler`: Handles event timing
- `max_cycles`: Simulation limit

**Methods:**
- `initialize(components, initial_events)`: Set up simulation
- `run()`: Main simulation loop
- `step()`: Process one time step
- `distribute_events(events)`: Route events to subscribers

## Execution Flow

1. **Initialization**
   - Register components with EventManager
   - Schedule initial events with EventScheduler

2. **Main Loop** (in SimulationEngine)
   ```
   while scheduler.has_events() and time < max_cycles:
       events = scheduler.get_next_time_events()
       grouped_events = group_by_component(events)
       for component_id, event_list in grouped_events:
           new_events = component.react_atomic(event_list)
           for event, delay in new_events:
               scheduler.schedule_event(event, targets, delay)
   ```

3. **Event Distribution**
   - EventManager determines subscribers
   - Components process events in parallel within time step
   - Generated events are scheduled for future cycles

## Key Design Decisions

- **Separation of Concerns**: Event routing (EventManager) vs timing (EventScheduler) vs execution (SimulationEngine)
- **Deterministic Ordering**: Priority queue + sequence numbers ensure reproducible execution
- **Batch Processing**: All events at same delay processed together
- **Static Subscriptions**: Declared at component creation (can extend to dynamic later)
- **Delay-Based Scheduling**: Events use relative delays instead of absolute times for simpler time management