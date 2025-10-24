use crate::eventchains::{ChainableEvent, EventContext, EventMiddleware, EventResult};
use std::cell::RefCell;
use std::time::Instant;

/// Logging middleware that tracks event execution
pub struct LoggingMiddleware {
    verbose: bool,
}

impl LoggingMiddleware {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl EventMiddleware for LoggingMiddleware {
    fn execute(
        &self,
        event: &dyn ChainableEvent,
        context: &RefCell<EventContext>,
        next: &mut dyn FnMut(&RefCell<EventContext>) -> EventResult<()>,
    ) -> EventResult<()> {
        if self.verbose {
            println!("  ▶ {} starting", event.name());
        }

        let result = next(context);

        if self.verbose {
            match &result {
                EventResult::Success(_) => println!("    ✓ {} completed", event.name()),
                EventResult::Failure(err) => println!("    ✗ {} failed: {}", event.name(), err),
            }
        }

        result
    }
}

/// Timing middleware that measures execution time
pub struct TimingMiddleware {
    log_timing: bool,
}

impl TimingMiddleware {
    pub fn new(log_timing: bool) -> Self {
        Self { log_timing }
    }
}

impl EventMiddleware for TimingMiddleware {
    fn execute(
        &self,
        event: &dyn ChainableEvent,
        context: &RefCell<EventContext>,
        next: &mut dyn FnMut(&RefCell<EventContext>) -> EventResult<()>,
    ) -> EventResult<()> {
        let mut borrowed = context.borrow_mut();

        let start = Instant::now();

        let result = next(context);

        let duration = start.elapsed();

        if self.log_timing {
            println!("    ⏱️  {} took {}μs", event.name(), duration.as_micros());
        }

        // Store timing in context for profiling
        let key = format!("{}_duration_ns", event.name());
        borrowed.set(&key, duration.as_nanos() as u64);

        result
    }
}

/// Performance profiling middleware
pub struct PerformanceMiddleware {
    pub event_count: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl PerformanceMiddleware {
    pub fn new() -> Self {
        Self {
            event_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    pub fn get_event_count(&self) -> u64 {
        *self.event_count.lock().unwrap()
    }
}

impl EventMiddleware for PerformanceMiddleware {
    fn execute(
        &self,
        _event: &dyn ChainableEvent,
        context: &RefCell<EventContext>,
        next: &mut dyn FnMut(&RefCell<EventContext>) -> EventResult<()>,
    ) -> EventResult<()> {
        {
            let mut count = self.event_count.lock().unwrap();
            *count += 1;
        }

        next(context)
    }
}

impl Default for PerformanceMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
