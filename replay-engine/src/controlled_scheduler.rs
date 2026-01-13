use trace_engine::{ThreadId, Event};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

/// Controlled scheduler for deterministic replay
pub struct ControlledScheduler {
    scheduled_events: VecDeque<ScheduledEvent>,
    thread_states: HashMap<ThreadId, ThreadState>,
}

#[derive(Debug, Clone)]
struct ScheduledEvent {
    sequence: u64,
    thread_id: ThreadId,
    event: Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThreadState {
    Ready,
    Running,
    Blocked,
    Completed,
}

impl ControlledScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_events: VecDeque::new(),
            thread_states: HashMap::new(),
        }
    }

    /// Load events for controlled replay
    pub fn load_events(&mut self, events: Vec<Event>) {
        for event in events {
            let thread_id = event.metadata().thread_id;
            let sequence = event.metadata().sequence_number;

            self.scheduled_events.push_back(ScheduledEvent {
                sequence,
                thread_id,
                event,
            });

            self.thread_states.entry(thread_id)
                .or_insert(ThreadState::Ready);
        }

        debug!("Loaded {} events for controlled replay", self.scheduled_events.len());
    }

    /// Get the next event to execute
    pub fn next_event(&mut self) -> Option<Event> {
        if let Some(scheduled) = self.scheduled_events.pop_front() {
            self.thread_states.insert(scheduled.thread_id, ThreadState::Running);
            debug!("Executing event #{} for thread {:?}", scheduled.sequence, scheduled.thread_id);
            Some(scheduled.event)
        } else {
            None
        }
    }

    /// Check if a thread should be blocked
    pub fn should_block(&self, thread_id: ThreadId) -> bool {
        matches!(
            self.thread_states.get(&thread_id),
            Some(ThreadState::Blocked) | Some(ThreadState::Completed)
        )
    }

    /// Mark a thread as completed
    pub fn mark_completed(&mut self, thread_id: ThreadId) {
        self.thread_states.insert(thread_id, ThreadState::Completed);
        debug!("Thread {:?} marked as completed", thread_id);
    }
}

impl Default for ControlledScheduler {
    fn default() -> Self {
        Self::new()
    }
}
