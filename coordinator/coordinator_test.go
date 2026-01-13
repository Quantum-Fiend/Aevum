package main

import (
	"encoding/json"
	"testing"
	"time"
)

func TestCoordinatorStartup(t *testing.T) {
	coordinator := NewCoordinator()

	// Start on a random port
	err := coordinator.Start(":0")
	if err != nil {
		t.Fatalf("Failed to start coordinator: %v", err)
	}
	defer coordinator.Stop()

	// Verify coordinator is running
	stats := coordinator.GetStats()
	if stats["total_traces"].(int) != 0 {
		t.Errorf("Expected 0 traces, got %d", stats["total_traces"])
	}
}

func TestTraceSubmission(t *testing.T) {
	coordinator := NewCoordinator()

	submission := &TraceSubmission{
		NodeID:    "test-node-1",
		TraceID:   "test-trace-1",
		Timestamp: time.Now().UnixNano(),
		Events: []Event{
			{
				EventType: "FunctionCall",
				Metadata: EventMetadata{
					TraceID:        "test-trace-1",
					ProcessID:      1234,
					ThreadID:       1,
					TimestampNs:    1000,
					SequenceNumber: 1,
				},
			},
		},
	}

	err := coordinator.SubmitTrace(submission)
	if err != nil {
		t.Fatalf("Failed to submit trace: %v", err)
	}

	// Verify trace was stored
	trace, err := coordinator.GetTrace("test-trace-1")
	if err != nil {
		t.Fatalf("Failed to get trace: %v", err)
	}

	if trace.EventCount != 1 {
		t.Errorf("Expected 1 event, got %d", trace.EventCount)
	}
}

func TestGlobalTimeline(t *testing.T) {
	coordinator := NewCoordinator()

	// Submit events from multiple nodes
	for i := 0; i < 3; i++ {
		submission := &TraceSubmission{
			NodeID:    "test-node-" + string(rune('A'+i)),
			TraceID:   "timeline-test",
			Timestamp: time.Now().UnixNano(),
			Events: []Event{
				{
					EventType: "FunctionCall",
					Metadata: EventMetadata{
						TraceID:        "timeline-test",
						ProcessID:      1234,
						ThreadID:       uint64(i + 1),
						TimestampNs:    int64(1000 + i*100),
						SequenceNumber: uint64(i + 1),
					},
				},
			},
		}
		coordinator.SubmitTrace(submission)
	}

	// Get global timeline
	timeline, err := coordinator.GetGlobalTimeline("timeline-test")
	if err != nil {
		t.Fatalf("Failed to get timeline: %v", err)
	}

	if len(timeline) != 3 {
		t.Errorf("Expected 3 events in timeline, got %d", len(timeline))
	}

	// Verify events are ordered by timestamp
	for i := 1; i < len(timeline); i++ {
		if timeline[i].Metadata.TimestampNs < timeline[i-1].Metadata.TimestampNs {
			t.Error("Timeline events are not ordered correctly")
		}
	}
}

func TestCausalGraph(t *testing.T) {
	events := []Event{
		{
			EventType: "FunctionCall",
			Metadata: EventMetadata{
				TraceID:        "causal-test",
				ThreadID:       1,
				SequenceNumber: 1,
			},
		},
		{
			EventType: "FunctionCall",
			Metadata: EventMetadata{
				TraceID:        "causal-test",
				ThreadID:       1,
				SequenceNumber: 2,
			},
		},
		{
			EventType: "FunctionCall",
			Metadata: EventMetadata{
				TraceID:        "causal-test",
				ThreadID:       2,
				SequenceNumber: 3,
			},
		},
	}

	graph := BuildCausalGraph(events)

	if len(graph.nodes) != 3 {
		t.Errorf("Expected 3 nodes, got %d", len(graph.nodes))
	}

	// Check same-thread causality edge
	node1 := graph.nodes[1]
	node2 := graph.nodes[2]

	found := false
	for _, n := range node1.HappensBefore {
		if n.SequenceNumber == node2.SequenceNumber {
			found = true
			break
		}
	}

	if !found {
		t.Error("Missing happens-before edge between same-thread events")
	}
}

func TestListTraces(t *testing.T) {
	coordinator := NewCoordinator()

	// Submit multiple traces
	for i := 0; i < 5; i++ {
		submission := &TraceSubmission{
			NodeID:    "test-node",
			TraceID:   "trace-" + string(rune('A'+i)),
			Timestamp: time.Now().UnixNano(),
			Events:    []Event{},
		}
		coordinator.SubmitTrace(submission)
	}

	traces := coordinator.ListTraces()
	if len(traces) != 5 {
		t.Errorf("Expected 5 traces, got %d", len(traces))
	}
}

func TestEventSerialization(t *testing.T) {
	event := Event{
		EventType: "FunctionCall",
		Metadata: EventMetadata{
			TraceID:        "serialize-test",
			ProcessID:      1234,
			ThreadID:       5678,
			TimestampNs:    1000000000,
			VectorClock:    map[int]uint64{1: 10, 2: 20},
			SequenceNumber: 42,
		},
	}

	data, err := json.Marshal(event)
	if err != nil {
		t.Fatalf("Failed to serialize event: %v", err)
	}

	var decoded Event
	err = json.Unmarshal(data, &decoded)
	if err != nil {
		t.Fatalf("Failed to deserialize event: %v", err)
	}

	if decoded.EventType != event.EventType {
		t.Errorf("Event type mismatch: %s vs %s", decoded.EventType, event.EventType)
	}

	if decoded.Metadata.SequenceNumber != event.Metadata.SequenceNumber {
		t.Errorf("Sequence number mismatch")
	}
}
