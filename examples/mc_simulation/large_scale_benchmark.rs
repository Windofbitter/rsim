use rsim::core::execution::cycle_engine::CycleEngine;
use rsim::core::execution::config::{SimulationConfig, ConcurrencyMode};
use std::time::Instant;

mod components;
mod simulation_builder;

use components::component_states::*;
use components::fifo_memory::FIFOMemory;
use simulation_builder::*;

/// Configuration for large-scale benchmarking
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub total_components: usize,
    pub cycles_to_run: usize,
    pub thread_count: Option<usize>,
    pub warmup_cycles: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            total_components: 116,
            cycles_to_run: 1000,
            thread_count: Some(2),
            warmup_cycles: 10,
        }
    }
}

/// Build a large-scale McDonald's simulation for benchmarking
fn build_large_scale_simulation(concurrency_mode: ConcurrencyMode) -> Result<CycleEngine, String> {
    // Scale up to achieve maximum components within manager constraints
    // McDonald's simulation has: bakers + fryers + assemblers + customers + managers + memory components
    // With 10 each: 10*4 = 40 processing + 3 managers = 43 processing
    // Memory components: 10*7 (state + buffers) + 3 inventory = 73 memory
    // Total: ~116 components (limited by manager component design)
    
    let scale_factor = 10; // 10 of each component type (max supported by managers)
    
    let config = McSimulationConfig {
        num_bakers: scale_factor,
        num_fryers: scale_factor,
        num_assemblers: scale_factor,
        num_customers: scale_factor,
        
        // Reduce buffer sizes to avoid memory issues
        individual_buffer_capacity: 5,
        inventory_buffer_capacity: 50,
        assembler_buffer_capacity: 3,
        burger_buffer_capacity: 25,
        customer_buffer_capacity: 4,
        
        delay_mode: simulation_builder::DelayMode::Random,
        fixed_delay_values: simulation_builder::FixedDelayValues::default(),
        
        // Faster timing to stress the system
        baker_timing: (1, 3),
        fryer_timing: (1, 3),
        assembler_timing: (1, 2),
        customer_timing: (1, 2),
        
        // Unique seeds for deterministic behavior
        baker_seed_base: 10000,
        fryer_seed_base: 20000,
        assembler_seed_base: 30000,
        customer_seed_base: 40000,
    };
    
    // Configure simulation based on concurrency mode
    let sim_config = match concurrency_mode {
        ConcurrencyMode::Sequential => SimulationConfig::default(),
        ConcurrencyMode::Rayon => SimulationConfig::new()
            .with_concurrency(ConcurrencyMode::Rayon)
            .with_thread_pool_size(2), // Force 2 threads for comparison
    };
    
    let (sim, components) = McSimulationBuilder::with_config(config)
        .build_with_config(sim_config)?;
    
    // Build simulation engine
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    // Print component counts for verification
    println!("📊 Component counts:");
    println!("   Processing: {} bakers, {} fryers, {} assemblers, {} customers", 
             components.bakers.len(), components.fryers.len(), 
             components.assemblers.len(), components.customers.len());
    println!("   Managers: 3 (bread, meat, assembler)");
    println!("   Memory: {} state + {} buffers + 3 inventory", 
             components.baker_states.len() + components.fryer_states.len() + 
             components.assembler_states.len() + components.customer_states.len(),
             components.bread_buffers.len() + components.meat_buffers.len() + 
             components.assembler_buffers.len() + components.customer_buffers.len());
    
    let total_processing = components.bakers.len() + components.fryers.len() + 
                          components.assemblers.len() + components.customers.len() + 3;
    let total_memory = components.baker_states.len() + components.fryer_states.len() + 
                      components.assembler_states.len() + components.customer_states.len() +
                      components.bread_buffers.len() + components.meat_buffers.len() + 
                      components.assembler_buffers.len() + components.customer_buffers.len() + 3;
    
    println!("   Total: {} processing + {} memory = {} components", 
             total_processing, total_memory, total_processing + total_memory);
    
    Ok(engine)
}

