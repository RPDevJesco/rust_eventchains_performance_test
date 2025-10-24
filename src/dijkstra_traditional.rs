use crate::graph::{DijkstraState, Graph, NodeId, QueueNode, ShortestPathResult};
use std::collections::BTreeSet;
use std::sync::Arc;

/// Traditional implementation of Dijkstra's algorithm
pub fn dijkstra_traditional(
    graph: Arc<Graph>,
    source: &NodeId,
    target: &NodeId,
) -> ShortestPathResult {
    let mut state = DijkstraState::new(graph.nodes, source);
    let mut queue = BTreeSet::new();

    queue.insert(QueueNode {
        node: *source,
        distance: 0,
    });

    while let Some(QueueNode { node, distance }) = queue.pop_last() {
        if state.visited[node.0] || distance > state.distances[node.0] {
            continue;
        }

        state.visited[node.0] = true;

        if &node == target {
            break;
        }

        for edge in &graph.adjacency_list[node.0] {
            let new_distance = distance.saturating_add(edge.weight);

            if new_distance < state.distances[edge.to.0] {
                state.distances[edge.to.0] = new_distance;
                state.predecessors[edge.to.0] = Some(node);

                queue.insert(QueueNode {
                    node: edge.to,
                    distance: new_distance,
                });
            }
        }
    }

    ShortestPathResult::reconstruct_path(&state, &source, &target)
}

/// Traditional implementation with logging
pub fn dijkstra_traditional_logged(
    graph: Arc<Graph>,
    source: &NodeId,
    target: &NodeId,
    verbose: bool,
) -> ShortestPathResult {
    if verbose {
        println!("  ▶ InitializeState starting");
    }

    let mut state = DijkstraState::new(graph.nodes, source);
    let mut queue = BTreeSet::new();

    if verbose {
        println!("    ✓ InitializeState completed");
        println!("  ▶ InitializePriorityQueue starting");
    }

    queue.insert(QueueNode {
        node: *source,
        distance: 0,
    });

    if verbose {
        println!("    ✓ InitializePriorityQueue completed");
    }

    let mut nodes_processed = 0;

    while let Some(QueueNode { node, distance }) = queue.pop_last() {
        if verbose && nodes_processed == 0 {
            println!("  ▶ ProcessNode starting");
        }

        if state.visited[node.0] || distance > state.distances[node.0] {
            continue;
        }

        state.visited[node.0] = true;
        nodes_processed += 1;

        if &node == target {
            break;
        }

        for edge in &graph.adjacency_list[node.0] {
            let new_distance = distance.saturating_add(edge.weight);

            if new_distance < state.distances[edge.to.0] {
                state.distances[edge.to.0] = new_distance;
                state.predecessors[edge.to.0] = Some(node);

                queue.insert(QueueNode {
                    node: edge.to,
                    distance: new_distance,
                });
            }
        }
    }

    if verbose {
        println!(
            "    ✓ ProcessNode completed ({} nodes processed)",
            nodes_processed
        );
        println!("  ▶ FinalizeResult starting");
    }

    let result = ShortestPathResult::reconstruct_path(&state, &source, &target);

    if verbose {
        println!("    ✓ FinalizeResult completed");
    }

    result
}
