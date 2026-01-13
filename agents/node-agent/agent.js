const inspector = require('inspector');
const { EventEmitter } = require('events');
const net = require('net');

/**
 * Aevum Node.js Agent - Inspector Protocol integration
 * 
 * Uses the V8 Inspector Protocol to capture execution events
 */

class EventMetadata {
    constructor(traceId, sequenceNumber) {
        this.trace_id = traceId;
        this.process_id = process.pid;
        this.thread_id = 0; // Node.js is single-threaded (main thread)
        this.timestamp_ns = process.hrtime.bigint().toString();
        this.sequence_number = sequenceNumber;
    }
}

class AevumNodeAgent extends EventEmitter {
    constructor(traceId, serverHost = 'localhost', serverPort = 9876) {
        super();
        this.traceId = traceId;
        this.serverHost = serverHost;
        this.serverPort = serverPort;
        this.sequenceNumber = 0;
        this.enabled = false;
        this.socket = null;
        this.session = null;
    }

    /**
     * Connect to the trace server
     */
    connect() {
        return new Promise((resolve, reject) => {
            this.socket = net.createConnection({
                host: this.serverHost,
                port: this.serverPort
            }, () => {
                console.log(`[Aevum] Connected to trace server at ${this.serverHost}:${this.serverPort}`);
                resolve();
            });

            this.socket.on('error', (err) => {
                console.error(`[Aevum] Socket error: ${err.message}`);
                reject(err);
            });
        });
    }

    /**
     * Disconnect from the trace server
     */
    disconnect() {
        if (this.socket) {
            this.socket.end();
            this.socket = null;
        }
    }

    /**
     * Send an event to the trace server
     */
    sendEvent(event) {
        if (!this.socket) {
            return;
        }

        try {
            const eventJson = JSON.stringify(event);
            const eventBuffer = Buffer.from(eventJson, 'utf-8');

            // Send length prefix (4 bytes, big-endian) + data
            const lengthBuffer = Buffer.allocUnsafe(4);
            lengthBuffer.writeUInt32BE(eventBuffer.length, 0);

            this.socket.write(Buffer.concat([lengthBuffer, eventBuffer]));
        } catch (err) {
            console.error(`[Aevum] Failed to send event: ${err.message}`);
        }
    }

    /**
     * Create event metadata
     */
    createMetadata() {
        this.sequenceNumber++;
        return new EventMetadata(this.traceId, this.sequenceNumber);
    }

    /**
     * Handle debugger events
     */
    handleDebuggerEvent(method, params) {
        if (!this.enabled) {
            return;
        }

        try {
            if (method === 'Debugger.paused') {
                this.handleFunctionCall(params);
            } else if (method === 'Runtime.executionContextCreated') {
                this.handleExecutionContext(params);
            } else if (method === 'Runtime.consoleAPICalled') {
                this.handleConsoleAPI(params);
            }
        } catch (err) {
            console.error(`[Aevum] Error handling debugger event: ${err.message}`);
        }
    }

    /**
     * Handle function call event
     */
    handleFunctionCall(params) {
        const callFrame = params.callFrames && params.callFrames[0];
        if (!callFrame) return;

        const event = {
            event_type: 'FunctionCall',
            metadata: this.createMetadata(),
            function_name: callFrame.functionName || '<anonymous>',
            module: callFrame.url || '<unknown>',
            line: callFrame.location.lineNumber,
            column: callFrame.location.columnNumber,
            stack_depth: params.callFrames.length
        };

        this.sendEvent(event);
    }

    /**
     * Handle execution context creation
     */
    handleExecutionContext(params) {
        const event = {
            event_type: 'ExecutionContextCreated',
            metadata: this.createMetadata(),
            context_id: params.context.id,
            context_name: params.context.name,
            origin: params.context.origin
        };

        this.sendEvent(event);
    }

    /**
     * Handle console API calls
     */
    handleConsoleAPI(params) {
        const event = {
            event_type: 'ConsoleAPI',
            metadata: this.createMetadata(),
            type: params.type,
            args: params.args.map(arg => arg.value || arg.description || '<object>')
        };

        this.sendEvent(event);
    }

    /**
     * Start the agent
     */
    async start() {
        await this.connect();

        // Open inspector session
        this.session = new inspector.Session();
        this.session.connect();

        // Enable debugging domains
        this.session.post('Debugger.enable');
        this.session.post('Runtime.enable');
        this.session.post('Profiler.enable');

        // Set up event listeners
        this.session.on('Debugger.paused', (params) => {
            this.handleDebuggerEvent('Debugger.paused', params);
            // Resume execution
            this.session.post('Debugger.resume');
        });

        this.session.on('Runtime.executionContextCreated', (params) => {
            this.handleDebuggerEvent('Runtime.executionContextCreated', params);
        });

        this.session.on('Runtime.consoleAPICalled', (params) => {
            this.handleDebuggerEvent('Runtime.consoleAPICalled', params);
        });

        this.enabled = true;
        console.log(`[Aevum] Node.js agent started (trace_id: ${this.traceId})`);
    }

    /**
     * Stop the agent
     */
    stop() {
        this.enabled = false;

        if (this.session) {
            this.session.post('Debugger.disable');
            this.session.post('Runtime.disable');
            this.session.post('Profiler.disable');
            this.session.disconnect();
            this.session = null;
        }

        this.disconnect();
        console.log('[Aevum] Node.js agent stopped');
    }
}

// Global agent instance
let globalAgent = null;

/**
 * Attach the Aevum agent to the current process
 */
async function attach(traceId, serverHost = 'localhost', serverPort = 9876) {
    if (globalAgent) {
        console.log('[Aevum] Agent already attached');
        return globalAgent;
    }

    globalAgent = new AevumNodeAgent(traceId, serverHost, serverPort);
    await globalAgent.start();
    return globalAgent;
}

/**
 * Detach the Aevum agent
 */
function detach() {
    if (globalAgent) {
        globalAgent.stop();
        globalAgent = null;
    }
}

module.exports = {
    AevumNodeAgent,
    attach,
    detach
};
