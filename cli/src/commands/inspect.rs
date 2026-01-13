use anyhow::{Context, Result};
use colored::Colorize;
use replay_engine::ReplayEngine;
use std::collections::HashMap;

pub async fn execute(trace_file: &str, causality: bool, event_type_filter: Option<&str>) -> Result<()> {
    println!("{}", "🔍 Inspecting trace...".cyan().bold());
    println!("   Trace file: {}", trace_file.green());
    
    let replay = ReplayEngine::load(trace_file)
        .context("Failed to load trace file")?;
    
    let total_events = replay.total_events();
    
    println!("\n{}", "=== Trace Summary ===".cyan().bold());
    println!("   Total events: {}", total_events.to_string().yellow());
    
    // Count events by type
    let mut event_counts: HashMap<String, usize> = HashMap::new();
    let mut thread_counts: HashMap<u64, usize> = HashMap::new();
    
    for i in 0..total_events {
        if let Some(event) = replay.events_in_range(i as u64 + 1, i as u64 + 1).first() {
            *event_counts.entry(event.event_type().to_string()).or_insert(0) += 1;
            *thread_counts.entry(event.metadata().thread_id.0).or_insert(0) += 1;
        }
    }
    
    println!("\n{}", "=== Event Types ===".cyan().bold());
    for (event_type, count) in event_counts.iter() {
        println!("   {}: {}", event_type.yellow(), count.to_string().green());
    }
    
    println!("\n{}", "=== Threads ===".cyan().bold());
    for (thread_id, count) in thread_counts.iter() {
        println!("   Thread {}: {} events", thread_id.to_string().magenta(), count.to_string().green());
    }
    
    if let Some(filter) = event_type_filter {
        println!("\n{}", format!("=== Filtered Events ({}) ===", filter).cyan().bold());
        let mut filtered_count = 0;
        
        for i in 0..total_events.min(20) {
            if let Some(event) = replay.events_in_range(i as u64 + 1, i as u64 + 1).first() {
                if event.event_type() == filter {
                    println!("   Event #{}: {} (Thread {:?})",
                        event.metadata().sequence_number.to_string().yellow(),
                        event.event_type().cyan(),
                        event.metadata().thread_id
                    );
                    filtered_count += 1;
                }
            }
        }
        
        if filtered_count == 0 {
            println!("   {}", "No matching events found".yellow());
        } else if filtered_count == 20 {
            println!("   {} (showing first 20)", "...".yellow());
        }
    }
    
    if causality {
        println!("\n{}", "=== Causality Analysis ===".cyan().bold());
        println!("   {}", "Causality analysis requires distributed coordinator".yellow());
        println!("   Use the coordinator API to analyze cross-node causality");
    }
    
    Ok(())
}
