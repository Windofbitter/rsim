use rsim::*;
use rsim::core::execution::cycle_engine::CycleEngine;
use rsim::core::execution::config::{SimulationConfig, ConcurrencyMode};
use rsim::core::builder::simulation_builder::Simulation;
use std::time::Instant;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// High-performance computational worker component for massive-scale benchmarking
/// This component performs intensive mathematical operations to stress parallel execution
#[derive(Debug)]
pub struct ComputeWorker {
    /// Component ID for tracking
    id: usize,
    /// Seed for deterministic random number generation
    seed: u64,
    /// Target computation cycles per update
    work_cycles: usize,
}

impl ComputeWorker {
    pub fn new(id: usize, seed: u64, work_cycles: usize) -> Self {
        Self {
            id,
            seed,
            work_cycles,
        }
    }
    
    /// Perform computationally intensive work (static version)
    fn do_computational_work_static(cycle_count: u64, seed: u64, work_cycles: usize) -> f64 {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(cycle_count));
        let mut result = 0.0;
        
        // Perform mathematical operations that can't be optimized away
        for _ in 0..work_cycles {
            let x: f64 = rng.gen_range(0.0..100.0);
            let y: f64 = rng.gen_range(0.0..100.0);
            
            // Complex mathematical operations
            result += (x * y).sin().cos().exp().ln().sqrt();
            result += (x / (y + 1.0)).tan().abs();
            result += (x.powi(2) + y.powi(2)).sqrt();
            
            // Bit operations for integer work
            let a = rng.gen::<u64>();
            let b = rng.gen::<u64>();
            let bit_result = (a ^ b).count_ones() as f64;
            result += bit_result;
        }
        
        result
    }
}

/// Memory component for storing worker state
#[derive(Debug, Clone)]
pub struct WorkerState {
    pub cycle_count: u64,
    pub computation_result: f64,
    pub total_work_done: u64,
}

impl WorkerState {
    pub fn new() -> Self {
        Self {
            cycle_count: 0,
            computation_result: 0.0,
            total_work_done: 0,
        }
    }
}

// Implement MemoryData trait so WorkerState can be stored in memory components
impl rsim::core::components::state::MemoryData for WorkerState {}

impl Cycle for WorkerState {
    type Output = f64;
    
    fn cycle(&mut self) -> Option<Self::Output> {
        Some(self.computation_result)
    }
}

// Implement MemoryComponent trait for WorkerState using macro
impl_memory_component!(WorkerState, {
    input: input,
    output: output
});

impl_component!(ComputeWorker, "ComputeWorker", {
    inputs: [],
    outputs: [],
    memory: [worker_state],
    react: |ctx, _outputs| {
        // Read current state
        let mut state = if let Ok(Some(current_state)) = ctx.memory.read::<WorkerState>("worker_state", "state") {
            current_state
        } else {
            WorkerState::new()
        };
        
        // Perform computational work (using fixed values for now)
        let work_result = Self::do_computational_work_static(state.cycle_count, 1000, 100);
        
        // Update state
        state.cycle_count += 1;
        state.computation_result = work_result;
        state.total_work_done += 100; // Fixed work cycles
        
        // Write back updated state
        memory_write!(ctx, "worker_state", "state", state);
        
        Ok(())
    }
});

/// Configuration for massive-scale benchmark
#[derive(Debug, Clone)]
pub struct MassiveScaleBenchmarkConfig {
    /// Number of compute workers to create
    pub num_workers: usize,
    /// Computational work per worker per cycle
    pub work_cycles_per_worker: usize,
    /// Number of simulation cycles to run
    pub cycles_to_run: usize,
    /// Number of warmup cycles
    pub warmup_cycles: usize,
    /// Thread count for parallel execution
    pub thread_count: Option<usize>,
    /// Random seed base
    pub seed_base: u64,
}

impl Default for MassiveScaleBenchmarkConfig {
    fn default() -> Self {
        Self {
            num_workers: 1000,
            work_cycles_per_worker: 100,
            cycles_to_run: 100,
            warmup_cycles: 10,
            thread_count: Some(4),
            seed_base: 12345,
        }
    }
}

