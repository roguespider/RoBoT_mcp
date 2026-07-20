// src/cli/commands/experience.rs
//! Experience statistics command

use anyhow::Result;
use crate::cli::output;

pub fn run() -> Result<()> {
    let db = crate::database::sqlite::SqliteDatabase::initialize()?;
    let conn = db.connection()?;
    
    let memories = crate::database::queries::search_memory(&conn, "Experience:", 1000)?;
    
    let mut success = 0;
    let mut failure = 0;
    let mut total_confidence: f32 = 0.0;
    
    for m in &memories {
        total_confidence += m.confidence;
        if m.content.contains("Success") || m.content.contains("success") {
            success += 1;
        } else {
            failure += 1;
        }
    }
    
    let total = memories.len();
    let avg_confidence = if total == 0 { 0.0 } else { total_confidence / total as f32 };
    let success_rate = if total > 0 { (success as f32 / total as f32) * 100.0 } else { 0.0 };
    
    output::section_header("Experience Statistics");
    output::kv("Total experiences", total);
    
    if total == 0 {
        output::warn_msg("No experiences recorded yet");
    } else {
        output::kv("Success rate", format!("{:.1}%", success_rate));
        output::kv("Average confidence", format!("{:.2}", avg_confidence));
        println!();
        println!("{}", output::bold("Breakdown:"));
        
        // Use table for breakdown
        let widths = [15usize, 10];
        output::table_header(&["Type", "Count"], &widths);
        output::table_row(&["Success", &success.to_string()], &widths);
        output::table_row(&["Failure", &failure.to_string()], &widths);
    }
    
    Ok(())
}
