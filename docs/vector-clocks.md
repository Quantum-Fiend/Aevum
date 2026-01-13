# Vector Clocks in Aevum

## Overview

Aevum uses **vector clocks** to track causality in distributed systems. This document explains how they work and why they're essential for time-travel debugging.

## The Problem

In a distributed system, events on different nodes don't have a shared clock. We can't simply use wall-clock time to order events because:

1. **Clock skew**: Different machines have different clock times
2. **No global ordering**: Some events are concurrent (neither happened before the other)
3. **Message delays**: Network latency makes timestamps unreliable

## Vector Clock Solution

A vector clock is a data structure that tracks logical time across all processes:

```
VectorClock = {
    process_1: logical_time_1,
    process_2: logical_time_2,
    ...
}
```

### Rules

1. **Local event**: Increment your own clock
   ```
   VC[my_process] += 1
   ```

2. **Send message**: Increment your clock, attach VC to message
   ```
   VC[my_process] += 1
   send(message, VC)
   ```

3. **Receive message**: Merge VCs, then increment
   ```
   VC = merge(VC, received_VC)
   VC[my_process] += 1
   ```

### Merge Operation

```
merge(VC1, VC2) = {
    process: max(VC1[process], VC2[process])
    for each process
}
```

## Happens-Before Relationship

Given two events A and B:

- **A → B** (A happens-before B): `VC(A) < VC(B)`
- **A || B** (A concurrent with B): Neither `VC(A) < VC(B)` nor `VC(B) < VC(A)`

### Comparison

`VC1 < VC2` if and only if:
1. For all processes: `VC1[p] <= VC2[p]`
2. For at least one process: `VC1[p] < VC2[p]`

## Example

```
Node A                          Node B
-------                         -------
[A:1, B:0]  Event 1
[A:2, B:0]  Send M1 ----------> [A:2, B:1]  Receive M1
                                [A:2, B:2]  Event 2
[A:3, B:0]  Event 3  (concurrent with Event 2)
            Receive M2 <------- [A:2, B:3]  Send M2
[A:4, B:3]
```

Analysis:
- Event 1 → Receive M1 (causal)
- Event 3 || Event 2 (concurrent - potential race condition!)

## Implementation in Aevum

### Rust

```rust
pub struct VectorClock {
    pub clocks: HashMap<ProcessId, u64>,
}

impl VectorClock {
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        // Check if self < other
    }
    
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}
```

### Every Event Carries a Vector Clock

```rust
pub struct EventMetadata {
    pub trace_id: TraceId,
    pub process_id: ProcessId,
    pub thread_id: ThreadId,
    pub timestamp_ns: u64,
    pub vector_clock: VectorClock,  // <-- Causality tracking
    pub sequence_number: u64,
}
```

## Race Detection

Two memory accesses are a **potential race** if:
1. They access the same address
2. At least one is a write
3. They are **concurrent** (neither happens-before the other)

```rust
if access1.concurrent_with(&access2) && 
   access1.address == access2.address &&
   (access1.is_write || access2.is_write) {
    report_race(access1, access2);
}
```

## Causal Graph

Aevum builds a **causal graph** from vector clocks:

- **Nodes**: Events
- **Edges**: Happens-before relationships
- **Concurrent events**: No edge between them

This graph powers:
- Timeline reconstruction
- Race condition detection
- Causality visualization in the UI

## Further Reading

- Lamport, L. (1978). "Time, Clocks, and the Ordering of Events in a Distributed System"
- Fidge, C. (1988). "Timestamps in Message-Passing Systems"
- Mattern, F. (1989). "Virtual Time and Global States of Distributed Systems"
