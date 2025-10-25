use crate::eventchains::{ChainableEvent, EventContext, EventMiddleware, EventResult};

/// No-op middleware for measuring overhead
pub struct NoOpMiddleware {
    id: usize,
}

impl NoOpMiddleware {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

impl EventMiddleware for NoOpMiddleware {
    fn execute(
        &self,
        _event: &dyn ChainableEvent,
        context: &mut EventContext,
        next: &mut dyn FnMut(&mut EventContext) -> EventResult<()>,
    ) -> EventResult<()> {
        // Minimal work: just increment a counter and call next
        let key = format!("noop_middleware_{}_called", self.id);
        let count: u32 = context.get(&key).unwrap_or(0);
        context.set(&key, count + 1);
        
        next(context)
    }
}

/// Counting middleware for validation
pub struct CountingMiddleware {
    counter: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl CountingMiddleware {
    pub fn new() -> Self {
        Self {
            counter: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }
    
    pub fn get_count(&self) -> u64 {
        *self.counter.lock().unwrap()
    }
}

impl EventMiddleware for CountingMiddleware {
    fn execute(
        &self,
        _event: &dyn ChainableEvent,
        context: &mut EventContext,
        next: &mut dyn FnMut(&mut EventContext) -> EventResult<()>,
    ) -> EventResult<()> {
        {
            let mut count = self.counter.lock().unwrap();
            *count += 1;
        }
        next(context)
    }
}

impl Default for CountingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
