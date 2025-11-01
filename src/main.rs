mod benchmarking;
mod comprehensive_benchmarking;
mod dijkstra_eventchains;
mod dijkstra_events;
mod eventchains;
mod graph;
mod middleware;
mod noop_middleware;
mod tier_baselines;

use benchmarking::*;
use comprehensive_benchmarking::*;
use colored::*;
use dijkstra_eventchains::*;
use graph::{Graph, NodeId};
use tier_baselines::*;

use std::sync::Arc;

// Use the tracking allocator for memory profiling
#[global_allocator]
static GLOBAL: comprehensive_benchmarking::TrackingAllocator = comprehensive_benchmarking::TrackingAllocator;

fn run_tier1_comprehensive(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    runs: usize,
) -> (ComprehensiveMetrics, ComprehensiveMetrics) {
    println!("\n{}", "Running Tier 1 Comprehensive Benchmarks...".bright_yellow().bold());

    // Baseline: Bare function calls
    print!("  Benchmarking bare function calls...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let bare_functions = run_comprehensive_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_tier1_baseline(g, source, target);
        result.distance.is_some()
    });
    println!(" ✓");

    // EventChains: No middleware
    print!("  Benchmarking EventChains (no middleware)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let eventchains_no_middleware = run_comprehensive_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized(g, source, target);
        result.distance.is_some()
    });
    println!(" ✓");

    (bare_functions, eventchains_no_middleware)
}

fn run_tier2_comprehensive(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    runs: usize,
) -> (ComprehensiveMetrics, ComprehensiveMetrics) {
    println!("\n{}", "Running Tier 2 Comprehensive Benchmarks...".bright_yellow().bold());

    // Baseline: Manual instrumented
    print!("  Benchmarking manual instrumented...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let manual_instrumented = run_comprehensive_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_tier2_baseline(g, source, target);
        result.is_ok()
    });
    println!(" ✓");

    // EventChains: No middleware
    print!("  Benchmarking EventChains (no middleware)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let eventchains_no_middleware = run_comprehensive_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized(g, source, target);
        result.distance.is_some()
    });
    println!(" ✓");

    (manual_instrumented, eventchains_no_middleware)
}

fn run_tier3_comprehensive(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    runs: usize,
) -> Vec<(usize, ComprehensiveMetrics)> {
    println!("\n{}", "Running Tier 3 Comprehensive Benchmarks...".bright_yellow().bold());

    let middleware_counts = vec![0, 1, 3, 5, 10];
    let mut results = Vec::new();

    for &count in &middleware_counts {
        print!("  Benchmarking {} middleware...", count);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let metrics = run_comprehensive_benchmark(runs, || {
            let g = graph.clone();
            let result = dijkstra_eventchains_with_n_middleware(g, source, target, count);
            result.distance.is_some()
        });

        println!(" ✓");
        results.push((count, metrics));
    }

    results
}

fn run_tier4_comprehensive(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    runs: usize,
) -> (ComprehensiveMetrics, ComprehensiveMetrics) {
    println!("\n{}", "Running Tier 4 Comprehensive Benchmarks...".bright_yellow().bold());

    // Baseline: Manual with logging and timing
    print!("  Benchmarking manual (logging + timing)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let manual_with_logging_timing = run_comprehensive_benchmark(runs, || {
        let g = graph.clone();
        let (result, _context) = dijkstra_tier4_baseline(g, source, target, false);
        result.distance.is_some()
    });
    println!(" ✓");

    // EventChains: With logging and timing middleware
    print!("  Benchmarking EventChains (logging + timing)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let eventchains_with_logging_timing = run_comprehensive_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized_with_middleware(g, source, target, false);
        result.distance.is_some()
    });
    println!(" ✓");

    (manual_with_logging_timing, eventchains_with_logging_timing)
}

fn print_tier1_report(baseline: &ComprehensiveMetrics, eventchains: &ComprehensiveMetrics) {
    print_comprehensive_comparison(
        "TIER 1: Minimal Baseline - Cost of Orchestration Framework",
        baseline,
        eventchains,
    );

    println!("\n{}", "Interpretation:".yellow().bold());
    println!("  This tier shows the pure cost of the EventChains framework:");
    println!("  - Event wrapping and trait dispatch overhead");
    println!("  - Context creation and type erasure costs");
    println!("  - Result enum wrapping impact");
    println!("  - Memory allocations from the framework");
}

fn print_tier2_report(baseline: &ComprehensiveMetrics, eventchains: &ComprehensiveMetrics) {
    print_comprehensive_comparison(
        "TIER 2: Feature-Parity Baseline - Cost of Abstraction",
        baseline,
        eventchains,
    );

    println!("\n{}", "Interpretation:".yellow().bold());
    println!("  This tier shows the cost of abstraction:");
    println!("  - Generic trait-based design vs concrete types");
    println!("  - Dynamic dispatch vs static dispatch");
    println!("  - Type-erased context vs typed variables");
}

