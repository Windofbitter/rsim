# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Run
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version  
- `cargo run` - Build and run the simulator
- `cargo check` - Fast type checking without building

### Testing and Quality
- `cargo test` - Run all tests
- `cargo test [test_name]` - Run specific test
- `cargo clippy` - Run linter for code quality checks
- `cargo fmt` - Format code according to Rust standards
- `cargo fmt --check` - Check formatting without modifying

## Architecture Overview

This is an event-based discrete time simulator built in Rust. The architecture follows a clean separation between the simulation framework and concrete component implementations.

### Core Simulation Flow
1. Components register with EventManager and declare their event subscriptions
2. EventScheduler maintains a priority queue of events ordered by execution time
3. SimulationEngine orchestrates the main loop:
   - Pull all events for current time from scheduler
   - Group events by target component
   - Components process events via `react_atomic()` and generate new events
   - New events are scheduled with delays
4. Simulation continues until no events remain or max cycles reached

### Key Design Patterns
- **Event-Driven**: Components communicate exclusively through events
- **Deterministic Execution**: Priority queue + sequence numbers ensure reproducible runs
- **Batch Processing**: All events at the same simulation time are processed together
- **Static Subscriptions**: Components declare event interests at creation time

### Module Responsibilities
- `core/component.rs`: BaseComponent trait, ComponentState, Event types
- `core/event_manager.rs`: Manages component registry and event routing
- `core/event_scheduler.rs`: Priority queue for time-based event scheduling  
- `core/simulation_engine.rs`: Main simulation loop orchestration
- `components/`: Concrete component implementations (counter, processor, memory, router)

### Important Types
- `ComponentId`: String identifier for components
- `EventType`: String identifier for event types
- `ComponentValue`: Enum for event data (Int, Float, String, Bool)
- `Event`: Contains type, data, source_id, and optional target_ids

Refer to `simulator_architecture.md` and `simulator_architecture_diagrams.md` for detailed documentation and visual diagrams of the system architecture.