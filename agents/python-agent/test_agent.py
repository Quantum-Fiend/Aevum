"""
Unit tests for Aevum Python Agent
"""

import unittest
import json
import sys
import os

# Add agent to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from aevum_agent import EventMetadata, FunctionCallEvent, AevumPythonAgent

class TestEventMetadata(unittest.TestCase):
    """Test EventMetadata dataclass"""
    
    def test_metadata_creation(self):
        metadata = EventMetadata(
            trace_id="test-trace",
            process_id=1234,
            thread_id=5678,
            timestamp_ns=1000000000,
            sequence_number=42
        )
        
        self.assertEqual(metadata.trace_id, "test-trace")
        self.assertEqual(metadata.process_id, 1234)
        self.assertEqual(metadata.thread_id, 5678)
        self.assertEqual(metadata.sequence_number, 42)

class TestFunctionCallEvent(unittest.TestCase):
    """Test FunctionCallEvent dataclass"""
    
    def test_event_creation(self):
        metadata = EventMetadata(
            trace_id="test-trace",
            process_id=1234,
            thread_id=5678,
            timestamp_ns=1000000000,
            sequence_number=1
        )
        
        event = FunctionCallEvent(
            metadata=metadata,
            function_name="test_function",
            module="test_module",
            args='{"x": 10}',
            stack_depth=3
        )
        
        self.assertEqual(event.event_type, "FunctionCall")
        self.assertEqual(event.function_name, "test_function")
        self.assertEqual(event.module, "test_module")
        self.assertEqual(event.stack_depth, 3)

class TestAevumPythonAgent(unittest.TestCase):
    """Test AevumPythonAgent class"""
    
    def test_agent_creation(self):
        agent = AevumPythonAgent(
            trace_id="test-agent",
            server_host="localhost",
            server_port=9876
        )
        
        self.assertEqual(agent.trace_id, "test-agent")
        self.assertEqual(agent.server_host, "localhost")
        self.assertEqual(agent.server_port, 9876)
        self.assertEqual(agent.sequence_number, 0)
        self.assertFalse(agent.enabled)

    def test_metadata_generation(self):
        agent = AevumPythonAgent(
            trace_id="test-agent",
            server_host="localhost",
            server_port=9876
        )
        
        metadata1 = agent.create_metadata()
        metadata2 = agent.create_metadata()
        
        self.assertEqual(metadata1.trace_id, "test-agent")
        self.assertEqual(metadata1.sequence_number, 1)
        self.assertEqual(metadata2.sequence_number, 2)

    def test_trace_function_hook(self):
        """Test that trace function returns itself for chaining"""
        agent = AevumPythonAgent(
            trace_id="test-agent",
            server_host="localhost",
            server_port=9876
        )
        agent.enabled = True
        
        # Create a mock frame
        class MockFrame:
            f_code = type('Code', (), {'co_name': 'test_func'})()
            f_globals = {'__name__': 'test_module'}
            f_locals = {}
        
        # trace_function should return itself
        result = agent.trace_function(MockFrame(), 'call', None)
        self.assertEqual(result, agent.trace_function)

class TestEventSerialization(unittest.TestCase):
    """Test event JSON serialization"""
    
    def test_event_to_json(self):
        from dataclasses import asdict
        
        metadata = EventMetadata(
            trace_id="test-trace",
            process_id=1234,
            thread_id=5678,
            timestamp_ns=1000000000,
            sequence_number=1
        )
        
        event = FunctionCallEvent(
            metadata=metadata,
            function_name="test_function",
            module="test_module",
            args='{}',
            stack_depth=1
        )
        
        event_dict = asdict(event)
        json_str = json.dumps(event_dict)
        
        # Verify JSON is valid
        parsed = json.loads(json_str)
        self.assertEqual(parsed['event_type'], 'FunctionCall')
        self.assertEqual(parsed['function_name'], 'test_function')
        self.assertEqual(parsed['metadata']['trace_id'], 'test-trace')

if __name__ == '__main__':
    unittest.main()
