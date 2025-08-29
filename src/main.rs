mod generator;
mod matcher;
mod utils;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use crossbeam_channel::unbounded;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::generator::{validate_bech32_chars, NostrKeyPair};
use crate::matcher::{MatchType, PatternMatcher};
use crate::utils::{
    estimate_time, parse_patterns_string, read_patterns_from_file, 
    write_csv_result, write_result_to_file, VanityResult
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Nostr vanity npub address generator", long_about = None)]
struct Args {
    #[arg(short, long, help = "Comma-separated list of patterns to search for")]
    patterns: Option<String>,
    
    #[arg(short = 'f', long, help = "Path to CSV file containing patterns")]
    file: Option<PathBuf>,
    
    #[arg(short, long, help = "Output file path (optional)")]
    output: Option<PathBuf>,
    
    #[arg(long, help = "Output in CSV format")]
    csv: bool,
    
    #[arg(short, long, default_value = "prefix", help = "Match type")]
    match_type: MatchTypeArg,
    
    #[arg(short = 'c', long, help = "Case sensitive matching")]
    case_sensitive: bool,
    
    #[arg(short = 't', long, help = "Number of CPU threads (default: all cores)")]
    threads: Option<usize>,
    
    #[arg(long, help = "Continue searching after finding first match")]
    continuous: bool,
    
    #[arg(short = 'q', long, help = "Quiet mode (less output)")]
    quiet: bool,
    
    #[arg(long, help = "Estimate time for patterns and exit")]
    estimate: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum MatchTypeArg {
    Prefix,
    Suffix,
    Contains,
}

impl From<MatchTypeArg> for MatchType {
    fn from(arg: MatchTypeArg) -> Self {
        match arg {
            MatchTypeArg::Prefix => MatchType::Prefix,
            MatchTypeArg::Suffix => MatchType::Suffix,
            MatchTypeArg::Contains => MatchType::Contains,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let patterns = collect_patterns(&args)?;
    
    if patterns.is_empty() {
        eprintln!("Error: No patterns provided. Use --patterns or --file");
        std::process::exit(1);
    }
    
    for pattern in &patterns {
        if !validate_bech32_chars(pattern) {
            eprintln!(
                "Error: Pattern '{}' contains invalid characters. \
                Valid: 023456789acdefghjklmnpqrstuvwxyz",
                pattern
            );
            std::process::exit(1);
        }
    }
    
    if args.estimate {
        estimate_patterns(&patterns);
        return Ok(());
    }
    
    let thread_count = args.threads.unwrap_or_else(num_cpus::get);
    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()?;
    
    if !args.quiet {
        println!("🔍 Nostr Vanity npub Generator");
        println!("Searching for {} pattern(s) with {} threads", patterns.len(), thread_count);
        println!("Patterns: {}", patterns.join(", "));
        println!("Match type: {:?}", args.match_type);
        println!();
    }
    
    let match_type = args.match_type.clone().into();
    let matcher = PatternMatcher::from_strings(
        patterns.clone(),
        match_type,
        args.case_sensitive,
    );
    
    run_search(args, matcher)?;
    
    Ok(())
}

fn collect_patterns(args: &Args) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    
    if let Some(pattern_str) = &args.patterns {
        patterns.extend(parse_patterns_string(pattern_str));
    }
    
    if let Some(file_path) = &args.file {
        patterns.extend(read_patterns_from_file(file_path)?);
    }
    
    Ok(patterns)
}

fn estimate_patterns(patterns: &[String]) {
    println!("⏱️  Time estimates (assuming ~100k keys/sec per core):");
    println!();
    
    for pattern in patterns {
        let time = estimate_time(pattern.len(), 100_000.0 * num_cpus::get() as f64);
        println!("  Pattern '{}' ({} chars): ~{}", pattern, pattern.len(), time);
    }
}

fn run_search(args: Args, matcher: PatternMatcher) -> Result<()> {
    let found = Arc::new(AtomicBool::new(false));
    let attempts = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();
    let (tx, rx) = unbounded();
    
    let progress = if !args.quiet {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg} [{elapsed_precise}] {per_sec}")?
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };
    
    let search_handle = std::thread::spawn({
        let found = found.clone();
        let attempts = attempts.clone();
        let continuous = args.continuous;
        let tx = tx.clone();
        
        move || {
            loop {
                if !continuous && found.load(Ordering::Relaxed) {
                    break;
                }
                
                let batch_size = 10000;
                let results: Vec<_> = (0..batch_size)
                    .into_par_iter()
                    .filter_map(|_| {
                        if !continuous && found.load(Ordering::Relaxed) {
                            return None;
                        }
                        
                        attempts.fetch_add(1, Ordering::Relaxed);
                        
                        match NostrKeyPair::generate() {
                            Ok(keypair) => {
                                if let Some(pattern) = matcher.find_match(&keypair) {
                                    Some((keypair, pattern))
                                } else {
                                    None
                                }
                            }
                            Err(_) => None,
                        }
                    })
                    .collect();
                
                for (keypair, pattern) in results {
                    found.store(true, Ordering::Relaxed);
                    let _ = tx.send((keypair, pattern));
                    if !continuous {
                        break;
                    }
                }
            }
        }
    });
    
    let output_handle = std::thread::spawn({
        let output = args.output.clone();
        let csv = args.csv;
        let quiet = args.quiet;
        let continuous = args.continuous;
        let attempts = attempts.clone();
        
        move || {
            for (keypair, pattern) in rx {
                let result = VanityResult {
                    keypair,
                    matched_pattern: pattern,
                    attempts: attempts.load(Ordering::Relaxed),
                    time_elapsed: start_time.elapsed(),
                };
                
                if !quiet {
                    println!("\n{}", result.format_output());
                }
                
                if let Some(ref path) = output {
                    let _ = if csv {
                        write_csv_result(&result, path)
                    } else {
                        write_result_to_file(&result, path)
                    };
                }
                
                if !continuous {
                    break;
                }
            }
        }
    });
    
    if let Some(pb) = &progress {
        while !found.load(Ordering::Relaxed) || args.continuous {
            let current_attempts = attempts.load(Ordering::Relaxed);
            let elapsed = start_time.elapsed().as_secs_f64();
            let rate = current_attempts as f64 / elapsed.max(0.1);
            
            pb.set_message(format!("Attempts: {}", current_attempts));
            pb.set_prefix(format!("{:.0} keys/sec", rate));
            
            std::thread::sleep(Duration::from_millis(100));
            
            if args.continuous && pb.elapsed() > Duration::from_secs(3600) {
                break;
            }
        }
        
        pb.finish_with_message("Complete!");
    } else {
        search_handle.join().unwrap();
    }
    
    drop(tx);
    output_handle.join().unwrap();
    
    Ok(())
}