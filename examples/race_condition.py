"""
Example: Race Condition Detection

This example demonstrates a program with a race condition
that Aevum can detect through causality analysis.
"""

import threading
import time
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'agents', 'python-agent'))

# Shared counter (potential race condition!)
counter = 0
lock = threading.Lock()

def unsafe_increment(n):
    """Increment counter without proper synchronization - RACE CONDITION!"""
    global counter
    for _ in range(n):
        # Read-modify-write without lock = race condition
        temp = counter
        time.sleep(0.0001)  # Simulate some work
        counter = temp + 1

def safe_increment(n):
    """Increment counter with proper synchronization"""
    global counter
    for _ in range(n):
        with lock:
            temp = counter
            time.sleep(0.0001)
            counter = temp + 1

def run_race_demo():
    global counter
    
    print("🏎️ Race Condition Detection Demo")
    print("=" * 50)
    
    # In production, attach Aevum agent here
    # import aevum_agent
    # aevum_agent.attach("race-demo", "localhost", 9876)
    
    # ============================================
    # UNSAFE: Race condition exists
    # ============================================
    print("\n⚠️  Running UNSAFE increment (race condition)...")
    counter = 0
    
    threads = []
    for i in range(4):
        t = threading.Thread(target=unsafe_increment, args=(100,))
        threads.append(t)
        t.start()
    
    for t in threads:
        t.join()
    
    print(f"   Expected: 400")
    print(f"   Actual:   {counter}")
    print(f"   Lost updates: {400 - counter}")
    
    if counter != 400:
        print("   ❌ RACE CONDITION DETECTED!")
    
    # ============================================
    # SAFE: Proper synchronization
    # ============================================
    print("\n✅ Running SAFE increment (with lock)...")
    counter = 0
    
    threads = []
    for i in range(4):
        t = threading.Thread(target=safe_increment, args=(100,))
        threads.append(t)
        t.start()
    
    for t in threads:
        t.join()
    
    print(f"   Expected: 400")
    print(f"   Actual:   {counter}")
    
    if counter == 400:
        print("   ✓ No race condition - counter is correct!")
    
    # ============================================
    # Analysis
    # ============================================
    print("\n📊 Aevum Analysis:")
    print("   With Aevum tracing, you would see:")
    print("   - Memory access events for 'counter' variable")
    print("   - Concurrent accesses from different threads")
    print("   - Happens-before relationships (or lack thereof)")
    print("   - Visual heatmap of race-prone code regions")
    print("\n   The causal graph would show:")
    print("   - Thread 1 read → Thread 2 read (concurrent!)")
    print("   - No synchronization edge between threads")
    print("   - Report: Potential race on address of 'counter'")


if __name__ == "__main__":
    run_race_demo()
