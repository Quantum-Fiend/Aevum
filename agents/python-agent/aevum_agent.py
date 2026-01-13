"""
Aevum Python Agent - Bytecode instrumentation for execution tracing

This agent uses sys.settrace to capture function calls, returns, and variable mutations
without requiring source code modification.
"""

import sys
import json
import socket
import struct
import time
import threading
from typing import Any, Dict, Optional
from dataclasses import dataclass, asdict
import traceback as tb

@dataclass
class EventMetadata:
    trace_id: str
    process_id: int
    thread_id: int
    timestamp_ns: int
    sequence_number: int

@dataclass
class FunctionCallEvent:
    metadata: EventMetadata
    function_name: str
    module: str
    args: str  # JSON serialized
    stack_depth: int
    event_type: str = "FunctionCall"

@dataclass
class FunctionReturnEvent:
    metadata: EventMetadata
    function_name: str
    return_value: str  # JSON serialized
    stack_depth: int
    event_type: str = "FunctionReturn"

@dataclass
class ExceptionEvent:
    metadata: EventMetadata
    exception_type: str
    exception_message: str
    traceback: str
    event_type: str = "Exception"

class AevumPythonAgent:
    """Python execution tracing agent"""
    
    def __init__(self, trace_id: str, server_host: str = "localhost", server_port: int = 9876):
        self.trace_id = trace_id
        self.server_host = server_host
        self.server_port = server_port
        self.sequence_number = 0
        self.stack_depth = 0
        self.enabled = False
        self.socket: Optional[socket.socket] = None
        self._lock = threading.Lock()
        
    def connect(self):
        """Connect to the trace server"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.server_host, self.server_port))
            print(f"[Aevum] Connected to trace server at {self.server_host}:{self.server_port}")
        except Exception as e:
            print(f"[Aevum] Failed to connect to trace server: {e}")
            self.socket = None
    
    def disconnect(self):
        """Disconnect from the trace server"""
        if self.socket:
            try:
                self.socket.close()
            except:
                pass
            self.socket = None
    
    def send_event(self, event: Any):
        """Send an event to the trace server"""
        if not self.socket:
            return
        
        try:
            event_dict = asdict(event)
            event_json = json.dumps(event_dict)
            event_bytes = event_json.encode('utf-8')
            
            # Send length prefix + data
            length = struct.pack('!I', len(event_bytes))
            self.socket.sendall(length + event_bytes)
        except Exception as e:
            print(f"[Aevum] Failed to send event: {e}")
    
    def create_metadata(self) -> EventMetadata:
        """Create event metadata"""
        with self._lock:
            self.sequence_number += 1
            return EventMetadata(
                trace_id=self.trace_id,
                process_id=os.getpid(),
                thread_id=threading.get_ident(),
                timestamp_ns=time.time_ns(),
                sequence_number=self.sequence_number
            )
    
    def trace_function(self, frame, event, arg):
        """Trace function for sys.settrace"""
        if not self.enabled:
            return None
        
        try:
            if event == 'call':
                self.stack_depth += 1
                self._handle_call(frame)
            elif event == 'return':
                self._handle_return(frame, arg)
                self.stack_depth -= 1
            elif event == 'exception':
                self._handle_exception(frame, arg)
        except Exception as e:
            # Don't let tracing errors crash the program
            print(f"[Aevum] Tracing error: {e}")
        
        return self.trace_function
    
    def _handle_call(self, frame):
        """Handle function call event"""
        func_name = frame.f_code.co_name
        module = frame.f_globals.get('__name__', '<unknown>')
        
        # Serialize arguments (simplified)
        args_dict = {}
        try:
            local_vars = frame.f_locals.copy()
            # Only capture simple types to avoid serialization issues
            for key, value in local_vars.items():
                if isinstance(value, (int, float, str, bool, type(None))):
                    args_dict[key] = value
                else:
                    args_dict[key] = f"<{type(value).__name__}>"
        except:
            args_dict = {"error": "failed to capture args"}
        
        event = FunctionCallEvent(
            metadata=self.create_metadata(),
            function_name=func_name,
            module=module,
            args=json.dumps(args_dict),
            stack_depth=self.stack_depth
        )
        
        self.send_event(event)
    
    def _handle_return(self, frame, return_value):
        """Handle function return event"""
        func_name = frame.f_code.co_name
        
        # Serialize return value (simplified)
        try:
            if isinstance(return_value, (int, float, str, bool, type(None))):
                ret_val = json.dumps(return_value)
            else:
                ret_val = json.dumps(f"<{type(return_value).__name__}>")
        except:
            ret_val = json.dumps("<unserializable>")
        
        event = FunctionReturnEvent(
            metadata=self.create_metadata(),
            function_name=func_name,
            return_value=ret_val,
            stack_depth=self.stack_depth
        )
        
        self.send_event(event)
    
    def _handle_exception(self, frame, exc_info):
        """Handle exception event"""
        exc_type, exc_value, exc_traceback = exc_info
        
        event = ExceptionEvent(
            metadata=self.create_metadata(),
            exception_type=exc_type.__name__ if exc_type else "Unknown",
            exception_message=str(exc_value),
            traceback=tb.format_exc()
        )
        
        self.send_event(event)
    
    def start(self):
        """Start tracing"""
        self.connect()
        self.enabled = True
        sys.settrace(self.trace_function)
        threading.settrace(self.trace_function)
        print(f"[Aevum] Python agent started (trace_id: {self.trace_id})")
    
    def stop(self):
        """Stop tracing"""
        self.enabled = False
        sys.settrace(None)
        threading.settrace(None)
        self.disconnect()
        print("[Aevum] Python agent stopped")

# Global agent instance
_agent: Optional[AevumPythonAgent] = None

def attach(trace_id: str, server_host: str = "localhost", server_port: int = 9876):
    """Attach the Aevum agent to the current process"""
    global _agent
    import os
    
    if _agent:
        print("[Aevum] Agent already attached")
        return _agent
    
    _agent = AevumPythonAgent(trace_id, server_host, server_port)
    _agent.start()
    return _agent

def detach():
    """Detach the Aevum agent"""
    global _agent
    if _agent:
        _agent.stop()
        _agent = None

import os
