use crate::eventchains::{ChainableEvent, EventContext, EventResult};
use crate::graph::{DijkstraState, Graph, NodeId, QueueNode};
use std::collections::BinaryHeap;
use std::sync::Arc;

/// Event: Initialize Dijkstra's algorithm state
pub struct InitializeStateEvent {
    source: NodeId,
    node_count: usize,
}

impl InitializeStateEvent {
    pub fn new(source: NodeId, node_count: usize) -> Self {
        Self { source, node_count }
    }
}

impl ChainableEvent for InitializeStateEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let state = DijkstraState::new(self.node_count, self.source);
        context.set("state", state);
        context.set("source", self.source);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "InitializeState"
    }
}

/// Event: Create and initialize priority queue
pub struct InitializePriorityQueueEvent;

impl ChainableEvent for InitializePriorityQueueEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let source: NodeId = match context.get::<NodeId>("source") {
            Some(s) => s.clone(),
            None => return EventResult::Failure("Source not found in context".to_string()),
        };

        let mut queue = BinaryHeap::new();
        queue.push(QueueNode {
            node: source,
            distance: 0,
        });

        context.set("queue", queue);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "InitializePriorityQueue"
    }
}

/// Event: Process one node from the priority queue
pub struct ProcessNodeEvent;

impl ChainableEvent for ProcessNodeEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let mut queue: BinaryHeap<QueueNode> = match context.take("queue") {
            Some(q) => *q,
            None => return EventResult::Failure("Queue not found in context".to_string()),
        };

        let mut state: DijkstraState = match context.take("state") {
            Some(s) => *s,
            None => return EventResult::Failure("State not found in context".to_string()),
        };

        let graph: Arc<Graph> = match context.get::<Arc<Graph>>("graph") {
            Some(g) => g.clone(),
            None => return EventResult::Failure("Graph not found in context".to_string()),
        };

        if let Some(QueueNode { node, distance }) = queue.pop() {
            // Skip if already visited or if distance is stale
            if state.visited[node.0] || distance > state.distances[node.0] {
                context.set("queue", queue);
                context.set("state", state);
                context.set("continue", true);
                return EventResult::Success(());
            }

            state.visited[node.0] = true;

            // Process neighbors
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

            context.set("continue", !queue.is_empty());
        } else {
            context.set("continue", false);
        }

        context.set("queue", queue);
        context.set("state", state);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "ProcessNode"
    }
}

/// Event: Finalize result
pub struct FinalizeResultEvent {
    target: NodeId,
}

impl FinalizeResultEvent {
    pub fn new(target: NodeId) -> Self {
        Self { target }
    }
}

impl ChainableEvent for FinalizeResultEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let state: DijkstraState = match context.take("state") {
            Some(s) => *s,
            None => return EventResult::Failure("State not found in context".to_string()),
        };

        let source: NodeId = match context.take("source") {
            Some(s) => *s,
            None => return EventResult::Failure("Source not found in context".to_string()),
        };

        let result =
            crate::graph::ShortestPathResult::reconstruct_path(&state, source, self.target);

        context.set("result", result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "FinalizeResult"
    }
}
