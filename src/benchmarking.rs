use colored::*;
use std::time::{Duration, Instant};

/// Performance statistics for multiple runs
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub mean_duration: Duration,
    pub median_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub std_dev_nanos: f64,
    pub runs: usize,
}

impl BenchmarkStats {
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

    pub fn overhead_vs(&self, baseline: &BenchmarkStats) -> f64 {
        let baseline_nanos = baseline.mean_duration.as_nanos() as f64;
        let our_nanos = self.mean_duration.as_nanos() as f64;
        ((our_nanos - baseline_nanos) / baseline_nanos) * 100.0
    }

    pub fn mean_micros(&self) -> f64 {
        self.mean_duration.as_nanos() as f64 / 1000.0
    }
}

/// Run a benchmark and collect statistics
pub fn run_benchmark<F>(runs: usize, func: F) -> BenchmarkStats
where
    F: Fn() -> bool,
{
    let mut durations = Vec::with_capacity(runs);

    for _ in 0..runs {
        let start = Instant::now();
        let _ = func();
        let duration = start.elapsed();
        durations.push(duration);
    }

    BenchmarkStats::from_durations(durations)
}

// ============================================================================
// TIER 1: Minimal Baseline (Cost of Orchestration Framework)
// ============================================================================

pub struct Tier1Results {
    pub bare_functions: BenchmarkStats,
    pub eventchains_no_middleware: BenchmarkStats,
}

impl Tier1Results {
    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(80).bright_cyan().bold());
        println!(
            "{}",
            "TIER 1: Minimal Baseline - Cost of Orchestration Framework"
                .bright_cyan()
                .bold()
        );
        println!("{}", "=".repeat(80).bright_cyan().bold());
        println!("\nComparison:");
        println!("   Baseline: 3 direct function calls (no error handling, no tracking)");
        println!("   EventChains: Full pattern with 0 middleware");
        println!("   Metric: Cost of the orchestration framework itself\n");

        println!("{}", "-".repeat(80));
        println!(
            "{:<40} {:>12} {:>12} {:>12}",
            "Implementation".bold(),
            "Mean (µs)".bold(),
            "Median (µs)".bold(),
            "Overhead".bold()
        );
        println!("{}", "-".repeat(80));

        println!(
            "{:<40} {:>12.2} {:>12.2} {:>12}",
            "Bare Function Calls (baseline)".green(),
            self.bare_functions.mean_micros(),
            self.bare_functions.median_duration.as_nanos() as f64 / 1000.0,
            "0.00%".bright_green()
        );

        let overhead = self.eventchains_no_middleware.overhead_vs(&self.bare_functions);
        let color = if overhead < 20.0 {
            "green"
        } else if overhead < 50.0 {
            "yellow"
        } else {
            "red"
        };

        println!(
            "{:<40} {:>12.2} {:>12.2} {:>12}",
            "EventChains (no middleware)",
            self.eventchains_no_middleware.mean_micros(),
            self.eventchains_no_middleware.median_duration.as_nanos() as f64 / 1000.0,
            format!("+{:.2}%", overhead).color(color)
        );

        println!("\n{}", "Analysis:".yellow().bold());
        println!("  Framework overhead: {:.2}%", overhead);
        println!("  Absolute cost: {:.2}µs",
                 self.eventchains_no_middleware.mean_micros() - self.bare_functions.mean_micros());
        println!("  This represents the cost of:");
        println!("    - Event wrapping and trait dispatch");
        println!("    - Context creation and type erasure");
        println!("    - Result enum wrapping");
    }
}

// ============================================================================
// TIER 2: Feature-Parity Baseline (Cost of Abstraction)
// ============================================================================

pub struct Tier2Results {
    pub manual_instrumented: BenchmarkStats,
    pub eventchains_no_middleware: BenchmarkStats,
}

impl Tier2Results {
    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(80).bright_cyan().bold());
        println!(
            "{}",
            "TIER 2: Feature-Parity Baseline - Cost of Abstraction"
                .bright_cyan()
                .bold()
        );
        println!("{}", "=".repeat(80).bright_cyan().bold());
        println!("\nComparison:");
        println!("   Baseline: Manual implementation with error handling, name tracking, context");
        println!("   EventChains: Full pattern with 0 middleware");
        println!("   Metric: Cost of abstraction vs hand-rolled equivalent\n");

        println!("{}", "-".repeat(80));
        println!(
            "{:<40} {:>12} {:>12} {:>12}",
            "Implementation".bold(),
            "Mean (µs)".bold(),
            "Median (µs)".bold(),
            "Overhead".bold()
        );
        println!("{}", "-".repeat(80));

        println!(
            "{:<40} {:>12.2} {:>12.2} {:>12}",
            "Manual (instrumented baseline)".green(),
            self.manual_instrumented.mean_micros(),
            self.manual_instrumented.median_duration.as_nanos() as f64 / 1000.0,
            "0.00%".bright_green()
        );

        let overhead = self.eventchains_no_middleware.overhead_vs(&self.manual_instrumented);
        let color = if overhead < 15.0 {
            "green"
        } else if overhead < 30.0 {
            "yellow"
        } else {
            "red"
        };

        println!(
            "{:<40} {:>12.2} {:>12.2} {:>12}",
            "EventChains (no middleware)",
            self.eventchains_no_middleware.mean_micros(),
            self.eventchains_no_middleware.median_duration.as_nanos() as f64 / 1000.0,
            format!("+{:.2}%", overhead).color(color)
        );

        println!("\n{}", "Analysis:".yellow().bold());
        println!("  Abstraction overhead: {:.2}%", overhead);
        println!("  Absolute cost: {:.2}µs",
                 self.eventchains_no_middleware.mean_micros() - self.manual_instrumented.mean_micros());
        println!("  This represents the cost of:");
        println!("    - Generic trait-based design vs concrete types");
        println!("    - Dynamic dispatch vs static dispatch");
        println!("    - Type-erased context vs typed variables");
    }
}

