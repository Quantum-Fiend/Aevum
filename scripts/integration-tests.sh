#!/bin/bash

# Integration test script for Aevum

set -e

echo "🧪 Running Aevum Integration Tests"
echo "=================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((TESTS_FAILED++))
}

info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# 1. Test: Build all Rust components
echo ""
info "Test 1: Building Rust components..."
if cargo build --release --workspace 2>&1 | grep -q "Finished"; then
    pass "Rust components built successfully"
else
    fail "Rust build failed"
fi

# 2. Test: Build Go coordinator
echo ""
info "Test 2: Building Go coordinator..."
cd coordinator
if go build -o aevum-coordinator 2>&1; then
    pass "Go coordinator built successfully"
    cd ..
else
    fail "Go coordinator build failed"
    cd ..
fi

# 3. Test: CLI help command
echo ""
info "Test 3: Testing CLI help..."
if ./target/release/aevum --help | grep -q "Time-Travel Debugging Platform"; then
    pass "CLI help works"
else
    fail "CLI help failed"
fi

# 4. Test: Start coordinator
echo ""
info "Test 4: Starting coordinator..."
cd coordinator
./aevum-coordinator &
COORDINATOR_PID=$!
sleep 3

if kill -0 $COORDINATOR_PID 2>/dev/null; then
    pass "Coordinator started successfully"
else
    fail "Coordinator failed to start"
fi
cd ..

# 5. Test: Coordinator health check
echo ""
info "Test 5: Checking coordinator health..."
if curl -s http://localhost:8080/health | grep -q "healthy"; then
    pass "Coordinator health check passed"
else
    fail "Coordinator health check failed"
fi

# 6. Test: Record a simple trace
echo ""
info "Test 6: Recording a trace..."
if ./target/release/aevum record echo "test" --output test.aevum 2>&1; then
    pass "Trace recording completed"
else
    fail "Trace recording failed"
fi

# 7. Test: Inspect trace
echo ""
info "Test 7: Inspecting trace..."
if ./target/release/aevum inspect test.aevum 2>&1 | grep -q "Trace Summary"; then
    pass "Trace inspection works"
else
    fail "Trace inspection failed"
fi

# 8. Test: Python agent import
echo ""
info "Test 8: Testing Python agent import..."
if python3 -c "import sys; sys.path.insert(0, 'agents/python-agent'); import aevum_agent" 2>&1; then
    pass "Python agent imports successfully"
else
    fail "Python agent import failed"
fi

# 9. Test: Go agent compilation
echo ""
info "Test 9: Testing Go agent compilation..."
cd agents/go-agent
if go build -o test-agent agent.go 2>&1; then
    pass "Go agent compiles successfully"
    rm -f test-agent
    cd ../..
else
    fail "Go agent compilation failed"
    cd ../..
fi

# 10. Test: Node.js agent syntax
echo ""
info "Test 10: Testing Node.js agent syntax..."
if node -c agents/node-agent/agent.js 2>&1; then
    pass "Node.js agent syntax valid"
else
    fail "Node.js agent syntax check failed"
fi

# Cleanup
echo ""
info "Cleaning up..."
kill $COORDINATOR_PID 2>/dev/null || true
rm -f test.aevum

# Summary
echo ""
echo "=================================="
echo "Test Results:"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo "=================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
