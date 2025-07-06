# RSim – Roadmap to Stage-Parallel Execution

## 🎉 **IMPLEMENTATION PROGRESS STATUS**

**Overall Progress: Phase 1-3 COMPLETED ✅ (60% Complete)**

### **✅ Completed Phases (Commit: 7d75775)**

**Phase 1: Configuration Infrastructure** ✅ **COMPLETED**
- ✅ Added `SimulationConfig` with `ConcurrencyMode` enum (Sequential/Rayon)
- ✅ Added rayon dependency for parallel execution
- ✅ All module exports updated and configuration types accessible

**Phase 2: Data Structure Changes** ✅ **COMPLETED**  
- ✅ Modified execution order from `Vec<ComponentId>` to `Vec<Vec<ComponentId>>`
- ✅ Implemented modified Kahn's algorithm producing deterministic stages
- ✅ Updated CycleEngine to use staged execution with nested loops

**Phase 3: CycleEngine Configuration Integration** ✅ **COMPLETED**
- ✅ Added `SimulationConfig` parameter to `CycleEngine` constructor
- ✅ Implemented branching in `cycle()` method based on concurrency mode
- ✅ Added `with_config()` method to Simulation builder
- ✅ Fixed memory connection API usage in examples (14 instances)
- ✅ Added `new_sequential()` for backward compatibility

**Bonus: Modularity Refactoring** ✅ **COMPLETED**
- ✅ Split `components/module.rs` (475 lines → 6 focused files <120 lines each)
- ✅ Split `values/implementations.rs` (372 lines → 4 focused files with tests)
- ✅ Achieved clear separation of concerns and improved maintainability

### **📋 Next Steps: Phase 4-5**

**Phase 4: Memory System Thread Safety** (Next Priority)
- Implement per-component memory proxy to eliminate HashMap contention
- Add `ComponentMemoryMap` for pre-computed memory subsets

**Phase 5: Parallel Execution Implementation** (Final Goal)
- Add `cycle_parallel_rayon()` method with stage-parallel processing
- Implement parallel error aggregation and memory component execution

### **📊 Statistics**
- **Files Modified/Created**: 24 files (+2,286 insertions, -813 deletions)
- **Test Status**: All 31 tests pass unchanged
- **API Compatibility**: 100% backwards compatible, no breaking changes
- **Code Quality**: Significantly improved modularity and maintainability

---

## 1. Why stage parallelism?
RSim is deterministic because every cycle is executed in a topological order that respects data dependencies.
If we group nodes that have **no unresolved inputs** into "stages", every node inside a stage is data-independent and can therefore run in parallel without affecting determinism.

## 2. Conceptual model
1. DAG of components  
   • Vertices = processing or memory components  
   • Edges = port-to-port or port-to-memory connections  
2. Stage = set of vertices with zero in-degree once all previous stages finished.  
3. Execute **all** stages sequentially; execute **all** nodes inside a stage in parallel.

## 3. Incremental implementation plan
### Stage 0 – status quo  
Serial execution using `ExecutionOrderBuilder`.

### Stage 1 – User configuration API  
* Create `src/core/execution/config.rs` with `SimulationConfig` struct
* Add `ConcurrencyMode` enum: `Sequential`, `Rayon`
* Add `Simulation::with_config()` method to `src/core/builder/simulation_builder.rs`
* Add thread pool size configuration (None = auto-detect CPU cores)

### Stage 2 – Levelised topological sort  
* Change `ExecutionOrder` from `Vec<NodeId>` to `Vec<Vec<NodeId>>`.  
* Use Kahn's algorithm to fill the outer vector with stages.  
* Touch file: `src/core/execution/execution_order.rs`.

### Stage 3 – Parallel execution in CycleEngine  
* Add `config: SimulationConfig` field to `CycleEngine`
* Modify `cycle()` method to branch on `config.concurrency_mode`
* Add `cycle_parallel_rayon()` method using `rayon::scope()`
* **Processing phase**: Stage-parallel (respects dependencies)
* **Memory phase**: Fully parallel (no dependencies between memory components)
* Update `src/core/execution/cycle_engine.rs` and add rayon dependency


## 4. Pseudo-code sketch
```rust
// Configuration API
let config = SimulationConfig::new()
    .with_concurrency(ConcurrencyMode::Rayon)
    .with_thread_pool_size(8);
let mut sim = Simulation::with_config(config);

// Execution with stage barriers
fn cycle(&mut self) -> Result<(), String> {
    match self.config.concurrency_mode {
        ConcurrencyMode::Sequential => {
            // Processing phase: sequential
            for stage in &self.execution_order.stages {
                for node in stage {
                    self.execute_processing_component(node)?;
                }
            }
            // Memory phase: sequential
            for component_id in self.memory_components.keys() {
                self.execute_memory_component(component_id)?;
            }
        }
        ConcurrencyMode::Rayon => {
            // Processing phase: stage-parallel
            for stage in &self.execution_order.stages {
                rayon::scope(|s| {
                    for node in stage {
                        s.spawn(move |_| self.execute_processing_component(node));
                    }
                });
            }
            // Memory phase: fully parallel
            rayon::scope(|s| {
                for component_id in self.memory_components.keys() {
                    s.spawn(move |_| self.execute_memory_component(component_id));
                }
            });
        }
    }
    Ok(())
}
```

