use crate::graph::{DijkstraState, Graph, NodeId, QueueNode, ShortestPathResult};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Instant;

// ============================================================================
// TIER 1 BASELINE: Bare Function Calls (Minimal)
// ============================================================================

/// Three direct function calls with no orchestration
pub fn dijkstra_tier1_baseline(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
) -> ShortestPathResult {
    // Step 1: Initialize state
    let mut state = DijkstraState::new(graph.nodes, source);
    
    // Step 2: Initialize queue
    let mut queue = BinaryHeap::new();
    queue.push(QueueNode {
        node: source,
        distance: 0,
    });
    
    // Step 3: Process all nodes
    while let Some(QueueNode { node, distance }) = queue.pop() {
        if state.visited[node.0] || distance > state.distances[node.0] {
            continue;
        }

        state.visited[node.0] = true;

        for edge in &graph.adjacency_list[node.0] {
            let new_distance = distance.saturating_add(edge.weight);

            if new_distance < state.distances[edge.to.0] {
                state.distances[edge.to.0] = new_distance;
                state.predecessors[edge.to.0] = Some(node);

                queue.push(QueueNode {
                    node: edge.to,
                    distance: new_distance,
                });
            }
        }
    }
    
    // Step 4: Finalize result
    ShortestPathResult::reconstruct_path(&state, source, target)
}

// ============================================================================
// TIER 2 BASELINE: Manual Instrumented (Feature-Parity)
// ============================================================================

/// Manual implementation with error handling, step tracking, and context
pub fn dijkstra_tier2_baseline(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
) -> Result<ShortestPathResult, String> {
    // Simulate context with manual tracking
    let mut step_names = Vec::new();
    let mut step_results = Vec::new();
    
    // Step 1: Initialize state
    step_names.push("InitializeState");
    let state = DijkstraState::new(graph.nodes, source);
    step_results.push(Ok(()));
    
    // Step 2: Initialize queue
    step_names.push("InitializePriorityQueue");
    let mut queue = BinaryHeap::new();
    queue.push(QueueNode {
        node: source,
        distance: 0,
    });
    step_results.push(Ok(()));
    
    // Step 3: Process all nodes
    step_names.push("ProcessAllNodes");
    let mut state = state;
    let result = process_nodes_with_validation(&graph, &mut state, &mut queue);
    step_results.push(result);
    
    // Check for errors
    for (i, result) in step_results.iter().enumerate() {
        if let Err(e) = result {
            return Err(format!("Step '{}' failed: {}", step_names[i], e));
        }
    }
    
    // Step 4: Finalize result
    step_names.push("FinalizeResult");
    Ok(ShortestPathResult::reconstruct_path(&state, source, target))
}

fn process_nodes_with_validation(
    graph: &Graph,
    state: &mut DijkstraState,
    queue: &mut BinaryHeap<QueueNode>,
) -> Result<(), String> {
    while let Some(QueueNode { node, distance }) = queue.pop() {
        if state.visited[node.0] || distance > state.distances[node.0] {
            continue;
        }

        state.visited[node.0] = true;

        for edge in &graph.adjacency_list[node.0] {
            let new_distance = distance.saturating_add(edge.weight);

            if new_distance < state.distances[edge.to.0] {
                state.distances[edge.to.0] = new_distance;
                state.predecessors[edge.to.0] = Some(node);

                queue.push(QueueNode {
                    node: edge.to,
                    distance: new_distance,
                });
            }
        }
    }
    Ok(())
}

// ============================================================================
// TIER 4 BASELINE: Manual with Logging and Timing
// ============================================================================

pub struct ManualLoggingContext {
    pub timings: Vec<(&'static str, u128)>,
    pub logs: Vec<String>,
}

impl ManualLoggingContext {
    pub fn new() -> Self {
        Self {
            timings: Vec::new(),
            logs: Vec::new(),
        }
    }
    
    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }
    
    pub fn record_timing(&mut self, name: &'static str, duration_nanos: u128) {
        self.timings.push((name, duration_nanos));
    }
}

/// Manual implementation with logging and timing (equivalent to EventChains with middleware)
pub fn dijkstra_tier4_baseline(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    logging_enabled: bool,
) -> (ShortestPathResult, ManualLoggingContext) {
    let mut context = ManualLoggingContext::new();
    
    // Step 1: Initialize state
    let start = Instant::now();
    if logging_enabled {
        context.log("▶ InitializeState starting".to_string());
    }
    
    let mut state = DijkstraState::new(graph.nodes, source);
    
    let duration = start.elapsed().as_nanos();
    context.record_timing("InitializeState", duration);
    if logging_enabled {
        context.log(format!("  ✓ InitializeState completed ({}μs)", duration / 1000));
    }
    
    // Step 2: Initialize queue
    let start = Instant::now();
    if logging_enabled {
        context.log("▶ InitializePriorityQueue starting".to_string());
    }
    
    let mut queue = BinaryHeap::new();
    queue.push(QueueNode {
        node: source,
        distance: 0,
    });
    
    let duration = start.elapsed().as_nanos();
    context.record_timing("InitializePriorityQueue", duration);
    if logging_enabled {
        context.log(format!("  ✓ InitializePriorityQueue completed ({}μs)", duration / 1000));
    }
    
    // Step 3: Process all nodes
    let start = Instant::now();
    if logging_enabled {
        context.log("▶ ProcessAllNodes starting".to_string());
    }
    
    while let Some(QueueNode { node, distance }) = queue.pop() {
        if state.visited[node.0] || distance > state.distances[node.0] {
            continue;
        }

        state.visited[node.0] = true;

        for edge in &graph.adjacency_list[node.0] {
            let new_distance = distance.saturating_add(edge.weight);

            if new_distance < state.distances[edge.to.0] {
                state.distances[edge.to.0] = new_distance;
                state.predecessors[edge.to.0] = Some(node);

                queue.push(QueueNode {
                    node: edge.to,
                    distance: new_distance,
                });
            }
        }
    }
    
    let duration = start.elapsed().as_nanos();
    context.record_timing("ProcessAllNodes", duration);
    if logging_enabled {
        context.log(format!("  ✓ ProcessAllNodes completed ({}μs)", duration / 1000));
    }
    
    // Step 4: Finalize result
    let start = Instant::now();
    if logging_enabled {
        context.log("▶ FinalizeResult starting".to_string());
    }
    
    let result = ShortestPathResult::reconstruct_path(&state, source, target);
    
    let duration = start.elapsed().as_nanos();
    context.record_timing("FinalizeResult", duration);
    if logging_enabled {
        context.log(format!("  ✓ FinalizeResult completed ({}μs)", duration / 1000));
    }
    
    (result, context)
}

impl Default for ManualLoggingContext {
    fn default() -> Self {
        Self::new()
    }
}
