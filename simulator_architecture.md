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

### Event
**Responsibility**: Encapsulate event data and metadata

**Properties:**
- `type`: Event classification (string/enum)
- `data`: Event payload
- `source_id`: Originating component ID
- `target_ids`: Optional specific target components

### EventManager
**Responsibility**: Manage component registration and event subscriptions

**Properties:**
- `components`: Dict of component_id → component instance
- `subscriptions`: Dict of event_type → set of component_ids

**Methods:**
- `register_component(component)`: Add component and process its subscriptions
- `unregister_component(component_id)`: Remove component and its subscriptions
- `get_subscribers(event_type)`: Return component IDs subscribed to event type
- `subscribe(component_id, event_type)`: Add subscription
- `unsubscribe(component_id, event_type)`: Remove subscription

### EventScheduler
**Responsibility**: Maintain priority queue of future events based on execution time

**Properties:**
- `event_queue`: Min-heap priority queue [(time, sequence_num, event, targets)]
- `current_time`: Current simulation cycle
- `sequence_counter`: Ensure FIFO for same-time events

**Methods:**
- `schedule_event(event, targets, delay_cycles)`: Add event to queue
- `get_next_time_events()`: Return all events for next time step
- `has_events()`: Check if queue is empty
- `peek_next_time()`: Get next event time without removing

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
- **Batch Processing**: All events at same time processed together
- **Static Subscriptions**: Declared at component creation (can extend to dynamic later)