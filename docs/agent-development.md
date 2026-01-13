# Agent Development Guide

This guide explains how to create a new language agent for Aevum.

## Overview

An Aevum agent is responsible for:

1. Instrumenting the target runtime
2. Capturing execution events
3. Serializing events to JSON
4. Streaming events to the coordinator

## Agent Architecture

```
┌─────────────────────────────────────────────────┐
│                  Your Program                    │
├─────────────────────────────────────────────────┤
│                Language Runtime                  │
├─────────────────────────────────────────────────┤
│              Aevum Agent (Your Code)            │
│  ┌─────────┐  ┌──────────┐  ┌─────────────────┐ │
│  │ Hooks   │→ │ Capture  │→ │ Event Sender    │ │
│  └─────────┘  └──────────┘  └─────────────────┘ │
└────────────────────────┬────────────────────────┘
                         │ TCP
                         ▼
                   ┌───────────┐
                   │Coordinator│
                   └───────────┘
```

## Minimal Implementation

### 1. Event Metadata

Every event must include:

```json
{
    "metadata": {
        "trace_id": "string",
        "process_id": 1234,
        "thread_id": 5678,
        "timestamp_ns": 1705153800000000000,
        "sequence_number": 1
    }
}
```

### 2. Core Functions

```pseudocode
class AevumAgent:
    trace_id: string
    socket: Connection
    sequence: atomic_int
    enabled: bool
    
    function connect(host, port):
        socket = tcp_connect(host, port)
    
    function send_event(event):
        if not enabled: return
        
        json = serialize_to_json(event)
        length = len(json)
        
        socket.send(length as 4-byte big-endian)
        socket.send(json)
    
    function create_metadata() -> EventMetadata:
        return EventMetadata(
            trace_id=self.trace_id,
            process_id=get_pid(),
            thread_id=get_thread_id(),
            timestamp_ns=get_nanoseconds(),
            sequence_number=atomic_increment(self.sequence)
        )
```

### 3. Instrumentation Hooks

Depending on the language, implement hooks for:

| Hook Type | When to Trigger |
|-----------|-----------------|
| Function Entry | Before function body executes |
| Function Exit | Before function returns |
| Memory Write | Before memory modification |
| Thread Create | When new thread starts |
| Thread Exit | When thread terminates |

## Language-Specific Techniques

### Python

Use `sys.settrace()`:

```python
def trace_function(frame, event, arg):
    if event == 'call':
        record_function_call(frame)
    elif event == 'return':
        record_function_return(frame, arg)
    return trace_function

sys.settrace(trace_function)
```

### Go

Use runtime hooks (requires build tags):

```go
//go:linkname gopark runtime.gopark
func gopark(unlockf func(*g, unsafe.Pointer) bool, lock unsafe.Pointer, reason waitReason, traceEv byte, traceskip int)
```

### Node.js

Use Inspector Protocol:

```javascript
const inspector = require('inspector');
const session = new inspector.Session();
session.connect();
session.post('Debugger.enable');
session.on('Debugger.paused', handlePause);
```

### Java

Use ASM for bytecode instrumentation:

```java
public class MethodVisitor extends AdviceAdapter {
    @Override
    protected void onMethodEnter() {
        // Insert call to agent
    }
}
```

### C/C++

Use LLVM IR instrumentation:

```cpp
// LLVM Pass
bool runOnFunction(Function &F) {
    // Insert call to __aevum_function_entry
}
```

## Event Types to Implement

### Required

1. **FunctionCall** - Function entry
2. **FunctionReturn** - Function exit

### Recommended

3. **ThreadCreate** - Thread/goroutine/async task creation
4. **ThreadExit** - Thread termination
5. **Syscall** - System calls (if accessible)

### Optional

6. **MemoryWrite** - Memory modifications
7. **NetworkIO** - Network operations
8. **GarbageCollection** - GC events
9. **MutexLock/Unlock** - Synchronization

## Testing Your Agent

### 1. Unit Tests

Test event serialization:

```python
def test_event_serialization():
    event = create_function_call_event()
    json = serialize(event)
    assert json is valid JSON
    assert json contains correct fields
```

### 2. Integration Tests

Test with coordinator:

```bash
# Start coordinator
./aevum-coordinator &

# Run instrumented program
python -c "import your_agent; your_agent.attach(...); ..."

# Verify events received
curl http://localhost:8080/api/stats
```

### 3. Replay Tests

Verify deterministic replay:

```bash
# Record
aevum record python test.py --output test.aevum

# Replay
aevum replay test.aevum --verify
```

## Performance Considerations

1. **Buffer events**: Don't send each event immediately
2. **Compress data**: Use zstd or similar
3. **Filter noise**: Exclude standard library calls
4. **Async sending**: Don't block the main program

## Example Agent Structure

```
agents/your-language-agent/
├── src/
│   ├── agent.ext        # Main agent code
│   ├── hooks.ext        # Instrumentation hooks
│   ├── events.ext       # Event types
│   └── transport.ext    # Network communication
├── tests/
│   └── test_agent.ext
├── examples/
│   └── basic_usage.ext
└── README.md
```

## Checklist

- [ ] Implement connect/disconnect
- [ ] Implement event metadata
- [ ] Implement function call/return hooks
- [ ] Implement event serialization
- [ ] Implement event sending (length-prefixed)
- [ ] Add thread safety
- [ ] Add error handling
- [ ] Add logging
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Create usage examples
- [ ] Document installation
- [ ] Add to CI/CD

## Resources

- [Protocol Buffers Definition](../common/proto/events.proto)
- [JSON Schema](../common/schemas/trace-file.schema.json)
- [Existing Agents](../agents/)
- [API Documentation](./api.md)
