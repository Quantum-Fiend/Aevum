import { useEffect, useRef } from 'react'
import * as d3 from 'd3'
import './Timeline.css'

interface Event {
    event_type: string
    metadata: {
        sequence_number: number
        timestamp_ns: number
        thread_id: number
    }
}

interface TimelineProps {
    events: Event[]
    currentPosition: number
    onPositionChange: (position: number) => void
}

const Timeline = ({ events, currentPosition, onPositionChange }: TimelineProps) => {
    const svgRef = useRef<SVGSVGElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)

    useEffect(() => {
        if (!svgRef.current || !containerRef.current || events.length === 0) return

        const container = containerRef.current
        const svg = d3.select(svgRef.current)
        svg.selectAll('*').remove()

        const width = container.clientWidth
        const height = container.clientHeight
        const margin = { top: 40, right: 40, bottom: 60, left: 60 }
        const innerWidth = width - margin.left - margin.right
        const innerHeight = height - margin.top - margin.bottom

        const g = svg
            .attr('width', width)
            .attr('height', height)
            .append('g')
            .attr('transform', `translate(${margin.left},${margin.top})`)

        // Scales
        const xScale = d3.scaleLinear()
            .domain([0, events.length - 1])
            .range([0, innerWidth])

        const threads = Array.from(new Set(events.map(e => e.metadata.thread_id)))
        const yScale = d3.scalePoint()
            .domain(threads.map(String))
            .range([0, innerHeight])
            .padding(0.5)

        // Grid
        g.append('g')
            .attr('class', 'grid')
            .selectAll('line')
            .data(threads)
            .join('line')
            .attr('x1', 0)
            .attr('x2', innerWidth)
            .attr('y1', d => yScale(String(d)) || 0)
            .attr('y2', d => yScale(String(d)) || 0)
            .attr('stroke', '#2d2d3f')
            .attr('stroke-width', 1)
            .attr('stroke-dasharray', '4,4')

        // Events
        const eventColors: Record<string, string> = {
            FunctionCall: '#6366f1',
            FunctionReturn: '#8b5cf6',
            MemoryWrite: '#ec4899',
            Syscall: '#f59e0b',
            ThreadSwitch: '#10b981',
            NetworkIO: '#3b82f6',
            default: '#6c757d'
        }

        g.selectAll('circle.event')
            .data(events)
            .join('circle')
            .attr('class', 'event')
            .attr('cx', (d, i) => xScale(i))
            .attr('cy', d => yScale(String(d.metadata.thread_id)) || 0)
            .attr('r', (d, i) => i === currentPosition ? 8 : 5)
            .attr('fill', d => eventColors[d.event_type] || eventColors.default)
            .attr('stroke', (d, i) => i === currentPosition ? '#fff' : 'none')
            .attr('stroke-width', 2)
            .style('cursor', 'pointer')
            .on('click', (event, d) => {
                const index = events.indexOf(d)
                onPositionChange(index)
            })
            .append('title')
            .text(d => `${d.event_type} #${d.metadata.sequence_number}`)

        // Axes
        const xAxis = d3.axisBottom(xScale)
            .ticks(10)
            .tickFormat(d => `#${d}`)

        g.append('g')
            .attr('class', 'x-axis')
            .attr('transform', `translate(0,${innerHeight})`)
            .call(xAxis)
            .selectAll('text')
            .attr('fill', '#adb5bd')

        const yAxis = d3.axisLeft(yScale)
            .tickFormat(d => `Thread ${d}`)

        g.append('g')
            .attr('class', 'y-axis')
            .call(yAxis)
            .selectAll('text')
            .attr('fill', '#adb5bd')

        // Current position indicator
        g.append('line')
            .attr('class', 'position-indicator')
            .attr('x1', xScale(currentPosition))
            .attr('x2', xScale(currentPosition))
            .attr('y1', 0)
            .attr('y2', innerHeight)
            .attr('stroke', '#ec4899')
            .attr('stroke-width', 2)
            .attr('stroke-dasharray', '5,5')

    }, [events, currentPosition, onPositionChange])

    return (
        <div className="timeline-container" ref={containerRef}>
            <svg ref={svgRef}></svg>
            <div className="timeline-legend">
                <div className="legend-item">
                    <span className="legend-dot" style={{ background: '#6366f1' }}></span>
                    FunctionCall
                </div>
                <div className="legend-item">
                    <span className="legend-dot" style={{ background: '#8b5cf6' }}></span>
                    FunctionReturn
                </div>
                <div className="legend-item">
                    <span className="legend-dot" style={{ background: '#ec4899' }}></span>
                    MemoryWrite
                </div>
                <div className="legend-item">
                    <span className="legend-dot" style={{ background: '#f59e0b' }}></span>
                    Syscall
                </div>
                <div className="legend-item">
                    <span className="legend-dot" style={{ background: '#10b981' }}></span>
                    ThreadSwitch
                </div>
                <div className="legend-item">
                    <span className="legend-dot" style={{ background: '#3b82f6' }}></span>
                    NetworkIO
                </div>
            </div>
        </div>
    )
}

export default Timeline
