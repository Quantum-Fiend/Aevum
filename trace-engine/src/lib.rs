pub mod event;
pub mod trace_log;
pub mod snapshot;
pub mod vector_clock;
pub mod scheduler;

pub use event::{Event, EventMetadata, ProcessId, ThreadId, TraceId, VectorClock, IODirection};
pub use trace_log::TraceLog;
pub use snapshot::{Snapshot, SnapshotManager, SnapshotType, MemoryRegion, MemoryDiff};
pub use scheduler::{DeterministicScheduler, SchedulerDecision};

use anyhow::Result;
use std::path::Path;
use tracing::info;

/// Main trace engine coordinator
pub struct TraceEngine {
    trace_log: TraceLog,
    snapshot_manager: SnapshotManager,
    scheduler: DeterministicScheduler,
    trace_id: TraceId,
}

impl TraceEngine {
    /// Create a new trace engine
    pub fn new<P: AsRef<Path>>(
        trace_dir: P,
        compress: bool,
        snapshot_interval: u64,
    ) -> Result<Self> {
        let trace_dir = trace_dir.as_ref();
        let trace_id = TraceId::new();
        
        let trace_log_path = trace_dir.join(format!("trace_{}.log", trace_id.0));
        let snapshot_dir = trace_dir.join(format!("snapshots_{}", trace_id.0));

        let trace_log = TraceLog::create(trace_log_path, compress)?;
        let snapshot_manager = SnapshotManager::new(snapshot_dir, snapshot_interval)?;
        let scheduler = DeterministicScheduler::new();

        info!("Initialized trace engine with ID: {:?}", trace_id);

        Ok(Self {
            trace_log,
            snapshot_manager,
            scheduler,
            trace_id,
        })
    }

    /// Record an event
    pub fn record_event(&mut self, event: Event) -> Result<u64> {
        let event_id = self.trace_log.append(&event)?;
        
        // Check if we should create a snapshot
        if self.snapshot_manager.should_snapshot(event_id) {
            info!("Snapshot checkpoint reached at event #{}", event_id);
            // Snapshot creation would happen here in a real implementation
        }

        Ok(event_id)
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> TraceId {
        self.trace_id
    }

    /// Get the number of recorded events
    pub fn event_count(&self) -> u64 {
        self.trace_log.event_count()
    }

    /// Read all events from the trace
    pub fn read_events(&self) -> Result<Vec<Event>> {
        self.trace_log.read_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_trace_engine_creation() {
        let dir = tempdir().unwrap();
        let engine = TraceEngine::new(dir.path(), true, 100).unwrap();
        assert_eq!(engine.event_count(), 0);
    }
}
