import { useEffect, useRef } from 'react'
import cytoscape from 'cytoscape'
import dagre from 'cytoscape-dagre'
import './CausalityGraph.css'

cytoscape.use(dagre)

interface Event {
    event_type: string
    metadata: {
        sequence_number: number
        thread_id: number
    }
}

interface CausalityGraphProps {
    events: Event[]
    currentPosition: number
}

const CausalityGraph = ({ events, currentPosition }: CausalityGraphProps) => {
    const containerRef = useRef<HTMLDivElement>(null)
    const cyRef = useRef<cytoscape.Core | null>(null)

    useEffect(() => {
        if (!containerRef.current || events.length === 0) return

        // Build causal graph
        const nodes: any[] = []
        const edges: any[] = []

        // Group events by thread
        const threadEvents = new Map<number, Event[]>()
        events.forEach(event => {
            const threadId = event.metadata.thread_id
            if (!threadEvents.has(threadId)) {
                threadEvents.set(threadId, [])
            }
            threadEvents.get(threadId)!.push(event)
        })

        // Create nodes and edges
        events.slice(0, Math.min(100, events.length)).forEach((event, index) => {
            nodes.push({
                data: {
                    id: `event-${event.metadata.sequence_number}`,
                    label: `${event.event_type}\n#${event.metadata.sequence_number}`,
                    type: event.event_type,
                    current: index === currentPosition
                }
            })

            // Add edge to previous event in same thread
            const threadId = event.metadata.thread_id
            const threadEventList = threadEvents.get(threadId)!
            const eventIndex = threadEventList.indexOf(event)
            if (eventIndex > 0) {
                const prevEvent = threadEventList[eventIndex - 1]
                edges.push({
                    data: {
                        source: `event-${prevEvent.metadata.sequence_number}`,
                        target: `event-${event.metadata.sequence_number}`,
                        type: 'thread'
                    }
                })
            }
        })

        // Initialize Cytoscape
        const cy = cytoscape({
            container: containerRef.current,
            elements: { nodes, edges },
            style: [
                {
                    selector: 'node',
                    style: {
                        'background-color': (ele: any) => {
                            const type = ele.data('type')
                            const colors: Record<string, string> = {
                                FunctionCall: '#6366f1',
                                FunctionReturn: '#8b5cf6',
                                MemoryWrite: '#ec4899',
                                Syscall: '#f59e0b',
                                ThreadSwitch: '#10b981',
                                NetworkIO: '#3b82f6',
                            }
                            return colors[type] || '#6c757d'
                        },
                        'label': 'data(label)',
                        'color': '#f8f9fa',
                        'text-valign': 'center',
                        'text-halign': 'center',
                        'font-size': '10px',
                        'width': (ele: any) => ele.data('current') ? 60 : 40,
                        'height': (ele: any) => ele.data('current') ? 60 : 40,
                        'border-width': (ele: any) => ele.data('current') ? 3 : 0,
                        'border-color': '#fff',
                    }
                },
                {
                    selector: 'edge',
                    style: {
                        'width': 2,
                        'line-color': '#2d2d3f',
                        'target-arrow-color': '#2d2d3f',
                        'target-arrow-shape': 'triangle',
                        'curve-style': 'bezier'
                    }
                }
            ],
            layout: {
                name: 'dagre',
                rankDir: 'TB',
                nodeSep: 50,
                rankSep: 80
            } as any
        })

        cyRef.current = cy

        return () => {
            cy.destroy()
        }
    }, [events, currentPosition])

    return (
        <div className="causality-graph-container">
            <div className="graph-header glass">
                <h3>Causal Dependency Graph</h3>
                <p>Showing up to 100 events with happens-before relationships</p>
            </div>
            <div className="graph-canvas" ref={containerRef}></div>
        </div>
    )
}

export default CausalityGraph
