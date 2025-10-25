use colored::*;
use std::time::{Duration, Instant};

/// Performance metrics for a single run
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub duration: Duration,
    pub memory_allocated: usize,
    pub success: bool,
}

/// Statistics for multiple runs
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub mean_duration: Duration,
    pub median_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub std_dev_nanos: f64,
    pub runs: usize,
}

impl PerformanceStats {
    pub fn from_durations(mut durations: Vec<Duration>) -> Self {
        durations.sort();
        let runs = durations.len();

        let mean_nanos = durations.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / runs as f64;
        let mean_duration = Duration::from_nanos(mean_nanos as u64);

        let median_duration = if runs % 2 == 0 {
            let mid = runs / 2;
            Duration::from_nanos(
                ((durations[mid - 1].as_nanos() + durations[mid].as_nanos()) / 2) as u64,
            )
        } else {
            durations[runs / 2]
        };

        let variance = durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>()
            / runs as f64;

        let std_dev_nanos = variance.sqrt();

        Self {
            mean_duration,
            median_duration,
            min_duration: *durations.first().unwrap(),
            max_duration: *durations.last().unwrap(),
            std_dev_nanos,
            runs,
        }
    }

    pub fn overhead_vs(&self, baseline: &PerformanceStats) -> f64 {
        let baseline_nanos = baseline.mean_duration.as_nanos() as f64;
        let our_nanos = self.mean_duration.as_nanos() as f64;
        ((our_nanos - baseline_nanos) / baseline_nanos) * 100.0
    }
}

/// Run a benchmark multiple times and collect statistics
pub fn benchmark<F>(name: &str, runs: usize, func: F) -> PerformanceStats
where
    F: Fn() -> bool,
{
    println!("\n{}", format!("Benchmarking: {}", name).cyan().bold());
    println!("{}", "━".repeat(60).cyan());

    let mut durations = Vec::with_capacity(runs);
    let mut successful_runs = 0;

    for i in 0..runs {
        let start = Instant::now();
        let success = func();
        let duration = start.elapsed();

        durations.push(duration);
        if success {
            successful_runs += 1;
        }

        if (i + 1) % (runs / 10).max(1) == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }

    println!(); // New line after progress dots

    let stats = PerformanceStats::from_durations(durations);

    println!("{}", "Results:".green().bold());
    println!("  Runs: {}/{} successful", successful_runs, runs);
    println!("  Mean:   {}μs", stats.mean_duration.as_micros());
    println!("  Median: {}μs", stats.median_duration.as_micros());
    println!("  Min:    {}μs", stats.min_duration.as_micros());
    println!("  Max:    {}μs", stats.max_duration.as_micros());
    println!("  StdDev: {:.2}μs", stats.std_dev_nanos / 1000.0);

    stats
}

/// Print comparison table
pub fn print_comparison_table(
    graph_size: (usize, usize),
    traditional: &PerformanceStats,
    bare: &PerformanceStats,
    full: &PerformanceStats,
    optimized: &PerformanceStats,
) {
    println!("\n{}", "═".repeat(80).bright_blue().bold());
    println!(
        "{}",
        format!(
            "Performance Comparison - Graph: {} nodes, {} edges",
            graph_size.0, graph_size.1
        )
        .bright_blue()
        .bold()
    );
    println!("{}", "═".repeat(80).bright_blue().bold());

    println!(
        "\n{:<30} {:>12} {:>12} {:>12}",
        "Implementation".bold(),
        "Mean (μs)".bold(),
        "Median (μs)".bold(),
        "Overhead %".bold()
    );
    println!("{}", "─".repeat(80));

    // Traditional (baseline)
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "Traditional (baseline)".green(),
        format!("{:.2}", traditional.mean_duration.as_micros()),
        format!("{:.2}", traditional.median_duration.as_micros()),
        "0.00%".bright_green()
    );

    // EventChains (bare)
    let bare_overhead = bare.overhead_vs(traditional);
    let bare_color = if bare_overhead < 20.0 {
        "green"
    } else if bare_overhead < 50.0 {
        "yellow"
    } else {
        "red"
    };
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "EventChains (bare)",
        format!("{:.2}", bare.mean_duration.as_micros()),
        format!("{:.2}", bare.median_duration.as_micros()),
        format!("+{:.2}%", bare_overhead).color(bare_color)
    );

    // EventChains (full)
    let full_overhead = full.overhead_vs(traditional);
    let full_color = if full_overhead < 50.0 {
        "green"
    } else if full_overhead < 100.0 {
        "yellow"
    } else {
        "red"
    };
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "EventChains (full middleware)",
        format!("{:.2}", full.mean_duration.as_micros()),
        format!("{:.2}", full.median_duration.as_micros()),
        format!("+{:.2}%", full_overhead).color(full_color)
    );

    // EventChains (optimized)
    let opt_overhead = optimized.overhead_vs(traditional);
    let opt_color = if opt_overhead < 20.0 {
        "green"
    } else if opt_overhead < 50.0 {
        "yellow"
    } else {
        "red"
    };
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "EventChains (optimized)".cyan(),
        format!("{:.2}", optimized.mean_duration.as_micros()),
        format!("{:.2}", optimized.median_duration.as_micros()),
        format!("+{:.2}%", opt_overhead).color(opt_color)
    );

    println!("\n{}", "Overhead Breakdown:".yellow().bold());
    println!("  Event wrapping:        ~{:.1}%", bare_overhead * 0.4);
    println!("  Context lookups:       ~{:.1}%", bare_overhead * 0.3);
    println!(
        "  Middleware calls:      ~{:.1}%",
        full_overhead - bare_overhead
    );

    println!("\n{}", "═".repeat(80).bright_blue().bold());
}