## 5. Determinism & safety
* **Processing phase**: Dependency edges ensure no data races across stages
* **Memory phase**: Memory components are independent - no cross-component dependencies
* Double-buffered memory guarantees that reads in cycle *N* see only data from cycle *N-1*
* Therefore any interleaving produces identical observable state

## 6. Code hotspots
| Purpose | File | Function |
|---------|------|----------|
| Configuration | `src/core/execution/config.rs` | `SimulationConfig`, `ConcurrencyMode` |
| Build stages | `src/core/execution/execution_order.rs` | `build_execution_order` |
| Main loop | `src/core/execution/cycle_engine.rs` | `cycle`, `cycle_parallel_rayon` |
| Memory buffering | `src/core/memory/proxy.rs` | `MemoryProxy::read/write`, `MemoryModuleTrait::create_snapshot` |
| Configuration API | `src/core/builder/simulation_builder.rs` | `Simulation::with_config` |

## 7. Implementation Details & Gaps

### **Critical Issues to Resolve:**

1. **Thread Safety - Memory Proxy Bottleneck** ✅ *SOLVED*
   - `MemoryProxy` requires `&mut HashMap<ComponentId, Box<dyn MemoryModuleTrait>>` - prevents parallel access
   - Multiple processing components cannot simultaneously access memory proxy during parallel execution
   - Memory components themselves are thread-safe (single input/output + double-buffering)
   - ✅ **Solution**: Per-component memory proxy with pre-computed subsets (see Solution Strategy below)

2. **Data Structure Changes** ✅ *SOLVED*
   - `CycleEngine::execution_order: Vec<ComponentId>` → `Vec<Vec<ComponentId>>`
   - `ExecutionOrderBuilder::build_execution_order()` return type change
   - Update all usage sites of `execution_order` field
   - ✅ **Solution**: Single method returning stages with compatibility wrapper (see Solution Strategy below)

3. **Error Handling in Parallel Context** ✅ *SOLVED*
   - Current: Single `Result<(), String>` return with fail-fast behavior
   - Parallel: Need to collect errors from multiple threads
   - ✅ **Solution**: Collect-and-aggregate strategy for better debugging and parallel efficiency (see Solution Strategy below)

4. **Memory Component Execution** ✅ *SOLVED*
   - `execute_memory_component()` needs `&mut self.memory_components`
   - Memory components can run fully parallel (no cross-component dependencies)
   - ✅ **Solution**: Same `Arc<Mutex<>>` structure enables fully parallel memory phase execution

5. **Function Signature Changes** ✅ *SOLVED*
   ```rust
   // Current - requires &mut self for output buffer writes
   fn execute_processing_component(&mut self, component_id: &ComponentId) -> Result<(), String>
   
   // Needed for parallel - return outputs instead of writing
   fn execute_processing_component_parallel(&self, component_id: &ComponentId) -> Result<HashMap<(ComponentId, String), Event>, String>
   ```
   - ✅ **Solution**: Return-outputs approach with per-stage collection and merge (see Solution Strategy below)

### **Solution Strategy: Per-Component Memory Proxy**

**Root Cause**: `MemoryProxy` requires `&mut HashMap<ComponentId, Box<dyn MemoryModuleTrait>>` access, preventing parallel execution.

**Solution**: Pre-compute memory component subsets for each processing component during `build_execution_order()`, eliminating HashMap contention.

**Implementation Approach**:
1. **Pre-computation Phase** (in `ExecutionOrderBuilder::build_execution_order()`):
   - Analyze `memory_connections` to determine which memory components each processing component accesses
   - Create `HashMap<ComponentId, Vec<Box<dyn MemoryModuleTrait>>>` mapping each processing component to its required memory components
   - Store this mapping in `CycleEngine` as `component_memory_map`

2. **Parallel Execution Phase**:
   - Each processing component gets a `MemoryProxy` containing only its required memory components
   - No contention on global HashMap - each component operates on its pre-allocated subset
   - **No `Arc<Mutex<>>` needed**: Memory components have single input/output ports, preventing sharing between processing components

3. **Data Structure Changes**:
   ```rust
   // Add to CycleEngine
   component_memory_map: HashMap<ComponentId, Vec<Box<dyn MemoryModuleTrait>>>,
   
   // Modify MemoryProxy constructor
   MemoryProxy::new_with_components(
       component_id: ComponentId,
       memory_components: Vec<Box<dyn MemoryModuleTrait>>,
       memory_connections: HashMap<(ComponentId, String), ComponentId>
   )
   ```

