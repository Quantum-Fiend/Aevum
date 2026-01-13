package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"
)

// EventMetadata represents metadata for all events
type EventMetadata struct {
	TraceID        string         `json:"trace_id"`
	ProcessID      int            `json:"process_id"`
	ThreadID       uint64         `json:"thread_id"`
	TimestampNs    int64          `json:"timestamp_ns"`
	VectorClock    map[int]uint64 `json:"vector_clock"`
	SequenceNumber uint64         `json:"sequence_number"`
}

// Event represents a traced event
type Event struct {
	EventType string          `json:"event_type"`
	Metadata  EventMetadata   `json:"metadata"`
	Data      json.RawMessage `json:"data,omitempty"`
}

// TraceSubmission represents a trace submitted by an agent
type TraceSubmission struct {
	NodeID    string  `json:"node_id"`
	TraceID   string  `json:"trace_id"`
	Events    []Event `json:"events"`
	Timestamp int64   `json:"timestamp"`
}

// Coordinator manages distributed trace collection and merging
type Coordinator struct {
	mu             sync.RWMutex
	traces         map[string]*MergedTrace
	nodeClockSkews map[string]int64
	listener       net.Listener
	ctx            context.Context
	cancel         context.CancelFunc
}

// MergedTrace represents a merged trace from multiple nodes
type MergedTrace struct {
	TraceID     string
	Events      []Event
	Nodes       map[string]bool
	CreatedAt   time.Time
	LastUpdated time.Time
	EventCount  int
	mu          sync.RWMutex
}

// NewCoordinator creates a new coordinator
func NewCoordinator() *Coordinator {
	ctx, cancel := context.WithCancel(context.Background())
	return &Coordinator{
		traces:         make(map[string]*MergedTrace),
		nodeClockSkews: make(map[string]int64),
		ctx:            ctx,
		cancel:         cancel,
	}
}

// Start starts the coordinator server
func (c *Coordinator) Start(addr string) error {
	listener, err := net.Listen("tcp", addr)
	if err != nil {
		return fmt.Errorf("failed to listen: %w", err)
	}

	c.listener = listener
	log.Printf("[Coordinator] Started on %s", addr)

	go c.acceptConnections()

	return nil
}

// Stop stops the coordinator
func (c *Coordinator) Stop() {
	c.cancel()
	if c.listener != nil {
		c.listener.Close()
	}
	log.Println("[Coordinator] Stopped")
}

// acceptConnections accepts incoming connections from agents
func (c *Coordinator) acceptConnections() {
	for {
		select {
		case <-c.ctx.Done():
			return
		default:
		}

		conn, err := c.listener.Accept()
		if err != nil {
			if c.ctx.Err() != nil {
				return
			}
			log.Printf("[Coordinator] Accept error: %v", err)
			continue
		}

		go c.handleConnection(conn)
	}
}

// handleConnection handles a connection from an agent
func (c *Coordinator) handleConnection(conn net.Conn) {
	defer conn.Close()

	log.Printf("[Coordinator] New connection from %s", conn.RemoteAddr())

	decoder := json.NewDecoder(conn)

	for {
		var submission TraceSubmission
		if err := decoder.Decode(&submission); err != nil {
			if err.Error() != "EOF" {
				log.Printf("[Coordinator] Decode error: %v", err)
			}
			return
		}

		if err := c.SubmitTrace(&submission); err != nil {
			log.Printf("[Coordinator] Failed to submit trace: %v", err)
		}
	}
}

