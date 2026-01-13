# Aevum API Reference

## REST API

The coordinator exposes a REST API on port 8080 (configurable).

### Base URL

```
http://localhost:8080/api
```

---

## Endpoints

### Health Check

```http
GET /health
```

**Response:**
```json
{
    "status": "healthy",
    "time": "2026-01-13T14:30:00Z"
}
```

---

### List Traces

```http
GET /api/traces
```

**Response:**
```json
{
    "traces": ["trace-abc123", "trace-def456"],
    "count": 2
}
```

---

### Get Trace

```http
GET /api/traces/{trace_id}
```

**Parameters:**
- `trace_id` (path): Trace identifier

**Response:**
```json
{
    "trace_id": "trace-abc123",
    "event_count": 1542,
    "node_count": 3,
    "created_at": "2026-01-13T14:00:00Z",
    "last_updated": "2026-01-13T14:30:00Z"
}
```

---

### Get Timeline

```http
GET /api/timeline/{trace_id}
```

**Parameters:**
- `trace_id` (path): Trace identifier

**Response:**
```json
{
    "trace_id": "trace-abc123",
    "events": [
        {
            "event_type": "FunctionCall",
            "metadata": {
                "trace_id": "trace-abc123",
                "process_id": 1234,
                "thread_id": 1,
                "timestamp_ns": 1705153800000000000,
                "sequence_number": 1
            },
            "function_name": "main",
            "module": "app"
        }
    ],
    "count": 1542
}
```

---

### Get Statistics

```http
GET /api/stats
```

**Response:**
```json
{
    "total_traces": 5,
    "total_events": 12543,
    "total_nodes": 8,
    "clock_skews": 3
}
```

---

## Trace Ingestion Protocol

Agents connect to the coordinator on port 9876 (configurable) using TCP.

### Message Format

```
+---------------+------------------+
| Length (4B)   | JSON Data        |
+---------------+------------------+
```

- **Length**: 4-byte big-endian unsigned integer
- **JSON Data**: UTF-8 encoded JSON

### Event Schema

```json
{
    "event_type": "FunctionCall",
    "metadata": {
        "trace_id": "string",
        "process_id": 1234,
        "thread_id": 5678,
        "timestamp_ns": 1705153800000000000,
        "vector_clock": {"1": 10, "2": 5},
        "sequence_number": 42
    },
    "function_name": "example_function",
    "module": "example_module",
    "args": "base64_encoded_data",
    "stack_depth": 3
}
```

### Event Types

| Type | Description | Additional Fields |
|------|-------------|-------------------|
| `FunctionCall` | Function entry | `function_name`, `module`, `args`, `stack_depth` |
| `FunctionReturn` | Function exit | `function_name`, `return_value`, `stack_depth` |
| `MemoryWrite` | Memory modification | `address`, `size`, `old_value`, `new_value` |
| `Syscall` | System call | `syscall_number`, `syscall_name`, `args`, `result` |
| `ThreadSwitch` | Context switch | `from_thread`, `to_thread`, `reason` |
| `NetworkIO` | Network operation | `direction`, `socket_fd`, `remote_addr`, `data` |
| `GarbageCollection` | GC event | `gc_type`, `duration_ns`, `bytes_collected` |
| `ThreadCreate` | Thread creation | `new_thread_id`, `parent_thread_id` |
| `ThreadExit` | Thread termination | `exit_code` |
| `MutexLock` | Lock acquisition | `mutex_id`, `acquired` |
| `MutexUnlock` | Lock release | `mutex_id` |
| `RandomBytes` | Nondeterministic input | `bytes` |
| `Timestamp` | Wall clock capture | `wall_clock_ns` |

---

## CLI Reference

### aevum record

Record execution of a command.

```bash
aevum record [OPTIONS] <COMMAND> [ARGS...]
```

**Options:**
- `-o, --output <FILE>`: Output trace file (default: `trace.aevum`)
- `--cluster`: Enable cluster recording mode
- `-v, --verbose`: Enable verbose output

**Example:**
```bash
aevum record python my_script.py --output my_trace.aevum
```

---

### aevum replay

Replay a trace file.

```bash
aevum replay [OPTIONS] <TRACE_FILE>
```

**Options:**
- `-i, --interactive`: Interactive replay mode

**Interactive Commands:**
- `n` / `next`: Step forward
- `p` / `prev`: Step backward
- `g <N>` / `goto <N>`: Jump to event N
- `q` / `quit`: Exit

---

### aevum rewind

Jump to a specific point in a trace.

```bash
aevum rewind [OPTIONS] <TRACE_FILE>
```

**Options:**
- `-s, --step <N>`: Target event number

---

### aevum inspect

Inspect trace contents.

```bash
aevum inspect [OPTIONS] <TRACE_FILE>
```

**Options:**
- `--causality`: Show causality analysis
- `-e, --event-type <TYPE>`: Filter by event type

---

### aevum attach

Attach to a running process.

```bash
aevum attach [OPTIONS]
```

**Options:**
- `-p, --pid <PID>`: Process ID to attach to
- `-o, --output <FILE>`: Output trace file

---

## Error Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Connection failed |
| 3 | Trace file not found |
| 4 | Invalid trace format |
| 5 | Event not found |