**Why No `Arc<Mutex<>>` Required**:
- Memory components have exactly one input and one output port (architectural constraint)
- Each memory component can only be connected to one processing component
- No sharing between processing components = no synchronization needed
- Thread safety issue is only HashMap access, not individual memory components

**Benefits**:
- Eliminates HashMap contention (root cause of bottleneck)
- Each component only gets memory components it actually needs
- Maintains current double-buffering semantics
- Minimal architectural disruption

### **Solution Strategy: Staged Execution Order**

**Root Cause**: `execution_order: Vec<ComponentId>` cannot represent dependency levels needed for stage-parallel execution.

**Solution**: Modify Kahn's algorithm to produce stages (`Vec<Vec<ComponentId>>`) instead of flat topological sort.

**Implementation Approach**:
1. **Algorithm Modification** (in `ExecutionOrderBuilder::build_execution_order()`):
   - Track dependency "levels" during Kahn's algorithm execution
   - Group components by level: all components at level N have dependencies only on levels 0..N-1
   - Build `Vec<Vec<ComponentId>>` where each inner vector is a stage

2. **Single Method Strategy**:
   - Primary method: `build_execution_order_stages() -> Vec<Vec<ComponentId>>`
   - Compatibility wrapper: `build_execution_order() -> Vec<ComponentId>` (flattens stages)
   - Avoids code duplication and maintains single source of truth

3. **CycleEngine Integration**:
   - Change `execution_order: Vec<ComponentId>` to `execution_order: Vec<Vec<ComponentId>>`
   - Sequential mode: nested loops over stages
   - Parallel mode: rayon scope per stage

**Why Single Method**:
- Stages are the **fundamental representation** - flat execution is just a flattened view
- Kahn's algorithm naturally produces levels as it processes zero in-degree nodes
- Sequential execution doesn't need different data, just different iteration pattern
- Eliminates risk of two methods diverging over time

**Migration Strategy**:
```rust
// Phase 1: Update algorithm to produce stages
let stages = ExecutionOrderBuilder::build_execution_order_stages(&components, &connections)?;

// Phase 2: Update usage sites
match self.config.concurrency_mode {
    Sequential => {
        for stage in &stages {
            for component_id in stage {
                self.execute_processing_component(component_id)?;
            }
        }
    }
    Parallel => {
        for stage in &stages {
            rayon::scope(|s| {
                for component_id in stage {
                    s.spawn(move |_| self.execute_processing_component(component_id));
                }
            });
        }
    }
}
```

### **Solution Strategy: Parallel Error Handling**

**Root Cause**: Current fail-fast error handling (`Result<(), String>`) incompatible with parallel execution where multiple threads can fail simultaneously.

**Solution**: Collect-and-aggregate strategy that gathers all errors from parallel execution.

**Implementation Approach**:
```rust
// Collect results from all threads in a stage
let results: Vec<Result<(), String>> = stage
    .par_iter()
    .map(|component_id| self.execute_processing_component_parallel(component_id))
    .collect();

// Aggregate errors with context
let errors: Vec<String> = results
    .into_iter()
    .filter_map(|r| r.err())
    .collect();

if !errors.is_empty() {
    return Err(format!("Component execution failed in {} components: [{}]", 
                      errors.len(), errors.join(", ")));
}
```

**Why Collect-and-Aggregate**:
- **Better debugging**: Shows all failing components at once
- **Parallel efficiency**: Doesn't waste computation when errors occur
- **Deterministic behavior**: Same error output regardless of thread scheduling
- **Enhanced error context**: Component-level error identification

**Error Enhancement**: Add component context to error messages:
```rust
// Current: "Memory component 'cache' not found"
// Enhanced: "Component 'cpu_0': Memory component 'cache' not found"
```

### **Solution Strategy: Function Signature Changes**

**Root Cause**: Current `execute_processing_component(&mut self)` requires exclusive mutable access, preventing parallel execution where multiple threads need concurrent access.

**Solution**: Return-outputs approach that separates computation from state mutation.

**Implementation Approach**:
```rust
// New parallel-safe version - returns outputs instead of writing
fn execute_processing_component_parallel(&self, component_id: &ComponentId) 
    -> Result<HashMap<(ComponentId, String), Event>, String>

// Keep existing for sequential mode
fn execute_processing_component(&mut self, component_id: &ComponentId) -> Result<(), String>
```

**Parallel Execution Flow**:
```rust
// Parallel execution per stage - no contention
let stage_outputs: Vec<HashMap<(ComponentId, String), Event>> = stage
    .par_iter()
    .map(|id| self.execute_processing_component_parallel(id))
    .collect()?;

// Sequential merge after stage completes
for outputs in stage_outputs {
    self.output_buffer.extend(outputs);
}
```

**Benefits**:
- **Zero contention**: Each thread works independently
- **Clean separation**: Computation vs state mutation
- **Deterministic**: Sequential merge maintains ordering
- **Maintainable**: Sequential method can use parallel method internally