// SubmitTrace submits a trace from a node
func (c *Coordinator) SubmitTrace(submission *TraceSubmission) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	trace, exists := c.traces[submission.TraceID]
	if !exists {
		trace = &MergedTrace{
			TraceID:     submission.TraceID,
			Events:      make([]Event, 0),
			Nodes:       make(map[string]bool),
			CreatedAt:   time.Now(),
			LastUpdated: time.Now(),
		}
		c.traces[submission.TraceID] = trace
	}

	trace.mu.Lock()
	defer trace.mu.Unlock()

	// Compensate for clock skew
	skew := c.getClockSkew(submission.NodeID, submission.Timestamp)

	// Add events with clock skew compensation
	for _, event := range submission.Events {
		event.Metadata.TimestampNs += skew
		trace.Events = append(trace.Events, event)
	}

	trace.Nodes[submission.NodeID] = true
	trace.LastUpdated = time.Now()
	trace.EventCount += len(submission.Events)

	log.Printf("[Coordinator] Received %d events from node %s for trace %s (total: %d events from %d nodes)",
		len(submission.Events), submission.NodeID, submission.TraceID, trace.EventCount, len(trace.Nodes))

	return nil
}

// getClockSkew calculates and caches clock skew for a node
func (c *Coordinator) getClockSkew(nodeID string, nodeTimestamp int64) int64 {
	coordinatorTime := time.Now().UnixNano()
	skew := coordinatorTime - nodeTimestamp

	// Cache the skew
	c.nodeClockSkews[nodeID] = skew

	return skew
}

// GetTrace retrieves a merged trace
func (c *Coordinator) GetTrace(traceID string) (*MergedTrace, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	trace, exists := c.traces[traceID]
	if !exists {
		return nil, fmt.Errorf("trace not found: %s", traceID)
	}

	return trace, nil
}

// GetGlobalTimeline returns events sorted by causal order
func (c *Coordinator) GetGlobalTimeline(traceID string) ([]Event, error) {
	trace, err := c.GetTrace(traceID)
	if err != nil {
		return nil, err
	}

	trace.mu.RLock()
	defer trace.mu.RUnlock()

	// Sort events by vector clock (causal order)
	events := make([]Event, len(trace.Events))
	copy(events, trace.Events)

	// Simple timestamp-based sorting (in production, use vector clock comparison)
	sortEventsByTimestamp(events)

	return events, nil
}

// sortEventsByTimestamp sorts events by timestamp
func sortEventsByTimestamp(events []Event) {
	// Bubble sort for simplicity (use a better algorithm in production)
	n := len(events)
	for i := 0; i < n-1; i++ {
		for j := 0; j < n-i-1; j++ {
			if events[j].Metadata.TimestampNs > events[j+1].Metadata.TimestampNs {
				events[j], events[j+1] = events[j+1], events[j]
			}
		}
	}
}

// ListTraces returns all trace IDs
func (c *Coordinator) ListTraces() []string {
	c.mu.RLock()
	defer c.mu.RUnlock()

	traces := make([]string, 0, len(c.traces))
	for traceID := range c.traces {
		traces = append(traces, traceID)
	}

	return traces
}

// GetStats returns coordinator statistics
func (c *Coordinator) GetStats() map[string]interface{} {
	c.mu.RLock()
	defer c.mu.RUnlock()

	totalEvents := 0
	totalNodes := 0

	for _, trace := range c.traces {
		trace.mu.RLock()
		totalEvents += trace.EventCount
		totalNodes += len(trace.Nodes)
		trace.mu.RUnlock()
	}

	return map[string]interface{}{
		"total_traces": len(c.traces),
		"total_events": totalEvents,
		"total_nodes":  totalNodes,
		"clock_skews":  len(c.nodeClockSkews),
	}
}

func main() {
	coordinator := NewCoordinator()

	// Start Agent Listener
	go func() {
		if err := coordinator.Start(":9876"); err != nil {
			log.Fatalf("Failed to start coordinator agent listener: %v", err)
		}
	}()

	// Start API Server
	apiServer := NewAPIServer(coordinator, ":8080")
	go func() {
		if err := apiServer.Start(); err != nil {
			log.Fatalf("Failed to start API server: %v", err)
		}
	}()

	// Print stats periodically
	go func() {
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()

		for range ticker.C {
			stats := coordinator.GetStats()
			log.Printf("[Stats] %+v", stats)
		}
	}()

	// Handle graceful shutdown
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)
	<-stop

	log.Println("Shutting down...")
	coordinator.Stop()
	apiServer.Stop()
}