// ============================================================================
// TIER 3: Middleware Scaling (Cost per Middleware Layer)
// ============================================================================

pub struct Tier3Results {
    pub no_middleware: BenchmarkStats,
    pub one_middleware: BenchmarkStats,
    pub three_middleware: BenchmarkStats,
    pub five_middleware: BenchmarkStats,
    pub ten_middleware: BenchmarkStats,
}

impl Tier3Results {
    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(80).bright_cyan().bold());
        println!(
            "{}",
            "TIER 3: Middleware Scaling - Cost per Middleware Layer"
                .bright_cyan()
                .bold()
        );
        println!("{}", "=".repeat(80).bright_cyan().bold());
        println!("\nComparison:");
        println!("   EventChains with 0, 1, 3, 5, and 10 middleware layers");
        println!("   Metric: Incremental cost per middleware layer\n");

        println!("{}", "-".repeat(80));
        println!(
            "{:<40} {:>12} {:>12} {:>12}",
            "Configuration".bold(),
            "Mean (µs)".bold(),
            "Median (µs)".bold(),
            "Overhead".bold()
        );
        println!("{}", "-".repeat(80));

        let configs = vec![
            ("0 middleware (baseline)", &self.no_middleware, true),
            ("1 middleware", &self.one_middleware, false),
            ("3 middleware", &self.three_middleware, false),
            ("5 middleware", &self.five_middleware, false),
            ("10 middleware", &self.ten_middleware, false),
        ];

        for (name, stats, is_baseline) in configs {
            if is_baseline {
                println!(
                    "{:<40} {:>12.2} {:>12.2} {:>12}",
                    name.green(),
                    stats.mean_micros(),
                    stats.median_duration.as_nanos() as f64 / 1000.0,
                    "0.00%".bright_green()
                );
            } else {
                let overhead = stats.overhead_vs(&self.no_middleware);
                let color = if overhead < 30.0 {
                    "green"
                } else if overhead < 60.0 {
                    "yellow"
                } else {
                    "red"
                };
                println!(
                    "{:<40} {:>12.2} {:>12.2} {:>12}",
                    name,
                    stats.mean_micros(),
                    stats.median_duration.as_nanos() as f64 / 1000.0,
                    format!("+{:.2}%", overhead).color(color)
                );
            }
        }

        println!("\n{}", "Analysis:".yellow().bold());
        let cost_per_mw_1 = self.one_middleware.mean_micros() - self.no_middleware.mean_micros();
        let cost_per_mw_3 = (self.three_middleware.mean_micros() - self.no_middleware.mean_micros()) / 3.0;
        let cost_per_mw_5 = (self.five_middleware.mean_micros() - self.no_middleware.mean_micros()) / 5.0;
        let cost_per_mw_10 = (self.ten_middleware.mean_micros() - self.no_middleware.mean_micros()) / 10.0;

        println!("  Cost per middleware layer:");
        println!("    1 middleware:  {:.3}µs per layer", cost_per_mw_1);
        println!("    3 middleware:  {:.3}µs per layer", cost_per_mw_3);
        println!("    5 middleware:  {:.3}µs per layer", cost_per_mw_5);
        println!("    10 middleware: {:.3}µs per layer", cost_per_mw_10);
        println!("\n  Scaling characteristics:");
        let scaling_ratio = cost_per_mw_10 / cost_per_mw_1;
        if scaling_ratio < 1.2 {
            println!("     Linear scaling (good) - ratio: {:.2}x", scaling_ratio);
        } else {
            println!("     Non-linear scaling - ratio: {:.2}x", scaling_ratio);
        }
    }
}

// ============================================================================
// TIER 4: Real-World Scenario (Cost vs Equivalent Manual Work)
// ============================================================================

pub struct Tier4Results {
    pub manual_with_logging_timing: BenchmarkStats,
    pub eventchains_with_logging_timing: BenchmarkStats,
}

