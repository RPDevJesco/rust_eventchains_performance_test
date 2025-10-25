use crate::Graph;
use crate::NodeId;
use crate::graph::DijkstraState;
use crate::graph::QueueNode;
use crate::graph::ShortestPathResult;
use std::collections::BinaryHeap;
use std::fmt;

/// Result of an event execution
#[derive(Debug, Clone)]
pub enum EventResult<T> {
    Success(T),
    Failure(String),
}

impl<T> EventResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, EventResult::Success(_))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, EventResult::Failure(_))
    }

    pub fn get_data(self) -> Option<T> {
        match self {
            EventResult::Success(data) => Some(data),
            EventResult::Failure(_) => None,
        }
    }

    pub fn get_error(&self) -> Option<&str> {
        match self {
            EventResult::Success(_) => None,
            EventResult::Failure(msg) => Some(msg),
        }
    }
}

/// Context that flows through the event chain
pub struct EventContext<'a> {
    graph: Option<&'a Graph>,
    state: Option<DijkstraState>,
    source: Option<NodeId>,
    result: Option<ShortestPathResult>,
    queue: Option<BinaryHeap<QueueNode>>,
}

impl<'a> EventContext<'a> {
    pub fn new() -> Self {
        Self {
            graph: None,
            state: None,
            source: None,
            result: None,
            queue: None,
        }
    }

    pub fn set_graph(&mut self, graph: &'a Graph) {
        self.graph = Some(graph);
    }

    pub fn set_state(&mut self, state: DijkstraState) {
        self.state = Some(state);
    }

    pub fn get_state(&self) -> Option<&DijkstraState> {
        self.state.as_ref()
    }

    pub fn set_source(&mut self, source: NodeId) {
        self.source = Some(source);
    }

    pub fn get_source(&self) -> Option<&NodeId> {
        self.source.as_ref()
    }

    pub fn set_result(&mut self, result: ShortestPathResult) {
        self.result = Some(result);
    }

    pub fn get_result(&mut self) -> Option<&ShortestPathResult> {
        self.result.as_ref()
    }

    pub fn set_queue(&mut self, queue: BinaryHeap<QueueNode>) {
        self.queue = Some(queue);
    }

    pub fn queue_pop(&mut self) -> Option<QueueNode> {
        self.queue.as_mut().map(|queue| queue.pop()).flatten()
    }

    pub fn process_node(&mut self, node: NodeId, distance: u32) -> EventResult<()> {
        let state: &mut DijkstraState = match &mut self.state {
            Some(s) => s,
            None => return EventResult::Failure("State not found in context".to_string()),
        };

        let queue: &mut BinaryHeap<QueueNode> = match &mut self.queue {
            Some(q) => q,
            None => return EventResult::Failure("Queue not found in context".to_string()),
        };

        let graph: &Graph = match self.graph {
            Some(g) => g,
            None => return EventResult::Failure("Graph not found in context".to_string()),
        };

        // Skip if already visited or if distance is stale
        if state.visited[node.0] || distance > state.distances[node.0] {
            return EventResult::Success(());
        }

        state.visited[node.0] = true;

        // Process neighbors
        graph.adjacency_list[node.0].iter().for_each(|edge| {
            let new_distance = distance.saturating_add(edge.weight);

            if new_distance < state.distances[edge.to.0] {
                {
                    state.distances[edge.to.0] = new_distance;
                    state.predecessors[edge.to.0] = Some(node);
                }

                let node = QueueNode {
                    node: edge.to,
                    distance: new_distance,
                };

                queue.push(node);
            }
        });

        EventResult::Success(())
    }
}

impl<'a> Default for EventContext<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for chainable events
pub trait ChainableEvent: Send + Sync {
    fn execute(&self, context: &mut EventContext) -> EventResult<()>;
    fn name(&self) -> &str;
}

/// Trait for middleware
pub trait EventMiddleware: Send + Sync {
    fn execute(
        &self,
        event: &dyn ChainableEvent,
        context: &mut EventContext,
        next: &mut dyn FnMut(&mut EventContext) -> EventResult<()>,
    ) -> EventResult<()>;
}