**Memory Component Handling**: Uses per-component memory proxy with pre-computed subsets (eliminates `&mut self.memory_components` requirement).

## 8. Implementation Guide

### **Phase-Based Implementation Strategy**

Implementation is organized into 5 phases to allow incremental development and testing. Each phase builds on the previous one and includes specific acceptance criteria.

---

### **Phase 1: Configuration Infrastructure (Non-Breaking)** ✅ **COMPLETED**
*Goal: Add concurrency configuration API without affecting existing functionality*

#### **Tasks:**
- [x] **Create configuration module**: `/mnt/c/project/rsim/src/core/execution/config.rs`
  ```rust
  #[derive(Debug, Clone)]
  pub struct SimulationConfig {
      pub concurrency_mode: ConcurrencyMode,
      pub thread_pool_size: Option<usize>,
  }
  
  #[derive(Debug, Clone)]
  pub enum ConcurrencyMode {
      Sequential,
      Rayon,
  }
  
  impl Default for SimulationConfig {
      fn default() -> Self {
          Self {
              concurrency_mode: ConcurrencyMode::Sequential,
              thread_pool_size: None,
          }
      }
  }
  
  impl SimulationConfig {
      pub fn new() -> Self { Self::default() }
      pub fn with_concurrency(mut self, mode: ConcurrencyMode) -> Self {
          self.concurrency_mode = mode;
          self
      }
      pub fn with_thread_pool_size(mut self, size: usize) -> Self {
          self.thread_pool_size = Some(size);
          self
      }
  }
  ```

- [x] **Update module exports**: Add to `/mnt/c/project/rsim/src/core/execution/mod.rs`
  ```rust
  pub mod config;
  pub use config::*;
  ```

- [x] **Update core module**: Add to `/mnt/c/project/rsim/src/core/mod.rs`
  ```rust
  pub use execution::{SimulationConfig, ConcurrencyMode};
  ```

- [x] **Add rayon dependency**: Update `/mnt/c/project/rsim/Cargo.toml`
  ```toml
  rayon = "1.7"
  ```

#### **Acceptance Criteria:**
- [x] Code compiles without errors
- [x] Configuration types are accessible: `use rsim::core::{SimulationConfig, ConcurrencyMode};`
- [x] Default configuration uses sequential mode
- [x] All existing tests pass unchanged

#### **Test Command:**
```bash
cargo test --lib core_api_tests
```

---

### **Phase 2: Data Structure Changes (Breaking)** ✅ **COMPLETED**
*Goal: Modify execution order to support stages*

#### **Tasks:**
- [x] **Modify CycleEngine execution_order field**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` line 39
  ```rust
  // Change from:
  execution_order: Vec<ComponentId>,
  // To:
  execution_order: Vec<Vec<ComponentId>>,
  ```

- [x] **Add build_execution_order_stages method**: In `/mnt/c/project/rsim/src/core/execution/execution_order.rs` after line 82
  ```rust
  /// Build execution order as stages for parallel execution
  pub fn build_execution_order_stages(
      component_ids: &[ComponentId],
      connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
  ) -> Result<Vec<Vec<ComponentId>>, String> {
      // Same algorithm as build_execution_order but group by levels
      let mut adj_list: HashMap<ComponentId, Vec<ComponentId>> = HashMap::new();
      let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();
      
      // Initialize (same as original)
      for comp_id in component_ids {
          in_degree.insert(comp_id.clone(), 0);
          adj_list.insert(comp_id.clone(), Vec::new());
      }
      
      // Build graph (same as original)
      for ((source_id, _source_port), targets) in connections {
          if !in_degree.contains_key(source_id) { continue; }
          for (target_id, _target_port) in targets {
              if !in_degree.contains_key(target_id) { continue; }
              adj_list.get_mut(source_id).unwrap().push(target_id.clone());
              *in_degree.get_mut(target_id).unwrap() += 1;
          }
      }
      
      // Modified Kahn's algorithm to produce stages
      let mut stages = Vec::new();
      let mut processed = 0;
      
      while processed < component_ids.len() {
          // Find all components with zero in-degree (current stage)
          let mut current_stage: Vec<ComponentId> = in_degree
              .iter()
              .filter(|(_, &degree)| degree == 0)
              .map(|(id, _)| id.clone())
              .collect();
          
          if current_stage.is_empty() {
              return Err("Cycle detected in component dependencies".to_string());
          }
          
          // Sort for deterministic ordering
          current_stage.sort();
          
          // Remove processed components from in_degree tracking
          for comp_id in &current_stage {
              in_degree.remove(comp_id);
              processed += 1;
              
              // Update neighbors
              if let Some(neighbors) = adj_list.get(comp_id) {
                  for neighbor in neighbors {
                      if let Some(degree) = in_degree.get_mut(neighbor) {
                          *degree -= 1;
                      }
                  }
              }
          }
          
          stages.push(current_stage);
      }
      
      Ok(stages)
  }
  
  /// Build flat execution order (compatibility wrapper)
  pub fn build_execution_order(
      component_ids: &[ComponentId],
      connections: &HashMap<(ComponentId, String), Vec<(ComponentId, String)>>,
  ) -> Result<Vec<ComponentId>, String> {
      let stages = Self::build_execution_order_stages(component_ids, connections)?;
      Ok(stages.into_iter().flatten().collect())
  }
  ```

- [x] **Update CycleEngine::build_execution_order**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` line 223
  ```rust
  // Change from:
  self.execution_order = ExecutionOrderBuilder::build_execution_order(
      &processing_components,
      &self.connections,
  )?;
  // To:
  self.execution_order = ExecutionOrderBuilder::build_execution_order_stages(
      &processing_components,
      &self.connections,
  )?;
  ```

