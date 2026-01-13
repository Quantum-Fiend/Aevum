use anyhow::{Context, Result};
use colored::Colorize;
use replay_engine::ReplayEngine;

pub async fn execute(trace_file: &str, step: u64) -> Result<()> {
    println!("{}", "⏪ Rewinding trace...".cyan().bold());
    println!("   Trace file: {}", trace_file.green());
    println!("   Target step: {}", step.to_string().yellow());
    
    let mut replay = ReplayEngine::load(trace_file)
        .context("Failed to load trace file")?;
    
    let event = replay.seek_to(step)
        .context("Failed to seek to step")?;
    
    println!("\n{}", "✓ Rewound successfully!".green().bold());
    println!("   Current event: #{}", event.metadata().sequence_number.to_string().yellow());
    println!("   Event type: {}", event.event_type().cyan());
    println!("   Thread: {:?}", event.metadata().thread_id);
    println!("   Timestamp: {} ns", event.metadata().timestamp_ns.to_string().magenta());
    
    Ok(())
}
