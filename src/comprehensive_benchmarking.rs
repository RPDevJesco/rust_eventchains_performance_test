use colored::*;
use std::time::{Duration, Instant};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// ============================================================================
// Memory Tracking Allocator
// ============================================================================

pub struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static PEAK_MEMORY: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let size = layout.size();
            ALLOCATED.fetch_add(size, Ordering::SeqCst);
            ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);

            // Update peak memory
            let current = ALLOCATED.load(Ordering::SeqCst) - DEALLOCATED.load(Ordering::SeqCst);
            let mut peak = PEAK_MEMORY.load(Ordering::SeqCst);
            while current > peak {
                match PEAK_MEMORY.compare_exchange_weak(peak, current, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        DEALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub net_allocated: usize,
    pub peak_memory: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

impl MemoryStats {
    pub fn reset() {
        ALLOCATED.store(0, Ordering::SeqCst);
        DEALLOCATED.store(0, Ordering::SeqCst);
        ALLOCATION_COUNT.store(0, Ordering::SeqCst);
        DEALLOCATION_COUNT.store(0, Ordering::SeqCst);
        PEAK_MEMORY.store(0, Ordering::SeqCst);
    }

    pub fn snapshot() -> Self {
        let total_allocated = ALLOCATED.load(Ordering::SeqCst);
        let total_deallocated = DEALLOCATED.load(Ordering::SeqCst);

        Self {
            total_allocated,
            total_deallocated,
            net_allocated: total_allocated.saturating_sub(total_deallocated),
            peak_memory: PEAK_MEMORY.load(Ordering::SeqCst),
            allocation_count: ALLOCATION_COUNT.load(Ordering::SeqCst),
            deallocation_count: DEALLOCATION_COUNT.load(Ordering::SeqCst),
        }
    }

    pub fn diff(&self, baseline: &MemoryStats) -> MemoryStatsDiff {
        MemoryStatsDiff {
            allocated_diff: self.total_allocated as i64 - baseline.total_allocated as i64,
            peak_diff: self.peak_memory as i64 - baseline.peak_memory as i64,
            allocation_count_diff: self.allocation_count as i64 - baseline.allocation_count as i64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStatsDiff {
    pub allocated_diff: i64,
    pub peak_diff: i64,
    pub allocation_count_diff: i64,
}

// ============================================================================
// Cache Performance Simulation (using timing patterns)
// ============================================================================

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub avg_access_time_ns: f64,
    pub variance_ns: f64,
    pub min_access_ns: u64,
    pub max_access_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
}

impl CacheStats {
    pub fn from_access_times(mut times: Vec<u64>) -> Self {
        if times.is_empty() {
            return Self::default();
        }

        times.sort_unstable();
        let len = times.len();

        let avg = times.iter().sum::<u64>() as f64 / len as f64;
        let variance = times.iter()
            .map(|&t| {
                let diff = t as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / len as f64;

        Self {
            avg_access_time_ns: avg,
            variance_ns: variance,
            min_access_ns: times[0],
            max_access_ns: times[len - 1],
            p50_ns: times[len / 2],
            p95_ns: times[(len as f64 * 0.95) as usize],
            p99_ns: times[(len as f64 * 0.99) as usize],
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            avg_access_time_ns: 0.0,
            variance_ns: 0.0,
            min_access_ns: 0,
            max_access_ns: 0,
            p50_ns: 0,
            p95_ns: 0,
            p99_ns: 0,
        }
    }
}

// ============================================================================
// Comprehensive Performance Metrics
// ============================================================================

#[derive(Debug, Clone)]
pub struct ComprehensiveMetrics {
    // Timing
    pub mean_duration: Duration,
    pub median_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub std_dev_nanos: f64,
    pub p95_duration: Duration,
    pub p99_duration: Duration,

    // Memory
    pub memory_stats: MemoryStats,

    // Cache behavior (approximated via timing variance)
    pub cache_stats: CacheStats,

    // Metadata
    pub runs: usize,
    pub success_rate: f64,
}

impl ComprehensiveMetrics {
    pub fn from_runs(durations: Vec<Duration>, memory_stats: MemoryStats, successes: usize) -> Self {
        let mut sorted_durations = durations.clone();
        sorted_durations.sort();
        let runs = sorted_durations.len();

        let mean_nanos = sorted_durations.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / runs as f64;
        let mean_duration = Duration::from_nanos(mean_nanos as u64);

        let median_duration = if runs % 2 == 0 {
            let mid = runs / 2;
            Duration::from_nanos(
                ((sorted_durations[mid - 1].as_nanos() + sorted_durations[mid].as_nanos()) / 2) as u64,
            )
        } else {
            sorted_durations[runs / 2]
        };

        let variance = sorted_durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / runs as f64;

        let std_dev_nanos = variance.sqrt();

        let p95_duration = sorted_durations[(runs as f64 * 0.95) as usize];
        let p99_duration = sorted_durations[(runs as f64 * 0.99) as usize];

        // Extract nanosecond timings for cache stats
        let nanos: Vec<u64> = sorted_durations.iter().map(|d| d.as_nanos() as u64).collect();
        let cache_stats = CacheStats::from_access_times(nanos);

        Self {
            mean_duration,
            median_duration,
            min_duration: *sorted_durations.first().unwrap(),
            max_duration: *sorted_durations.last().unwrap(),
            std_dev_nanos,
            p95_duration,
            p99_duration,
            memory_stats,
            cache_stats,
            runs,
            success_rate: (successes as f64 / runs as f64) * 100.0,
        }
    }

    pub fn mean_micros(&self) -> f64 {
        self.mean_duration.as_nanos() as f64 / 1000.0
    }

    pub fn overhead_vs(&self, baseline: &ComprehensiveMetrics) -> f64 {
        let baseline_nanos = baseline.mean_duration.as_nanos() as f64;
        let our_nanos = self.mean_duration.as_nanos() as f64;
        ((our_nanos - baseline_nanos) / baseline_nanos) * 100.0
    }

    pub fn memory_overhead_vs(&self, baseline: &ComprehensiveMetrics) -> f64 {
        let baseline_mem = baseline.memory_stats.peak_memory as f64;
        let our_mem = self.memory_stats.peak_memory as f64;
        if baseline_mem == 0.0 {
            return 0.0;
        }
        ((our_mem - baseline_mem) / baseline_mem) * 100.0
    }

    pub fn coefficient_of_variation(&self) -> f64 {
        let mean_nanos = self.mean_duration.as_nanos() as f64;
        if mean_nanos == 0.0 {
            return 0.0;
        }
        (self.std_dev_nanos / mean_nanos) * 100.0
    }
}

// ============================================================================
// Comprehensive Benchmark Runner
// ============================================================================

pub fn run_comprehensive_benchmark<F>(runs: usize, mut func: F) -> ComprehensiveMetrics
where
    F: FnMut() -> bool,
{
    let mut durations = Vec::with_capacity(runs);
    let mut successes = 0;

    // Reset memory tracking
    MemoryStats::reset();
    let baseline_memory = MemoryStats::snapshot();

    for _ in 0..runs {
        // Warm up - run once without measuring to populate caches
        if durations.is_empty() {
            let _ = func();
        }

        let start = Instant::now();
        let success = func();
        let duration = start.elapsed();

        durations.push(duration);
        if success {
            successes += 1;
        }
    }

    let memory_stats = MemoryStats::snapshot().diff(&baseline_memory);
    let final_memory = MemoryStats {
        total_allocated: memory_stats.allocated_diff.max(0) as usize,
        total_deallocated: 0,
        net_allocated: memory_stats.allocated_diff.max(0) as usize,
        peak_memory: memory_stats.peak_diff.max(0) as usize,
        allocation_count: memory_stats.allocation_count_diff.max(0) as usize,
        deallocation_count: 0,
    };

    ComprehensiveMetrics::from_runs(durations, final_memory, successes)
}

// ============================================================================
// Comprehensive Results Display
// ============================================================================

pub fn print_comprehensive_comparison(
    name: &str,
    baseline: &ComprehensiveMetrics,
    tested: &ComprehensiveMetrics,
) {
    println!("\n{}", "=".repeat(90).bright_cyan().bold());
    println!("{}", name.bright_cyan().bold());
    println!("{}", "=".repeat(90).bright_cyan().bold());

    // Timing Metrics
    println!("\n{}", "⏱️  Timing Metrics".yellow().bold());
    println!("{}", "-".repeat(90));
    println!(
        "{:<30} {:>15} {:>15} {:>15}",
        "Metric".bold(),
        "Baseline".bold(),
        "Tested".bold(),
        "Overhead".bold()
    );
    println!("{}", "-".repeat(90));

    let timing_overhead = tested.overhead_vs(baseline);
    let timing_color = if timing_overhead < 15.0 {
        "green"
    } else if timing_overhead < 30.0 {
        "yellow"
    } else {
        "red"
    };

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Mean (μs)",
        baseline.mean_micros(),
        tested.mean_micros(),
        format!("+{:.2}%", timing_overhead).color(timing_color)
    );

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Median (μs)",
        baseline.median_duration.as_nanos() as f64 / 1000.0,
        tested.median_duration.as_nanos() as f64 / 1000.0,
        "-"
    );

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "P95 (μs)",
        baseline.p95_duration.as_nanos() as f64 / 1000.0,
        tested.p95_duration.as_nanos() as f64 / 1000.0,
        "-"
    );

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "P99 (μs)",
        baseline.p99_duration.as_nanos() as f64 / 1000.0,
        tested.p99_duration.as_nanos() as f64 / 1000.0,
        "-"
    );

    // Latency Variance
    println!("\n{}", "📊 Latency Variance".yellow().bold());
    println!("{}", "-".repeat(90));

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Std Dev (μs)",
        baseline.std_dev_nanos / 1000.0,
        tested.std_dev_nanos / 1000.0,
        format!(
            "{:+.2}%",
            ((tested.std_dev_nanos - baseline.std_dev_nanos) / baseline.std_dev_nanos) * 100.0
        )
    );

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Coefficient of Var (%)",
        baseline.coefficient_of_variation(),
        tested.coefficient_of_variation(),
        format!(
            "{:+.2}pp",
            tested.coefficient_of_variation() - baseline.coefficient_of_variation()
        )
    );

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Min-Max Range (μs)",
        (baseline.max_duration.as_nanos() - baseline.min_duration.as_nanos()) as f64 / 1000.0,
        (tested.max_duration.as_nanos() - tested.min_duration.as_nanos()) as f64 / 1000.0,
        "-"
    );

    // Memory Metrics
    println!("\n{}", "💾 Memory Metrics".yellow().bold());
    println!("{}", "-".repeat(90));
    println!(
        "{:<30} {:>15} {:>15} {:>15}",
        "Metric".bold(),
        "Baseline".bold(),
        "Tested".bold(),
        "Overhead".bold()
    );
    println!("{}", "-".repeat(90));

    let memory_overhead = tested.memory_overhead_vs(baseline);
    let memory_color = if memory_overhead < 20.0 {
        "green"
    } else if memory_overhead < 50.0 {
        "yellow"
    } else {
        "red"
    };

    println!(
        "{:<30} {:>15} {:>15} {:>15}",
        "Peak Memory (KB)",
        format!("{:.2}", baseline.memory_stats.peak_memory as f64 / 1024.0),
        format!("{:.2}", tested.memory_stats.peak_memory as f64 / 1024.0),
        format!("+{:.2}%", memory_overhead).color(memory_color)
    );

    println!(
        "{:<30} {:>15} {:>15} {:>15}",
        "Total Allocated (KB)",
        format!("{:.2}", baseline.memory_stats.total_allocated as f64 / 1024.0),
        format!("{:.2}", tested.memory_stats.total_allocated as f64 / 1024.0),
        "-"
    );

    println!(
        "{:<30} {:>15} {:>15} {:>15}",
        "Allocation Count",
        baseline.memory_stats.allocation_count,
        tested.memory_stats.allocation_count,
        format!(
            "{:+}",
            tested.memory_stats.allocation_count as i64 - baseline.memory_stats.allocation_count as i64
        )
    );

    // Cache Behavior (approximated)
    println!("\n{}", "🎯 Cache Behavior (approximated via timing)".yellow().bold());
    println!("{}", "-".repeat(90));

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Avg Access Time (ns)",
        baseline.cache_stats.avg_access_time_ns,
        tested.cache_stats.avg_access_time_ns,
        format!(
            "{:+.2}%",
            ((tested.cache_stats.avg_access_time_ns - baseline.cache_stats.avg_access_time_ns)
                / baseline.cache_stats.avg_access_time_ns) * 100.0
        )
    );

    println!(
        "{:<30} {:>15.2} {:>15.2} {:>15}",
        "Variance (ns²)",
        baseline.cache_stats.variance_ns,
        tested.cache_stats.variance_ns,
        "-"
    );

    // Summary
    println!("\n{}", "📋 Summary".green().bold());
    println!("{}", "-".repeat(90));
    println!("  CPU Overhead:    {:>8.2}%", timing_overhead);
    println!("  Memory Overhead: {:>8.2}%", memory_overhead);
    println!("  Latency Variance: {:>7.2}% (CoV)", tested.coefficient_of_variation());
    println!("  Success Rate:    {:>8.2}%", tested.success_rate);
}
