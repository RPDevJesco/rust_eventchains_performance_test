use crate::dijkstra_events::*;
use crate::eventchains::{EventChain, EventContext, FaultToleranceMode};
use crate::graph::{Graph, NodeId, ShortestPathResult};
use crate::middleware::{LoggingMiddleware, PerformanceMiddleware, TimingMiddleware};

use std::sync::Arc;

/// Run Dijkstra using EventChains pattern (bare - no middleware)
pub fn dijkstra_eventchains_bare(
    graph: &Graph,
    source: &NodeId,
    target: &NodeId,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set_graph(graph);

    let mut chain = EventChain::new().with_fault_tolerance(FaultToleranceMode::Strict);

    chain.add_event(Box::new(InitializeStateEvent::new(source, node_count)));
    chain.add_event(Box::new(InitializePriorityQueueEvent));

    // Process nodes in a loop-like fashion
    // This is a limitation of the pattern for inherently iterative algorithms
    for _ in 0..node_count {
        chain.add_event(Box::new(ProcessNodeEvent));
    }

    chain.add_event(Box::new(FinalizeResultEvent::new(target)));

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.take_result().unwrap()
    } else {
        ShortestPathResult {
            source: *source,
            target: *target,
            distance: None,
            path: Vec::new(),
        }
    }
}

/// Run Dijkstra using EventChains pattern with full middleware
pub fn dijkstra_eventchains_full(
    graph: &Graph,
    source: &NodeId,
    target: &NodeId,
    verbose: bool,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set_graph(graph);

    let mut chain = EventChain::new().with_fault_tolerance(FaultToleranceMode::Strict);

    // Add middleware (reverse order of execution)
    let perf_middleware = PerformanceMiddleware::new();
    chain.use_middleware(Box::new(perf_middleware));
    chain.use_middleware(Box::new(TimingMiddleware::new(verbose)));
    chain.use_middleware(Box::new(LoggingMiddleware::new(verbose)));

    // Add events
    chain.add_event(Box::new(InitializeStateEvent::new(source, node_count)));
    chain.add_event(Box::new(InitializePriorityQueueEvent));

    // Process nodes
    for _ in 0..node_count {
        chain.add_event(Box::new(ProcessNodeEvent));
    }

    chain.add_event(Box::new(FinalizeResultEvent::new(target)));

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.take_result().unwrap()
    } else {
        ShortestPathResult {
            source: *source,
            target: *target,
            distance: None,
            path: Vec::new(),
        }
    }
}

/// Run Dijkstra using a more efficient EventChains approach
/// This version uses fewer events by processing multiple nodes per event
pub fn dijkstra_eventchains_optimized(
    graph: &Graph,
    source: &NodeId,
    target: &NodeId,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set_graph(graph);

    let mut chain = EventChain::new().with_fault_tolerance(FaultToleranceMode::Strict);

    // Add events
    chain.add_event(Box::new(InitializeStateEvent::new(source, node_count)));
    chain.add_event(Box::new(InitializePriorityQueueEvent));

    // Use a single "process all nodes" event
    chain.add_event(Box::new(ProcessAllNodesEvent));
    chain.add_event(Box::new(FinalizeResultEvent::new(target)));

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.take_result().unwrap()
    } else {
        ShortestPathResult {
            source: *source,
            target: *target,
            distance: None,
            path: Vec::new(),
        }
    }
}

/// Event that processes all nodes in one go (more efficient)
struct ProcessAllNodesEvent;

impl crate::eventchains::ChainableEvent for ProcessAllNodesEvent {
    fn execute(&self, context: &mut EventContext) -> crate::eventchains::EventResult<()> {
        use crate::eventchains::EventResult;
        use crate::graph::{DijkstraState, Graph, QueueNode};
        use std::collections::BinaryHeap;

        // Get references from context - note: these will be cloned
        let mut queue: BinaryHeap<QueueNode> = match context.take_queue() {
            Some(q) => q,
            None => return EventResult::Failure("Queue not found".to_string()),
        };

        let mut state: DijkstraState = match context.take_state() {
            Some(s) => s,
            None => return EventResult::Failure("State not found".to_string()),
        };

        let graph: &Graph = match context.get_graph() {
            Some(g) => g,
            None => return EventResult::Failure("Graph not found".to_string()),
        };

        while let Some(QueueNode { node, distance }) = queue.pop() {
            if state.visited[node.0] || distance > state.distances[node.0] {
                continue;
            }

            state.visited[node.0] = true;

            graph.adjacency_list[node.0].iter().for_each(|edge| {
                let new_distance = distance.saturating_add(edge.weight);

                if new_distance < state.distances[edge.to.0] {
                    state.distances[edge.to.0] = new_distance;
                    state.predecessors[edge.to.0] = Some(node);

                    queue.push(QueueNode {
                        node: edge.to,
                        distance: new_distance,
                    });
                }
            })
        }

        context.set_state(state);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "ProcessAllNodes"
    }
}
