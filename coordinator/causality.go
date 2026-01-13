package main

import (
	"fmt"
)

// CausalGraph represents a directed graph of causal relationships
type CausalGraph struct {
	nodes map[uint64]*CausalNode
	edges []*CausalEdge
}

// CausalNode represents a node in the causal graph (an event)
type CausalNode struct {
	SequenceNumber uint64
	Event          Event
	HappensBefore  []*CausalNode
	HappensAfter   []*CausalNode
}

// CausalEdge represents a happens-before relationship
type CausalEdge struct {
	From *CausalNode
	To   *CausalNode
	Type EdgeType
}

// EdgeType represents the type of causal relationship
type EdgeType int

const (
	EdgeTypeSameThread EdgeType = iota
	EdgeTypeSynchronization
	EdgeTypeNetworkMessage
	EdgeTypeMemoryDependency
)

// NewCausalGraph creates a new causal graph
func NewCausalGraph() *CausalGraph {
	return &CausalGraph{
		nodes: make(map[uint64]*CausalNode),
		edges: make([]*CausalEdge, 0),
	}
}

// AddEvent adds an event to the causal graph
func (g *CausalGraph) AddEvent(event Event) *CausalNode {
	node := &CausalNode{
		SequenceNumber: event.Metadata.SequenceNumber,
		Event:          event,
		HappensBefore:  make([]*CausalNode, 0),
		HappensAfter:   make([]*CausalNode, 0),
	}

	g.nodes[event.Metadata.SequenceNumber] = node
	return node
}

// AddEdge adds a causal edge between two events
func (g *CausalGraph) AddEdge(from, to *CausalNode, edgeType EdgeType) {
	edge := &CausalEdge{
		From: from,
		To:   to,
		Type: edgeType,
	}

	from.HappensBefore = append(from.HappensBefore, to)
	to.HappensAfter = append(to.HappensAfter, from)
	g.edges = append(g.edges, edge)
}

// BuildCausalGraph builds a causal graph from events
func BuildCausalGraph(events []Event) *CausalGraph {
	graph := NewCausalGraph()

	// Add all events as nodes
	for _, event := range events {
		graph.AddEvent(event)
	}

	// Build edges based on causal relationships
	threadLastEvent := make(map[uint64]*CausalNode)

	for i := range events {
		event := &events[i]
		currentNode := graph.nodes[event.Metadata.SequenceNumber]

		// Same-thread causality
		if lastEvent, exists := threadLastEvent[event.Metadata.ThreadID]; exists {
			graph.AddEdge(lastEvent, currentNode, EdgeTypeSameThread)
		}
		threadLastEvent[event.Metadata.ThreadID] = currentNode

		// TODO: Add synchronization, network, and memory dependency edges
	}

	return graph
}

// DetectRaceConditions detects potential race conditions in the trace
func (g *CausalGraph) DetectRaceConditions() []RaceCondition {
	races := make([]RaceCondition, 0)

	// Find concurrent memory accesses to the same address
	memoryAccesses := make(map[uint64][]*CausalNode)

	for _, node := range g.nodes {
		if node.Event.EventType == "MemoryWrite" {
			// Extract address from event data
			// This is simplified; in production, parse the event data
			address := uint64(0) // Placeholder
			memoryAccesses[address] = append(memoryAccesses[address], node)
		}
	}

	// Check for concurrent accesses
	for address, accesses := range memoryAccesses {
		if len(accesses) < 2 {
			continue
		}

		for i := 0; i < len(accesses); i++ {
			for j := i + 1; j < len(accesses); j++ {
				if g.areConcurrent(accesses[i], accesses[j]) {
					races = append(races, RaceCondition{
						Address: address,
						Event1:  accesses[i].Event,
						Event2:  accesses[j].Event,
					})
				}
			}
		}
	}

	return races
}

// areConcurrent checks if two nodes are concurrent (neither happens-before the other)
func (g *CausalGraph) areConcurrent(node1, node2 *CausalNode) bool {
	return !g.happensBefore(node1, node2) && !g.happensBefore(node2, node1)
}

// happensBefore checks if node1 happens-before node2
func (g *CausalGraph) happensBefore(node1, node2 *CausalNode) bool {
	// BFS to check reachability
	visited := make(map[uint64]bool)
	queue := []*CausalNode{node1}

	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]

		if current.SequenceNumber == node2.SequenceNumber {
			return true
		}

		if visited[current.SequenceNumber] {
			continue
		}
		visited[current.SequenceNumber] = true

		queue = append(queue, current.HappensBefore...)
	}

	return false
}

// RaceCondition represents a detected race condition
type RaceCondition struct {
	Address uint64
	Event1  Event
	Event2  Event
}

func (rc RaceCondition) String() string {
	return fmt.Sprintf("Race on address 0x%x between events %d and %d",
		rc.Address, rc.Event1.Metadata.SequenceNumber, rc.Event2.Metadata.SequenceNumber)
}