/// Event failure information
#[derive(Debug, Clone)]
pub struct EventFailure {
    pub event_name: String,
    pub error_message: String,
    pub timestamp: u64,
}

impl EventFailure {
    pub fn new(event_name: String, error_message: String) -> Self {
        Self {
            event_name,
            error_message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Chain execution result
#[derive(Debug)]
pub struct ChainResult {
    pub success: bool,
    pub failures: Vec<EventFailure>,
    pub status: ChainStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainStatus {
    Completed,
    CompletedWithWarnings,
    Failed,
}

impl ChainResult {
    pub fn success() -> Self {
        Self {
            success: true,
            failures: Vec::new(),
            status: ChainStatus::Completed,
        }
    }

    pub fn partial_success(failures: Vec<EventFailure>) -> Self {
        Self {
            success: true,
            failures,
            status: ChainStatus::CompletedWithWarnings,
        }
    }

    pub fn failure(failures: Vec<EventFailure>) -> Self {
        Self {
            success: false,
            failures,
            status: ChainStatus::Failed,
        }
    }
}

/// Fault tolerance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultToleranceMode {
    Strict,
    Lenient,
    BestEffort,
}

/// Main EventChain orchestrator
pub struct EventChain<'a> {
    events: Vec<&'a dyn ChainableEvent>,
    middlewares: Vec<&'a dyn EventMiddleware>,
    fault_tolerance: FaultToleranceMode,
}

impl<'a> EventChain<'a> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            middlewares: Vec::new(),
            fault_tolerance: FaultToleranceMode::Strict,
        }
    }

    pub fn with_fault_tolerance(mut self, mode: FaultToleranceMode) -> Self {
        self.fault_tolerance = mode;
        self
    }

    pub fn add_event(&mut self, event: &'a dyn ChainableEvent) -> &mut Self {
        self.events.push(event);
        self
    }

    pub fn use_middleware(&mut self, middleware: &'a dyn EventMiddleware) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn execute(&self, context: &mut EventContext) -> ChainResult {
        let mut failures = Vec::new();

        for event in self.events.iter() {
            // Build middleware pipeline (LIFO - last registered executes first)
            let result = self.execute_with_middleware(*event, context);

            if result.is_failure() {
                let failure = EventFailure::new(
                    event.name().to_string(),
                    result.get_error().unwrap_or("Unknown error").to_string(),
                );
                failures.push(failure);

                match self.fault_tolerance {
                    FaultToleranceMode::Strict => {
                        return ChainResult::failure(failures);
                    }
                    FaultToleranceMode::Lenient | FaultToleranceMode::BestEffort => {
                        // Continue execution
                        continue;
                    }
                }
            }
        }

        if failures.is_empty() {
            ChainResult::success()
        } else {
            ChainResult::partial_success(failures)
        }
    }

    fn execute_with_middleware(
        &self,
        event: &dyn ChainableEvent,
        context: &mut EventContext,
    ) -> EventResult<()> {
        if self.middlewares.is_empty() {
            return event.execute(context);
        }

        // Execute middleware in reverse order by recursively building the call stack
        self.execute_middleware_recursive(0, event, context)
    }

    fn execute_middleware_recursive(
        &self,
        middleware_index: usize,
        event: &dyn ChainableEvent,
        context: &mut EventContext,
    ) -> EventResult<()> {
        if middleware_index >= self.middlewares.len() {
            // Base case: execute the actual event
            return event.execute(context);
        }

        // Get the current middleware (reverse order)
        let middleware_idx = self.middlewares.len() - 1 - middleware_index;
        let middleware = self.middlewares[middleware_idx];

        // Create a closure that calls the next middleware (or event)
        let mut next = |ctx: &mut EventContext| -> EventResult<()> {
            self.execute_middleware_recursive(middleware_index + 1, event, ctx)
        };

        // Execute this middleware with the next closure
        middleware.execute(event, context, &mut next)
    }
}

impl<'a> Default for EventChain<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ChainStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainStatus::Completed => write!(f, "COMPLETED"),
            ChainStatus::CompletedWithWarnings => write!(f, "COMPLETED_WITH_WARNINGS"),
            ChainStatus::Failed => write!(f, "FAILED"),
        }
    }
}
