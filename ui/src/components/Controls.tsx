import './Controls.css'
import { SkipBack, ChevronLeft, ChevronRight, SkipForward } from 'lucide-react'

interface Event {
    event_type: string
    metadata: {
        sequence_number: number
    }
}

interface ControlsProps {
    currentPosition: number
    totalEvents: number
    onPositionChange: (position: number) => void
    currentEvent: Event
}

const Controls = ({ currentPosition, totalEvents, onPositionChange, currentEvent }: ControlsProps) => {
    const handlePrevious = () => {
        if (currentPosition > 0) {
            onPositionChange(currentPosition - 1)
        }
    }

    const handleNext = () => {
        if (currentPosition < totalEvents - 1) {
            onPositionChange(currentPosition + 1)
        }
    }

    const handleFirst = () => {
        onPositionChange(0)
    }

    const handleLast = () => {
        onPositionChange(totalEvents - 1)
    }

    const handleSliderChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        onPositionChange(parseInt(e.target.value))
    }

    return (
        <div className="controls glass">
            <div className="controls-content">
                <div className="playback-buttons">
                    <button onClick={handleFirst} disabled={currentPosition === 0} title="First Event">
                        <SkipBack size={20} />
                    </button>
                    <button onClick={handlePrevious} disabled={currentPosition === 0} title="Previous Event">
                        <ChevronLeft size={20} />
                    </button>
                    <button onClick={handleNext} disabled={currentPosition === totalEvents - 1} title="Next Event">
                        <ChevronRight size={20} />
                    </button>
                    <button onClick={handleLast} disabled={currentPosition === totalEvents - 1} title="Last Event">
                        <SkipForward size={20} />
                    </button>
                </div>

                <div className="timeline-slider">
                    <input
                        type="range"
                        min="0"
                        max={totalEvents - 1}
                        value={currentPosition}
                        onChange={handleSliderChange}
                        className="slider"
                    />
                    <div className="slider-labels">
                        <span>Event #{currentEvent?.metadata.sequence_number || 0}</span>
                        <span>{currentPosition + 1} / {totalEvents}</span>
                    </div>
                </div>

                <div className="event-info">
                    <div className="info-badge">
                        <span className="badge-label">Type:</span>
                        <span className="badge-value">{currentEvent?.event_type || 'N/A'}</span>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default Controls
