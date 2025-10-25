use crate::dijkstra_events::*;
use crate::eventchains::{EventChain, EventContext, FaultToleranceMode};
use crate::graph::{Graph, NodeId, ShortestPathResult};
use crate::middleware::{LoggingMiddleware, PerformanceMiddleware, TimingMiddleware};

use std::sync::Arc;
use crate::noop_middleware::NoOpMiddleware;

/// Run Dijkstra using EventChains pattern (bare - no middleware)
pub fn dijkstra_eventchains_bare(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set("graph", graph);

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
        context.get("result").unwrap()
    } else {
        ShortestPathResult {
            source,
            target,
            distance: None,
            path: Vec::new(),
        }
    }
}

/// Run Dijkstra using EventChains pattern with full middleware
pub fn dijkstra_eventchains_full(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    verbose: bool,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set("graph", graph);

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
        context.get("result").unwrap()
    } else {
        ShortestPathResult {
            source,
            target,
            distance: None,
            path: Vec::new(),
        }
    }
}

/// Run Dijkstra using a more efficient EventChains approach
/// This version uses fewer events by processing multiple nodes per event
pub fn dijkstra_eventchains_optimized(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set("graph", graph);

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
        context.get("result").unwrap()
    } else {
        ShortestPathResult {
            source,
            target,
            distance: None,
            path: Vec::new(),
        }
    }
}

/// Run Dijkstra using optimized EventChains with logging and timing middleware
/// This is for fair comparison in Tier 4 benchmarks
pub fn dijkstra_eventchains_optimized_with_middleware(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    verbose: bool,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set("graph", graph);

    let mut chain = EventChain::new().with_fault_tolerance(FaultToleranceMode::Strict);

    // Add middleware (same as full version)
    chain.use_middleware(Box::new(TimingMiddleware::new(verbose)));
    chain.use_middleware(Box::new(LoggingMiddleware::new(verbose)));

    // Add events (using optimized version - only 4 events total)
    chain.add_event(Box::new(InitializeStateEvent::new(source, node_count)));
    chain.add_event(Box::new(InitializePriorityQueueEvent));
    chain.add_event(Box::new(ProcessAllNodesEvent));
    chain.add_event(Box::new(FinalizeResultEvent::new(target)));

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.get("result").unwrap()
    } else {
        ShortestPathResult {
            source,
            target,
            distance: None,
            path: Vec::new(),
        }
    }
}

pub fn dijkstra_eventchains_with_n_middleware(
    graph: Arc<Graph>,
    source: NodeId,
    target: NodeId,
    n: usize,
) -> ShortestPathResult {
    let mut context = EventContext::new();
    let node_count = graph.nodes;
    context.set("graph", graph);

    let mut chain = EventChain::new()
        .with_fault_tolerance(FaultToleranceMode::Strict);

    // Add n no-op middleware layers
    for i in 0..n {
        chain.use_middleware(Box::new(NoOpMiddleware::new(i)));
    }

    // Add events (using optimized version)
    chain.add_event(Box::new(InitializeStateEvent::new(source, node_count)));
    chain.add_event(Box::new(InitializePriorityQueueEvent));
    chain.add_event(Box::new(ProcessAllNodesEvent));
    chain.add_event(Box::new(FinalizeResultEvent::new(target)));

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.get("result").unwrap()
    } else {
        ShortestPathResult {
            source,
            target,
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
        let queue: BinaryHeap<QueueNode> = match context.get("queue") {
            Some(q) => q,
            None => return EventResult::Failure("Queue not found".to_string()),
        };

        let mut state: DijkstraState = match context.get("state") {
            Some(s) => s,
            None => return EventResult::Failure("State not found".to_string()),
        };

        let graph: Arc<Graph> = match context.get("graph") {
            Some(g) => g,
            None => return EventResult::Failure("Graph not found".to_string()),
        };

        let mut queue = queue;

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

        context.set("state", state);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "ProcessAllNodes"
    }
}
