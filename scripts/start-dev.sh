#!/bin/bash
# Aevum Startup Script
# Starts all Aevum components for development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# Cleanup function
cleanup() {
    log_info "Shutting down Aevum..."
    kill $COORDINATOR_PID 2>/dev/null || true
    kill $UI_PID 2>/dev/null || true
    log_success "Aevum stopped"
}

trap cleanup EXIT

echo ""
echo "═══════════════════════════════════════════"
echo "   Aevum - Time-Travel Debugging Platform"
echo "═══════════════════════════════════════════"
echo ""

# Check dependencies
command -v go >/dev/null 2>&1 || { log_warn "Go not installed"; }
command -v cargo >/dev/null 2>&1 || { log_warn "Rust/Cargo not installed"; }
command -v npm >/dev/null 2>&1 || { log_warn "Node.js/npm not installed"; }

# Start Coordinator
log_info "Starting Coordinator..."
cd "$PROJECT_ROOT/coordinator"
if [ ! -f "aevum-coordinator" ]; then
    log_info "Building coordinator..."
    go build -o aevum-coordinator
fi
./aevum-coordinator &
COORDINATOR_PID=$!
sleep 2

if kill -0 $COORDINATOR_PID 2>/dev/null; then
    log_success "Coordinator started on :9876 (trace) and :8080 (API)"
else
    log_warn "Coordinator failed to start"
    exit 1
fi

# Start UI
log_info "Starting UI..."
cd "$PROJECT_ROOT/ui"
if [ ! -d "node_modules" ]; then
    log_info "Installing UI dependencies..."
    npm install
fi
npm run dev &
UI_PID=$!
sleep 3

log_success "UI started on http://localhost:3000"

echo ""
echo "═══════════════════════════════════════════"
echo "   Aevum is running!"
echo "═══════════════════════════════════════════"
echo ""
echo "   Coordinator (Trace): localhost:9876"
echo "   Coordinator (API):   localhost:8080"
echo "   UI Dashboard:        http://localhost:3000"
echo ""
echo "   Press Ctrl+C to stop all services"
echo ""

# Wait for user interrupt
wait