impl MassiveScaleBenchmarkConfig {
    /// Create configuration for small-scale testing
    pub fn small_scale() -> Self {
        Self {
            num_workers: 100,
            work_cycles_per_worker: 50,
            cycles_to_run: 50,
            warmup_cycles: 5,
            thread_count: Some(2),
            seed_base: 11111,
        }
    }
    
    /// Create configuration for medium-scale testing
    pub fn medium_scale() -> Self {
        Self {
            num_workers: 1000,
            work_cycles_per_worker: 100,
            cycles_to_run: 100,
            warmup_cycles: 10,
            thread_count: Some(4),
            seed_base: 22222,
        }
    }
    
    /// Create configuration for large-scale testing
    pub fn large_scale() -> Self {
        Self {
            num_workers: 5000,
            work_cycles_per_worker: 200,
            cycles_to_run: 100,
            warmup_cycles: 10,
            thread_count: Some(8),
            seed_base: 33333,
        }
    }
    
    /// Create configuration for massive-scale testing
    pub fn massive_scale() -> Self {
        Self {
            num_workers: 10000,
            work_cycles_per_worker: 300,
            cycles_to_run: 50,
            warmup_cycles: 5,
            thread_count: Some(16),
            seed_base: 44444,
        }
    }
    
    /// Create configuration with specific parameters
    pub fn with_params(num_workers: usize, work_cycles: usize, threads: usize) -> Self {
        Self {
            num_workers,
            work_cycles_per_worker: work_cycles,
            thread_count: Some(threads),
            ..Default::default()
        }
    }
}

/// Results from benchmark execution
#[derive(Debug)]
pub struct BenchmarkResults {
    pub config: MassiveScaleBenchmarkConfig,
    pub concurrency_mode: ConcurrencyMode,
    pub total_duration: std::time::Duration,
    pub cycles_per_second: f64,
    pub microseconds_per_cycle: f64,
    pub total_computational_work: u64,
    pub work_per_second: f64,
    pub memory_usage_mb: f64,
}

impl BenchmarkResults {
    pub fn new(
        config: MassiveScaleBenchmarkConfig,
        mode: ConcurrencyMode,
        duration: std::time::Duration,
    ) -> Self {
        let cycles_per_second = config.cycles_to_run as f64 / duration.as_secs_f64();
        let microseconds_per_cycle = duration.as_micros() as f64 / config.cycles_to_run as f64;
        let total_work = config.num_workers as u64 * config.work_cycles_per_worker as u64 * config.cycles_to_run as u64;
        let work_per_second = total_work as f64 / duration.as_secs_f64();
        
        // Rough memory estimate: each worker + state ~= 1KB
        let memory_usage_mb = (config.num_workers * 2) as f64 / 1024.0;
        
        Self {
            config,
            concurrency_mode: mode,
            total_duration: duration,
            cycles_per_second,
            microseconds_per_cycle,
            total_computational_work: total_work,
            work_per_second,
            memory_usage_mb,
        }
    }
    
    pub fn print_summary(&self) {
        println!("📊 Benchmark Results Summary");
        println!("============================");
        println!("Mode: {:?}", self.concurrency_mode);
        println!("Workers: {}", self.config.num_workers);
        println!("Work per worker: {} cycles", self.config.work_cycles_per_worker);
        println!("Simulation cycles: {}", self.config.cycles_to_run);
        println!("Threads: {:?}", self.config.thread_count);
        println!();
        println!("Performance:");
        println!("  Total time: {:.2}ms", self.total_duration.as_millis());
        println!("  Cycles/sec: {:.1}", self.cycles_per_second);
        println!("  μs/cycle: {:.1}", self.microseconds_per_cycle);
        println!("  Work/sec: {:.0} operations", self.work_per_second);
        println!("  Memory: ~{:.1} MB", self.memory_usage_mb);
        println!();
    }
}

