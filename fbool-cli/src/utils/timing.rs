use std::time::{Duration, Instant};

/// Utility struct to measure execution times
pub struct Timer {
    start: Option<Instant>,
}

impl Timer {
    /// Create a new timer
    pub fn new() -> Self {
        Self { start: None }
    }

    /// Start timing if enabled
    pub fn start_if(&mut self, enabled: bool) {
        if enabled {
            self.start = Some(Instant::now());
        }
    }

    /// Get elapsed time if timing was started
    pub fn elapsed(&self) -> Option<Duration> {
        self.start.map(|start| start.elapsed())
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct to hold both generation and computation timings
pub struct ExecutionTimings {
    pub generation: Option<Duration>,
    pub computation: Option<Duration>,
}

impl ExecutionTimings {
    pub fn new() -> Self {
        Self {
            generation: None,
            computation: None,
        }
    }

    /// Print timing information if available
    pub fn print_if_available(&self) {
        if let (Some(gen), Some(comp)) = (self.generation, self.computation) {
            println!("Generation time: {}ms", gen.as_millis());
            println!("Compute time: {}ms", comp.as_millis());
        }
    }
}

impl Default for ExecutionTimings {
    fn default() -> Self {
        Self::new()
    }
}
