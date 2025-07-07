use std::time::Instant;

mod massive_scale_benchmark;
use massive_scale_benchmark::*;
use rsim::core::execution::config::ConcurrencyMode;

fn main() -> Result<(), String> {
    println!("🚀 ULTRA-MASSIVE SCALE TEST");
    println!("===========================");
    println!("Testing RSim with extreme scale configurations");
    
    // Ultra-massive configuration: 50,000 workers
    let ultra_config = MassiveScaleBenchmarkConfig {
        num_workers: 50000,
        work_cycles_per_worker: 500,
        cycles_to_run: 20,
        warmup_cycles: 2,
        thread_count: Some(32),
        seed_base: 99999,
    };
    
    println!("\n🔥 ULTRA-MASSIVE CONFIGURATION:");
    println!("  Workers: {}", ultra_config.num_workers);
    println!("  Work per worker: {} cycles", ultra_config.work_cycles_per_worker);
    println!("  Total cycles: {}", ultra_config.cycles_to_run);
    println!("  Threads: {:?}", ultra_config.thread_count);
    println!("  Expected components: {} (workers + states)", ultra_config.num_workers * 2);
    
    // Test sequential first
    println!("\n🔬 Testing Sequential Mode...");
    let start = Instant::now();
    match run_benchmark(ultra_config.clone(), ConcurrencyMode::Sequential) {
        Ok(result) => {
            println!("✅ Sequential completed successfully!");
            result.print_summary();
        }
        Err(e) => {
            println!("❌ Sequential failed: {}", e);
            return Err(e);
        }
    }
    
    // Test parallel 
    println!("\n🔬 Testing Parallel Mode...");
    match run_benchmark(ultra_config.clone(), ConcurrencyMode::Rayon) {
        Ok(result) => {
            println!("✅ Parallel completed successfully!");
            result.print_summary();
        }
        Err(e) => {
            println!("❌ Parallel failed: {}", e);
            return Err(e);
        }
    }
    
    println!("\n🎉 ULTRA-MASSIVE SCALE TEST COMPLETED!");
    println!("The RSim framework successfully handled 50,000+ components!");
    
    Ok(())
}