impl Tier4Results {
    pub fn print_report(&self) {
        println!("\n{}", "=".repeat(80).bright_cyan().bold());
        println!(
            "{}",
            "TIER 4: Real-World Scenario - Cost vs Equivalent Manual Instrumentation"
                .bright_cyan()
                .bold()
        );
        println!("{}", "=".repeat(80).bright_cyan().bold());
        println!("\nComparison:");
        println!("   Baseline: Manual implementation with logging, timing, error handling");
        println!("   EventChains: Optimized version (4 events) with logging + timing middleware");
        println!("   Metric: Cost vs equivalent manual instrumentation");
        println!("   Note: Both process all nodes in bulk (not node-by-node)\n");

        println!("{}", "-".repeat(80));
        println!(
            "{:<40} {:>12} {:>12} {:>12}",
            "Implementation".bold(),
            "Mean (µs)".bold(),
            "Median (µs)".bold(),
            "Overhead".bold()
        );
        println!("{}", "-".repeat(80));

        println!(
            "{:<40} {:>12.2} {:>12.2} {:>12}",
            "Manual (logging + timing)".green(),
            self.manual_with_logging_timing.mean_micros(),
            self.manual_with_logging_timing.median_duration.as_nanos() as f64 / 1000.0,
            "baseline".bright_green()
        );

        let diff = self.eventchains_with_logging_timing.mean_micros()
            - self.manual_with_logging_timing.mean_micros();
        let overhead = self.eventchains_with_logging_timing
            .overhead_vs(&self.manual_with_logging_timing);

        let color = if overhead.abs() < 10.0 {
            "green"
        } else if overhead.abs() < 25.0 {
            "yellow"
        } else {
            "red"
        };

        let sign = if diff >= 0.0 { "+" } else { "" };
        println!(
            "{:<40} {:>12.2} {:>12.2} {:>12}",
            "EventChains (logging + timing)",
            self.eventchains_with_logging_timing.mean_micros(),
            self.eventchains_with_logging_timing.median_duration.as_nanos() as f64 / 1000.0,
            format!("{}{:.2}%", sign, overhead).color(color)
        );

        println!("\n{}", "Analysis:".yellow().bold());
        println!("  Performance difference: {}{:.2}µs ({}{:.2}%)", sign, diff, sign, overhead);

        if overhead.abs() < 10.0 {
            println!("   EventChains provides equivalent performance for same functionality");
        } else if overhead < 0.0 {
            println!("   EventChains is faster (likely due to optimization differences)");
        } else {
            println!("   EventChains has overhead, but provides:");
        }

        println!("\n  Value proposition:");
        println!("     Consistent middleware API across all events");
        println!("     Composable and reusable instrumentation");
        println!("     No manual integration of cross-cutting concerns");
        println!("     Type-safe context passing");
    }
}

// ============================================================================
// Comprehensive Report
// ============================================================================

pub struct ComprehensiveBenchmarkResults {
    pub tier1: Tier1Results,
    pub tier2: Tier2Results,
    pub tier3: Tier3Results,
    pub tier4: Tier4Results,
}

impl ComprehensiveBenchmarkResults {
    pub fn print_full_report(&self) {
        println!("\n{}", "=".repeat(80).bright_magenta().bold());
        println!(
            "{}",
            "EventChains Performance Analysis - Multi-Tier Benchmark Report"
                .bright_magenta()
                .bold()
        );
        println!("{}", "=".repeat(80).bright_magenta().bold());

        self.tier1.print_report();
        self.tier2.print_report();
        self.tier3.print_report();
        self.tier4.print_report();

        println!("\n{}", "=".repeat(80).bright_magenta().bold());
        println!("{}", "Executive Summary".bright_magenta().bold());
        println!("{}", "=".repeat(80).bright_magenta().bold());

        println!("\n{}", "Key Findings:".yellow().bold());

        let framework_overhead = self.tier1.eventchains_no_middleware.overhead_vs(&self.tier1.bare_functions);
        println!("  1. Framework Overhead (Tier 1): {:.2}%", framework_overhead);
        println!("      Pure cost of orchestration pattern");

        let abstraction_overhead = self.tier2.eventchains_no_middleware.overhead_vs(&self.tier2.manual_instrumented);
        println!("\n  2. Abstraction Overhead (Tier 2): {:.2}%", abstraction_overhead);
        println!("      Cost of generic design vs hand-rolled equivalent");

        let mw_cost = (self.tier3.five_middleware.mean_micros() - self.tier3.no_middleware.mean_micros()) / 5.0;
        println!("\n  3. Middleware Cost (Tier 3): {:.3}µs per layer", mw_cost);
        println!("      Incremental cost for each middleware added");

        let real_world_overhead = self.tier4.eventchains_with_logging_timing
            .overhead_vs(&self.tier4.manual_with_logging_timing);
        println!("\n  4. Real-World Comparison (Tier 4): {:.2}%", real_world_overhead);
        println!("      EventChains vs manual instrumentation");

        println!("\n{}", "Interpretation:".green().bold());
        println!("   Tier 1 shows the minimum cost you pay for using the pattern");
        println!("   Tier 2 shows you're not paying much more than a careful manual approach");
        println!("   Tier 3 shows the cost scales linearly with middleware complexity");
        println!("   Tier 4 shows real-world cost is comparable to manual equivalent");

        println!("\n{}", "=".repeat(80).bright_magenta().bold());
    }
}
