use crate::event::{Event, ThreadId};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

/// Decision made by the deterministic scheduler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerDecision {
    /// Allow the thread to continue execution
    Continue,
    /// Block the thread and switch to another
    Block,
    /// Yield to another thread
    Yield,
}

/// Deterministic scheduler for replay
/// 
/// During recording, captures all scheduling decisions.
/// During replay, enforces the exact same execution order.
pub struct DeterministicScheduler {
    mode: SchedulerMode,
    recorded_decisions: VecDeque<SchedulingEvent>,
    current_thread: Option<ThreadId>,
    thread_states: HashMap<ThreadId, ThreadSchedulingState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchedulerMode {
    Recording,
    Replaying,
}

#[derive(Debug, Clone)]
struct SchedulingEvent {
    sequence: u64,
    thread_id: ThreadId,
    decision: SchedulerDecision,
    reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThreadSchedulingState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

impl DeterministicScheduler {
    pub fn new() -> Self {
        Self {
            mode: SchedulerMode::Recording,
            recorded_decisions: VecDeque::new(),
            current_thread: None,
            thread_states: HashMap::new(),
        }
    }

    /// Switch to replay mode
    pub fn set_replay_mode(&mut self, decisions: Vec<SchedulingEvent>) {
        self.mode = SchedulerMode::Replaying;
        self.recorded_decisions = decisions.into();
        debug!("Scheduler switched to replay mode with {} decisions", self.recorded_decisions.len());
    }

    /// Make a scheduling decision for a thread
    pub fn schedule(&mut self, thread_id: ThreadId, sequence: u64) -> SchedulerDecision {
        match self.mode {
            SchedulerMode::Recording => {
                // In recording mode, make decisions based on actual execution
                let decision = self.make_decision(thread_id);
                
                self.recorded_decisions.push_back(SchedulingEvent {
                    sequence,
                    thread_id,
                    decision,
                    reason: "recorded".to_string(),
                });

                decision
            }
            SchedulerMode::Replaying => {
                // In replay mode, enforce recorded decisions
                if let Some(event) = self.recorded_decisions.front() {
                    if event.thread_id == thread_id && event.sequence == sequence {
                        let decision = event.decision;
                        self.recorded_decisions.pop_front();
                        debug!("Replaying decision for thread {:?}: {:?}", thread_id, decision);
                        return decision;
                    }
                }

                // If no matching decision found, block by default
                debug!("No matching decision found for thread {:?}, blocking", thread_id);
                SchedulerDecision::Block
            }
        }
    }

    /// Register a new thread
    pub fn register_thread(&mut self, thread_id: ThreadId) {
        self.thread_states.insert(thread_id, ThreadSchedulingState::Ready);
        debug!("Registered thread {:?}", thread_id);
    }

    /// Mark a thread as terminated
    pub fn terminate_thread(&mut self, thread_id: ThreadId) {
        self.thread_states.insert(thread_id, ThreadSchedulingState::Terminated);
        debug!("Terminated thread {:?}", thread_id);
    }

    /// Get recorded scheduling decisions
    pub fn get_decisions(&self) -> Vec<SchedulingEvent> {
        self.recorded_decisions.iter().cloned().collect()
    }

    fn make_decision(&mut self, thread_id: ThreadId) -> SchedulerDecision {
        // Simple round-robin scheduling for recording
        // In a real implementation, this would capture actual OS scheduling
        
        let state = self.thread_states.get(&thread_id).copied()
            .unwrap_or(ThreadSchedulingState::Ready);

        match state {
            ThreadSchedulingState::Ready | ThreadSchedulingState::Running => {
                self.current_thread = Some(thread_id);
                self.thread_states.insert(thread_id, ThreadSchedulingState::Running);
                SchedulerDecision::Continue
            }
            ThreadSchedulingState::Blocked => {
                SchedulerDecision::Block
            }
            ThreadSchedulingState::Terminated => {
                SchedulerDecision::Block
            }
        }
    }
}

impl Default for DeterministicScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_recording() {
        let mut scheduler = DeterministicScheduler::new();
        let thread1 = ThreadId(1);
        let thread2 = ThreadId(2);

        scheduler.register_thread(thread1);
        scheduler.register_thread(thread2);

        let decision1 = scheduler.schedule(thread1, 1);
        let decision2 = scheduler.schedule(thread2, 2);

        assert_eq!(decision1, SchedulerDecision::Continue);
        assert_eq!(decision2, SchedulerDecision::Continue);
        assert_eq!(scheduler.get_decisions().len(), 2);
    }

    #[test]
    fn test_scheduler_replay() {
        let mut scheduler = DeterministicScheduler::new();
        let thread1 = ThreadId(1);

        scheduler.register_thread(thread1);
        let _ = scheduler.schedule(thread1, 1);
        
        let decisions = scheduler.get_decisions();
        
        let mut replay_scheduler = DeterministicScheduler::new();
        replay_scheduler.set_replay_mode(decisions);
        replay_scheduler.register_thread(thread1);

        let decision = replay_scheduler.schedule(thread1, 1);
        assert_eq!(decision, SchedulerDecision::Continue);
    }
}
