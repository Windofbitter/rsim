use rsim::core::execution::cycle_engine::CycleEngine;
use std::time::Instant;

mod components;
mod simulation_builder;

use components::component_states::*;
use components::fifo_memory::FIFOMemory;
use simulation_builder::*;

/// Builds a complete McDonald's simulation with all components using helper
fn build_mcdonalds_simulation() -> Result<CycleEngine, String> {
    let (mut sim, _components) = build_large_mc_simulation()?;
    
    // Build simulation engine
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    Ok(engine)
}

/// Times the execution of a simulation for a given number of cycles
fn time_simulation(cycles: usize) -> Result<std::time::Duration, String> {
    let mut engine = build_mcdonalds_simulation()?;
    
    let start_time = Instant::now();
    
    for _ in 1..=cycles {
        engine.cycle()?;
    }
    
    let duration = start_time.elapsed();
    Ok(duration)
}

fn main() -> Result<(), String> {
    println!("🍔 McDonald's Simulation Timing Benchmark 🍔");
    println!("==============================================");
    
    let test_cases = [100, 1000, 10000];
    
    for &cycles in &test_cases {
        println!("\n⏱️  Testing {} cycles...", cycles);
        
        match time_simulation(cycles) {
            Ok(duration) => {
                let millis = duration.as_millis();
                let micros = duration.as_micros();
                let cycles_per_second = if millis > 0 { 
                    (cycles as f64 / (millis as f64 / 1000.0)) as u64
                } else {
                    cycles as u64 * 1_000_000 / micros as u64
                };
                
                println!("✅ {} cycles completed in {} ms ({} μs)", cycles, millis, micros);
                println!("📊 Performance: ~{} cycles/second", cycles_per_second);
            }
            Err(e) => {
                println!("❌ Error running {} cycles: {}", cycles, e);
            }
        }
    }
    
    println!("\n🎯 Benchmark completed!");
    
    Ok(())
}