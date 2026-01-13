use anyhow::{Context, Result};
use colored::Colorize;
use replay_engine::ReplayEngine;
use std::io::{self, Write};

pub async fn execute(trace_file: &str, interactive: bool) -> Result<()> {
    println!("{}", "⏮️  Loading trace for replay...".cyan().bold());
    println!("   Trace file: {}", trace_file.green());
    
    let mut replay = ReplayEngine::load(trace_file)
        .context("Failed to load trace file")?;
    
    println!("\n{}", "✓ Trace loaded successfully!".green().bold());
    println!("   Total events: {}", replay.total_events().to_string().yellow());
    
    if interactive {
        println!("\n{}", "Interactive replay mode".cyan());
        println!("   Commands: [n]ext, [p]revious, [g]oto <step>, [q]uit");
        
        loop {
            print!("\n{}> ", "aevum".cyan().bold());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            match input {
                "n" | "next" => {
                    if let Some(event) = replay.next_event() {
                        println!("Event #{}: {} in {}",
                            event.metadata().sequence_number.to_string().yellow(),
                            event.event_type().cyan(),
                            format!("{:?}", event.metadata().thread_id).magenta()
                        );
                    } else {
                        println!("{}", "End of trace reached".yellow());
                    }
                }
                "p" | "prev" | "previous" => {
                    if let Some(event) = replay.previous_event() {
                        println!("Event #{}: {} in {}",
                            event.metadata().sequence_number.to_string().yellow(),
                            event.event_type().cyan(),
                            format!("{:?}", event.metadata().thread_id).magenta()
                        );
                    } else {
                        println!("{}", "Beginning of trace reached".yellow());
                    }
                }
                "q" | "quit" | "exit" => {
                    println!("{}", "Exiting replay".green());
                    break;
                }
                _ if input.starts_with("g ") || input.starts_with("goto ") => {
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(step) = parts[1].parse::<u64>() {
                            match replay.seek_to(step) {
                                Ok(event) => {
                                    println!("Jumped to event #{}: {}",
                                        step.to_string().yellow(),
                                        event.event_type().cyan()
                                    );
                                }
                                Err(e) => {
                                    println!("{}: {}", "Error".red(), e);
                                }
                            }
                        }
                    }
                }
                "" => continue,
                _ => {
                    println!("{}: Unknown command", "Error".red());
                }
            }
        }
    } else {
        // Non-interactive replay
        println!("\n{}", "Replaying trace...".cyan());
        
        let mut count = 0;
        while let Some(event) = replay.next_event() {
            count += 1;
            if count % 100 == 0 {
                println!("Replayed {} events...", count.to_string().yellow());
            }
        }
        
        println!("\n{}", "✓ Replay complete!".green().bold());
        println!("   Events replayed: {}", count.to_string().yellow());
    }
    
    Ok(())
}
