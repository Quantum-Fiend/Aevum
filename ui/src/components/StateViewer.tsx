import './StateViewer.css'

interface Event {
    event_type: string
    metadata: {
        sequence_number: number
        timestamp_ns: number
        thread_id: number
        process_id: number
    }
}

interface StateViewerProps {
    event: Event | undefined
    events: Event[]
    position: number
}

const StateViewer = ({ event, events, position }: StateViewerProps) => {
    if (!event) {
        return (
            <div className="state-viewer">
                <div className="empty-message">No event selected</div>
            </div>
        )
    }

    const previousEvent = position > 0 ? events[position - 1] : null

    return (
        <div className="state-viewer">
            <div className="state-grid">
                <div className="state-card glass">
                    <h3>Current Event</h3>
                    <div className="state-details">
                        <div className="detail-row">
                            <span className="label">Sequence:</span>
                            <span className="value">#{event.metadata.sequence_number}</span>
                        </div>
                        <div className="detail-row">
                            <span className="label">Type:</span>
                            <span className="value event-type">{event.event_type}</span>
                        </div>
                        <div className="detail-row">
                            <span className="label">Thread:</span>
                            <span className="value">{event.metadata.thread_id}</span>
                        </div>
                        <div className="detail-row">
                            <span className="label">Process:</span>
                            <span className="value">{event.metadata.process_id}</span>
                        </div>
                        <div className="detail-row">
                            <span className="label">Timestamp:</span>
                            <span className="value">{event.metadata.timestamp_ns.toLocaleString()} ns</span>
                        </div>
                    </div>
                </div>

                {previousEvent && (
                    <div className="state-card glass">
                        <h3>Previous Event</h3>
                        <div className="state-details">
                            <div className="detail-row">
                                <span className="label">Sequence:</span>
                                <span className="value">#{previousEvent.metadata.sequence_number}</span>
                            </div>
                            <div className="detail-row">
                                <span className="label">Type:</span>
                                <span className="value event-type">{previousEvent.event_type}</span>
                            </div>
                            <div className="detail-row">
                                <span className="label">Time Delta:</span>
                                <span className="value">
                                    {(event.metadata.timestamp_ns - previousEvent.metadata.timestamp_ns).toLocaleString()} ns
                                </span>
                            </div>
                        </div>
                    </div>
                )}

                <div className="state-card glass full-width">
                    <h3>Stack Trace</h3>
                    <div className="stack-trace">
                        <div className="stack-frame">
                            <span className="frame-number">0</span>
                            <span className="frame-function">{event.event_type}</span>
                            <span className="frame-location">Thread {event.metadata.thread_id}</span>
                        </div>
                        <div className="stack-info">
                            Stack trace reconstruction requires runtime integration
                        </div>
                    </div>
                </div>

                <div className="state-card glass full-width">
                    <h3>Event Context</h3>
                    <div className="context-info">
                        <p>Position in trace: {position + 1} / {events.length}</p>
                        <p>Progress: {((position / events.length) * 100).toFixed(1)}%</p>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default StateViewer
