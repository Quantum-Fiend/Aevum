use trace_engine::{Event, TraceLog, DeterministicScheduler, ThreadId};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, debug};

/// Deterministic replay engine
pub struct ReplayEngine {
    trace_log: TraceLog,
    scheduler: DeterministicScheduler,
    current_position: usize,
    events: Vec<Event>,
    nondeterministic_inputs: HashMap<u64, Vec<u8>>,
}

impl ReplayEngine {
    /// Load a trace for replay
    pub fn load<P: AsRef<Path>>(trace_path: P) -> Result<Self> {
        let trace_log = TraceLog::open(trace_path)
            .context("Failed to open trace log")?;
        
        let events = trace_log.read_all()
            .context("Failed to read events from trace")?;
        
        info!("Loaded {} events for replay", events.len());

        let mut scheduler = DeterministicScheduler::new();
        let nondeterministic_inputs = Self::extract_nondeterministic_inputs(&events);

        Ok(Self {
            trace_log,
            scheduler,
            current_position: 0,
            events,
            nondeterministic_inputs,
        })
    }

    /// Get the next event in the replay
    pub fn next_event(&mut self) -> Option<&Event> {
        if self.current_position < self.events.len() {
            let event = &self.events[self.current_position];
            self.current_position += 1;
            debug!("Replaying event #{}: {}", self.current_position, event.event_type());
            Some(event)
        } else {
            None
        }
    }

    /// Step backward in the replay
    pub fn previous_event(&mut self) -> Option<&Event> {
        if self.current_position > 0 {
            self.current_position -= 1;
            let event = &self.events[self.current_position];
            debug!("Stepped back to event #{}: {}", self.current_position + 1, event.event_type());
            Some(event)
        } else {
            None
        }
    }

    /// Jump to a specific event by sequence number
    pub fn seek_to(&mut self, sequence_number: u64) -> Result<&Event> {
        let position = self.events
            .iter()
            .position(|e| e.metadata().sequence_number == sequence_number)
            .context("Event not found")?;
        
        self.current_position = position;
        info!("Seeked to event #{}", sequence_number);
        Ok(&self.events[position])
    }

    /// Get the current position in the replay
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Get the total number of events
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// Get a nondeterministic input by sequence number
    pub fn get_nondeterministic_input(&self, sequence: u64) -> Option<&Vec<u8>> {
        self.nondeterministic_inputs.get(&sequence)
    }

    /// Extract nondeterministic inputs from events
    fn extract_nondeterministic_inputs(events: &[Event]) -> HashMap<u64, Vec<u8>> {
        let mut inputs = HashMap::new();

        for event in events {
            match event {
                Event::RandomBytes { metadata, bytes } => {
                    inputs.insert(metadata.sequence_number, bytes.clone());
                }
                Event::Timestamp { metadata, wall_clock_ns } => {
                    inputs.insert(metadata.sequence_number, wall_clock_ns.to_le_bytes().to_vec());
                }
                _ => {}
            }
        }

        info!("Extracted {} nondeterministic inputs", inputs.len());
        inputs
    }

    /// Get events in a time range
    pub fn events_in_range(&self, start_seq: u64, end_seq: u64) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| {
                let seq = e.metadata().sequence_number;
                seq >= start_seq && seq <= end_seq
            })
            .collect()
    }

    /// Get all events for a specific thread
    pub fn events_for_thread(&self, thread_id: ThreadId) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.metadata().thread_id == thread_id)
            .collect()
    }

    /// Reset replay to the beginning
    pub fn reset(&mut self) {
        self.current_position = 0;
        info!("Reset replay to beginning");
    }

    /// Check if replay is complete
    pub fn is_complete(&self) -> bool {
        self.current_position >= self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trace_engine::{TraceEngine, EventMetadata, ProcessId, TraceId, VectorClock};
    use tempfile::tempdir;

    #[test]
    fn test_replay_engine() {
        let dir = tempdir().unwrap();
        
        // Create a trace
        let mut engine = TraceEngine::new(dir.path(), true, 100).unwrap();
        
        for i in 0..10 {
            let metadata = EventMetadata {
                trace_id: engine.trace_id(),
                process_id: ProcessId(1),
                thread_id: ThreadId(1),
                timestamp_ns: 1000 + i,
                vector_clock: VectorClock::new(),
                sequence_number: i + 1,
            };

            let event = Event::FunctionCall {
                metadata,
                function_name: format!("func_{}", i),
                module: "test".to_string(),
                args: vec![],
                stack_depth: 1,
            };

            engine.record_event(event).unwrap();
        }

        // Load for replay
        let trace_path = dir.path().join(format!("trace_{}.log", engine.trace_id().0));
        let mut replay = ReplayEngine::load(&trace_path).unwrap();

        assert_eq!(replay.total_events(), 10);
        assert_eq!(replay.current_position(), 0);

        // Step forward
        let event = replay.next_event().unwrap();
        assert_eq!(event.metadata().sequence_number, 1);

        // Step backward
        let event = replay.previous_event().unwrap();
        assert_eq!(event.metadata().sequence_number, 1);
    }
}
