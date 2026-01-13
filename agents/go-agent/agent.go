package main

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"runtime"
	"sync"
	"sync/atomic"
	"time"
)

// EventMetadata contains metadata for all events
type EventMetadata struct {
	TraceID        string `json:"trace_id"`
	ProcessID      int    `json:"process_id"`
	ThreadID       uint64 `json:"thread_id"`
	TimestampNs    int64  `json:"timestamp_ns"`
	SequenceNumber uint64 `json:"sequence_number"`
}

// GoroutineCreateEvent represents goroutine creation
type GoroutineCreateEvent struct {
	Metadata         EventMetadata `json:"metadata"`
	NewGoroutineID   uint64        `json:"new_goroutine_id"`
	ParentGoroutineID uint64       `json:"parent_goroutine_id"`
	EventType        string        `json:"event_type"`
}

// FunctionCallEvent represents a function call
type FunctionCallEvent struct {
	Metadata     EventMetadata `json:"metadata"`
	FunctionName string        `json:"function_name"`
	Module       string        `json:"module"`
	StackDepth   int           `json:"stack_depth"`
	EventType    string        `json:"event_type"`
}

// ChannelOpEvent represents a channel operation
type ChannelOpEvent struct {
	Metadata  EventMetadata `json:"metadata"`
	ChannelID uint64        `json:"channel_id"`
	Operation string        `json:"operation"` // "send" or "receive"
	EventType string        `json:"event_type"`
}

// Agent is the Go tracing agent
type Agent struct {
	traceID        string
	serverAddr     string
	conn           net.Conn
	sequenceNumber uint64
	enabled        bool
	mu             sync.Mutex
}

// NewAgent creates a new Go agent
func NewAgent(traceID, serverAddr string) *Agent {
	return &Agent{
		traceID:    traceID,
		serverAddr: serverAddr,
		enabled:    false,
	}
}

// Connect establishes connection to the trace server
func (a *Agent) Connect() error {
	conn, err := net.Dial("tcp", a.serverAddr)
	if err != nil {
		return fmt.Errorf("failed to connect to trace server: %w", err)
	}
	a.conn = conn
	fmt.Printf("[Aevum] Connected to trace server at %s\n", a.serverAddr)
	return nil
}

// Disconnect closes the connection
func (a *Agent) Disconnect() {
	if a.conn != nil {
		a.conn.Close()
		a.conn = nil
	}
}

// SendEvent sends an event to the trace server
func (a *Agent) SendEvent(event interface{}) error {
	if a.conn == nil {
		return fmt.Errorf("not connected to trace server")
	}

	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}

	// Send length prefix
	length := uint32(len(data))
	if err := binary.Write(a.conn, binary.BigEndian, length); err != nil {
		return fmt.Errorf("failed to write length: %w", err)
	}

	// Send data
	if _, err := a.conn.Write(data); err != nil {
		return fmt.Errorf("failed to write data: %w", err)
	}

	return nil
}

// CreateMetadata creates event metadata
func (a *Agent) CreateMetadata() EventMetadata {
	seq := atomic.AddUint64(&a.sequenceNumber, 1)
	return EventMetadata{
		TraceID:        a.traceID,
		ProcessID:      os.Getpid(),
		ThreadID:       getGoroutineID(),
		TimestampNs:    time.Now().UnixNano(),
		SequenceNumber: seq,
	}
}

// RecordFunctionCall records a function call
func (a *Agent) RecordFunctionCall(functionName, module string, stackDepth int) {
	if !a.enabled {
		return
	}

	event := FunctionCallEvent{
		Metadata:     a.CreateMetadata(),
		FunctionName: functionName,
		Module:       module,
		StackDepth:   stackDepth,
		EventType:    "FunctionCall",
	}

	if err := a.SendEvent(event); err != nil {
		fmt.Printf("[Aevum] Failed to send event: %v\n", err)
	}
}

// RecordGoroutineCreate records goroutine creation
func (a *Agent) RecordGoroutineCreate(newGoroutineID, parentGoroutineID uint64) {
	if !a.enabled {
		return
	}

	event := GoroutineCreateEvent{
		Metadata:          a.CreateMetadata(),
		NewGoroutineID:    newGoroutineID,
		ParentGoroutineID: parentGoroutineID,
		EventType:         "GoroutineCreate",
	}

	if err := a.SendEvent(event); err != nil {
		fmt.Printf("[Aevum] Failed to send event: %v\n", err)
	}
}

// RecordChannelOp records a channel operation
func (a *Agent) RecordChannelOp(channelID uint64, operation string) {
	if !a.enabled {
		return
	}

	event := ChannelOpEvent{
		Metadata:  a.CreateMetadata(),
		ChannelID: channelID,
		Operation: operation,
		EventType: "ChannelOp",
	}

	if err := a.SendEvent(event); err != nil {
		fmt.Printf("[Aevum] Failed to send event: %v\n", err)
	}
}

// Start starts the agent
func (a *Agent) Start() error {
	if err := a.Connect(); err != nil {
		return err
	}
	a.enabled = true
	fmt.Printf("[Aevum] Go agent started (trace_id: %s)\n", a.traceID)
	return nil
}

// Stop stops the agent
func (a *Agent) Stop() {
	a.enabled = false
	a.Disconnect()
	fmt.Println("[Aevum] Go agent stopped")
}

// getGoroutineID returns the current goroutine ID
func getGoroutineID() uint64 {
	// This is a simplified implementation
	// In production, you'd use runtime.Stack() parsing or unsafe tricks
	return uint64(runtime.NumGoroutine())
}

// Global agent instance
var globalAgent *Agent
var agentMu sync.Mutex

// Attach attaches the agent to the current process
func Attach(traceID, serverAddr string) (*Agent, error) {
	agentMu.Lock()
	defer agentMu.Unlock()

	if globalAgent != nil {
		return globalAgent, fmt.Errorf("agent already attached")
	}

	agent := NewAgent(traceID, serverAddr)
	if err := agent.Start(); err != nil {
		return nil, err
	}

	globalAgent = agent
	return agent, nil
}

// Detach detaches the agent
func Detach() {
	agentMu.Lock()
	defer agentMu.Unlock()

	if globalAgent != nil {
		globalAgent.Stop()
		globalAgent = nil
	}
}

// GetAgent returns the global agent instance
func GetAgent() *Agent {
	agentMu.Lock()
	defer agentMu.Unlock()
	return globalAgent
}