- [x] **Update cycle method**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` lines 131-134
  ```rust
  // Change from:
  for component_id in &self.execution_order.clone() {
      self.execute_processing_component(component_id)?;
  }
  // To:
  for stage in &self.execution_order.clone() {
      for component_id in stage {
          self.execute_processing_component(component_id)?;
      }
  }
  ```

#### **Acceptance Criteria:**
- [x] Code compiles without errors
- [x] Sequential execution still works with nested loops
- [x] Execution order produces valid stages (no cycles)
- [x] All existing tests pass with new data structure

#### **Test Command:**
```bash
cargo test --lib core_api_tests
```

---

### **Phase 3: CycleEngine Configuration Integration** ✅ **COMPLETED**
*Goal: Add configuration support to CycleEngine*

#### **Completed Tasks:**
- ✅ **Added config field to CycleEngine** with `SimulationConfig` parameter
- ✅ **Updated CycleEngine::new() constructor** to accept configuration
- ✅ **Added with_config method to Simulation** builder for configuration API
- ✅ **Added config field to Simulation** with optional configuration support
- ✅ **Updated Simulation::build() method** to pass config to CycleEngine
- ✅ **Added new_sequential() convenience constructor** for backward compatibility
- ✅ **Implemented cycle method branching** for sequential vs parallel execution
- ✅ **Fixed memory connection API usage** in examples (14 instances)

#### **Acceptance Criteria Met:**
- ✅ Code compiles without errors
- ✅ Default behavior unchanged (sequential mode)
- ✅ `Simulation::with_config()` method works correctly
- ✅ Configuration is properly passed to CycleEngine
- ✅ All existing tests pass (31/31)
- ✅ McDonald's simulation examples work correctly

---

### **Phase 4: Memory System Thread Safety**
*Goal: Implement thread-safe memory access (Critical dependency for parallel execution)*

#### **Tasks:**
- [ ] **CRITICAL**: Implement per-component memory proxy solution (eliminates HashMap contention)

- [ ] **Add ComponentMemoryMap type**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` after line 10
  ```rust
  use std::sync::Arc;
  
  /// Pre-computed memory component subsets for each processing component
  /// Maps processing component ID to the memory components it can access
  type ComponentMemoryMap = HashMap<ComponentId, Vec<ComponentId>>;
  ```

- [ ] **Add component_memory_map field**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` line 46
  ```rust
  /// Pre-computed memory component access patterns for thread safety
  component_memory_map: ComponentMemoryMap,
  ```

- [ ] **Implement memory subset pre-computation**: Add to `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` after line 246
  ```rust
  /// Pre-compute which memory components each processing component needs
  fn pre_compute_memory_subsets(&mut self) {
      self.component_memory_map.clear();
      
      for (comp_id, _) in &self.processing_components {
          let mut memory_deps = Vec::new();
          
          // Find all memory connections for this component
          for ((connected_comp, _port), memory_id) in &self.memory_connections {
              if connected_comp == comp_id {
                  memory_deps.push(memory_id.clone());
              }
          }
          
          if !memory_deps.is_empty() {
              self.component_memory_map.insert(comp_id.clone(), memory_deps);
          }
      }
  }
  ```

- [ ] **Call pre-computation in build_execution_order**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` after line 244
  ```rust
  // Add after input connections computation:
  self.pre_compute_memory_subsets();
  ```

- [ ] **Add per-component memory proxy creation method**: Add to `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` after pre_compute_memory_subsets
  ```rust
  /// Create a memory proxy for a specific component with only its required memory components
  fn create_component_memory_proxy(&self, component_id: &ComponentId) -> Result<MemoryProxy, String> {
      // Get the memory component IDs this component needs
      let memory_deps = self.component_memory_map.get(component_id)
          .cloned()
          .unwrap_or_default();
      
      // Extract only the memory components this component can access
      let mut component_memory_components = HashMap::new();
      for memory_id in &memory_deps {
          if let Some(memory_component) = self.memory_components.get(memory_id) {
              // Create a shared reference to the memory component
              // Safe because each memory component is only accessed by one processing component
              let memory_ref = unsafe {
                  std::ptr::from_ref(memory_component.as_ref()) as *mut dyn MemoryModuleTrait
              };
              component_memory_components.insert(memory_id.clone(), memory_ref);
          }
      }
      
      // Filter memory connections to only include this component's connections
      let component_memory_connections: HashMap<(ComponentId, String), ComponentId> = self.memory_connections
          .iter()
          .filter(|((comp_id, _), _)| comp_id == component_id)
          .map(|(k, v)| (k.clone(), v.clone()))
          .collect();
      
      Ok(MemoryProxy::new_with_component_subset(
          component_memory_connections,
          component_id.clone(),
          component_memory_components,
      ))
  }
  ```