/// Benchmark a simulation with warmup and measurement phases
fn benchmark_simulation(
    mode: ConcurrencyMode,
    config: &BenchmarkConfig
) -> Result<(std::time::Duration, f64), String> {
    println!("\n🚀 Building simulation with {:?} mode...", mode);
    let mut engine = build_large_scale_simulation(mode)?;
    
    // Warmup phase
    println!("🔥 Warming up with {} cycles...", config.warmup_cycles);
    for _ in 0..config.warmup_cycles {
        engine.cycle()?;
    }
    
    // Measurement phase
    println!("⏱️  Measuring {} cycles...", config.cycles_to_run);
    let start_time = Instant::now();
    
    for _ in 0..config.cycles_to_run {
        engine.cycle()?;
    }
    
    let duration = start_time.elapsed();
    let cycles_per_second = config.cycles_to_run as f64 / duration.as_secs_f64();
    
    Ok((duration, cycles_per_second))
}

/// Run comparative benchmark between sequential and parallel execution
fn run_comparative_benchmark(config: BenchmarkConfig) -> Result<(), String> {
    println!("🏁 Large-Scale RSim Benchmark");
    println!("============================");
    println!("Target: {} components", config.total_components);
    println!("Cycles: {} (+ {} warmup)", config.cycles_to_run, config.warmup_cycles);
    if let Some(threads) = config.thread_count {
        println!("Thread limit: {}", threads);
    }
    
    // Run sequential benchmark
    println!("\n📈 SEQUENTIAL EXECUTION");
    println!("========================");
    let (seq_duration, seq_cps) = benchmark_simulation(ConcurrencyMode::Sequential, &config)?;
    println!("✅ Sequential: {:.2}ms total, {:.1} cycles/sec", 
             seq_duration.as_millis(), seq_cps);
    
    // Run parallel benchmark
    println!("\n⚡ PARALLEL EXECUTION (2 threads)");
    println!("=================================");
    let (par_duration, par_cps) = benchmark_simulation(ConcurrencyMode::Rayon, &config)?;
    println!("✅ Parallel: {:.2}ms total, {:.1} cycles/sec", 
             par_duration.as_millis(), par_cps);
    
    // Calculate speedup/slowdown
    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    let throughput_ratio = par_cps / seq_cps;
    
    println!("\n📊 PERFORMANCE COMPARISON");
    println!("==========================");
    println!("Parallel vs Sequential:");
    println!("  Time ratio: {:.2}x {}", speedup, 
             if speedup > 1.0 { "(speedup)" } else { "(slowdown)" });
    println!("  Throughput ratio: {:.2}x", throughput_ratio);
    println!("  Parallel overhead: {:.1}%", (1.0 - speedup) * 100.0);
    
    // Per-cycle timing analysis
    let seq_per_cycle = seq_duration.as_micros() as f64 / config.cycles_to_run as f64;
    let par_per_cycle = par_duration.as_micros() as f64 / config.cycles_to_run as f64;
    
    println!("\n🔍 PER-CYCLE ANALYSIS");
    println!("=====================");
    println!("Sequential: {:.1}μs per cycle", seq_per_cycle);
    println!("Parallel:   {:.1}μs per cycle", par_per_cycle);
    println!("Difference: {:.1}μs per cycle", par_per_cycle - seq_per_cycle);
    
    // Memory and component analysis
    println!("\n💾 SYSTEM UTILIZATION");
    println!("====================");
    println!("Thread utilization: 2 threads (parallel mode)");
    println!("Memory pressure: High (1000+ components)");
    
    // Performance assessment
    println!("\n🎯 ASSESSMENT");
    println!("=============");
    if speedup > 1.5 {
        println!("🚀 Excellent parallel performance! Significant speedup achieved.");
    } else if speedup > 1.1 {
        println!("✅ Good parallel performance. Moderate speedup achieved.");
    } else if speedup > 0.9 {
        println!("⚠️  Parallel performance is comparable to sequential.");
    } else {
        println!("❌ Parallel performance is slower than sequential.");
        println!("   This may be due to overhead from thread synchronization.");
    }
    
    if config.cycles_to_run >= 1000 {
        println!("📈 Large cycle count provides reliable performance measurements.");
    }
    
    Ok(())
}

