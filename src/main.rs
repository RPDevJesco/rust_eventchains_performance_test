mod dijkstra_eventchains;
mod dijkstra_events;
mod dijkstra_traditional;
mod eventchains;
mod graph;
mod middleware;
mod profiling;

use colored::*;
use dijkstra_eventchains::*;
use dijkstra_traditional::*;
use graph::{Graph, NodeId};
use profiling::{benchmark, print_comparison_table};

use std::sync::Arc;

fn main() {
    println!("{}", "═".repeat(80).bright_magenta().bold());
    println!(
        "{}",
        "Dijkstra's Algorithm: EventChains vs Traditional Implementation"
            .bright_magenta()
            .bold()
    );
    println!("{}", "Performance Comparison Study".bright_magenta().bold());
    println!("{}", "═".repeat(80).bright_magenta().bold());

    let test_cases = vec![
        (100, 500, 50),    // Small graph
        (500, 2500, 30),   // Medium graph
        (1000, 5000, 20),  // Large graph
        (2000, 10000, 10), // Extra large graph
    ];

    for (nodes, edges, runs) in test_cases {
        println!(
            "\n\n{}",
            format!("TEST CASE: {} nodes, {} edges", nodes, edges)
                .bright_yellow()
                .bold()
        );
        println!("{}", "═".repeat(80).bright_yellow());

        // Generate graph
        let graph = Arc::new(Graph::random_connected(nodes, edges, 100));
        let source = NodeId(0);
        let target = NodeId(nodes - 1);

        println!("\n{}", "Graph generated successfully!".green());
        println!("  Source node: {}", source.0);
        println!("  Target node: {}", target.0);

        // Run single test with logging first (only for small graphs)
        if nodes <= 100 {
            println!("\n{}", "Sample run with logging:".cyan().bold());
            println!("{}", "─".repeat(80).cyan());

            println!("\n{}", "Traditional Implementation:".green().bold());
            let graph_clone = graph.clone();
            let result_trad = dijkstra_traditional_logged(graph_clone, source, target, true);
            if let Some(dist) = result_trad.distance {
                println!(
                    "  {} Distance: {}, Path length: {}",
                    "✓".green(),
                    dist,
                    result_trad.path.len()
                );
            }

            println!("\n{}", "EventChains Implementation:".green().bold());
            let graph_clone = graph.clone();
            let result_ec = dijkstra_eventchains_full(graph_clone, source, target, true);
            if let Some(dist) = result_ec.distance {
                println!(
                    "  {} Distance: {}, Path length: {}",
                    "✓".green(),
                    dist,
                    result_ec.path.len()
                );
            }

            if result_trad.distance == result_ec.distance {
                println!("\n  {} Results match!", "✓".bright_green().bold());
            } else {
                println!("\n  {} Results don't match!", "✗".bright_red().bold());
            }
        }

        // Benchmark: Traditional
        let stats_traditional = {
            benchmark("Traditional Dijkstra", runs, || {
                let graph_clone = graph.clone();
                let result = dijkstra_traditional(graph_clone, source, target);
                result.distance.is_some()
            })
        };

        // Benchmark: EventChains (bare)
        let stats_bare = {
            benchmark("EventChains (bare - no middleware)", runs, || {
                let graph_clone = graph.clone();
                let result = dijkstra_eventchains_bare(graph_clone, source, target);
                result.distance.is_some()
            })
        };

        // Benchmark: EventChains (full)
        let stats_full = {
            benchmark("EventChains (full middleware)", runs, || {
                let graph_clone = graph.clone();
                let result = dijkstra_eventchains_full(graph_clone, source, target, false);
                result.distance.is_some()
            })
        };

        // Benchmark: EventChains (optimized)
        let stats_optimized = {
            benchmark("EventChains (optimized)", runs, || {
                let graph_clone = graph.clone();
                let result = dijkstra_eventchains_optimized(graph_clone, source, target);
                result.distance.is_some()
            })
        };

        print_comparison_table(
            (nodes, edges),
            &stats_traditional,
            &stats_bare,
            &stats_full,
            &stats_optimized,
        );

        println!("\n{}", "Complexity Analysis:".yellow().bold());
        println!("  Time Complexity:  O(E log V) for all implementations");
        println!("  Space Complexity: O(V) for all implementations");
        println!("  Event Count (bare):     {} events per run", nodes + 3);
        println!("  Event Count (optimized): 4 events per run");
        println!("  Context Lookups (bare):  ~{} lookups per run", nodes * 4);
        println!(
            "  Middleware Calls (full): ~{} calls per run",
            (nodes + 3) * 3
        );
    }

    println!("\n{}", "═".repeat(80).bright_magenta().bold());
    println!("{}", "Benchmark Complete!".bright_green().bold());
    println!("{}", "═".repeat(80).bright_magenta().bold());
}