/// Build a massive-scale simulation with compute workers
pub fn build_massive_scale_simulation(
    config: &MassiveScaleBenchmarkConfig,
    concurrency_mode: ConcurrencyMode,
) -> Result<CycleEngine, String> {
    println!("🏗️  Building massive-scale simulation...");
    println!("  Workers: {}", config.num_workers);
    println!("  Work per worker: {} cycles", config.work_cycles_per_worker);
    println!("  Mode: {:?}", concurrency_mode);
    
    // Configure simulation
    let sim_config = match concurrency_mode {
        ConcurrencyMode::Sequential => SimulationConfig::default(),
        ConcurrencyMode::Rayon => {
            let mut sim_config = SimulationConfig::new().with_concurrency(ConcurrencyMode::Rayon);
            if let Some(threads) = config.thread_count {
                sim_config = sim_config.with_thread_pool_size(threads);
            }
            sim_config
        }
    };
    
    let mut sim = Simulation::with_config(sim_config);
    
    // Create compute workers and their state components
    let mut workers = Vec::new();
    let mut worker_states = Vec::new();
    
    for i in 0..config.num_workers {
        // Create worker component
        let worker = ComputeWorker::new(
            i,
            config.seed_base + i as u64,
            config.work_cycles_per_worker,
        );
        let worker_id = sim.add_component(worker);
        workers.push(worker_id.clone());
        
        // Create state component
        let state = WorkerState::new();
        let state_id = sim.add_memory_component(state);
        worker_states.push(state_id.clone());
        
        // Connect worker to its state
        sim.connect_memory_port(worker_id.memory_port("worker_state"), state_id)?;
    }
    
    // Build and prepare the simulation engine
    let mut engine = sim.build()?;
    engine.build_execution_order()?;
    
    println!("✅ Simulation built successfully!");
    println!("  Total components: {} workers + {} states = {}",
             workers.len(), worker_states.len(), workers.len() + worker_states.len());
    
    Ok(engine)
}

/// Run a single benchmark with the given configuration
pub fn run_benchmark(
    config: MassiveScaleBenchmarkConfig,
    mode: ConcurrencyMode,
) -> Result<BenchmarkResults, String> {
    println!("\n🚀 Running benchmark: {:?} with {} workers", mode, config.num_workers);
    
    // Build simulation
    let mut engine = build_massive_scale_simulation(&config, mode)?;
    
    // Warmup phase
    if config.warmup_cycles > 0 {
        println!("🔥 Warming up: {} cycles", config.warmup_cycles);
        for _ in 0..config.warmup_cycles {
            engine.cycle()?;
        }
    }
    
    // Measurement phase
    println!("⏱️  Measuring: {} cycles", config.cycles_to_run);
    let start_time = Instant::now();
    
    for cycle in 0..config.cycles_to_run {
        engine.cycle()?;
        
        // Progress indicator for long runs
        if config.cycles_to_run > 100 && cycle % (config.cycles_to_run / 10) == 0 {
            println!("  Progress: {:.0}%", (cycle as f64 / config.cycles_to_run as f64) * 100.0);
        }
    }
    
    let duration = start_time.elapsed();
    
    Ok(BenchmarkResults::new(config, mode, duration))
}

/// Compare performance between sequential and parallel execution
pub fn run_comparative_benchmark(config: MassiveScaleBenchmarkConfig) -> Result<(), String> {
    println!("🎯 Massive-Scale RSim Parallel Benchmark");
    println!("=========================================");
    
    // Run sequential benchmark
    let sequential_results = run_benchmark(config.clone(), ConcurrencyMode::Sequential)?;
    sequential_results.print_summary();
    
    // Run parallel benchmark
    let parallel_results = run_benchmark(config.clone(), ConcurrencyMode::Rayon)?;
    parallel_results.print_summary();
    
    // Calculate speedup
    let speedup = sequential_results.total_duration.as_secs_f64() / parallel_results.total_duration.as_secs_f64();
    let throughput_improvement = parallel_results.cycles_per_second / sequential_results.cycles_per_second;
    let efficiency = speedup / config.thread_count.unwrap_or(1) as f64;
    
    println!("🏆 PERFORMANCE COMPARISON");
    println!("=========================");
    println!("Parallel vs Sequential:");
    println!("  Speedup: {:.2}x {}", speedup, if speedup > 1.0 { "🚀" } else { "🐌" });
    println!("  Throughput improvement: {:.2}x", throughput_improvement);
    println!("  Parallel efficiency: {:.1}% (vs {} threads)", efficiency * 100.0, config.thread_count.unwrap_or(1));
    
    // Per-thread analysis
    if let Some(threads) = config.thread_count {
        println!("  Per-thread speedup: {:.2}x", speedup / threads as f64);
        println!("  Threading overhead: {:.1}%", (1.0 - efficiency) * 100.0);
    }
    
    // Performance assessment
    println!("\n🎯 ASSESSMENT");
    println!("=============");
    if speedup > 2.0 {
        println!("🚀 EXCELLENT: Significant parallel speedup achieved!");
    } else if speedup > 1.5 {
        println!("✅ GOOD: Solid parallel performance gains.");
    } else if speedup > 1.1 {
        println!("⚠️  MODEST: Some parallel benefit, but room for improvement.");
    } else if speedup > 0.9 {
        println!("😐 NEUTRAL: Parallel performance comparable to sequential.");
    } else {
        println!("❌ POOR: Parallel execution is slower than sequential.");
        println!("   This indicates threading overhead exceeds parallel benefits.");
    }
    
    // Scale analysis
    let work_per_component = config.work_cycles_per_worker;
    if work_per_component < 50 {
        println!("\n💡 RECOMMENDATION: Increase work per component (currently {}) to improve parallel efficiency.", work_per_component);
    } else if work_per_component > 500 {
        println!("\n💡 NOTE: High work per component ({}) should show good parallel scaling.", work_per_component);
    }
    
    Ok(())
}

