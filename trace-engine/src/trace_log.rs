use crate::event::Event;
use anyhow::{Context, Result};
use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Append-only trace log with compression and memory-mapped I/O
pub struct TraceLog {
    path: PathBuf,
    file: Arc<RwLock<File>>,
    mmap: Arc<RwLock<Option<MmapMut>>>,
    event_count: Arc<RwLock<u64>>,
    compress: bool,
}

impl TraceLog {
    /// Create a new trace log at the specified path
    pub fn create<P: AsRef<Path>>(path: P, compress: bool) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create trace log directory")?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&path)
            .context("Failed to create trace log file")?;

        info!("Created trace log at: {:?}", path);

        Ok(Self {
            path,
            file: Arc::new(RwLock::new(file)),
            mmap: Arc::new(RwLock::new(None)),
            event_count: Arc::new(RwLock::new(0)),
            compress,
        })
    }

    /// Open an existing trace log
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .context("Failed to open trace log file")?;

        info!("Opened trace log at: {:?}", path);

        // Count existing events
        let event_count = Self::count_events(&file)?;

        Ok(Self {
            path,
            file: Arc::new(RwLock::new(file)),
            mmap: Arc::new(RwLock::new(None)),
            event_count: Arc::new(RwLock::new(event_count)),
            compress: true, // Assume compression for existing logs
        })
    }

    /// Append an event to the trace log
    pub fn append(&self, event: &Event) -> Result<u64> {
        let mut file = self.file.write();
        
        // Serialize the event
        let serialized = bincode::serialize(event)
            .context("Failed to serialize event")?;

        // Optionally compress
        let data = if self.compress {
            zstd::encode_all(&serialized[..], 3)
                .context("Failed to compress event")?
        } else {
            serialized
        };

        // Write length prefix (4 bytes) + compressed flag (1 byte) + data
        let len = data.len() as u32;
        file.write_all(&len.to_le_bytes())
            .context("Failed to write event length")?;
        file.write_all(&[self.compress as u8])
            .context("Failed to write compression flag")?;
        file.write_all(&data)
            .context("Failed to write event data")?;
        file.flush()
            .context("Failed to flush trace log")?;

        let mut count = self.event_count.write();
        *count += 1;
        let event_id = *count;

        debug!(
            "Appended event #{}: {} ({} bytes, compressed: {})",
            event_id,
            event.event_type(),
            data.len(),
            self.compress
        );

        Ok(event_id)
    }

    /// Read all events from the trace log
    pub fn read_all(&self) -> Result<Vec<Event>> {
        let file = self.file.read();
        let metadata = file.metadata()
            .context("Failed to get file metadata")?;
        
        if metadata.len() == 0 {
            return Ok(Vec::new());
        }

        drop(file);

        let file = File::open(&self.path)
            .context("Failed to open trace log for reading")?;
        
        let mut events = Vec::new();
        let mut reader = std::io::BufReader::new(file);
        
        use std::io::Read;
        
        loop {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e).context("Failed to read event length")?,
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            // Read compression flag
            let mut compress_flag = [0u8; 1];
            reader.read_exact(&mut compress_flag)
                .context("Failed to read compression flag")?;
            let compressed = compress_flag[0] != 0;

            // Read event data
            let mut data = vec![0u8; len];
            reader.read_exact(&mut data)
                .context("Failed to read event data")?;

            // Decompress if needed
            let serialized = if compressed {
                zstd::decode_all(&data[..])
                    .context("Failed to decompress event")?
            } else {
                data
            };

            // Deserialize
            let event: Event = bincode::deserialize(&serialized)
                .context("Failed to deserialize event")?;
            
            events.push(event);
        }

        info!("Read {} events from trace log", events.len());
        Ok(events)
    }

    /// Get the number of events in the log
    pub fn event_count(&self) -> u64 {
        *self.event_count.read()
    }

    /// Get the path to the trace log file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Count events in a file
    fn count_events(file: &File) -> Result<u64> {
        let metadata = file.metadata()
            .context("Failed to get file metadata")?;
        
        if metadata.len() == 0 {
            return Ok(0);
        }

        let mut count = 0u64;
        let mut reader = std::io::BufReader::new(file);
        
        use std::io::{Read, Seek};
        
        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e).context("Failed to read event length")?,
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            // Skip compression flag (1 byte) + data
            reader.seek(std::io::SeekFrom::Current((1 + len) as i64))
                .context("Failed to seek past event")?;
            
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventMetadata, ProcessId, ThreadId, TraceId, VectorClock};
    use tempfile::tempdir;

    #[test]
    fn test_trace_log_create_and_append() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.trace");
        
        let log = TraceLog::create(&log_path, true).unwrap();
        
        let metadata = EventMetadata {
            trace_id: TraceId::new(),
            process_id: ProcessId(1),
            thread_id: ThreadId(1),
            timestamp_ns: 1000,
            vector_clock: VectorClock::new(),
            sequence_number: 1,
        };

        let event = Event::FunctionCall {
            metadata,
            function_name: "test_function".to_string(),
            module: "test_module".to_string(),
            args: vec![1, 2, 3],
            stack_depth: 1,
        };

        let event_id = log.append(&event).unwrap();
        assert_eq!(event_id, 1);
        assert_eq!(log.event_count(), 1);
    }

    #[test]
    fn test_trace_log_read_all() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("test.trace");
        
        let log = TraceLog::create(&log_path, true).unwrap();
        
        let trace_id = TraceId::new();
        
        for i in 0..10 {
            let metadata = EventMetadata {
                trace_id,
                process_id: ProcessId(1),
                thread_id: ThreadId(1),
                timestamp_ns: 1000 + i,
                vector_clock: VectorClock::new(),
                sequence_number: i + 1,
            };

            let event = Event::FunctionCall {
                metadata,
                function_name: format!("function_{}", i),
                module: "test_module".to_string(),
                args: vec![],
                stack_depth: 1,
            };

            log.append(&event).unwrap();
        }

        let events = log.read_all().unwrap();
        assert_eq!(events.len(), 10);
    }
}
