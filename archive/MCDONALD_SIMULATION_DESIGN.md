# McDonald's Simulation Design

## Overview
A multi-worker fast food production line simulation using RSim's component-based architecture with dedicated memory buffers and backpressure control.

## Architecture

### Production Pipeline
```
[Bakers] → [Bread Buffers] → [Bread Manager] ──→ [Assembler Manager] ──→ [Assembler Buffers] → [Assemblers] → [Burger Buffer] → [Consumers]
[Fryers] → [Meat Buffers]  → [Meat Manager]  ──→                   ↗
```

### Component Types

#### 1. **Producer Components** (Processing Components)
- **Bakers** (10 instances): Produce bread with random timing
- **Fryers** (10 instances): Produce meat with random timing
- **Ports**: 1 memory port to dedicated buffer
- **Logic**: Check buffer status → produce if not full → write to buffer

#### 2. **Individual Buffer Components** (Memory Components)
- **BreadBuffer1-10**: One per baker (FIFO queues)
- **MeatBuffer1-10**: One per fryer (FIFO queues)
- **Ports**: 1 input, 1 output (RSim constraint)
- **Data**: Queue of items + buffer status

#### 3. **Manager Components** (Processing Components)
- **BreadManager**: Collects from all bread buffers
- **MeatManager**: Collects from all meat buffers
- **AssemblerManager**: Coordinates ingredient distribution to assemblers
- **Ports**: 
  - BreadManager: 10 memory ports (read from bread buffers) + 1 memory port (write to assembler manager)
  - MeatManager: 10 memory ports (read from meat buffers) + 1 memory port (write to assembler manager)
  - AssemblerManager: 2 memory ports (read bread/meat) + 10 memory ports (write to assembler buffers)
- **Logic**: Collect ingredients → distribute to available assemblers

#### 4. **Assembler Components** (Processing Components)
- **Assemblers** (10 instances): Combine bread + meat → burger
- **Ports**: 1 memory port (read from dedicated assembler buffer) + 1 memory port (write burger)
- **Logic**: Read ingredient pairs from buffer → random assembly time → produce burger

#### 5. **Assembler Buffer Components** (Memory Components)
- **AssemblerBuffer1-10**: One per assembler (stores bread+meat pairs)
- **Ports**: 1 input, 1 output (RSim constraint)
- **Data**: Queue of {bread, meat} ingredient pairs

#### 6. **Consumer Components** (Processing Components)
- **Consumers** (10 instances): Consume burgers
- **Ports**: 1 memory port (read from burger buffer)
- **Logic**: Random consumption time → remove burger

## Memory Architecture

### 1-to-1 Memory Connections
```
Baker1 ─memory_port─→ BreadBuffer1 ─┐                                    ┌─→ AssemblerBuffer1 ─→ Assembler1
Baker2 ─memory_port─→ BreadBuffer2 ─┼─→ BreadManager ─memory_port─→ ┐    │
Baker3 ─memory_port─→ BreadBuffer3 ─┤                              │    ├─→ AssemblerBuffer2 ─→ Assembler2
Baker4 ─memory_port─→ BreadBuffer4 ─┤                              ├──→ AssemblerManager ─┤
Baker5 ─memory_port─→ BreadBuffer5 ─┘                              │    ├─→ AssemblerBuffer3 ─→ Assembler3
                                                                   │    │
Fryer1 ─memory_port─→ MeatBuffer1 ──┐                              │    └─→ AssemblerBuffer10 ─→ Assembler10
Fryer2 ─memory_port─→ MeatBuffer2 ──┼─→ MeatManager ─memory_port──→ ┘
Fryer3 ─memory_port─→ MeatBuffer3 ──┤
Fryer4 ─memory_port─→ MeatBuffer4 ──┤
Fryer5 ─memory_port─→ MeatBuffer5 ──┘
```

### Buffer Data Structure
```rust
#[derive(Clone)]
struct BufferData {
    items: VecDeque<String>,
    capacity: usize,
    is_full: bool,
    total_produced: u64,
    total_consumed: u64,
}
```

## Backpressure Control

### Individual Buffer Status
- Each producer checks its dedicated buffer before producing
- No production when buffer is full
- Producers resume when buffer has space

### Manager Ingredient Matching
- **BreadManager/MeatManager**: If no ingredients available, skip cycle (no output)
- **AssemblerManager**: Only outputs to assembler buffers when both bread and meat available
- **Assemblers**: Check both ingredient availability and output buffer capacity before processing

### Memory Read/Write Pattern
- **Read**: From snapshot (previous cycle state)
- **Write**: To current state (affects next cycle)
- **No cycles**: All memory connections are acyclic

## Random Processing Times

### Implementation
- Each component maintains internal timer state
- Random delay ranges configurable per component type
- Components skip processing during delay periods

```rust
struct ComponentTimer {
    current_delay: u32,
    remaining_cycles: u32,
    min_delay: u32,
    max_delay: u32,
    rng_seed: u64,  // For deterministic randomness
}
```

### Timing Ranges
- **Bakers**: 1-3 cycles per bread
- **Fryers**: 2-4 cycles per meat
- **Assemblers**: 1-2 cycles per burger
- **Consumers**: 1-5 cycles per burger

### Component Processing Logic
- Check if timer is active (remaining_cycles > 0)
- If timer active: decrement and skip processing
- If timer expired: perform operation and reset timer with new random delay

## Key Features

1. **Scalable Workers**: Easy to add/remove workers by changing instance count
2. **Individual Buffers**: Each worker has dedicated buffer (no contention)
3. **Backpressure**: Natural flow control through buffer status
4. **No Cycles**: Acyclic memory connections prevent deadlocks
5. **Deterministic**: Reproducible simulation results
6. **Configurable**: Buffer sizes, timing ranges, worker counts

## Component Count Summary
- **Bakers**: 10 processing components
- **Fryers**: 10 processing components
- **Assemblers**: 10 processing components
- **Consumers**: 10 processing components
- **Individual Buffers**: 20 memory components (10 bread + 10 meat)
- **Assembler Buffers**: 10 memory components (ingredient pairs)
- **Shared Buffers**: 1 memory component (burger buffer)
- **Managers**: 3 processing components (bread + meat + assembler managers)
- **Total**: 43 processing components + 31 memory components

## Connection Pattern
- **Memory Connections**: 74 total (1-to-1 dedicated channels)
  - Baker→BreadBuffer: 10 connections
  - BreadBuffer→BreadManager: 10 connections  
  - Fryer→MeatBuffer: 10 connections
  - MeatBuffer→MeatManager: 10 connections
  - BreadManager→AssemblerManager: 1 connection
  - MeatManager→AssemblerManager: 1 connection
  - AssemblerManager→AssemblerBuffer: 10 connections
  - AssemblerBuffer→Assembler: 10 connections
  - Assembler→BurgerBuffer: 10 connections
  - BurgerBuffer→Consumer: 10 connections
- **No Direct Component Connections**: All data flows through memory
- **Acyclic Graph**: Ensures deterministic execution order

This design provides realistic McDonald's production simulation with proper backpressure, scalability, and adherence to RSim's architectural constraints.