use anyhow::Result;
use bs58;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signer};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    StartsWith,
    EndsWith,
    Contains,
}

impl std::str::FromStr for PatternType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "starts_with" | "starts" | "start" => Ok(PatternType::StartsWith),
            "ends_with" | "ends" | "end" => Ok(PatternType::EndsWith),
            "contains" | "contain" => Ok(PatternType::Contains),
            _ => Err(format!("Invalid pattern type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VanityOptions {
    pub pattern: String,
    pub pattern_type: PatternType,
    pub case_sensitive: bool,
    #[allow(dead_code)]
    pub max_attempts: u64,
    pub max_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanityResult {
    pub public_key: String,
    pub private_key: String,
    pub attempts: u64,
    pub time_elapsed: Duration,
}

pub struct VanityGenerator {
    // No state needed - all operations are stateless
}

impl VanityGenerator {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a single vanity address
    #[allow(dead_code)]
    pub async fn generate_single(&self, options: &VanityOptions) -> Result<Option<VanityResult>> {
        let start_time = Instant::now();
        let mut attempts = 0u64;

        while attempts < options.max_attempts && start_time.elapsed() < options.max_time {
            attempts += 1;

            // Generate a new keypair
            let keypair = Keypair::new();
            let public_key = keypair.pubkey().to_string();

            // Check if it matches our criteria
            if self.matches_pattern(&public_key, &options.pattern, &options.pattern_type, options.case_sensitive) {
                return Ok(Some(VanityResult {
                    public_key,
                    private_key: bs58::encode(&keypair.to_bytes()).into_string(),
                    attempts,
                    time_elapsed: start_time.elapsed(),
                }));
            }

            // Yield control periodically to prevent blocking
            if attempts % 10000 == 0 {
                tokio::task::yield_now().await;
            }
        }

        Ok(None)
    }

    /// Generate multiple addresses in parallel
    pub async fn generate_multiple_parallel(
        &self,
        count: usize,
        options: VanityOptions,
        thread_count: usize,
    ) -> Result<(Vec<VanityResult>, u64)> {
        let results = Arc::new(std::sync::Mutex::new(Vec::new()));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let total_attempts = Arc::new(AtomicU64::new(0));

        // Create a progress bar
        let pb = indicatif::ProgressBar::new(count as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} addresses ({percent}%) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Spawn worker threads
        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let options = options.clone();
                let results = Arc::clone(&results);
                let stop_flag = Arc::clone(&stop_flag);
                let total_attempts = Arc::clone(&total_attempts);
                let pb = pb.clone();

                tokio::spawn(async move {
                    let mut local_attempts = 0u64;
                    let start_time = Instant::now();

                    loop {
                        // Check if we should stop
                        if stop_flag.load(Ordering::Relaxed) {
                            break;
                        }

                        // Check time limit
                        if start_time.elapsed() > options.max_time {
                            break;
                        }

                        // Check if we have enough results
                        {
                            let results_guard = results.lock().unwrap();
                            if results_guard.len() >= count {
                                break;
                            }
                        }

                        local_attempts += 1;

                        // Generate a new keypair
                        let keypair = Keypair::new();
                        let public_key = keypair.pubkey().to_string();

                        // Check if it matches our criteria
                        if Self::matches_pattern_static(&public_key, &options.pattern, &options.pattern_type, options.case_sensitive) {
                            let result = VanityResult {
                                public_key,
                                private_key: bs58::encode(&keypair.to_bytes()).into_string(),
                                attempts: local_attempts,
                                time_elapsed: start_time.elapsed(),
                            };

                            // Add to results
                            {
                                let mut results_guard = results.lock().unwrap();
                                if results_guard.len() < count {
                                    results_guard.push(result);
                                    pb.inc(1);
                                    
                                    if results_guard.len() >= count {
                                        stop_flag.store(true, Ordering::Relaxed);
                                    }
                                }
                            }
                        }

                        // Update total attempts periodically
                        if local_attempts % 1000 == 0 {
                            total_attempts.fetch_add(1000, Ordering::Relaxed);
                            local_attempts = 0;
                            
                            // Update progress message
                            let current_attempts = total_attempts.load(Ordering::Relaxed);
                            let elapsed = start_time.elapsed();
                            let speed = current_attempts as f64 / elapsed.as_secs_f64();
                            pb.set_message(format!("{:.0} attempts/sec", speed));
                        }
                    }

                    // Add remaining attempts
                    total_attempts.fetch_add(local_attempts, Ordering::Relaxed);
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.await?;
        }

        pb.finish_with_message("Generation complete!");

        let final_results = results.lock().unwrap().clone();
        let final_total_attempts = total_attempts.load(Ordering::Relaxed);
        Ok((final_results, final_total_attempts))
    }

    /// Check if a public key matches the specified pattern
    #[allow(dead_code)]
    fn matches_pattern(&self, public_key: &str, pattern: &str, pattern_type: &PatternType, case_sensitive: bool) -> bool {
        Self::matches_pattern_static(public_key, pattern, pattern_type, case_sensitive)
    }

    /// Static version for use in parallel contexts
    fn matches_pattern_static(public_key: &str, pattern: &str, pattern_type: &PatternType, case_sensitive: bool) -> bool {
        let (key, pat) = if case_sensitive {
            (public_key.to_string(), pattern.to_string())
        } else {
            (public_key.to_lowercase(), pattern.to_lowercase())
        };

        match pattern_type {
            PatternType::StartsWith => key.starts_with(&pat),
            PatternType::EndsWith => key.ends_with(&pat),
            PatternType::Contains => key.contains(&pat),
        }
    }

    /// Estimate the probability of finding a vanity address
    pub fn estimate_probability(&self, options: &VanityOptions) -> f64 {
        let alphabet_size: f64 = 58.0; // Base58 alphabet size
        let pattern_length = options.pattern.len() as f64;

        let base_probability = 1.0 / alphabet_size.powf(pattern_length);

        // Adjust for case sensitivity
        if !options.case_sensitive {
            // For case insensitive, we need to account for case variations
            // This is a rough estimate - actual probability is higher
            let case_variations = 2.0_f64.powf(pattern_length);
            base_probability * case_variations.min(alphabet_size)
        } else {
            base_probability
        }
    }

    /// Estimate expected number of attempts
    pub fn estimate_expected_attempts(&self, options: &VanityOptions) -> u64 {
        let probability = self.estimate_probability(options);
        if probability > 0.0 {
            (1.0 / probability).ceil() as u64
        } else {
            u64::MAX
        }
    }

    /// Estimate expected time in seconds
    pub fn estimate_expected_time(&self, options: &VanityOptions) -> Duration {
        let expected_attempts = self.estimate_expected_attempts(options);
        
        // Conservative estimate: 50,000 attempts per second per thread
        // This is much faster than the TypeScript version's ~2,000/sec
        let attempts_per_second = 50_000.0;
        let seconds = expected_attempts as f64 / attempts_per_second;
        
        Duration::from_secs_f64(seconds)
    }

    /// Format duration in a human-readable format
    pub fn format_duration(&self, duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        
        if total_seconds < 1 {
            "< 1 second".to_string()
        } else if total_seconds < 60 {
            format!("{} seconds", total_seconds)
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            if seconds == 0 {
                format!("{} minutes", minutes)
            } else {
                format!("{}m {}s", minutes, seconds)
            }
        } else if total_seconds < 86400 {
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            if minutes == 0 {
                format!("{} hours", hours)
            } else {
                format!("{}h {}m", hours, minutes)
            }
        } else {
            let days = total_seconds / 86400;
            let hours = (total_seconds % 86400) / 3600;
            if hours == 0 {
                format!("{} days", days)
            } else {
                format!("{}d {}h", days, hours)
            }
        }
    }
}

/// Validate that a pattern only contains valid Base58 characters
#[allow(dead_code)]
pub fn is_valid_base58_pattern(pattern: &str) -> bool {
    // Base58 excludes: 0, O, I, l
    let invalid_chars = ['0', 'O', 'I', 'l'];
    !pattern.chars().any(|c| invalid_chars.contains(&c))
}

/// Validate Base58 pattern and return specific invalid characters
pub fn validate_base58_pattern(pattern: &str) -> Result<(), Vec<char>> {
    // Base58 excludes: 0, O, I, l
    let invalid_chars = ['0', 'O', 'I', 'l'];
    let found_invalid: Vec<char> = pattern.chars()
        .filter(|c| invalid_chars.contains(c))
        .collect();
    
    if found_invalid.is_empty() {
        Ok(())
    } else {
        Err(found_invalid)
    }
}

/// Get all valid Base58 characters as a string
#[allow(dead_code)]
pub fn get_valid_base58_chars() -> &'static str {
    "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        let public_key = "ABC123def456GHI789jkl";
        
        // Test starts_with
        assert!(VanityGenerator::matches_pattern_static(
            public_key, "ABC", &PatternType::StartsWith, true
        ));
        assert!(!VanityGenerator::matches_pattern_static(
            public_key, "XYZ", &PatternType::StartsWith, true
        ));

        // Test ends_with
        assert!(VanityGenerator::matches_pattern_static(
            public_key, "jkl", &PatternType::EndsWith, true
        ));
        assert!(!VanityGenerator::matches_pattern_static(
            public_key, "XYZ", &PatternType::EndsWith, true
        ));

        // Test contains
        assert!(VanityGenerator::matches_pattern_static(
            public_key, "def", &PatternType::Contains, true
        ));
        assert!(!VanityGenerator::matches_pattern_static(
            public_key, "XYZ", &PatternType::Contains, true
        ));

        // Test case insensitive
        assert!(VanityGenerator::matches_pattern_static(
            public_key, "abc", &PatternType::StartsWith, false
        ));
        assert!(VanityGenerator::matches_pattern_static(
            public_key, "DEF", &PatternType::Contains, false
        ));
    }

    #[test]
    fn test_base58_validation() {
        assert!(is_valid_base58_pattern("ABC123"));
        assert!(is_valid_base58_pattern("xyz789"));
        assert!(!is_valid_base58_pattern("ABC0")); // Contains 0
        assert!(!is_valid_base58_pattern("ABCO")); // Contains O
        assert!(!is_valid_base58_pattern("ABCI")); // Contains I
        assert!(!is_valid_base58_pattern("ABCl")); // Contains l
    }

    #[test]
    fn test_validate_base58_pattern() {
        // Valid patterns
        assert!(validate_base58_pattern("ABC123").is_ok());
        assert!(validate_base58_pattern("xyz789").is_ok());
        assert!(validate_base58_pattern("RUST").is_ok());
        assert!(validate_base58_pattern("BYTE").is_ok());

        // Invalid patterns
        assert_eq!(validate_base58_pattern("ABC0").unwrap_err(), vec!['0']);
        assert_eq!(validate_base58_pattern("ABCO").unwrap_err(), vec!['O']);
        assert_eq!(validate_base58_pattern("ABCI").unwrap_err(), vec!['I']);
        assert_eq!(validate_base58_pattern("ABCl").unwrap_err(), vec!['l']);
        
        // Multiple invalid characters
        let result = validate_base58_pattern("AB0Ol");
        assert!(result.is_err());
        let invalid_chars = result.unwrap_err();
        assert!(invalid_chars.contains(&'0'));
        assert!(invalid_chars.contains(&'O'));
        assert!(invalid_chars.contains(&'l'));
    }

    #[test]
    fn test_get_valid_base58_chars() {
        let valid_chars = get_valid_base58_chars();
        assert!(valid_chars.contains('1'));
        assert!(valid_chars.contains('A'));
        assert!(valid_chars.contains('a'));
        assert!(!valid_chars.contains('0'));
        assert!(!valid_chars.contains('O'));
        assert!(!valid_chars.contains('I'));
        assert!(!valid_chars.contains('l'));
    }

    #[test]
    fn test_probability_estimation() {
        let generator = VanityGenerator::new();
        let options = VanityOptions {
            pattern: "A".to_string(),
            pattern_type: PatternType::StartsWith,
            case_sensitive: true,
            max_attempts: 1000000,
            max_time: Duration::from_secs(60),
        };

        let probability = generator.estimate_probability(&options);
        assert!(probability > 0.0);
        assert!(probability < 1.0);

        let expected_attempts = generator.estimate_expected_attempts(&options);
        assert!(expected_attempts > 0);
        assert!(expected_attempts < 1000); // Should be around 58 for single character
    }
}