/// Memory usage estimation
fn estimate_memory_usage() {
    println!("\n💾 MEMORY USAGE ESTIMATION");
    println!("==========================");
    
    // Component estimates (rough)
    let processing_components = 43; // 40 + 3 managers
    let memory_components = 73; // states + buffers + inventory
    
    // Assume ~1KB per component (processing logic + state)
    let estimated_mb = (processing_components + memory_components) as f64 / 1024.0;
    
    println!("Processing components: {}", processing_components);
    println!("Memory components: {}", memory_components);
    println!("Estimated memory: ~{:.1} MB", estimated_mb);
    println!("Note: This is a rough estimate for component overhead only.");
}

/// Extended benchmark with multiple configurations
fn run_extended_benchmark() -> Result<(), String> {
    println!("🔬 EXTENDED BENCHMARK SUITE");
    println!("============================");
    
    let test_configs = vec![
        BenchmarkConfig {
            total_components: 116,
            cycles_to_run: 100,
            thread_count: Some(2),
            warmup_cycles: 5,
        },
        BenchmarkConfig {
            total_components: 116,
            cycles_to_run: 1000,
            thread_count: Some(2),
            warmup_cycles: 10,
        },
        BenchmarkConfig {
            total_components: 116,
            cycles_to_run: 5000,
            thread_count: Some(2),
            warmup_cycles: 50,
        },
    ];
    
    for (i, config) in test_configs.iter().enumerate() {
        println!("\n🔄 Test Configuration {} of {}", i + 1, test_configs.len());
        println!("Cycles: {}, Warmup: {}", config.cycles_to_run, config.warmup_cycles);
        
        match run_comparative_benchmark(config.clone()) {
            Ok(_) => println!("✅ Test {} completed successfully", i + 1),
            Err(e) => println!("❌ Test {} failed: {}", i + 1, e),
        }
    }
    
    Ok(())
}

fn main() -> Result<(), String> {
    println!("🎯 RSim Large-Scale Parallel Execution Benchmark");
    println!("=================================================");
    
    // Memory estimation
    estimate_memory_usage();
    
    // Run standard benchmark
    let standard_config = BenchmarkConfig::default();
    run_comparative_benchmark(standard_config)?;
    
    // Ask user if they want extended benchmark
    println!("\n❓ Run extended benchmark suite? (This will take longer)");
    println!("   Press Enter to continue with extended tests, or Ctrl+C to exit.");
    
    // For automation, we'll run a shorter extended benchmark
    let extended_config = BenchmarkConfig {
        cycles_to_run: 100,
        warmup_cycles: 5,
        ..BenchmarkConfig::default()
    };
    
    println!("\n🔬 Running abbreviated extended benchmark...");
    run_comparative_benchmark(extended_config)?;
    
    println!("\n🎉 Benchmark completed!");
    println!("This benchmark demonstrates RSim's parallel execution capabilities");
    println!("with a realistic McDonald's simulation scenario (~116 components).");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_large_scale_simulation_builds() {
        let result = build_large_scale_simulation(ConcurrencyMode::Sequential);
        assert!(result.is_ok(), "Failed to build large-scale simulation");
    }
    
    #[test]
    fn test_parallel_simulation_builds() {
        let result = build_large_scale_simulation(ConcurrencyMode::Rayon);
        assert!(result.is_ok(), "Failed to build parallel large-scale simulation");
    }
    
    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.total_components, 116);
        assert_eq!(config.cycles_to_run, 1000);
        assert_eq!(config.thread_count, Some(2));
    }
    
    #[test]
    fn test_short_benchmark_sequential() {
        let config = BenchmarkConfig {
            cycles_to_run: 10,
            warmup_cycles: 2,
            ..BenchmarkConfig::default()
        };
        
        let result = benchmark_simulation(ConcurrencyMode::Sequential, &config);
        assert!(result.is_ok(), "Sequential benchmark failed");
    }
    
    #[test]
    fn test_short_benchmark_parallel() {
        let config = BenchmarkConfig {
            cycles_to_run: 10,
            warmup_cycles: 2,
            ..BenchmarkConfig::default()
        };
        
        let result = benchmark_simulation(ConcurrencyMode::Rayon, &config);
        assert!(result.is_ok(), "Parallel benchmark failed");
    }
}