mod components;
mod simulation_builder;

use components::component_states::*;
use components::fifo_memory::FIFOMemory;
use simulation_builder::*;
use rsim::core::execution::config::{ConcurrencyMode, SimulationConfig};
use std::time::Instant;

/// Simple test to identify the parallel execution issue
fn main() -> Result<(), String> {
    println!("🔬 Simple Parallel Execution Test");
    println!("==================================");
    
    // Create a very simple configuration with minimal components
    let test_config = McSimulationConfig {
        // Just 1 of each component for simplicity
        num_bakers: 1,
        num_fryers: 1,
        num_assemblers: 1,
        num_customers: 1,
        
        // Small buffers
        individual_buffer_capacity: 5,
        inventory_buffer_capacity: 20,
        assembler_buffer_capacity: 3,
        burger_buffer_capacity: 10,
        customer_buffer_capacity: 5,
        
        // Fixed delays for deterministic behavior
        delay_mode: DelayMode::Fixed,
        fixed_delay_values: FixedDelayValues {
            baker_delay: 2,
            fryer_delay: 3,
            assembler_delay: 1,
            customer_delay: 2,
        },
        
        // Fallback values (not used in fixed mode)
        baker_timing: (2, 5),
        fryer_timing: (3, 7),
        assembler_timing: (1, 3),
        customer_timing: (1, 5),
        
        // Deterministic seeds
        baker_seed_base: 1000,
        fryer_seed_base: 2000,
        assembler_seed_base: 3000,
        customer_seed_base: 4000,
    };
    
    let test_cycles = 20;
    
    // Test 1: Sequential execution (baseline)
    println!("\n📊 TEST 1: Sequential Execution");
    println!("===============================");
    
    let sequential_config = SimulationConfig::new()
        .with_concurrency(ConcurrencyMode::Sequential);
    
    let (mut sim, components) = McSimulationBuilder::with_config(test_config.clone())
        .build_with_config(sequential_config)?;
    
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    let start_time = Instant::now();
    
    for cycle in 1..=test_cycles {
        engine.cycle()?;
        if cycle % 5 == 0 {
            println!("   Cycle {}: Running...", cycle);
        }
    }
    
    let seq_time = start_time.elapsed();
    
    // Query sequential results
    let seq_bread = if let Ok(Some(state)) = engine.query_memory_component_state::<BakerState>(&components.baker_states[0]) {
        state.total_produced
    } else {
        -1
    };
    
    let seq_meat = if let Ok(Some(state)) = engine.query_memory_component_state::<FryerState>(&components.fryer_states[0]) {
        state.total_produced
    } else {
        -1
    };
    
    let seq_assembled = if let Ok(Some(state)) = engine.query_memory_component_state::<AssemblerState>(&components.assembler_states[0]) {
        state.total_assembled
    } else {
        -1
    };
    
    let seq_consumed = if let Ok(Some(state)) = engine.query_memory_component_state::<CustomerState>(&components.customer_states[0]) {
        state.total_consumed
    } else {
        -1
    };
    
    let seq_remaining = if let Ok(Some(buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&components.burger_buffer, "buffer") {
        buffer_state.data_count
    } else {
        -1
    };
    
    println!("   ✅ Sequential completed in {} ms", seq_time.as_millis());
    println!("   📊 Results: bread={}, meat={}, assembled={}, consumed={}, remaining={}",
        seq_bread, seq_meat, seq_assembled, seq_consumed, seq_remaining);
    
    // Test 2: Parallel execution
    println!("\n🔀 TEST 2: Parallel Execution");
    println!("=============================");
    
    let parallel_config = SimulationConfig::new()
        .with_concurrency(ConcurrencyMode::Rayon)
        .with_thread_pool_size(2);
    
    let (mut sim, components) = McSimulationBuilder::with_config(test_config.clone())
        .build_with_config(parallel_config)?;
    
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    let start_time = Instant::now();
    
    for cycle in 1..=test_cycles {
        engine.cycle()?;
        if cycle % 5 == 0 {
            println!("   Cycle {}: Running...", cycle);
        }
    }
    
    let par_time = start_time.elapsed();
    
    // Query parallel results
    let par_bread = if let Ok(Some(state)) = engine.query_memory_component_state::<BakerState>(&components.baker_states[0]) {
        state.total_produced
    } else {
        -1
    };
    
    let par_meat = if let Ok(Some(state)) = engine.query_memory_component_state::<FryerState>(&components.fryer_states[0]) {
        state.total_produced
    } else {
        -1
    };
    
    let par_assembled = if let Ok(Some(state)) = engine.query_memory_component_state::<AssemblerState>(&components.assembler_states[0]) {
        state.total_assembled
    } else {
        -1
    };
    
    let par_consumed = if let Ok(Some(state)) = engine.query_memory_component_state::<CustomerState>(&components.customer_states[0]) {
        state.total_consumed
    } else {
        -1
    };
    
    let par_remaining = if let Ok(Some(buffer_state)) = engine.query_memory_component_data::<FIFOMemory>(&components.burger_buffer, "buffer") {
        buffer_state.data_count
    } else {
        -1
    };
    
    println!("   ✅ Parallel completed in {} ms", par_time.as_millis());
    println!("   📊 Results: bread={}, meat={}, assembled={}, consumed={}, remaining={}",
        par_bread, par_meat, par_assembled, par_consumed, par_remaining);
    
    // Compare results
    println!("\n🔍 COMPARISON:");
    println!("==============");
    println!("Sequential: bread={}, meat={}, assembled={}, consumed={}, remaining={}",
        seq_bread, seq_meat, seq_assembled, seq_consumed, seq_remaining);
    println!("Parallel:   bread={}, meat={}, assembled={}, consumed={}, remaining={}",
        par_bread, par_meat, par_assembled, par_consumed, par_remaining);
    
    let all_match = seq_bread == par_bread && seq_meat == par_meat && 
                   seq_assembled == par_assembled && seq_consumed == par_consumed && 
                   seq_remaining == par_remaining;
    
    if all_match {
        println!("✅ RESULT: All results match! Parallel execution is working correctly.");
    } else {
        println!("❌ RESULT: Results differ! Parallel execution has issues.");
        
        // Identify specific differences
        if seq_bread != par_bread {
            println!("   ❌ Bread production differs: {} vs {}", seq_bread, par_bread);
        }
        if seq_meat != par_meat {
            println!("   ❌ Meat production differs: {} vs {}", seq_meat, par_meat);
        }
        if seq_assembled != par_assembled {
            println!("   ❌ Burger assembly differs: {} vs {}", seq_assembled, par_assembled);
        }
        if seq_consumed != par_consumed {
            println!("   ❌ Burger consumption differs: {} vs {}", seq_consumed, par_consumed);
        }
        if seq_remaining != par_remaining {
            println!("   ❌ Remaining burgers differs: {} vs {}", seq_remaining, par_remaining);
        }
    }
    
    println!("\n⏱️  Performance:");
    println!("Sequential: {} ms", seq_time.as_millis());
    println!("Parallel:   {} ms", par_time.as_millis());
    
    if par_time.as_millis() > 0 {
        let speedup = seq_time.as_millis() as f64 / par_time.as_millis() as f64;
        if speedup > 1.0 {
            println!("🚀 Parallel is {:.2}x faster", speedup);
        } else {
            println!("⚠️  Parallel is {:.2}x slower", 1.0 / speedup);
        }
    }
    
    Ok(())
}