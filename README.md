# Nostr Vanity npub Generator

A high-performance Nostr vanity address generator written in Rust with parallel processing support.

## Features

- 🚀 **High Performance**: Utilizes all CPU cores with Rayon parallelization
- 🔍 **Multiple Patterns**: Search for multiple patterns simultaneously
- 📁 **Flexible Input**: Accept patterns via command line or CSV file
- 💾 **Output Options**: Display to stdout or save to file (text/CSV)
- 🎯 **Match Types**: Support for prefix, suffix, and contains matching
- 📊 **Progress Tracking**: Real-time statistics and time estimates
- 🔒 **Secure**: Uses cryptographically secure random number generation

## Installation

### Pre-built Binaries (Recommended)

Download the latest pre-built binary for your platform from the [releases page](https://github.com/typedcypher/nostr_vanity/releases):

#### Linux
```bash
# Download and extract (replace with actual release URL)
curl -L https://github.com/typedcypher/nostr_vanity/releases/latest/download/nostr_vanity-x86_64-unknown-linux-gnu.tar.gz | tar xzv
sudo mv nostr_vanity /usr/local/bin/

# Or for musl systems
curl -L https://github.com/typedcypher/nostr_vanity/releases/latest/download/nostr_vanity-x86_64-unknown-linux-musl.tar.gz | tar xzv
sudo mv nostr_vanity /usr/local/bin/
```

#### macOS
```bash
# Download and extract (replace with actual release URL)
curl -L https://github.com/typedcypher/nostr_vanity/releases/latest/download/nostr_vanity-x86_64-apple-darwin.tar.gz | tar xzv
sudo mv nostr_vanity /usr/local/bin/

# For Apple Silicon Macs
curl -L https://github.com/typedcypher/nostr_vanity/releases/latest/download/nostr_vanity-aarch64-apple-darwin.tar.gz | tar xzv
sudo mv nostr_vanity /usr/local/bin/
```

#### Windows
1. Download `nostr_vanity-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/typedcypher/nostr_vanity/releases)
2. Extract the ZIP file
3. Add the extracted `nostr_vanity.exe` to your PATH or run it directly

*Note: Only x86_64 Windows builds are provided. ARM64 Windows is not currently supported.*

### From Source

Requires Rust 1.89+ installed.

```bash
git clone https://github.com/typedcypher/nostr_vanity.git
cd nostr_vanity
cargo build --release
```

The binary will be available at `target/release/nostr_vanity`

### Cargo Install

```bash
cargo install --git https://github.com/typedcypher/nostr_vanity.git
```

## Usage

### Basic Examples

Search for a single pattern (prefix):
```bash
nostr_vanity --patterns "alice"
```

Search for multiple patterns:
```bash
nostr_vanity --patterns "alice,bob,charlie"
```

Use patterns from a file:
```bash
nostr_vanity --file patterns.csv
```

### Advanced Options

```bash
# Suffix matching
nostr_vanity --patterns "end" --match-type suffix

# Contains matching
nostr_vanity --patterns "middle" --match-type contains

# Case-sensitive search (faster)
nostr_vanity --patterns "Test" --case-sensitive

# Save results to file
nostr_vanity --patterns "rare" --output results.txt

# CSV output format
nostr_vanity --patterns "data" --output results.csv --csv

# Continuous mode (find multiple matches)
nostr_vanity --patterns "test" --continuous --output collection.txt

# Specify thread count
nostr_vanity --patterns "fast" --threads 8

# Quiet mode (minimal output)
nostr_vanity --patterns "silent" --quiet

# Estimate time for patterns
nostr_vanity --patterns "bitcoin,lightning" --estimate
```

### Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--patterns` | `-p` | Comma-separated list of patterns |
| `--file` | `-f` | Path to patterns file |
| `--output` | `-o` | Output file path |
| `--csv` | | Output in CSV format |
| `--match-type` | `-m` | Match type: prefix, suffix, contains |
| `--case-sensitive` | `-c` | Case-sensitive matching |
| `--threads` | `-t` | Number of CPU threads |
| `--continuous` | | Continue after finding matches |
| `--quiet` | `-q` | Minimal output |
| `--estimate` | | Show time estimates and exit |

## Pattern File Format

Create a text file with one pattern per line:
```
# Comments start with #
alice
bob
charlie
# Longer patterns take more time
satoshi
```

## Valid Characters

npub addresses use bech32 encoding. Valid characters for patterns:
```
023456789acdefghjklmnpqrstuvwxyz
```

Note: The characters `1`, `b`, `i`, and `o` are NOT valid in bech32.

## Performance

Performance varies by hardware. On a modern multi-core CPU:

| Pattern Length | Approximate Time | Keys/sec |
|---------------|------------------|----------|
| 4 characters | < 1 minute | ~500k |
| 5 characters | Minutes | ~500k |
| 6 characters | Hours | ~500k |
| 7 characters | Days | ~500k |
| 8+ characters | Weeks+ | ~500k |

### Performance Tips

1. **Use case-sensitive matching** when possible (faster)
2. **Search for multiple patterns** simultaneously (efficient)
3. **Shorter patterns** are exponentially faster to find
4. **More CPU cores** = proportionally faster searching

## Security

- Uses cryptographically secure random number generation
- Private keys are generated using `secp256k1` library
- Never share your `nsec` (private key) with anyone
- Consider running offline for maximum security
- Verify the source code before using for important keys

## Output Format

### Standard Output
```
✨ Found vanity address!
Pattern: alice
npub: npub1alice7x4k9hl5wl3x5hxqkp4w8x8u5qxh9lq5xr
nsec: nsec1qzkzp6rpp5jqrgu3wfvdnwzvl9rkzln3clhqxp
Hex pubkey: a1ce45f3a...
Attempts: 15234
Time: 0.35s
Speed: 43525 keys/sec
```

### CSV Output
```csv
pattern,npub,nsec,hex_pubkey,attempts,time_seconds
alice,npub1alice...,nsec1...,a1ce45...,15234,0.35
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests.

## License

MIT License - see LICENSE file for details

## Disclaimer

This tool generates cryptographic keys. Always verify the generated keys work correctly with your Nostr client before using them. The authors are not responsible for any loss of funds or data.