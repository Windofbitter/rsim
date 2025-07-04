# Event-Based Simulator Architecture Diagrams

## Class Architecture

```mermaid
classDiagram
    class BaseComponent {
        +component_id: str
        +state: dict
        +subscriptions: list
        +react_atomic(events) list
    }
    
    class Event {
        +type: str
        +data: any
        +source_id: str
        +target_ids: list
    }
    
    class EventManager {
        +components: dict
        +subscriptions: dict
        +register_component(component)
        +unregister_component(component_id)
        +get_subscribers(event_type)
        +subscribe(component_id, event_type)
        +unsubscribe(component_id, event_type)
    }
    
    class EventScheduler {
        +event_queue: PriorityQueue
        +current_time: int
        +sequence_counter: int
        +schedule_event(event, targets, delay)
        +get_next_time_events()
        +has_events()
        +peek_next_time()
    }
    
    class SimulationEngine {
        +event_manager: EventManager
        +scheduler: EventScheduler
        +max_cycles: int
        +initialize(components, events)
        +run()
        +step()
        +distribute_events(events)
    }
    
    SimulationEngine --> EventManager : uses
    SimulationEngine --> EventScheduler : uses
    EventManager --> BaseComponent : manages
    EventScheduler --> Event : schedules
    BaseComponent ..> Event : generates
```

## Component Registration Workflow

```mermaid
sequenceDiagram
    participant C as Component
    participant SE as SimulationEngine
    participant EM as EventManager
    
    C->>SE: register
    SE->>EM: register_component(component)
    EM->>EM: Store component
    EM->>EM: Process subscriptions
    loop For each subscription
        EM->>EM: Add to event_type → components mapping
    end
    EM->>SE: Registration complete
```

## Event Processing Workflow

```mermaid
sequenceDiagram
    participant SE as SimulationEngine
    participant ES as EventScheduler
    participant EM as EventManager
    participant C as Component
    
    SE->>ES: get_next_time_events()
    ES->>SE: Events at time T
    SE->>EM: get_subscribers(event_type)
    EM->>SE: Component IDs
    SE->>SE: Group events by component
    
    loop For each component
        SE->>C: react_atomic(event_list)
        C->>C: Process events
        C->>C: Update state
        C->>SE: Return [(event, delay)]
        
        loop For each new event
            SE->>ES: schedule_event(event, targets, delay)
        end
    end
    
    SE->>ES: Advance time
```

## Time-Based Event Flow

```mermaid
graph TB
    subgraph "Time = 0"
        E1[Event A] --> C1[Component 1]
        E2[Event B] --> C2[Component 2]
    end
    
    subgraph "Time = 5"
        C1 -.->|generates| E3[Event C]
        E3 --> C3[Component 3]
    end
    
    subgraph "Time = 10"
        C2 -.->|generates| E4[Event D]
        C3 -.->|generates| E5[Event E]
        E4 --> C1
        E5 --> C2
    end
    
    style E1 fill:#f9f,stroke:#333
    style E2 fill:#f9f,stroke:#333
    style E3 fill:#9ff,stroke:#333
    style E4 fill:#ff9,stroke:#333
    style E5 fill:#ff9,stroke:#333
```

## Priority Queue Structure

```mermaid
graph TD
    subgraph "Event Queue (Min-Heap)"
        PQ[Priority Queue]
        PQ --> T5["(5, 0, Event A, [C1])"]
        PQ --> T5b["(5, 1, Event B, [C2])"]
        PQ --> T10["(10, 2, Event C, [C1,C3])"]
        PQ --> T15["(15, 3, Event D, [C2])"]
    end
    
    subgraph "Legend"
        L["(time, seq_num, event, targets)"]
    end
```

## Example: Ping-Pong Workflow

```mermaid
sequenceDiagram
    participant Client as Client Component
    participant Server as Server Component
    participant ES as EventScheduler
    
    Note over Client: Subscribed to "pong"
    Note over Server: Subscribed to "ping"
    
    Client->>ES: Schedule "ping" event
    ES->>Server: Deliver "ping" at T=0
    Server->>Server: react_atomic(["ping"])
    Server->>ES: Schedule "pong" at T=5
    
    Note over ES: Time advances to T=5
    
    ES->>Client: Deliver "pong" at T=5
    Client->>Client: react_atomic(["pong"])
    Client->>ES: Schedule "ping" at T=10
    
    Note over Client,Server: Cycle continues...
```

## Component State Machine

```mermaid
stateDiagram-v2
    [*] --> Created: new Component()
    Created --> Registered: register_component()
    Registered --> Active: receive events
    
    Active --> Processing: react_atomic()
    Processing --> Active: return events
    
    Active --> Unregistered: unregister_component()
    Unregistered --> [*]
    
    note right of Processing
        1. Update internal state
        2. Generate new events
        3. Return with delays
    end note
```

## System Overview

```mermaid
graph LR
    subgraph "SimulationEngine"
        SE[Engine Loop]
    end
    
    subgraph "EventManager"
        REG[Component Registry]
        SUB[Subscription Map]
    end
    
    subgraph "EventScheduler"
        PQ[Priority Queue]
        TIME[Current Time]
    end
    
    subgraph "Components"
        C1[Component 1]
        C2[Component 2]
        C3[Component 3]
    end
    
    SE --> REG
    SE --> PQ
    REG --> C1
    REG --> C2
    REG --> C3
    SUB --> C1
    SUB --> C2
    SUB --> C3
    PQ --> SE
    
    C1 -.->|events| PQ
    C2 -.->|events| PQ
    C3 -.->|events| PQ
```