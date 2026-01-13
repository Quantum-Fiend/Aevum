# Aevum Development Guide

## Building from Source

### Rust Components

```bash
# Build all Rust workspace members
cargo build --release --workspace

# Run tests
cargo test --workspace

# Build specific component
cargo build --release -p trace-engine
cargo build --release -p replay-engine
cargo build --release -p aevum-cli
```

### Go Coordinator

```bash
cd coordinator

# Download dependencies
go mod download

# Build
go build -o aevum-coordinator

# Run tests
go test ./...

# Run coordinator
./aevum-coordinator
```

### TypeScript UI

```bash
cd ui

# Install dependencies
npm install

# Development server
npm run dev

# Production build
npm run build

# Preview production build
npm run preview
```

## Running the Full Stack

### Option 1: Docker Compose (Recommended)

```bash
docker-compose up -d
```

Services:
- Coordinator: `localhost:9876` (trace ingestion), `localhost:8080` (HTTP API)
- UI: `localhost:3000`

### Option 2: Manual

Terminal 1 - Coordinator:
```bash
cd coordinator
go run main.go causality.go api.go
```

Terminal 2 - UI:
```bash
cd ui
npm run dev
```

## Testing

### Unit Tests

```bash
# Rust
cargo test --workspace

# Go
cd coordinator && go test ./...

# TypeScript
cd ui && npm test
```

### Integration Tests

```bash
# Run example programs
python examples/python_fibonacci.py
go run examples/go_concurrent.go
node examples/node_async.js
```

### CLI Testing

```bash
# Build CLI
cargo build --release -p aevum-cli

# Test recording
./target/release/aevum record python examples/python_fibonacci.py

# Test replay
./target/release/aevum replay trace.aevum --interactive

# Test inspection
./target/release/aevum inspect trace.aevum --causality
```

## Architecture Deep Dive

### Trace Engine (Rust)

The trace engine is the core of Aevum, responsible for:

1. **Event Capture**: Receiving events from language agents
2. **Compression**: Using zstd to compress events
3. **Persistence**: Writing to append-only log
4. **Vector Clocks**: Tracking distributed causality
5. **Snapshots**: Creating checkpoints for fast seeking

Key files:
- `trace-engine/src/event.rs` - Event types and vector clocks
- `trace-engine/src/trace_log.rs` - Append-only log implementation
- `trace-engine/src/snapshot.rs` - Snapshot system

### Language Agents

Each language agent must:

1. Hook into the runtime
2. Capture events without modifying source code
3. Serialize events to a common format
4. Stream events to the coordinator

Agent implementations:
- **Python**: Uses `sys.settrace` for bytecode instrumentation
- **Go**: Uses runtime hooks for goroutine tracking
- **Node.js**: Uses Inspector Protocol for V8 integration

### Distributed Coordinator (Go)

The coordinator:

1. Accepts trace submissions from multiple nodes
2. Compensates for clock skew
3. Merges traces into a global timeline
4. Builds causal graphs
5. Detects race conditions
6. Serves HTTP API for the UI

### Replay Engine (Rust)

The replay engine:

1. Loads trace files
2. Supports forward/backward stepping
3. Allows seeking to arbitrary points
4. Provides nondeterministic input replay
5. Enforces deterministic execution order

### UI (TypeScript + React)

The UI provides:

1. **Timeline View**: D3-based visualization of events
2. **State Viewer**: Event details and context
3. **Causality Graph**: Cytoscape.js graph visualization
4. **Controls**: Playback controls with timeline scrubber

## Performance Considerations

### Trace Compression

Events are compressed using zstd level 3, providing:
- ~70% size reduction
- Fast compression/decompression
- Minimal CPU overhead

### Snapshot Intervals

Default snapshot interval: 1000 events

Adjust based on:
- Memory constraints
- Seek performance requirements
- Trace size

### Network Optimization

For distributed tracing:
- Batch events before sending
- Use compression for network transfer
- Implement backpressure handling

## Troubleshooting

### Agent Connection Issues

If agents can't connect to the coordinator:

1. Check coordinator is running: `netstat -an | grep 9876`
2. Verify firewall settings
3. Check agent configuration (host/port)

### Large Trace Files

For very large traces:

1. Increase snapshot interval
2. Use event filtering
3. Split traces by time window

### UI Performance

If the UI is slow:

1. Limit events displayed in timeline (default: all)
2. Reduce causality graph size (default: 100 events)
3. Use Chrome DevTools to profile

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT - see [LICENSE](LICENSE)
