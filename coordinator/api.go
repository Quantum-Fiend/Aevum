package main

import (
	"encoding/json"
	"log"
	"net/http"
	"time"
)

// APIServer provides HTTP API for querying traces
type APIServer struct {
	coordinator *Coordinator
	server      *http.Server
}

// NewAPIServer creates a new API server
func NewAPIServer(coordinator *Coordinator, addr string) *APIServer {
	mux := http.NewServeMux()

	api := &APIServer{
		coordinator: coordinator,
		server: &http.Server{
			Addr:         addr,
			Handler:      mux,
			ReadTimeout:  10 * time.Second,
			WriteTimeout: 10 * time.Second,
		},
	}

	// Register routes
	mux.HandleFunc("/api/traces", api.handleListTraces)
	mux.HandleFunc("/api/traces/", api.handleGetTrace)
	mux.HandleFunc("/api/timeline/", api.handleGetTimeline)
	mux.HandleFunc("/api/stats", api.handleGetStats)
	mux.HandleFunc("/health", api.handleHealth)

	return api
}

// Start starts the API server
func (s *APIServer) Start() error {
	log.Printf("[API] Starting on %s", s.server.Addr)
	return s.server.ListenAndServe()
}

// Stop stops the API server
func (s *APIServer) Stop() error {
	return s.server.Close()
}

// handleListTraces lists all traces
func (s *APIServer) handleListTraces(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	traces := s.coordinator.ListTraces()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"traces": traces,
		"count":  len(traces),
	})
}

// handleGetTrace gets a specific trace
func (s *APIServer) handleGetTrace(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	traceID := r.URL.Path[len("/api/traces/"):]
	if traceID == "" {
		http.Error(w, "Trace ID required", http.StatusBadRequest)
		return
	}

	trace, err := s.coordinator.GetTrace(traceID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusNotFound)
		return
	}

	trace.mu.RLock()
	defer trace.mu.RUnlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"trace_id":     trace.TraceID,
		"event_count":  trace.EventCount,
		"node_count":   len(trace.Nodes),
		"created_at":   trace.CreatedAt,
		"last_updated": trace.LastUpdated,
	})
}

// handleGetTimeline gets the global timeline for a trace
func (s *APIServer) handleGetTimeline(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	traceID := r.URL.Path[len("/api/timeline/"):]
	if traceID == "" {
		http.Error(w, "Trace ID required", http.StatusBadRequest)
		return
	}

	events, err := s.coordinator.GetGlobalTimeline(traceID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"trace_id": traceID,
		"events":   events,
		"count":    len(events),
	})
}

// handleGetStats gets coordinator statistics
func (s *APIServer) handleGetStats(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	stats := s.coordinator.GetStats()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(stats)
}

// handleHealth health check endpoint
func (s *APIServer) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status": "healthy",
		"time":   time.Now().Format(time.RFC3339),
	})
}