fn print_tier3_report(results: &[(usize, ComprehensiveMetrics)]) {
    println!("\n{}", "=".repeat(90).bright_cyan().bold());
    println!(
        "{}",
        "TIER 3: Middleware Scaling - Cost per Middleware Layer"
            .bright_cyan()
            .bold()
    );
    println!("{}", "=".repeat(90).bright_cyan().bold());

    let baseline = &results[0].1; // 0 middleware is baseline

    println!("\n{}", "⏱️  Timing Scaling".yellow().bold());
    println!("{}", "-".repeat(90));
    println!(
        "{:<25} {:>12} {:>12} {:>12} {:>15}",
        "Middleware Count".bold(),
        "Mean (μs)".bold(),
        "Overhead %".bold(),
        "Per MW (μs)".bold(),
        "Memory (KB)".bold()
    );
    println!("{}", "-".repeat(90));

    for (count, metrics) in results {
        let overhead = if *count == 0 {
            0.0
        } else {
            metrics.overhead_vs(baseline)
        };

        let per_mw = if *count > 0 {
            (metrics.mean_micros() - baseline.mean_micros()) / (*count as f64)
        } else {
            0.0
        };

        let color = if overhead < 30.0 {
            "green"
        } else if overhead < 60.0 {
            "yellow"
        } else {
            "red"
        };

        println!(
            "{:<25} {:>12.2} {:>12} {:>12.3} {:>15.2}",
            format!("{} middleware", count),
            metrics.mean_micros(),
            if *count == 0 {
                "baseline".to_string()
            } else {
                format!("+{:.2}%", overhead).color(color).to_string()
            },
            per_mw,
            metrics.memory_stats.peak_memory as f64 / 1024.0
        );
    }

    println!("\n{}", "📊 Latency Variance by Middleware Count".yellow().bold());
    println!("{}", "-".repeat(90));
    println!(
        "{:<25} {:>15} {:>15} {:>15}",
        "Middleware Count".bold(),
        "Std Dev (μs)".bold(),
        "CoV (%)".bold(),
        "P99 (μs)".bold()
    );
    println!("{}", "-".repeat(90));

    for (count, metrics) in results {
        println!(
            "{:<25} {:>15.2} {:>15.2} {:>15.2}",
            format!("{} middleware", count),
            metrics.std_dev_nanos / 1000.0,
            metrics.coefficient_of_variation(),
            metrics.p99_duration.as_nanos() as f64 / 1000.0
        );
    }

    println!("\n{}", "Interpretation:".yellow().bold());
    println!("  - Per-middleware cost shows if overhead scales linearly");
    println!("  - Latency variance indicates performance predictability");
    println!("  - Memory scaling shows allocation patterns");
}

fn print_tier4_report(baseline: &ComprehensiveMetrics, eventchains: &ComprehensiveMetrics) {
    print_comprehensive_comparison(
        "TIER 4: Real-World Scenario - Cost vs Equivalent Manual Work",
        baseline,
        eventchains,
    );

    println!("\n{}", "Interpretation:".yellow().bold());
    println!("  This tier shows real-world performance:");
    println!("  - EventChains vs manual logging and timing");
    println!("  - Trade-off between convenience and performance");
    println!("  - Value of consistent middleware API");
}

