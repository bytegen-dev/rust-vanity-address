use clap::Parser;
use console::style;
use std::time::{Duration, Instant};

mod vanity;
use vanity::{VanityGenerator, VanityOptions, VanityResult, PatternType};

#[derive(Parser)]
#[command(name = "solana-vanity")]
#[command(about = "High-performance Solana vanity address generator")]
#[command(version)]
struct Cli {
    /// Pattern to match (e.g., "ABC", "SOL", "XYZ")
    #[arg(short, long)]
    pattern: String,

    /// Type of pattern matching
    #[arg(long, value_enum, default_value = "starts_with")]
    pattern_type: PatternType,

    /// Maximum number of attempts
    #[arg(long, default_value = "10000000")]
    max_attempts: u64,

    /// Maximum time in seconds
    #[arg(long, default_value = "300")]
    max_time: u64,

    /// Number of threads to use
    #[arg(long, default_value = "0")]
    threads: usize,

    /// Case sensitive matching
    #[arg(short, long)]
    case_sensitive: bool,

    /// Generate multiple addresses
    #[arg(long, default_value = "1")]
    count: usize,

    /// Output format (json, csv, text)
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Output file (optional)
    #[arg(long)]
    output: Option<String>,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Validate pattern
    match vanity::validate_base58_pattern(&cli.pattern) {
        Ok(_) => {},
        Err(invalid_chars) => {
            eprintln!("{}", style("❌ Error: Pattern contains invalid Base58 characters").red().bold());
            let invalid_chars_str: String = invalid_chars.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ");
            eprintln!("{}{}", style("Invalid characters found: ").red(), style(invalid_chars_str).yellow().bold());
            eprintln!();
            eprintln!("{}", style("Base58 encoding excludes these characters:").yellow());
            eprintln!("  • {} (zero)", style("0").red());
            eprintln!("  • {} (capital O)", style("O").red());
            eprintln!("  • {} (capital I)", style("I").red());
            eprintln!("  • {} (lowercase L)", style("l").red());
            eprintln!();
            eprintln!("{}", style("Valid Base58 characters: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz").green());
            eprintln!();
            eprintln!("{}", style("Example valid patterns:").cyan());
            eprintln!("  • {}", style("ABC").green());
            eprintln!("  • {}", style("RUST").green());
            eprintln!("  • {}", style("BYTE").green());
            eprintln!("  • {} {}", style("SOL").red().strikethrough(), style("(contains 'O')").red());
            std::process::exit(1);
        }
    }

    // Set up thread count
    let thread_count = if cli.threads == 0 {
        num_cpus::get()
    } else {
        cli.threads
    };

    println!("{}", style("🦀 Solana Vanity Address Generator").bold().cyan());
    println!("{}", style("Built with Rust for maximum performance").dim());
    println!();

    // Display configuration
    println!("{}", style("Configuration:").bold().yellow());
    println!("  Pattern: {}", style(&cli.pattern).green());
    println!("  Type: {}", style(format!("{:?}", cli.pattern_type)).green());
    println!("  Case sensitive: {}", style(cli.case_sensitive).green());
    println!("  Max attempts: {}", style(cli.max_attempts.to_string()).green());
    println!("  Max time: {}", style(format!("{}s", cli.max_time)).green());
    println!("  Threads: {}", style(thread_count.to_string()).green());
    println!("  Count: {}", style(cli.count.to_string()).green());
    println!();

    // Estimate difficulty
    let options = VanityOptions {
        pattern: cli.pattern.clone(),
        pattern_type: cli.pattern_type.clone(),
        case_sensitive: cli.case_sensitive,
        max_attempts: cli.max_attempts,
        max_time: Duration::from_secs(cli.max_time),
    };

    let generator = VanityGenerator::new();
    let probability = generator.estimate_probability(&options);
    let expected_attempts = generator.estimate_expected_attempts(&options);
    let estimated_time = generator.estimate_expected_time(&options);

    println!("{}", style("Difficulty Estimate:").bold().yellow());
    println!("  Probability: {}", style(format!("{:.6}%", probability * 100.0)).green());
    println!("  Expected attempts: {}", style(expected_attempts.to_string()).green());
    println!("  Estimated time: {}", style(generator.format_duration(estimated_time)).green());
    println!();

    // Start generation
    let start_time = Instant::now();
    let (results, total_attempts) = generator.generate_multiple_parallel(
        cli.count,
        options,
        thread_count,
    ).await?;

    let total_time = start_time.elapsed();

    // Display results
    if results.is_empty() {
        println!("{}", style("❌ No addresses found within the specified limits").red());
        return Ok(());
    }

    println!("{}", style("✅ Generation Complete!").bold().green());
    println!("  Total time: {}", style(format!("{:.2}s", total_time.as_secs_f64())).green());
    println!("  Total attempts: {}", style(total_attempts.to_string()).green());
    println!("  Average speed: {}", style(format!("{:.0} attempts/sec", 
        total_attempts as f64 / total_time.as_secs_f64())).green());
    println!();

    // Output results
    match cli.format {
        OutputFormat::Text => output_text(&results),
        OutputFormat::Json => output_json(&results)?,
        OutputFormat::Csv => output_csv(&results)?,
    }

    // Save to file if specified
    if let Some(output_file) = cli.output {
        save_results(&results, &output_file, &cli.format)?;
        println!("{}", style(format!("Results saved to: {}", output_file)).green());
    }

    Ok(())
}

fn output_text(results: &[VanityResult]) {
    for (i, result) in results.iter().enumerate() {
        println!("{}", style(format!("Address #{}", i + 1)).bold().cyan());
        println!("  Public Key:  {}", style(&result.public_key).green());
        println!("  Private Key: {}", style(&result.private_key).red());
        println!("  Time:        {}", style(format!("{:.2}s", result.time_elapsed.as_secs_f64())).yellow());
        println!();
    }
}

fn output_json(results: &[VanityResult]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{}", json);
    Ok(())
}

fn output_csv(results: &[VanityResult]) -> anyhow::Result<()> {
    println!("public_key,private_key,attempts,time_seconds");
    for result in results {
        println!("{},{},{},{}", 
            result.public_key, 
            result.private_key, 
            result.attempts, 
            result.time_elapsed.as_secs_f64()
        );
    }
    Ok(())
}

fn save_results(results: &[VanityResult], filename: &str, format: &OutputFormat) -> anyhow::Result<()> {
    let content = match format {
        OutputFormat::Text => {
            let mut text = String::new();
            for (i, result) in results.iter().enumerate() {
                text.push_str(&format!("Address #{}\n", i + 1));
                text.push_str(&format!("Public Key:  {}\n", result.public_key));
                text.push_str(&format!("Private Key: {}\n", result.private_key));
                text.push_str(&format!("Attempts:    {}\n", result.attempts));
                text.push_str(&format!("Time:        {:.2}s\n\n", result.time_elapsed.as_secs_f64()));
            }
            text
        },
        OutputFormat::Json => serde_json::to_string_pretty(results)?,
        OutputFormat::Csv => {
            let mut csv = String::from("public_key,private_key,attempts,time_seconds\n");
            for result in results {
                csv.push_str(&format!("{},{},{},{}\n", 
                    result.public_key, 
                    result.private_key, 
                    result.attempts, 
                    result.time_elapsed.as_secs_f64()
                ));
            }
            csv
        },
    };

    std::fs::write(filename, content)?;
    Ok(())
}