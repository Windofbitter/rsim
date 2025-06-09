# Event-Based Simulator Architecture

## Overview

A simple event-based simulator with reusable components that can subscribe to events, maintain internal state, and generate new events in response to received events.

## Core Components

### BaseComponent
The fundamental building block that all simulator components inherit from.

**Properties:**
- `component_id`: Unique identifier
- `state`: Internal component state (dictionary)
- `subscriptions`: List of event types this component subscribes to

**Methods:**
- `react_atomic(events)`: Process a list of events in parallel and return list of (event, cycles) tuples

### Event
Simple event structure containing type, data, and metadata.

**Properties:**
- `type`: Event classification
- `data`: Event payload
- `source_id`: Component that generated the event
- `timestamp`: When the event occurs

### EventScheduler
Manages the event queue and schedules future events.

**Properties:**
- `event_queue`: Priority queue ordered by time
- `current_time`: Current simulation time

**Methods:**
- `schedule_event(event, target_component, delay_cycles)`: Schedule future event
- `get_next_event()`: Retrieve next event to process

### SimulationEngine
Main orchestrator that runs the simulation loop.

**Properties:**
- `components`: Registry of all components
- `scheduler`: Event scheduling system
- `event_subscriptions`: Mapping of event types to subscribed components

**Methods:**
- `add_component(component)`: Register component and subscriptions
- `run_simulation(max_cycles)`: Execute main simulation loop

## Simulation Flow

1. **Initialization Phase**
   - Components are created and registered
   - Event subscriptions are established
   - Initial events are scheduled

2. **Main Execution Loop**
   - Retrieve all events for current time step
   - Group events by subscribed components
   - Each component processes its event list via `react_atomic(events)`
   - Schedule any generated events
   - Advance simulation time
   - Continue until termination condition

3. **Event Processing**
   - Component receives list of events for current time step
   - Internal state is updated based on all events
   - Zero or more new events are generated
   - Events are scheduled with specified delays

## Key Design Principles

- **Modularity**: Components are self-contained and reusable
- **Event-Driven**: All interactions happen through events
- **Deterministic**: Events processed in time order for reproducible results
- **Extensible**: Easy to add new component types and event types
- **Parallel Processing**: Components can process multiple events simultaneously within a time step

## Component Interaction Model

Components interact exclusively through events:
- No direct method calls between components
- All communication is asynchronous
- Components can generate multiple events in response to one input
- Events can be scheduled for immediate or future execution

## Timing Model

- Discrete time steps (cycles)
- Events scheduled with cycle delays
- Components can introduce processing delays
- Deterministic event ordering within same time step

## Future Extensions

- Event priorities within the same time step
- Component hierarchies and communication patterns
- State snapshots for debugging/rollback
- Performance metrics and profiling
- Parallel event processing for independent components