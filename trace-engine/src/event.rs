use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a trace session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub Uuid);

impl TraceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub u32);

/// Thread identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreadId(pub u64);

/// Vector clock for distributed causality tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    pub clocks: HashMap<ProcessId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn increment(&mut self, process: ProcessId) {
        *self.clocks.entry(process).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (process, &clock) in &other.clocks {
            let entry = self.clocks.entry(*process).or_insert(0);
            *entry = (*entry).max(clock);
        }
    }

    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;
        
        for (process, &our_clock) in &self.clocks {
            let their_clock = other.clocks.get(process).copied().unwrap_or(0);
            if our_clock > their_clock {
                return false;
            }
            if our_clock < their_clock {
                strictly_less = true;
            }
        }

        for (process, &their_clock) in &other.clocks {
            if !self.clocks.contains_key(process) && their_clock > 0 {
                strictly_less = true;
            }
        }

        strictly_less
    }

    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Event metadata attached to every captured event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub trace_id: TraceId,
    pub process_id: ProcessId,
    pub thread_id: ThreadId,
    pub timestamp_ns: u64,
    pub vector_clock: VectorClock,
    pub sequence_number: u64,
}

/// Core event types captured during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Function call entry
    FunctionCall {
        metadata: EventMetadata,
        function_name: String,
        module: String,
        args: Vec<u8>, // Serialized arguments
        stack_depth: u32,
    },

    /// Function return
    FunctionReturn {
        metadata: EventMetadata,
        function_name: String,
        return_value: Vec<u8>, // Serialized return value
        stack_depth: u32,
    },

    /// Memory write operation
    MemoryWrite {
        metadata: EventMetadata,
        address: u64,
        size: usize,
        old_value: Vec<u8>,
        new_value: Vec<u8>,
    },

    /// System call
    Syscall {
        metadata: EventMetadata,
        syscall_number: u64,
        syscall_name: String,
        args: Vec<u64>,
        result: i64,
    },

    /// Thread context switch
    ThreadSwitch {
        metadata: EventMetadata,
        from_thread: ThreadId,
        to_thread: ThreadId,
        reason: String,
    },

    /// Network I/O operation
    NetworkIO {
        metadata: EventMetadata,
        direction: IODirection,
        socket_fd: i32,
        remote_addr: String,
        data: Vec<u8>,
        bytes_transferred: usize,
    },

    /// Garbage collection event
    GarbageCollection {
        metadata: EventMetadata,
        gc_type: String,
        duration_ns: u64,
        bytes_collected: usize,
        heap_size_before: usize,
        heap_size_after: usize,
    },

    /// Thread lifecycle
    ThreadCreate {
        metadata: EventMetadata,
        new_thread_id: ThreadId,
        parent_thread_id: ThreadId,
    },

    ThreadExit {
        metadata: EventMetadata,
        exit_code: i32,
    },

    /// Process lifecycle
    ProcessFork {
        metadata: EventMetadata,
        child_process_id: ProcessId,
    },

    ProcessExec {
        metadata: EventMetadata,
        executable: String,
        args: Vec<String>,
    },

    /// Synchronization primitives
    MutexLock {
        metadata: EventMetadata,
        mutex_id: u64,
        acquired: bool,
    },

    MutexUnlock {
        metadata: EventMetadata,
        mutex_id: u64,
    },

    /// Nondeterministic input capture
    RandomBytes {
        metadata: EventMetadata,
        bytes: Vec<u8>,
    },

    Timestamp {
        metadata: EventMetadata,
        wall_clock_ns: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IODirection {
    Send,
    Receive,
}

impl Event {
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            Event::FunctionCall { metadata, .. } => metadata,
            Event::FunctionReturn { metadata, .. } => metadata,
            Event::MemoryWrite { metadata, .. } => metadata,
            Event::Syscall { metadata, .. } => metadata,
            Event::ThreadSwitch { metadata, .. } => metadata,
            Event::NetworkIO { metadata, .. } => metadata,
            Event::GarbageCollection { metadata, .. } => metadata,
            Event::ThreadCreate { metadata, .. } => metadata,
            Event::ThreadExit { metadata, .. } => metadata,
            Event::ProcessFork { metadata, .. } => metadata,
            Event::ProcessExec { metadata, .. } => metadata,
            Event::MutexLock { metadata, .. } => metadata,
            Event::MutexUnlock { metadata, .. } => metadata,
            Event::RandomBytes { metadata, .. } => metadata,
            Event::Timestamp { metadata, .. } => metadata,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Event::FunctionCall { .. } => "FunctionCall",
            Event::FunctionReturn { .. } => "FunctionReturn",
            Event::MemoryWrite { .. } => "MemoryWrite",
            Event::Syscall { .. } => "Syscall",
            Event::ThreadSwitch { .. } => "ThreadSwitch",
            Event::NetworkIO { .. } => "NetworkIO",
            Event::GarbageCollection { .. } => "GarbageCollection",
            Event::ThreadCreate { .. } => "ThreadCreate",
            Event::ThreadExit { .. } => "ThreadExit",
            Event::ProcessFork { .. } => "ProcessFork",
            Event::ProcessExec { .. } => "ProcessExec",
            Event::MutexLock { .. } => "MutexLock",
            Event::MutexUnlock { .. } => "MutexUnlock",
            Event::RandomBytes { .. } => "RandomBytes",
            Event::Timestamp { .. } => "Timestamp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_happens_before() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        let p1 = ProcessId(1);
        let p2 = ProcessId(2);

        vc1.increment(p1);
        vc2.clocks.insert(p1, 2);
        vc2.clocks.insert(p2, 1);

        assert!(vc1.happens_before(&vc2));
        assert!(!vc2.happens_before(&vc1));
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        let p1 = ProcessId(1);
        let p2 = ProcessId(2);

        vc1.increment(p1);
        vc2.increment(p2);

        assert!(vc1.concurrent_with(&vc2));
        assert!(vc2.concurrent_with(&vc1));
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        let p1 = ProcessId(1);
        let p2 = ProcessId(2);

        vc1.clocks.insert(p1, 5);
        vc1.clocks.insert(p2, 2);

        vc2.clocks.insert(p1, 3);
        vc2.clocks.insert(p2, 7);

        vc1.merge(&vc2);

        assert_eq!(vc1.clocks.get(&p1), Some(&5));
        assert_eq!(vc1.clocks.get(&p2), Some(&7));
    }
}
