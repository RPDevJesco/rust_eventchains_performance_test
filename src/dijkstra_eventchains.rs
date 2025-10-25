use crate::dijkstra_events::*;
use crate::eventchains::{EventChain, EventContext, FaultToleranceMode};
use crate::graph::{Graph, NodeId, ShortestPathResult};
use crate::middleware::{LoggingMiddleware, PerformanceMiddleware, TimingMiddleware};

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

    let init_state = InitializeStateEvent::new(source, node_count);
    chain.add_event(&init_state);
    let init_queue = InitializePriorityQueueEvent;
    chain.add_event(&init_queue);

    // Process nodes in a loop-like fashion
    // This is a limitation of the pattern for inherently iterative algorithms
    for _ in 0..node_count {
        chain.add_event(&ProcessNodeEvent);
    }

    let finalize = FinalizeResultEvent::new(target);
    chain.add_event(&finalize);

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.get_result().unwrap().clone()
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
    chain.use_middleware(&perf_middleware);

    let timing_middle = TimingMiddleware::new(verbose);
    chain.use_middleware(&timing_middle);
    let logging_middle = LoggingMiddleware::new(verbose);
    chain.use_middleware(&logging_middle);

    // Add events
    let init_state = InitializeStateEvent::new(source, node_count);
    chain.add_event(&init_state);
    let init_queue = InitializePriorityQueueEvent;
    chain.add_event(&init_queue);

    // Process nodes
    for _ in 0..node_count {
        chain.add_event(&ProcessNodeEvent);
    }

    let finalize = FinalizeResultEvent::new(target);
    chain.add_event(&finalize);

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.get_result().unwrap().clone()
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
    let init_state = InitializeStateEvent::new(source, node_count);
    chain.add_event(&init_state);
    let init_queue = InitializePriorityQueueEvent;
    chain.add_event(&init_queue);

    // Use a single "process all nodes" event
    let process_all_nodes = ProcessAllNodesEvent;
    chain.add_event(&process_all_nodes);
    let finalize = FinalizeResultEvent::new(target);
    chain.add_event(&finalize);

    // Execute chain
    let result = chain.execute(&mut context);

    if result.success {
        context.get_result().unwrap().clone()
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

        context.process_all_nodes();

        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "ProcessAllNodes"
    }
}
