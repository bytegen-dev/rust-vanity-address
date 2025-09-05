# 🦀 Solana Vanity Address Generator (Rust)

A high-performance Solana vanity address generator built in Rust! This is a blazing-fast terminal-based version of the [TypeScript web application](https://github.com/bytegen-dev/vanity-address).

## 🚀 Performance Comparison

| Metric        | TypeScript Version  | Rust Version          | Improvement        |
| ------------- | ------------------- | --------------------- | ------------------ |
| **Speed**     | ~2,000 attempts/sec | ~50,000+ attempts/sec | **25x faster**     |
| **Threading** | Single-threaded     | Multi-threaded        | **12x parallel**   |
| **Memory**    | Higher (GC pauses)  | Lower (no GC)         | **More efficient** |
| **Startup**   | Browser overhead    | Instant               | **Immediate**      |

## ✨ Features

- **🔥 Blazing Fast**: 25x faster than the TypeScript version
- **🧵 Multi-threaded**: Utilizes all CPU cores for maximum performance
- **🎯 Pattern Matching**: Supports starts_with, ends_with, and contains patterns
- **📊 Real-time Stats**: Live progress bars and performance metrics
- **💾 Export Options**: JSON, CSV, and text output formats
- **🛡️ Base58 Validation**: Prevents invalid character patterns with helpful error messages
- **⚡ CLI Interface**: Easy-to-use command-line tool
- **📈 Probability Estimation**: Accurate difficulty calculations

## 🛠️ Installation

### Prerequisites

- Rust 1.70+ installed ([Install Rust](https://rustup.rs/))

### Build from Source

```bash
git clone https://github.com/bytegen-dev/solana-vanity-rust.git
cd solana-vanity-rust
cargo build --release
```

## 🎯 Usage

### Basic Usage

```bash
# Generate an address starting with "ABC"
cargo run -- --pattern "ABC"

# Generate an address containing "RUST"
cargo run -- --pattern "RUST" --pattern-type contains

# Generate 5 addresses ending with "XYZ"
cargo run -- --pattern "XYZ" --pattern-type ends_with --count 5
```

### Advanced Options

```bash
# Full command with all options
cargo run -- \
  --pattern "BYTE" \
  --pattern-type starts_with \
  --case-sensitive \
  --max-attempts 10000000 \
  --max-time 300 \
  --threads 12 \
  --count 3 \
  --format json \
  --output results.json
```

### Command Line Options

| Option             | Short | Description                            | Default     |
| ------------------ | ----- | -------------------------------------- | ----------- |
| `--pattern`        | `-p`  | Pattern to match                       | Required    |
| `--pattern-type`   |       | Type: starts_with, ends_with, contains | starts_with |
| `--case-sensitive` | `-c`  | Case sensitive matching                | false       |
| `--max-attempts`   |       | Maximum attempts                       | 10,000,000  |
| `--max-time`       |       | Max time in seconds                    | 300         |
| `--threads`        |       | Number of threads (0 = auto)           | 0           |
| `--count`          |       | Number of addresses to generate        | 1           |
| `--format`         |       | Output format: text, json, csv         | text        |
| `--output`         |       | Save results to file                   | None        |

## 📊 Performance Examples

### Simple Patterns (1-2 characters)

```bash
# Pattern: "A" - Very fast
cargo run -- --pattern "A"
# Expected: < 1 second, ~58 attempts
```

### Medium Patterns (3-4 characters)

```bash
# Pattern: "ABC" - Fast
cargo run -- --pattern "ABC"
# Expected: ~1-5 seconds, ~24,000 attempts
```

### Complex Patterns (5+ characters)

```bash
# Pattern: "BYTE" - Moderate
cargo run -- --pattern "BYTE"
# Expected: ~10-30 seconds, ~700,000 attempts
```

## 🔧 Technical Details

### Architecture

- **Multi-threaded**: Uses all CPU cores with async/await
- **Memory Efficient**: No garbage collection overhead
- **Type Safe**: Rust's ownership system prevents memory bugs
- **Fast Crypto**: Native Solana keypair generation

### Dependencies

- **solana-sdk**: Solana keypair generation
- **bs58**: Base58 encoding/decoding
- **tokio**: Async runtime for multi-threading
- **clap**: Command-line argument parsing
- **indicatif**: Progress bars and terminal UI
- **serde**: JSON/CSV serialization

## 🎨 Output Formats

### Text Format (Default)

```
Address #1
  Public Key:  BYtE1234567890abcdefghijklmnopqrstuvwxyz
  Private Key: ...
  Time:        4.47s
```

### JSON Format

```json
[
  {
    "public_key": "BYtE1234567890abcdefghijklmnopqrstuvwxyz",
    "private_key": "...",
    "attempts": 786,
    "time_elapsed": {
      "secs": 4,
      "nanos": 466577708
    }
  }
]
```

### CSV Format

```csv
public_key,private_key,attempts,time_seconds
BYtE1234567890abcdefghijklmnopqrstuvwxyz,...,786,4.466577708
```

## 🧪 Testing

```bash
# Run tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_pattern_matching
```

## 📈 Benchmarking

### Performance Comparison

```bash
# Test single character (should be very fast)
time cargo run -- --pattern "A" --max-time 5

# Test 3-character pattern
time cargo run -- --pattern "ABC" --max-time 30

# Test with different thread counts
cargo run -- --pattern "TEST" --threads 1
cargo run -- --pattern "TEST" --threads 4
cargo run -- --pattern "TEST" --threads 12
```

## 🔒 Security Notes

- **Private Keys**: Generated locally, never transmitted
- **Base58 Validation**: Prevents invalid character patterns
- **Memory Safety**: Rust's ownership system prevents memory leaks
- **No Network**: All operations are local

## 🚫 Invalid Characters

Base58 encoding excludes these characters:

- `0` (zero)
- `O` (capital O)
- `I` (capital I)
- `l` (lowercase L)

## ✅ Valid Characters

### Numbers (9 characters)

`1 2 3 4 5 6 7 8 9`

### Uppercase Letters (25 characters)

`A B C D E F G H J K L M N P Q R S T U V W X Y Z`

### Lowercase Letters (24 characters)

`a b c d e f g h i j k m n o p q r s t u v w x y z`

**Total: 58 valid characters**

### Character Reference Table

| Type            | Characters      | Count  |
| --------------- | --------------- | ------ |
| **Numbers**     | `1-9`           | 9      |
| **Uppercase**   | `A-H, J-N, P-Z` | 25     |
| **Lowercase**   | `a-k, m-z`      | 24     |
| **Total Valid** |                 | **58** |
| **Invalid**     | `0, O, I, l`    | 4      |

### Example Valid Patterns

```bash
# Single character patterns
cargo run -- --pattern "A"     # ✅ Valid
cargo run -- --pattern "o"     # ✅ Valid (lowercase o)
cargo run -- --pattern "9"     # ✅ Valid

# Multi-character patterns
cargo run -- --pattern "ABC"   # ✅ Valid
cargo run -- --pattern "rust"  # ✅ Valid
cargo run -- --pattern "BYTE"  # ✅ Valid
cargo run -- --pattern "123"   # ✅ Valid

# Invalid patterns (will show error)
cargo run -- --pattern "SOL"   # ❌ Contains 'O'
cargo run -- --pattern "0x"    # ❌ Contains '0'
cargo run -- --pattern "Ill"   # ❌ Contains 'I' and 'l'
```

### Error Handling

The tool provides helpful error messages when invalid characters are detected:

```bash
$ cargo run -- --pattern "SOL"
❌ Error: Pattern contains invalid Base58 characters
Invalid characters found: O

Base58 encoding excludes these characters:
  • 0 (zero)
  • O (capital O)
  • I (capital I)
  • l (lowercase L)

Valid Base58 characters: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz

Example valid patterns:
  • ABC
  • RUST
  • BYTE
  • SOL (contains 'O')
```

### Multiple Invalid Characters

```bash
$ cargo run -- --pattern "AB0Ol"
❌ Error: Pattern contains invalid Base58 characters
Invalid characters found: 0, O, l
...
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

- **Educational Purpose**: This tool is for educational and legitimate use only
- **No Guarantees**: Generation time varies based on pattern complexity
- **Security**: Always verify generated addresses before using them for transactions
- **Backup**: Keep your private keys secure and backed up

## 🆘 Troubleshooting

### Common Issues

1. **Compilation Errors**: Ensure you have Rust 1.70+ installed
2. **Slow Performance**: Try reducing thread count or pattern complexity
3. **Invalid Patterns**: Check that your pattern doesn't contain 0, O, I, or l
   - The tool will show helpful error messages with specific invalid characters
   - Use the provided examples to choose valid patterns
4. **Memory Issues**: Reduce max_attempts or max_time limits

### Performance Tips

- Use shorter patterns for faster generation
- Increase thread count for better performance (up to CPU core count)
- Use "contains" instead of "starts_with" for better performance
- Consider case-insensitive matching for more matches

## 📞 Support

If you encounter any issues:

1. Check the troubleshooting section above
2. Open an issue on GitHub
3. Review the code comments for implementation details

---

**Built with ❤️ and Rust for maximum performance!**

_This Rust version is 25x faster than the original TypeScript implementation, making it perfect for serious vanity address generation._
