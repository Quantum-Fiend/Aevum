use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use trace_engine::{TraceEngine, Event, EventMetadata, ProcessId, ThreadId, TraceId, VectorClock};
use tempfile::tempdir;

fn benchmark_event_recording(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_recording");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let dir = tempdir().unwrap();
                let mut engine = TraceEngine::new(dir.path(), true, 1000).unwrap();
                
                for i in 0..size {
                    let metadata = EventMetadata {
                        trace_id: engine.trace_id(),
                        process_id: ProcessId(1),
                        thread_id: ThreadId(1),
                        timestamp_ns: 1000 + i,
                        vector_clock: VectorClock::new(),
                        sequence_number: i + 1,
                    };

                    let event = Event::FunctionCall {
                        metadata,
                        function_name: format!("func_{}", i),
                        module: "benchmark".to_string(),
                        args: vec![],
                        stack_depth: 1,
                    };

                    engine.record_event(event).unwrap();
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_trace_replay(c: &mut Criterion) {
    use replay_engine::ReplayEngine;
    
    let mut group = c.benchmark_group("trace_replay");
    
    // Setup: Create a trace file
    let dir = tempdir().unwrap();
    let mut engine = TraceEngine::new(dir.path(), true, 1000).unwrap();
    
    for i in 0..1000 {
        let metadata = EventMetadata {
            trace_id: engine.trace_id(),
            process_id: ProcessId(1),
            thread_id: ThreadId(1),
            timestamp_ns: 1000 + i,
            vector_clock: VectorClock::new(),
            sequence_number: i + 1,
        };

        let event = Event::FunctionCall {
            metadata,
            function_name: format!("func_{}", i),
            module: "benchmark".to_string(),
            args: vec![],
            stack_depth: 1,
        };

        engine.record_event(event).unwrap();
    }
    
    let trace_path = dir.path().join(format!("trace_{}.log", engine.trace_id().0));
    
    group.bench_function("replay_1000_events", |b| {
        b.iter(|| {
            let mut replay = ReplayEngine::load(&trace_path).unwrap();
            while replay.next_event().is_some() {
                black_box(());
            }
        });
    });
    
    group.finish();
}

fn benchmark_vector_clock_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_clock");
    
    group.bench_function("happens_before", |b| {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();
        
        vc1.increment(ProcessId(1));
        vc2.increment(ProcessId(1));
        vc2.increment(ProcessId(1));
        
        b.iter(|| {
            black_box(vc1.happens_before(&vc2));
        });
    });
    
    group.bench_function("merge", |b| {
        let mut vc1 = VectorClock::new();
        let vc2 = VectorClock::new();
        
        vc1.increment(ProcessId(1));
        
        b.iter(|| {
            let mut vc = vc1.clone();
            vc.merge(&vc2);
            black_box(vc);
        });
    });
    
    group.finish();
}

fn benchmark_compression(c: &mut Criterion) {
    use trace_engine::TraceLog;
    
    let mut group = c.benchmark_group("compression");
    
    let dir = tempdir().unwrap();
    let log_path = dir.path().join("bench.trace");
    
    group.bench_function("compressed_write", |b| {
        b.iter(|| {
            let log = TraceLog::create(&log_path, true).unwrap();
            
            let metadata = EventMetadata {
                trace_id: TraceId::new(),
                process_id: ProcessId(1),
                thread_id: ThreadId(1),
                timestamp_ns: 1000,
                vector_clock: VectorClock::new(),
                sequence_number: 1,
            };

            let event = Event::FunctionCall {
                metadata,
                function_name: "benchmark_function".to_string(),
                module: "benchmark".to_string(),
                args: vec![1, 2, 3, 4, 5],
                stack_depth: 1,
            };

            log.append(&event).unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_event_recording,
    benchmark_trace_replay,
    benchmark_vector_clock_operations,
    benchmark_compression
);
criterion_main!(benches);
