"""
Example: Fibonacci with Aevum Tracing

This demonstrates how to use the Aevum Python agent to trace
a simple recursive function.
"""

import sys
import os

# Add agent to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'agents', 'python-agent'))

import aevum_agent

def fibonacci(n):
    """Compute the nth Fibonacci number recursively"""
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

def main():
    print("🎬 Starting Fibonacci computation with Aevum tracing...")
    
    # Attach the Aevum agent
    agent = aevum_agent.attach(
        trace_id="fibonacci-example",
        server_host="localhost",
        server_port=9876
    )
    
    # Compute Fibonacci numbers
    for i in range(10):
        result = fibonacci(i)
        print(f"fibonacci({i}) = {result}")
    
    # Detach the agent
    aevum_agent.detach()
    
    print("\n✅ Tracing complete! View the trace with:")
    print("   aevum inspect <trace_file> --causality")

if __name__ == "__main__":
    main()
