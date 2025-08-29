use anyhow::Result;
use crate::generator::NostrKeyPair;
use crate::matcher::Pattern;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::path::Path;

pub struct VanityResult {
    pub keypair: NostrKeyPair,
    pub matched_pattern: Pattern,
    pub attempts: u64,
    pub time_elapsed: std::time::Duration,
}

impl VanityResult {
    pub fn format_output(&self) -> String {
        format!(
            "✨ Found vanity address!\n\
            Pattern: {}\n\
            npub: {}\n\
            nsec: {}\n\
            Hex pubkey: {}\n\
            Attempts: {}\n\
            Time: {:.2}s\n\
            Speed: {:.0} keys/sec\n\
            ---",
            self.matched_pattern.value,
            self.keypair.npub,
            self.keypair.nsec,
            self.keypair.hex_pubkey,
            self.attempts,
            self.time_elapsed.as_secs_f64(),
            self.attempts as f64 / self.time_elapsed.as_secs_f64()
        )
    }
    
    pub fn format_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{:.2}",
            self.matched_pattern.value,
            self.keypair.npub,
            self.keypair.nsec,
            self.keypair.hex_pubkey,
            self.attempts,
            self.time_elapsed.as_secs_f64()
        )
    }
}

pub fn write_result_to_file(result: &VanityResult, path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    
    writeln!(file, "{}", result.format_output())?;
    Ok(())
}

pub fn write_csv_result(result: &VanityResult, path: &Path) -> Result<()> {
    let file_exists = path.exists();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    
    if !file_exists {
        writeln!(file, "pattern,npub,nsec,hex_pubkey,attempts,time_seconds")?;
    }
    
    writeln!(file, "{}", result.format_csv())?;
    Ok(())
}

pub fn read_patterns_from_file(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut patterns = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            patterns.push(trimmed.to_string());
        }
    }
    
    Ok(patterns)
}

pub fn parse_patterns_string(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn estimate_time(pattern_length: usize, keys_per_sec: f64) -> String {
    let possibilities = 32_f64.powi(pattern_length as i32);
    let expected_attempts = possibilities / 2.0;
    let seconds = expected_attempts / keys_per_sec;
    
    if seconds < 60.0 {
        format!("{:.1} seconds", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1} minutes", seconds / 60.0)
    } else if seconds < 86400.0 {
        format!("{:.1} hours", seconds / 3600.0)
    } else if seconds < 31536000.0 {
        format!("{:.1} days", seconds / 86400.0)
    } else {
        format!("{:.1} years", seconds / 31536000.0)
    }
}