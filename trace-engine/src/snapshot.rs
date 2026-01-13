use crate::event::{Event, ProcessId};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

/// Memory snapshot with delta compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub sequence_number: u64,
    pub timestamp_ns: u64,
    pub process_id: ProcessId,
    pub snapshot_type: SnapshotType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotType {
    /// Full memory snapshot
    Full {
        memory_regions: Vec<MemoryRegion>,
        register_state: RegisterState,
        thread_states: HashMap<u64, ThreadState>,
    },
    /// Delta from previous snapshot
    Delta {
        base_sequence: u64,
        memory_diffs: Vec<MemoryDiff>,
        register_diffs: Vec<RegisterDiff>,
        thread_state_changes: HashMap<u64, ThreadState>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub start_addr: u64,
    pub size: usize,
    pub permissions: u8, // rwx bits
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDiff {
    pub address: u64,
    pub old_value: Vec<u8>,
    pub new_value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterState {
    pub registers: HashMap<String, u64>,
    pub flags: u64,
    pub instruction_pointer: u64,
    pub stack_pointer: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDiff {
    pub register_name: String,
    pub old_value: u64,
    pub new_value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadState {
    pub thread_id: u64,
    pub stack_pointer: u64,
    pub instruction_pointer: u64,
    pub state: ThreadExecutionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadExecutionState {
    Running,
    Blocked,
    Waiting,
    Terminated,
}

/// Snapshot manager for creating and restoring checkpoints
pub struct SnapshotManager {
    snapshots: Vec<Snapshot>,
    snapshot_dir: PathBuf,
    snapshot_interval: u64, // Number of events between snapshots
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(snapshot_dir: P, snapshot_interval: u64) -> Result<Self> {
        let snapshot_dir = snapshot_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&snapshot_dir)
            .context("Failed to create snapshot directory")?;

        info!(
            "Initialized snapshot manager at {:?} with interval {}",
            snapshot_dir, snapshot_interval
        );

        Ok(Self {
            snapshots: Vec::new(),
            snapshot_dir,
            snapshot_interval,
        })
    }

    /// Create a new snapshot
    pub fn create_snapshot(
        &mut self,
        sequence_number: u64,
        timestamp_ns: u64,
        process_id: ProcessId,
        snapshot_type: SnapshotType,
    ) -> Result<()> {
        let snapshot = Snapshot {
            sequence_number,
            timestamp_ns,
            process_id,
            snapshot_type,
        };

        // Save to disk
        let snapshot_path = self.snapshot_path(sequence_number);
        let serialized = bincode::serialize(&snapshot)
            .context("Failed to serialize snapshot")?;
        let compressed = zstd::encode_all(&serialized[..], 3)
            .context("Failed to compress snapshot")?;
        
        std::fs::write(&snapshot_path, compressed)
            .context("Failed to write snapshot to disk")?;

        self.snapshots.push(snapshot);
        info!("Created snapshot #{} at {:?}", sequence_number, snapshot_path);

        Ok(())
    }

    /// Get the closest snapshot before or at the given sequence number
    pub fn get_snapshot(&self, sequence_number: u64) -> Option<&Snapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.sequence_number <= sequence_number)
    }

    /// Load a snapshot from disk
    pub fn load_snapshot(&self, sequence_number: u64) -> Result<Snapshot> {
        let snapshot_path = self.snapshot_path(sequence_number);
        let compressed = std::fs::read(&snapshot_path)
            .context("Failed to read snapshot from disk")?;
        let serialized = zstd::decode_all(&compressed[..])
            .context("Failed to decompress snapshot")?;
        let snapshot: Snapshot = bincode::deserialize(&serialized)
            .context("Failed to deserialize snapshot")?;

        Ok(snapshot)
    }

    /// Check if a snapshot should be created at this sequence number
    pub fn should_snapshot(&self, sequence_number: u64) -> bool {
        sequence_number % self.snapshot_interval == 0
    }

    /// Compute memory diff between two memory regions
    pub fn compute_memory_diff(old: &[u8], new: &[u8], base_addr: u64) -> Vec<MemoryDiff> {
        let mut diffs = Vec::new();
        let mut i = 0;

        while i < old.len().min(new.len()) {
            if old[i] != new[i] {
                let start = i;
                let mut end = i;

                // Find contiguous changed region
                while end < old.len().min(new.len()) && old[end] != new[end] {
                    end += 1;
                }

                diffs.push(MemoryDiff {
                    address: base_addr + start as u64,
                    old_value: old[start..end].to_vec(),
                    new_value: new[start..end].to_vec(),
                });

                i = end;
            } else {
                i += 1;
            }
        }

        diffs
    }

    fn snapshot_path(&self, sequence_number: u64) -> PathBuf {
        self.snapshot_dir.join(format!("snapshot_{:016x}.snap", sequence_number))
    }

    /// Get total number of snapshots
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = SnapshotManager::new(dir.path(), 100).unwrap();
        assert_eq!(manager.snapshot_count(), 0);
    }

    #[test]
    fn test_should_snapshot() {
        let dir = tempdir().unwrap();
        let manager = SnapshotManager::new(dir.path(), 100).unwrap();
        
        assert!(!manager.should_snapshot(50));
        assert!(manager.should_snapshot(100));
        assert!(manager.should_snapshot(200));
    }

    #[test]
    fn test_compute_memory_diff() {
        let old = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let new = vec![1, 2, 9, 10, 5, 6, 11, 8];
        
        let diffs = SnapshotManager::compute_memory_diff(&old, &new, 0x1000);
        
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].address, 0x1002);
        assert_eq!(diffs[0].old_value, vec![3, 4]);
        assert_eq!(diffs[0].new_value, vec![9, 10]);
        assert_eq!(diffs[1].address, 0x1006);
        assert_eq!(diffs[1].old_value, vec![7]);
        assert_eq!(diffs[1].new_value, vec![11]);
    }
}