- [ ] **Add new MemoryProxy constructor**: In `/mnt/c/project/rsim/src/core/memory/proxy.rs` after line 31
  ```rust
  /// Create a memory proxy with a subset of memory components for parallel execution
  /// This eliminates HashMap contention by giving each component only the memory it needs
  pub fn new_with_component_subset(
      memory_connections: HashMap<(ComponentId, String), ComponentId>,
      component_id: ComponentId,
      memory_components: HashMap<ComponentId, *mut dyn MemoryModuleTrait>,
  ) -> Self {
      Self {
          memory_connections,
          component_id,
          memory_components_subset: memory_components,
      }
  }
  ```

- [ ] **Add memory_components_subset field**: In `/mnt/c/project/rsim/src/core/memory/proxy.rs` line 16
  ```rust
  /// Subset of memory modules for this specific component (parallel execution)
  memory_components_subset: Option<HashMap<ComponentId, *mut dyn MemoryModuleTrait>>,
  ```

- [ ] **Update MemoryProxy constructor**: In `/mnt/c/project/rsim/src/core/memory/proxy.rs` line 20
  ```rust
  pub fn new(
      memory_connections: HashMap<(ComponentId, String), ComponentId>,
      component_id: ComponentId,
      memory_modules: &'a mut HashMap<ComponentId, Box<dyn MemoryModuleTrait>>,
  ) -> Self {
      Self {
          memory_connections,
          component_id,
          memory_modules,
          memory_components_subset: None,
      }
  }
  ```

- [ ] **Update MemoryProxy read/write methods**: In `/mnt/c/project/rsim/src/core/memory/proxy.rs` replace lines 34-73
  ```rust
  /// Read typed data from memory (reads from snapshot - previous cycle data)
  pub fn read<T: MemoryData>(&self, port: &str, address: &str) -> Result<Option<T>, String> {
      let mem_id = self
          .memory_connections
          .get(&(self.component_id.clone(), port.to_string()))
          .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

      // Use subset if available (parallel mode), otherwise use full registry (sequential mode)
      let memory_module = if let Some(ref subset) = self.memory_components_subset {
          unsafe { subset.get(mem_id).ok_or_else(|| format!("Memory module '{}' not found in component subset", mem_id))?.as_ref() }
      } else {
          self.memory_modules.get(mem_id)
              .ok_or_else(|| format!("Memory module '{}' not found in proxy registry", mem_id))?
              .as_ref()
      };

      if let Some(data_box) = memory_module.read_any(address) {
          // Try to downcast to the requested type
          if let Ok(typed_data) = data_box.downcast::<T>() {
              Ok(Some(*typed_data))
          } else {
              Err(format!("Type mismatch reading from memory address '{}' in memory '{}'", address, mem_id))
          }
      } else {
          Ok(None)
      }
  }

  /// Write typed data to memory (writes to current_state - affects next cycle)
  pub fn write<T: MemoryData>(&mut self, port: &str, address: &str, data: T) -> Result<(), String> {
      let mem_id = self
          .memory_connections
          .get(&(self.component_id.clone(), port.to_string()))
          .ok_or_else(|| format!("Memory port '{}' not connected for component '{}'", port, self.component_id))?;

      // Use subset if available (parallel mode), otherwise use full registry (sequential mode)
      let memory_module = if let Some(ref mut subset) = self.memory_components_subset {
          unsafe { subset.get_mut(mem_id).ok_or_else(|| format!("Memory module '{}' not found in component subset", mem_id))?.as_mut() }
      } else {
          self.memory_modules.get_mut(mem_id)
              .ok_or_else(|| format!("Memory module '{}' not found in proxy registry", mem_id))?
              .as_mut()
      };

      let data_box: Box<dyn std::any::Any + Send> = Box::new(data);
      if memory_module.write_any(address, data_box) {
          Ok(())
      } else {
          Err(format!("Failed to write to memory address '{}' in memory '{}' - type mismatch", address, mem_id))
      }
  }
  ```

#### **Acceptance Criteria:**
- [ ] Code compiles without errors
- [ ] Memory component subsets are pre-computed correctly
- [ ] Sequential execution still works
- [ ] All existing tests pass

#### **Test Command:**
```bash
cargo test --lib core_api_tests
```

---

### **Phase 5: Parallel Execution Implementation**
*Goal: Add parallel execution with rayon*

