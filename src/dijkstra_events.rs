use crate::eventchains::{ChainableEvent, EventContext, EventResult};
use crate::graph::{DijkstraState, NodeId, QueueNode};
use std::collections::BinaryHeap;

/// Event: Initialize Dijkstra's algorithm state
pub struct InitializeStateEvent {
    source: NodeId,
    node_count: usize,
}

impl InitializeStateEvent {
    pub fn new(source: &NodeId, node_count: usize) -> Self {
        Self {
            source: *source,
            node_count,
        }
    }
}

impl ChainableEvent for InitializeStateEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let state = DijkstraState::new(self.node_count, &self.source);
        context.set_state(state);
        context.set_source(self.source);
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
        let source: &NodeId = match context.get_source() {
            Some(s) => s,
            None => return EventResult::Failure("Source not found in context".to_string()),
        };

        let mut queue = BinaryHeap::new();
        queue.push(QueueNode {
            node: *source,
            distance: 0,
        });

        context.set_queue(queue);
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
        context.process_one_node();

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
    pub fn new(target: &NodeId) -> Self {
        Self { target: *target }
    }
}

impl ChainableEvent for FinalizeResultEvent {
    fn execute(&self, context: &mut EventContext) -> EventResult<()> {
        let state: &DijkstraState = match context.get_state() {
            Some(s) => s,
            None => return EventResult::Failure("State not found in context".to_string()),
        };

        let source: &NodeId = match context.get_source() {
            Some(s) => s,
            None => return EventResult::Failure("Source not found in context".to_string()),
        };

        let result =
            crate::graph::ShortestPathResult::reconstruct_path(state, source, &self.target);

        context.set_result(result);
        EventResult::Success(())
    }

    fn name(&self) -> &str {
        "FinalizeResult"
    }
}
