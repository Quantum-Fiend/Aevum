import { useState } from 'react'
import Timeline from './components/Timeline'
import StateViewer from './components/StateViewer'
import CausalityGraph from './components/CausalityGraph'
import Controls from './components/Controls'
import {
    Clock,
    Search,
    Play,
    GitBranch,
    Database,
    Activity,
    Rewind,
    FastForward
} from 'lucide-react'
import './App.css'

interface Event {
    event_type: string
    metadata: {
        trace_id: string
        process_id: number
        thread_id: number
        timestamp_ns: number
        sequence_number: number
    }
}

function App() {
    const [traceId, setTraceId] = useState<string>('')
    const [events, setEvents] = useState<Event[]>([])
    const [currentPosition, setCurrentPosition] = useState<number>(0)
    const [loading, setLoading] = useState(false)
    const [view, setView] = useState<'timeline' | 'state' | 'causality'>('timeline')

    const loadTrace = async (id: string) => {
        setLoading(true)
        try {
            const response = await fetch(`/api/timeline/${id}`)
            const data = await response.json()
            setEvents(data.events || [])
            setTraceId(id)
            setCurrentPosition(0)
        } catch (error) {
            console.error('Failed to load trace:', error)
        } finally {
            setLoading(false)
        }
    }

    const currentEvent = events[currentPosition]

    return (
        <div className="app">
            <header className="header glass">
                <div className="header-content">
                    <div className="logo">
                        <h1>
                            <Clock className="logo-icon" size={28} />
                            Aevum
                        </h1>
                        <span className="subtitle">Time-Travel Debugger</span>
                    </div>

                    <div className="trace-selector">
                        <div className="input-wrapper">
                            <Search className="input-icon" size={16} />
                            <input
                                type="text"
                                placeholder="Enter Trace ID..."
                                value={traceId}
                                onChange={(e) => setTraceId(e.target.value)}
                                onKeyPress={(e) => e.key === 'Enter' && loadTrace(traceId)}
                                className="trace-input"
                            />
                        </div>
                        <button
                            onClick={() => loadTrace(traceId)}
                            className="load-btn"
                            disabled={!traceId || loading}
                        >
                            {loading ? <Activity className="spin" size={18} /> : 'Load Trace'}
                        </button>
                    </div>

                    <div className="view-tabs">
                        <button
                            className={`tab ${view === 'timeline' ? 'active' : ''}`}
                            onClick={() => setView('timeline')}
                        >
                            <Activity size={16} /> Timeline
                        </button>
                        <button
                            className={`tab ${view === 'state' ? 'active' : ''}`}
                            onClick={() => setView('state')}
                        >
                            <Database size={16} /> State
                        </button>
                        <button
                            className={`tab ${view === 'causality' ? 'active' : ''}`}
                            onClick={() => setView('causality')}
                        >
                            <GitBranch size={16} /> Causality
                        </button>
                    </div>
                </div>
            </header>

            <main className="main-content">
                {events.length === 0 ? (
                    <div className="empty-state">
                        <div className="empty-icon-wrapper">
                            <Play size={64} className="empty-icon" />
                        </div>
                        <h2>No Trace Loaded</h2>
                        <p>Enter a trace ID above to begin time-travel debugging</p>
                        <div className="empty-features">
                            <div className="feature">
                                <Rewind className="feature-icon" />
                                <span>Step backward through execution</span>
                            </div>
                            <div className="feature">
                                <Database className="feature-icon" />
                                <span>Inspect state at any point</span>
                            </div>
                            <div className="feature">
                                <GitBranch className="feature-icon" />
                                <span>Visualize distributed causality</span>
                            </div>
                        </div>
                    </div>
                ) : (
                    <>
                        {view === 'timeline' && (
                            <Timeline
                                events={events}
                                currentPosition={currentPosition}
                                onPositionChange={setCurrentPosition}
                            />
                        )}
                        {view === 'state' && (
                            <StateViewer
                                event={currentEvent}
                                events={events}
                                position={currentPosition}
                            />
                        )}
                        {view === 'causality' && (
                            <CausalityGraph
                                events={events}
                                currentPosition={currentPosition}
                            />
                        )}
                    </>
                )}
            </main>

            {events.length > 0 && (
                <Controls
                    currentPosition={currentPosition}
                    totalEvents={events.length}
                    onPositionChange={setCurrentPosition}
                    currentEvent={currentEvent}
                />
            )}
        </div>
    )
}

export default App