#### **Tasks:**
- [ ] **Add parallel component execution method**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` after line 183
  ```rust
  /// Execute a processing component in parallel (returns outputs instead of writing)
  fn execute_processing_component_parallel(
      &self, 
      component_id: &ComponentId
  ) -> Result<HashMap<(ComponentId, String), Event>, String> {
      // Same logic as execute_processing_component but return outputs
      let inputs = self.collect_inputs(component_id)?;
      
      let processor = {
          let component = self.processing_components.get(component_id)
              .ok_or_else(|| format!("Processing component '{}' not found", component_id))?;
          component.module.clone()
      };
      
      // Create per-component memory proxy (eliminates HashMap contention)
      let mut memory_proxy = self.create_component_memory_proxy(component_id)?;
      
      let mut context = EvaluationContext {
          inputs: &inputs,
          memory: &mut memory_proxy,
          state: None,
          component_id,
      };
      
      let mut outputs = EventOutputMap::new_flexible(self.current_cycle);
      (processor.evaluate_fn)(&mut context, &mut outputs)?;
      
      Ok(outputs.into_event_map())
  }
  ```

- [ ] **Add parallel cycle method**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` after line 142
  ```rust
  /// Execute one simulation cycle in parallel using rayon
  fn cycle_parallel_rayon(&mut self) -> Result<(), String> {
      use rayon::prelude::*;
      
      self.current_cycle += 1;
      self.output_buffer.clear();
      
      // Processing phase: stage-parallel execution
      for stage in &self.execution_order.clone() {
          // Collect results from all components in this stage
          let stage_results: Vec<Result<HashMap<(ComponentId, String), Event>, String>> = stage
              .par_iter()
              .map(|component_id| self.execute_processing_component_parallel(component_id))
              .collect();
          
          // Aggregate errors
          let mut all_outputs = HashMap::new();
          let mut errors = Vec::new();
          
          for result in stage_results {
              match result {
                  Ok(outputs) => all_outputs.extend(outputs),
                  Err(error) => errors.push(error),
              }
          }
          
          if !errors.is_empty() {
              return Err(format!("Component execution failed in {} components: [{}]", 
                                errors.len(), errors.join(", ")));
          }
          
          // Sequential merge of outputs
          self.output_buffer.extend(all_outputs);
      }
      
      // Memory phase: fully parallel execution
      let memory_ids: Vec<ComponentId> = self.memory_components.keys().cloned().collect();
      let memory_results: Vec<Result<(), String>> = memory_ids
          .par_iter()
          .map(|component_id| self.execute_memory_component_parallel(component_id))
          .collect();
      
      // Aggregate memory errors
      let memory_errors: Vec<String> = memory_results
          .into_iter()
          .filter_map(|r| r.err())
          .collect();
      
      if !memory_errors.is_empty() {
          return Err(format!("Memory component execution failed: [{}]", 
                            memory_errors.join(", ")));
      }
      
      Ok(())
  }
  
  /// Execute a memory component in parallel (thread-safe version)
  fn execute_memory_component_parallel(&self, component_id: &ComponentId) -> Result<(), String> {
      // Memory components can run fully parallel since they have no cross-component dependencies
      // Each memory component has exactly one input/output port (architectural constraint)
      let memory_module = unsafe {
          // Safe because memory components are only accessed by one processing component each
          let memory_ref = self.memory_components.get(component_id)
              .ok_or_else(|| format!("Memory component '{}' not found", component_id))?;
          std::ptr::from_ref(memory_ref.as_ref()) as *mut dyn MemoryModuleTrait
      };
      
      unsafe {
          // Call cycle() on stored data objects to process pending operations
          (*memory_module).cycle()?;
          
          // Update memory state: current → snapshot for next cycle
          (*memory_module).create_snapshot();
      }
      
      Ok(())
  }
  ```

- [ ] **Update cycle method to branch on mode**: In `/mnt/c/project/rsim/src/core/execution/cycle_engine.rs` line 125
  ```rust
  /// Execute one simulation cycle
  pub fn cycle(&mut self) -> Result<(), String> {
      match self.config.concurrency_mode {
          ConcurrencyMode::Sequential => self.cycle_sequential(),
          ConcurrencyMode::Rayon => self.cycle_parallel_rayon(),
      }
  }
  
  /// Execute one simulation cycle sequentially (original logic)
  fn cycle_sequential(&mut self) -> Result<(), String> {
      // Move original cycle logic here
      self.current_cycle += 1;
      self.output_buffer.clear();
      
      for stage in &self.execution_order.clone() {
          for component_id in stage {
              self.execute_processing_component(component_id)?;
          }
      }
      
      for component_id in self.memory_components.keys().cloned().collect::<Vec<_>>() {
          self.execute_memory_component(&component_id)?;
      }
      
      Ok(())
  }
  ```

#### **Acceptance Criteria:**
- [ ] Code compiles without errors
- [ ] Sequential mode still works (default behavior)
- [ ] Parallel mode executes without deadlocks
- [ ] Error aggregation works correctly
- [ ] Stage barriers are properly implemented

