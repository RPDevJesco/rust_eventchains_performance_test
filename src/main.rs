mod benchmarking;
mod dijkstra_eventchains;
mod dijkstra_events;
mod eventchains;
mod graph;
mod middleware;
mod noop_middleware;
mod tier_baselines;

use benchmarking::*;
use colored::*;
use dijkstra_eventchains::*;
use graph::{Graph, NodeId};
use noop_middleware::NoOpMiddleware;
use tier_baselines::*;

use std::sync::Arc;

fn run_tier1_benchmark(graph: Arc<Graph>, source: NodeId, target: NodeId, runs: usize) -> Tier1Results {
    println!("\n{}", "Running Tier 1 Benchmarks...".bright_yellow().bold());

    // Baseline: Bare function calls
    print!("  Benchmarking bare function calls...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let bare_functions = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_tier1_baseline(g, source, target);
        result.distance.is_some()
    });
    println!(" ");

    // EventChains: No middleware
    print!("  Benchmarking EventChains (no middleware)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let eventchains_no_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized(g, source, target);
        result.distance.is_some()
    });
    println!(" ");

    Tier1Results {
        bare_functions,
        eventchains_no_middleware,
    }
}

fn run_tier2_benchmark(graph: Arc<Graph>, source: NodeId, target: NodeId, runs: usize) -> Tier2Results {
    println!("\n{}", "Running Tier 2 Benchmarks...".bright_yellow().bold());

    // Baseline: Manual instrumented
    print!("  Benchmarking manual instrumented...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let manual_instrumented = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_tier2_baseline(g, source, target);
        result.is_ok()
    });
    println!(" ");

    // EventChains: No middleware
    print!("  Benchmarking EventChains (no middleware)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let eventchains_no_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized(g, source, target);
        result.distance.is_some()
    });
    println!(" ");

    Tier2Results {
        manual_instrumented,
        eventchains_no_middleware,
    }
}

fn run_tier3_benchmark(graph: Arc<Graph>, source: NodeId, target: NodeId, runs: usize) -> Tier3Results {
    println!("\n{}", "Running Tier 3 Benchmarks...".bright_yellow().bold());

    // 0 middleware
    print!("  Benchmarking 0 middleware...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let no_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized(g, source, target);
        result.distance.is_some()
    });
    println!(" ");

    // 1 middleware
    print!("  Benchmarking 1 middleware...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let one_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_with_n_middleware(g, source, target, 1);
        result.distance.is_some()
    });
    println!(" ");

    // 3 middleware
    print!("  Benchmarking 3 middleware...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let three_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_with_n_middleware(g, source, target, 3);
        result.distance.is_some()
    });
    println!(" ");

    // 5 middleware
    print!("  Benchmarking 5 middleware...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let five_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_with_n_middleware(g, source, target, 5);
        result.distance.is_some()
    });
    println!(" ");

    // 10 middleware
    print!("  Benchmarking 10 middleware...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let ten_middleware = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_with_n_middleware(g, source, target, 10);
        result.distance.is_some()
    });
    println!(" ");

    Tier3Results {
        no_middleware,
        one_middleware,
        three_middleware,
        five_middleware,
        ten_middleware,
    }
}

fn run_tier4_benchmark(graph: Arc<Graph>, source: NodeId, target: NodeId, runs: usize) -> Tier4Results {
    println!("\n{}", "Running Tier 4 Benchmarks...".bright_yellow().bold());

    // Baseline: Manual with logging and timing
    print!("  Benchmarking manual (logging + timing)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let manual_with_logging_timing = run_benchmark(runs, || {
        let g = graph.clone();
        let (result, _context) = dijkstra_tier4_baseline(g, source, target, false);
        result.distance.is_some()
    });
    println!(" ");

    // EventChains: With logging and timing middleware (optimized version for fair comparison)
    print!("  Benchmarking EventChains (logging + timing)...");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    let eventchains_with_logging_timing = run_benchmark(runs, || {
        let g = graph.clone();
        let result = dijkstra_eventchains_optimized_with_middleware(g, source, target, false);
        result.distance.is_some()
    });
    println!(" ");

    Tier4Results {
        manual_with_logging_timing,
        eventchains_with_logging_timing,
    }
}

fn main() {
    println!("{}", "=".repeat(80).bright_magenta().bold());
    println!(
        "{}",
        "EventChains Multi-Tier Performance Benchmark"
            .bright_magenta()
            .bold()
    );
    println!("{}", "=".repeat(80).bright_magenta().bold());

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
        let tier1 = run_tier1_benchmark(graph.clone(), source, target, runs);
        let tier2 = run_tier2_benchmark(graph.clone(), source, target, runs);
        let tier3 = run_tier3_benchmark(graph.clone(), source, target, runs);
        let tier4 = run_tier4_benchmark(graph.clone(), source, target, runs);

        // Print results
        let results = ComprehensiveBenchmarkResults {
            tier1,
            tier2,
            tier3,
            tier4,
        };

        results.print_full_report();
    }

    println!("\n{}", "=".repeat(80).bright_magenta().bold());
    println!("{}", "All Benchmarks Complete!".bright_green().bold());
    println!("{}", "=".repeat(80).bright_magenta().bold());
}