/// Run scalability tests across different configurations
pub fn run_scalability_tests() -> Result<(), String> {
    println!("🧪 SCALABILITY ANALYSIS");
    println!("=======================");
    
    let test_configs = vec![
        ("Small", MassiveScaleBenchmarkConfig::small_scale()),
        ("Medium", MassiveScaleBenchmarkConfig::medium_scale()),
        ("Large", MassiveScaleBenchmarkConfig::large_scale()),
    ];
    
    let mut results = Vec::new();
    
    for (name, config) in test_configs {
        println!("\n🧪 Testing {} scale configuration...", name);
        println!("  Workers: {}, Work: {}, Threads: {:?}", 
                 config.num_workers, config.work_cycles_per_worker, config.thread_count);
        
        // Run both sequential and parallel
        let seq_result = run_benchmark(config.clone(), ConcurrencyMode::Sequential)?;
        let par_result = run_benchmark(config.clone(), ConcurrencyMode::Rayon)?;
        
        let speedup = seq_result.total_duration.as_secs_f64() / par_result.total_duration.as_secs_f64();
        results.push((name, config.num_workers, speedup, par_result.cycles_per_second));
        
        println!("  Result: {:.2}x speedup, {:.1} cycles/sec", speedup, par_result.cycles_per_second);
    }
    
    // Summary of scalability
    println!("\n📈 SCALABILITY SUMMARY");
    println!("======================");
    println!("Scale      | Workers | Speedup | Cycles/sec");
    println!("-----------|---------|---------|----------");
    for (name, workers, speedup, cps) in results {
        println!("{:<10} | {:<7} | {:<7.2} | {:<10.1}", name, workers, speedup, cps);
    }
    
    Ok(())
}

/// Run thread count analysis
pub fn run_thread_analysis() -> Result<(), String> {
    println!("\n🧵 THREAD COUNT ANALYSIS");
    println!("========================");
    
    let base_config = MassiveScaleBenchmarkConfig::medium_scale();
    let thread_counts = vec![1, 2, 4, 8, 16];
    
    let mut results = Vec::new();
    
    for thread_count in thread_counts {
        println!("\n🧵 Testing with {} threads...", thread_count);
        
        let mut config = base_config.clone();
        config.thread_count = Some(thread_count);
        
        let mode = if thread_count == 1 {
            ConcurrencyMode::Sequential
        } else {
            ConcurrencyMode::Rayon
        };
        
        let result = run_benchmark(config, mode)?;
        results.push((thread_count, result.cycles_per_second, result.microseconds_per_cycle));
        
        println!("  {:.1} cycles/sec, {:.1}μs/cycle", result.cycles_per_second, result.microseconds_per_cycle);
    }
    
    // Thread analysis summary
    println!("\n📊 THREAD ANALYSIS SUMMARY");
    println!("===========================");
    println!("Threads | Cycles/sec | μs/cycle | vs 1-thread");
    println!("--------|------------|----------|----------");
    
    let baseline_cps = results[0].1;
    for (threads, cps, us_per_cycle) in results {
        let speedup = cps / baseline_cps;
        println!("{:<7} | {:<10.1} | {:<8.1} | {:<8.2}x", threads, cps, us_per_cycle, speedup);
    }
    
    Ok(())
}

