# Event-Based Simulator Architecture

## Overview
Event-based discrete time simulator where components subscribe to events, maintain state, and generate new events in response.

## Core Components

### BaseComponent (Trait)
**Responsibility**: Define interface for all simulation components

**Methods:**
- `component_id()`: Returns unique identifier (`ComponentId`)
- `subscriptions()`: Returns list of event types this component listens to (`&[&'static str]`)
- `react_atomic(events)`: Process list of events, return list of `(Box<dyn Event>, u64)` tuples

**Note**: Components manage their own internal state privately - no exposed state dictionary.

### Event (Trait-Based Design)
**Responsibility**: Define interface for all event types using flexible trait-based approach

**Event Trait Interface:**
- `id()`: Unique identifier for this event instance (`EventId`)
- `event_type()`: String classification for routing/subscriptions
- `source_id()`: Originating component ID
- `target_ids()`: Optional specific target components (`Option<Vec<ComponentId>>`)
- `data()`: Event payload as `HashMap<String, ComponentValue>`
- `clone_event()`: Create a boxed clone of the event

**ComponentValue Enum:**
Flexible data type supporting:
- `Int(i64)`: Integer values
- `Float(f64)`: Floating point values  
- `String(String)`: Text data
- `Bool(bool)`: Boolean flags

**Benefits:**
- Flexibility: Generic data payload supports diverse event types
- Extensibility: Easy to add new event types without touching core framework
- Runtime adaptability: Events can carry arbitrary structured data

### EventManager
**Responsibility**: Manage component registration and event routing

**Properties:**
- `components`: `HashMap<ComponentId, Box<dyn BaseComponent>>` - Registered components
- `subscriptions`: `HashMap<EventType, HashSet<ComponentId>>` - Event type subscriptions

**Methods:**
- `new()`: Create a new EventManager
- `register_component(component)`: Add component and process its subscriptions, returns `Result<(), String>`
- `route_event(event)`: Determine target components based on event.target_ids or subscriptions, returns `Vec<ComponentId>`
- `get_component_mut(component_id)`: Get mutable reference to a component by ID

**Error Handling:** Registration prevents duplicate component IDs and returns descriptive errors.

### EventScheduler
**Responsibility**: Maintain priority queue of future events based on relative delays

**Properties:**
- `event_queue`: `BinaryHeap<ScheduledEvent>` - Min-heap priority queue of scheduled events
- `sequence_counter`: `u64` - Ensures FIFO ordering for same-delay events

**ScheduledEvent Structure:**
- `delay_cycles`: `u64` - Remaining delay until execution
- `sequence_num`: `u64` - Ordering sequence for deterministic execution
- `event`: `Box<dyn Event>` - The event to be executed

**Methods:**
- `new()`: Create a new EventScheduler
- `schedule_event(event, delay_cycles)`: Add event to queue with delay
- `get_next_time_events()`: Return all events with minimum delay as `Vec<Box<dyn Event>>`
- `has_events()`: Check if queue is empty
- `peek_next_delay()`: Get minimum delay without removing events, returns `Option<u64>`
- `advance_time(cycles)`: Reduce all event delays by specified cycles

**Implementation Note:** Uses custom `Ord` implementation for `ScheduledEvent` to create min-heap behavior from Rust's max-heap `BinaryHeap`.

### SimulationEngine
**Responsibility**: Orchestrate simulation execution and coordinate between managers

**Properties:**
- `event_manager`: `EventManager` - Handles component registration and event routing
- `scheduler`: `EventScheduler` - Handles event timing and priority queue
- `current_cycle`: `u64` - Tracks current simulation time
- `max_cycles`: `Option<u64>` - Optional simulation limit

**Methods:**
- `new(max_cycles)`: Create new engine with optional cycle limit
- `register_component(component)`: Register component with EventManager, returns `Result<(), String>`
- `schedule_initial_event(event, delay_cycles)`: Schedule initial events to start simulation
- `run()`: Main simulation loop, returns `Result<u64, String>` with final cycle count
- `step()`: Process one time step, returns `Result<bool, String>` (true if events remain)
- `current_cycle()`: Get current simulation time
- `has_pending_events()`: Check if simulation can continue

**Error Handling:** All major operations return `Result` types with descriptive error messages for robust error handling.

## Execution Flow

1. **Initialization**
   - Register components with EventManager using `register_component()`
   - Schedule initial events with EventScheduler using `schedule_initial_event()`

2. **Main Loop** (in SimulationEngine)
   ```rust
   while self.has_pending_events() {
       // Check cycle limit
       if let Some(max) = self.max_cycles {
           if self.current_cycle >= max { break; }
       }
       
       // Get next event delay and advance simulation time
       let next_delay = self.scheduler.peek_next_delay().unwrap_or(0);
       self.scheduler.advance_time(next_delay);
       self.current_cycle += next_delay;
       
       // Extract all events for current time
       let events = self.scheduler.get_next_time_events();
       
       // Group events by target component for batch processing
       let mut events_by_component = HashMap::new();
       for event in events {
           let target_ids = self.event_manager.route_event(event.as_ref());
           for target_id in target_ids {
               events_by_component
                   .entry(target_id)
                   .or_insert_with(Vec::new)
                   .push(event.clone_event());
           }
       }
       
       // Process events and handle component reactions
       for (component_id, component_events) in events_by_component {
           if let Some(component) = self.event_manager.get_component_mut(&component_id) {
               let new_events = component.react_atomic(component_events);
               
               // Schedule each emitted event
               for (new_event, delay) in new_events {
                   self.scheduler.schedule_event(new_event, delay);
               }
           }
       }
   }
   ```

3. **Event Processing Flow**
   - **Time Advancement**: SimulationEngine advances current_cycle by next event delay
   - **Event Extraction**: All events at current time are pulled from scheduler
   - **Event Routing & Grouping**: Events are routed to targets and grouped by component for batch processing
   - **Component Reactions**: Components process their events via `react_atomic()` and return `Vec<(Box<dyn Event>, u64)>`
   - **Event Scheduling**: New events are directly scheduled in EventScheduler with their delays
   - **Logging**: Debug logging shows simulation cycle progression

## Key Design Decisions

- **Separation of Concerns**: Event routing (EventManager) vs timing (EventScheduler) vs execution (SimulationEngine)
- **Deterministic Ordering**: Priority queue + sequence numbers ensure reproducible execution
- **Batch Processing**: All events at same delay processed together for efficiency  
- **Component Event Emission**: Components return `Vec<(Box<dyn Event>, u64)>` tuples for delayed event generation
- **Centralized Orchestration**: SimulationEngine coordinates time advancement, event routing, and scheduling
- **Static Subscriptions**: Declared at component creation via `subscriptions()` method
- **Delay-Based Scheduling**: Events use relative delays instead of absolute times for simpler time management
- **Time Tracking**: SimulationEngine maintains current_cycle and advances by minimum event delays
- **Error Handling**: Comprehensive `Result<T, String>` return types for robust error management
- **Trait-Based Design**: `BaseComponent` and `Event` traits provide flexible, extensible interfaces
- **Generic Event Data**: `ComponentValue` enum allows arbitrary event payloads with type-safe accessors
- **Memory Efficiency**: Events stored as `Box<dyn Event>` for dynamic dispatch with minimal overhead
- **Clone Support**: Events implement `clone_event()` for efficient event duplication during routing