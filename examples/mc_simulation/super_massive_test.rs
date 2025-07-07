use std::time::Instant;

mod massive_scale_benchmark;
use massive_scale_benchmark::*;
use rsim::core::execution::config::ConcurrencyMode;

fn main() -> Result<(), String> {
    println!("🚀 SUPER-MASSIVE SCALE TEST");
    println!("===========================");
    println!("Testing RSim with 25,000 components");
    
    // Super-massive configuration: 25,000 workers
    let super_config = MassiveScaleBenchmarkConfig {
        num_workers: 25000,
        work_cycles_per_worker: 400,
        cycles_to_run: 10,
        warmup_cycles: 1,
        thread_count: Some(16),
        seed_base: 77777,
    };
    
    println!("\n🔥 SUPER-MASSIVE CONFIGURATION:");
    println!("  Workers: {}", super_config.num_workers);
    println!("  Work per worker: {} cycles", super_config.work_cycles_per_worker);
    println!("  Total cycles: {}", super_config.cycles_to_run);
    println!("  Threads: {:?}", super_config.thread_count);
    println!("  Expected components: {} (workers + states)", super_config.num_workers * 2);
    println!("  Expected memory: ~{:.1} MB", (super_config.num_workers * 2) as f64 / 1024.0);
    
    // Test sequential first
    println!("\n🔬 Testing Sequential Mode...");
    let seq_start = Instant::now();
    let seq_result = match run_benchmark(super_config.clone(), ConcurrencyMode::Sequential) {
        Ok(result) => {
            println!("✅ Sequential completed in {:.2}s!", seq_start.elapsed().as_secs_f64());
            result
        }
        Err(e) => {
            println!("❌ Sequential failed: {}", e);
            return Err(e);
        }
    };
    
    // Test parallel 
    println!("\n🔬 Testing Parallel Mode...");
    let par_start = Instant::now();
    let par_result = match run_benchmark(super_config.clone(), ConcurrencyMode::Rayon) {
        Ok(result) => {
            println!("✅ Parallel completed in {:.2}s!", par_start.elapsed().as_secs_f64());
            result
        }
        Err(e) => {
            println!("❌ Parallel failed: {}", e);
            return Err(e);
        }
    };
    
    // Performance comparison
    let speedup = seq_result.total_duration.as_secs_f64() / par_result.total_duration.as_secs_f64();
    
    println!("\n🏆 SUPER-MASSIVE SCALE RESULTS:");
    println!("================================");
    println!("Components: {} workers + {} states = {} total", 
             super_config.num_workers, super_config.num_workers, super_config.num_workers * 2);
    println!("Sequential: {:.2}s ({:.1} cycles/sec)", 
             seq_result.total_duration.as_secs_f64(), seq_result.cycles_per_second);
    println!("Parallel:   {:.2}s ({:.1} cycles/sec)", 
             par_result.total_duration.as_secs_f64(), par_result.cycles_per_second);
    println!("Speedup:    {:.2}x {}", speedup, if speedup > 1.0 { "🚀" } else { "🐌" });
    
    if speedup > 1.1 {
        println!("🎉 SUCCESS: Parallel execution shows meaningful speedup at super-massive scale!");
    } else if speedup > 0.9 {
        println!("✅ SUCCESS: Framework handles super-massive scale reliably!");
    } else {
        println!("⚠️  NOTE: Parallel overhead exceeds benefits at this scale.");
    }
    
    println!("\n🎯 ACHIEVEMENT UNLOCKED:");
    println!("🏆 RSim successfully handled 50,000+ components!");
    println!("🏆 Memory usage: ~{:.1} MB", par_result.memory_usage_mb);
    println!("🏆 Total computational work: {} million operations", 
             (super_config.num_workers as u64 * super_config.work_cycles_per_worker as u64 * super_config.cycles_to_run as u64) / 1_000_000);
    
    Ok(())
}