/// Entry point for massive-scale benchmarking
fn main() -> Result<(), String> {
    println!("🚀 RSim Massive-Scale Parallel Benchmark");
    println!("=========================================");
    println!("This benchmark tests RSim's ability to handle large-scale parallel workloads");
    println!("with 1,000 to 10,000+ components performing intensive computational work.");
    
    // Get test mode from command line arguments
    let args: Vec<String> = std::env::args().collect();
    let test_mode = args.get(1).map(|s| s.as_str()).unwrap_or("standard");
    
    match test_mode {
        "small" => {
            println!("\n🔬 Running small-scale test...");
            run_comparative_benchmark(MassiveScaleBenchmarkConfig::small_scale())?;
        }
        "medium" => {
            println!("\n🔬 Running medium-scale test...");
            run_comparative_benchmark(MassiveScaleBenchmarkConfig::medium_scale())?;
        }
        "large" => {
            println!("\n🔬 Running large-scale test...");
            run_comparative_benchmark(MassiveScaleBenchmarkConfig::large_scale())?;
        }
        "massive" => {
            println!("\n🔬 Running massive-scale test...");
            run_comparative_benchmark(MassiveScaleBenchmarkConfig::massive_scale())?;
        }
        "scalability" => {
            println!("\n🔬 Running scalability analysis...");
            run_scalability_tests()?;
        }
        "threads" => {
            println!("\n🔬 Running thread count analysis...");
            run_thread_analysis()?;
        }
        "all" => {
            println!("\n🔬 Running comprehensive analysis...");
            run_comparative_benchmark(MassiveScaleBenchmarkConfig::medium_scale())?;
            run_scalability_tests()?;
            run_thread_analysis()?;
        }
        _ => {
            println!("\n🔬 Running standard benchmark...");
            run_comparative_benchmark(MassiveScaleBenchmarkConfig::default())?;
        }
    }
    
    println!("\n🎉 Benchmark completed!");
    println!("\n💡 Usage: cargo run --release --bin massive_scale_benchmark [mode]");
    println!("   Modes: small, medium, large, massive, scalability, threads, all");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_worker_creation() {
        let worker = ComputeWorker::new(0, 12345, 100);
        assert_eq!(worker.id, 0);
        assert_eq!(worker.seed, 12345);
        assert_eq!(worker.work_cycles, 100);
    }
    
    #[test]
    fn test_worker_state_creation() {
        let state = WorkerState::new();
        assert_eq!(state.cycle_count, 0);
        assert_eq!(state.computation_result, 0.0);
        assert_eq!(state.total_work_done, 0);
    }
    
    #[test]
    fn test_benchmark_configs() {
        let small = MassiveScaleBenchmarkConfig::small_scale();
        assert_eq!(small.num_workers, 100);
        
        let medium = MassiveScaleBenchmarkConfig::medium_scale();
        assert_eq!(medium.num_workers, 1000);
        
        let large = MassiveScaleBenchmarkConfig::large_scale();
        assert_eq!(large.num_workers, 5000);
        
        let massive = MassiveScaleBenchmarkConfig::massive_scale();
        assert_eq!(massive.num_workers, 10000);
    }
    
    #[test]
    fn test_small_scale_simulation_builds() {
        let config = MassiveScaleBenchmarkConfig::small_scale();
        let result = build_massive_scale_simulation(&config, ConcurrencyMode::Sequential);
        assert!(result.is_ok(), "Failed to build small-scale simulation");
    }
    
    #[test]
    fn test_computational_work() {
        let result = ComputeWorker::do_computational_work_static(0, 12345, 10);
        assert!(result.is_finite(), "Computational work should produce finite result");
    }
    
    #[test]
    fn test_benchmark_results() {
        let config = MassiveScaleBenchmarkConfig::small_scale();
        let duration = std::time::Duration::from_millis(100);
        let results = BenchmarkResults::new(config, ConcurrencyMode::Sequential, duration);
        
        assert!(results.cycles_per_second > 0.0);
        assert!(results.microseconds_per_cycle > 0.0);
        assert!(results.total_computational_work > 0);
    }
}