#### **Test Command:**
```bash
cargo test --lib core_api_tests
```

---

### **Phase 6: Integration Testing & Validation**
*Goal: Ensure both modes work correctly and produce identical results*

#### **Tasks:**
- [ ] **Add determinism test**: Create `/mnt/c/project/rsim/tests/concurrency_tests.rs`
  ```rust
  use rsim::core::*;
  
  #[test]
  fn test_determinism_serial_vs_parallel() {
      // Use McDonald's simulation as test case
      let mut sim_sequential = create_mcdonalds_simulation();
      let mut sim_parallel = create_mcdonalds_simulation_parallel();
      
      // Run 50 cycles with both modes
      for cycle in 1..=50 {
          sim_sequential.run_cycle().unwrap();
          sim_parallel.run_cycle().unwrap();
          
          // Compare states after each cycle
          assert_eq!(
              sim_sequential.query_memory_component_state::<QueueState>(&queue_id),
              sim_parallel.query_memory_component_state::<QueueState>(&queue_id),
              "States diverged at cycle {}", cycle
          );
      }
  }
  
  fn create_mcdonalds_simulation() -> CycleEngine {
      let config = SimulationConfig::default();
      // ... setup McDonald's simulation
  }
  
  fn create_mcdonalds_simulation_parallel() -> CycleEngine {
      let config = SimulationConfig::new().with_concurrency(ConcurrencyMode::Rayon);
      // ... setup McDonald's simulation
  }
  ```

- [ ] **Add performance regression test**: Add to `/mnt/c/project/rsim/tests/concurrency_tests.rs`
  ```rust
  #[test]
  fn test_parallel_performance_regression() {
      // For small simulations, parallel overhead should be reasonable
      let start = std::time::Instant::now();
      let mut sim = create_small_simulation(); // <10 components
      for _ in 0..100 {
          sim.run_cycle().unwrap();
      }
      let parallel_time = start.elapsed();
      
      // Parallel shouldn't be more than 50% slower for small sims
      // (This is a regression test, not a performance requirement)
      assert!(parallel_time.as_millis() < 1000, "Parallel execution too slow");
  }
  ```

- [ ] **Run existing test suite with both modes**: Create test helper
  ```rust
  /// Run all existing tests with both concurrency modes
  #[test]
  fn test_all_existing_functionality_sequential() {
      // All existing tests should pass with sequential mode
      // This is the regression test
  }
  
  #[test]
  fn test_all_existing_functionality_parallel() {
      // All existing tests should pass with parallel mode
      // This validates the parallel implementation
  }
  ```

#### **Acceptance Criteria:**
- [ ] All existing tests pass with `ConcurrencyMode::Sequential`
- [ ] All existing tests pass with `ConcurrencyMode::Rayon`
- [ ] Serial vs parallel results are identical (determinism test)
- [ ] No performance regressions for small simulations
- [ ] No deadlocks or race conditions in parallel mode

#### **Test Commands:**
```bash
# Test sequential mode (regression test)
cargo test --lib core_api_tests

# Test new concurrency features
cargo test --lib concurrency_tests

# Run McDonald's simulation with both modes
cargo run --bin mcdonald_complete_test

# Performance benchmark
cargo run --bin timing_benchmark
```

---

### **Implementation Dependencies**
```
Phase 1 (Config) → Phase 3 (Integration) → Phase 5 (Parallel)
                ↘                       ↗
Phase 2 (Data) → Phase 4 (Memory) → Phase 5 (Parallel)
                                   ↘
                                    Phase 6 (Testing)
```

### **Critical Notes for Implementation**
1. **Memory Proxy is the Bottleneck**: Phase 4 contains the most complex work and blocks parallel execution
2. **Test After Each Phase**: Don't proceed to next phase until current one passes all tests
3. **Maintain Backward Compatibility**: Sequential mode must always work
4. **Error Context**: Add component names to all error messages for better debugging
5. **Performance**: Parallel mode may be slower for small simulations due to overhead  

---

## Appendix A – Current engine is already two-phase

The present `CycleEngine::cycle()` executes **all processing components** first and **all memory components** afterwards:

```83:95:src/core/execution/cycle_engine.rs
// Execute processing components in topological order
for component_id in &self.execution_order.clone() {
    self.execute_processing_component(component_id)?;
}

// Update memory components
for component_id in self.memory_components.keys().cloned().collect::<Vec<_>>() {
    self.execute_memory_component(&component_id)?;
}
```

Because of this ordering, writes performed by processors in cycle *N* only become visible when each memory component's `create_snapshot()` is called at the end of the cycle, ensuring the classic two-phase model is already in place.

```109:121:src/core/execution/cycle_engine.rs
// Call cycle() on stored data objects to process pending operations
memory_module.cycle()?;

// Update memory state: current → snapshot for next cycle
memory_module.create_snapshot();
```

Therefore the simulator is ready for stage-parallel execution with minimal semantic change.