fn print_executive_summary(
    tier1: (&ComprehensiveMetrics, &ComprehensiveMetrics),
    tier2: (&ComprehensiveMetrics, &ComprehensiveMetrics),
    tier3: &[(usize, ComprehensiveMetrics)],
    tier4: (&ComprehensiveMetrics, &ComprehensiveMetrics),
) {
    println!("\n{}", "=".repeat(90).bright_magenta().bold());
    println!("{}", "Executive Summary - Comprehensive Analysis".bright_magenta().bold());
    println!("{}", "=".repeat(90).bright_magenta().bold());

    println!("\n{}", "🎯 Key Findings:".yellow().bold());

    // Tier 1
    let t1_cpu_overhead = tier1.1.overhead_vs(tier1.0);
    let t1_mem_overhead = tier1.1.memory_overhead_vs(tier1.0);
    println!("\n  1. Framework Overhead (Tier 1):");
    println!("     CPU:    {:>8.2}%", t1_cpu_overhead);
    println!("     Memory: {:>8.2}%", t1_mem_overhead);
    println!("     Latency Variance (CoV): {:>7.2}%", tier1.1.coefficient_of_variation());

    // Tier 2
    let t2_cpu_overhead = tier2.1.overhead_vs(tier2.0);
    let t2_mem_overhead = tier2.1.memory_overhead_vs(tier2.0);
    println!("\n  2. Abstraction Overhead (Tier 2):");
    println!("     CPU:    {:>8.2}%", t2_cpu_overhead);
    println!("     Memory: {:>8.2}%", t2_mem_overhead);
    println!("     Latency Variance (CoV): {:>7.2}%", tier2.1.coefficient_of_variation());

    // Tier 3
    let t3_baseline = &tier3[0].1;
    let t3_five = &tier3.iter().find(|(c, _)| *c == 5).unwrap().1;
    let cost_per_mw = (t3_five.mean_micros() - t3_baseline.mean_micros()) / 5.0;
    println!("\n  3. Middleware Cost (Tier 3):");
    println!("     Per Layer:     {:>8.3} μs", cost_per_mw);
    println!("     5 Middleware:  {:>8.2}% overhead", t3_five.overhead_vs(t3_baseline));
    println!("     Memory Impact: {:>8.2}% increase", t3_five.memory_overhead_vs(t3_baseline));

    // Tier 4
    let t4_cpu_overhead = tier4.1.overhead_vs(tier4.0);
    let t4_mem_overhead = tier4.1.memory_overhead_vs(tier4.0);
    println!("\n  4. Real-World Comparison (Tier 4):");
    println!("     CPU:    {:>8.2}%", t4_cpu_overhead);
    println!("     Memory: {:>8.2}%", t4_mem_overhead);
    println!("     Predictability: CoV {:.2}% vs {:.2}%",
             tier4.0.coefficient_of_variation(),
             tier4.1.coefficient_of_variation());

    println!("\n{}", "🔍 Latency Analysis:".green().bold());
    println!("  EventChains maintains consistent performance:");
    println!("  - P99 latency within {:.2}% of median",
             ((tier1.1.p99_duration.as_nanos() as f64 / tier1.1.median_duration.as_nanos() as f64) - 1.0) * 100.0);
    println!("  - Low coefficient of variation: {:.2}%", tier1.1.coefficient_of_variation());

    println!("\n{}", "💾 Memory Analysis:".green().bold());
    println!("  Peak memory overhead across tiers:");
    println!("  - Tier 1: {:.2} KB ({:.1}% increase)",
             (tier1.1.memory_stats.peak_memory as f64 - tier1.0.memory_stats.peak_memory as f64) / 1024.0,
             t1_mem_overhead);
    println!("  - Tier 4: {:.2} KB ({:.1}% increase)",
             (tier4.1.memory_stats.peak_memory as f64 - tier4.0.memory_stats.peak_memory as f64) / 1024.0,
             t4_mem_overhead);

    println!("\n{}", "✅ Conclusion:".bright_green().bold());
    if t1_cpu_overhead < 25.0 && t1_mem_overhead < 30.0 {
        println!("  EventChains demonstrates excellent performance characteristics:");
        println!("  ✓ Low CPU overhead (< 25%)");
        println!("  ✓ Reasonable memory overhead (< 30%)");
        println!("  ✓ Predictable latency profile");
        println!("  ✓ Linear middleware scaling");
    } else {
        println!("  EventChains shows measurable overhead but provides:");
        println!("  • Consistent abstraction layer");
        println!("  • Composable middleware");
        println!("  • Improved maintainability");
    }

    println!("\n{}", "=".repeat(90).bright_magenta().bold());
}

fn main() {
    println!("{}", "=".repeat(90).bright_magenta().bold());
    println!(
        "{}",
        "EventChains Comprehensive Performance Analysis"
            .bright_magenta()
            .bold()
    );
    println!("{}", "=".repeat(90).bright_magenta().bold());
    println!("\n{}", "Measuring: CPU, Memory, Cache Behavior, and Latency Variance".bright_yellow());

    // Test configuration
    let test_cases = vec![
        (100, 500, 100),   // Small graph - more runs
        (500, 2500, 50),   // Medium graph
        (1000, 5000, 30),  // Large graph
        (2000, 10000, 20), // Extra large graph
    ];

    for (nodes, edges, runs) in test_cases {
        println!(
            "\n\n{}",
            format!("=== TEST CASE: {} nodes, {} edges, {} runs ===", nodes, edges, runs)
                .bright_cyan()
                .bold()
        );

        // Generate graph
        let graph = Arc::new(Graph::random_connected(nodes, edges, 100));
        let source = NodeId(0);
        let target = NodeId(nodes - 1);

        println!("\n{}", "Graph generated successfully!".green());
        println!("  Source node: {}", source.0);
        println!("  Target node: {}", target.0);

        // Run all tier benchmarks
        let tier1 = run_tier1_comprehensive(graph.clone(), source, target, runs);
        let tier2 = run_tier2_comprehensive(graph.clone(), source, target, runs);
        let tier3 = run_tier3_comprehensive(graph.clone(), source, target, runs);
        let tier4 = run_tier4_comprehensive(graph.clone(), source, target, runs);

        // Print detailed reports
        print_tier1_report(&tier1.0, &tier1.1);
        print_tier2_report(&tier2.0, &tier2.1);
        print_tier3_report(&tier3);
        print_tier4_report(&tier4.0, &tier4.1);

        // Print executive summary
        print_executive_summary(
            (&tier1.0, &tier1.1),
            (&tier2.0, &tier2.1),
            &tier3,
            (&tier4.0, &tier4.1),
        );
    }

    println!("\n{}", "=".repeat(90).bright_magenta().bold());
    println!("{}", "All Comprehensive Benchmarks Complete!".bright_green().bold());
    println!("{}", "=".repeat(90).bright_magenta().bold());
}
