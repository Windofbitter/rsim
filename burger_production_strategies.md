# Burger Production Strategies Implementation Plan

## Current Production Approach

### Buffer-Based Production (Current System)
The existing system implements **buffer-based production** where components produce continuously whenever there is available space in downstream buffers:

- **Fryer/Baker**: Produce continuously, stop only when downstream buffer is full
- **Backpressure Control**: `BUFFER_FULL_EVENT` stops production, `BUFFER_SPACE_AVAILABLE_EVENT` resumes
- **Order Fulfillment**: Orders are served from existing buffer inventory

This is essentially a "produce when buffer has space" strategy.

---

## New Strategy: Order-Based Production

### Concept
Implement **demand-driven production** where components only produce when there are actual customer orders, creating a just-in-time manufacturing system.

### Key Differences from Current System

| Current (Buffer-Based) | New (Order-Based) |
|----------------------|-------------------|
| Produce when buffer has space | Produce only when orders arrive |
| Continuous production loops | Idle until demand signal |
| Orders served from inventory | Orders trigger production |
| High inventory, fast fulfillment | Low inventory, variable fulfillment |

### Implementation Approach

#### 1. Configuration
```rust
#[derive(Debug, Clone)]
pub enum ProductionStrategy {
    BufferBased,    // Current system
    OrderBased,     // New demand-driven system
}

pub struct BurgerSimulationConfig {
    // Existing fields...
    pub production_strategy: ProductionStrategy,
    
    // Order-based specific settings
    pub order_batch_size: Option<u32>,  // Batch small orders together
    pub max_order_wait_time: Option<u64>, // SLA for order fulfillment
}
```

#### 2. New Events
```rust
pub const PRODUCTION_REQUEST_EVENT: &str = "production_request";
pub const ORDER_QUEUED_EVENT: &str = "order_queued";

pub struct ProductionRequestEvent {
    pub item_type: String,        // "burger", "meat", "bread"
    pub quantity: u32,
    pub order_id: String,
    pub deadline: Option<u64>,    // When order needs to be ready
}
```

#### 3. Component Changes

**Client Component**:
- Orders immediately trigger production requests instead of just checking inventory
- Track order fulfillment timing and SLA

**Assembly Buffer**:
- When receiving orders, check inventory first
- If insufficient inventory, send production requests upstream
- Queue orders until ingredients are available

**Fryer/Baker Components**:
- Wait in IDLE state instead of continuous production
- Respond to `PRODUCTION_REQUEST_EVENT` by starting production
- Produce exact quantities requested

#### 4. Production Flow
```
Order Placed → Check Assembly Buffer → If insufficient inventory:
  └─→ Request Burgers → Check Ingredient Buffers → If insufficient:
      └─→ Request Meat/Bread → Fryer/Baker produce → Assembly → Fulfill Order
```

---

## Configuration Implementation Strategy

### Recommended Approach: Strategy Enum with Event Subscription

**Why This Works Best**:
- Minimal code changes to existing components
- Leverages existing event-driven architecture
- Easy A/B testing with simple config parameter switch
- Maintains backward compatibility

### Configuration Structure
```rust
#[derive(Debug, Clone)]
pub enum ProductionStrategy {
    BufferBased,    // Current system (default)
    OrderBased,     // New demand-driven system
}

pub struct BurgerSimulationConfig {
    // Existing fields unchanged for backward compatibility...
    pub production_strategy: ProductionStrategy,
    
    // Optional strategy-specific parameters
    pub order_batch_size: Option<u32>,        // For order-based: batch small orders
    pub max_order_wait_time: Option<u64>,     // For order-based: SLA limit
}

impl Default for BurgerSimulationConfig {
    fn default() -> Self {
        Self {
            // Existing defaults...
            production_strategy: ProductionStrategy::BufferBased, // Backward compatible
            order_batch_size: None,
            max_order_wait_time: None,
        }
    }
}
```

### Component Implementation Pattern
```rust
impl BaseComponent for Fryer {
    fn subscriptions(&self) -> &[&str] {
        match self.production_strategy {
            BufferBased => &["start_frying", "buffer_full", "buffer_space_available"],
            OrderBased => &["production_request", "buffer_full"], // No continuous production
        }
    }

    fn react_atomic(&mut self, events: Vec<Event>) -> Vec<Event> {
        for event in events {
            match (event.event_type(), &self.production_strategy) {
                ("start_frying", BufferBased) => {
                    // Current continuous production logic
                }
                ("production_request", OrderBased) => {
                    // New demand-driven production logic
                }
                ("buffer_full", _) => {
                    // Common backpressure handling for both strategies
                }
                _ => {} // Ignore events not relevant to current strategy
            }
        }
    }
}
```

### Testing Configuration
```rust
// Easy A/B testing with identical parameters
fn create_test_configs() -> (BurgerSimulationConfig, BurgerSimulationConfig) {
    let base_config = BurgerSimulationConfig {
        frying_delay: 10,
        baking_delay: 8,
        // ... identical timing and capacity parameters
        random_seed: 42, // Same seed for fair comparison
        production_strategy: ProductionStrategy::BufferBased,
    };
    
    let order_based_config = BurgerSimulationConfig {
        production_strategy: ProductionStrategy::OrderBased,
        order_batch_size: Some(2),
        max_order_wait_time: Some(50),
        ..base_config.clone()
    };
    
    (base_config, order_based_config)
}
```

---

## Implementation Steps

### Phase 1: Configuration System
1. Add `ProductionStrategy` enum to existing config struct
2. Update component constructors to accept strategy parameter
3. Maintain full backward compatibility with current system

### Phase 2: Order-Based Events
1. Add `PRODUCTION_REQUEST_EVENT` and `ORDER_QUEUED_EVENT` constants
2. Create event structs for production requests
3. No changes to event manager - uses existing routing

### Phase 3: Component Modifications
1. **Assembly Buffer**: Add order queuing and upstream request logic when OrderBased
2. **Fryer/Baker**: Add conditional event subscriptions and demand-triggered production
3. **Client**: Add order-to-production-request mapping for OrderBased strategy

### Phase 4: Testing & Optimization
1. Run identical simulations with different strategies for direct comparison
2. Add metrics for inventory levels, fulfillment times, resource utilization
3. Optimize order batching and production timing parameters

---

## Expected Benefits

### Buffer-Based (Current)
- **Fast Response**: Orders fulfilled immediately from inventory
- **Predictable**: Consistent production and inventory levels
- **Simple**: Straightforward backpressure control

### Order-Based (New)
- **Efficient**: No overproduction or excess inventory
- **Responsive**: Production matches actual demand patterns
- **Scalable**: Naturally handles varying demand without waste

This implementation will allow direct comparison of push vs pull manufacturing strategies within the same simulation framework.