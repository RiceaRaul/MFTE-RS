# mfte-rs

Cross-platform NTFS file system artifact parser written in Rust, providing a modern alternative to the C# MFTECmd tool.

## Features

- **Cross-platform**: Runs on Windows, Linux, macOS, and other Unix-like systems
- **NTFS Artifact Support**: Parse MFT, USN Journal, Boot sectors, SDS, and I30 index files
- **Multiple Output Formats**: CSV, JSON, and Bodyfile formats
- **High Performance**: Memory-mapped file access and optimized parsing
- **Modern CLI**: Rich command-line interface with comprehensive options
- **Safe**: Memory-safe Rust implementation with proper error handling

## Supported File Types

| File Type | Description | Status |
|-----------|-------------|--------|
| `$MFT` | Master File Table | ✅ Implemented |
| `$J` | USN Journal | ✅ Implemented |
| `$Boot` | Boot Sector | ✅ Implemented |
| `$SDS` | Security Descriptors | ✅ Implemented |
| `$I30` | Directory Index | ✅ Implemented |
| `$LogFile` | NTFS Log File | 🚧 Planned |

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-username/mfte-rs.git
cd mfte-rs

# Build the project
cargo build --release

# The binary will be available at target/release/mfte-rs
```

### Using Cargo

```bash
cargo install mfte-rs
```

## Usage

### Basic Examples

```bash
# Parse MFT file and output to CSV
mfte-rs -f /path/to/\$MFT --csv /output/directory

# Parse MFT file and output to JSON
mfte-rs -f /path/to/\$MFT --json /output/directory

# Parse USN Journal with MFT context
mfte-rs -f /path/to/\$J -m /path/to/\$MFT --csv /output/directory

# Parse Boot sector
mfte-rs -f /path/to/\$Boot --json /output/directory

# Generate bodyfile format for timeline analysis
mfte-rs -f /path/to/\$MFT --body /output/directory --bdl C

# Dump specific MFT entry details
mfte-rs -f /path/to/\$MFT --de 5

# Dump specific MFT entry with sequence number
mfte-rs -f /path/to/\$MFT --de 624-5

# Dump security descriptor
mfte-rs -f /path/to/\$SDS --ds 1234 --csv /output/directory
```

### Advanced Usage

```bash
# Enable debug logging
mfte-rs -f /path/to/\$MFT --csv /output --debug

# Enable trace logging for detailed analysis
mfte-rs -f /path/to/\$MFT --csv /output --trace

# Custom output filename
mfte-rs -f /path/to/\$MFT --csv /output --csvf custom_name.csv

# Table output format to console
mfte-rs -f /path/to/\$MFT --csv /output --format table

# Show progress bar for large files
mfte-rs -f /path/to/\$MFT --csv /output --progress
```

## Command Line Options

| Option | Description |
|--------|-------------|
| `-f, --file <FILE>` | File to process (required) |
| `-m, --mft <FILE>` | MFT file for USN Journal path resolution |
| `--json <DIR>` | Output directory for JSON format |
| `--jsonf <NAME>` | Custom JSON filename |
| `--csv <DIR>` | Output directory for CSV format |
| `--csvf <NAME>` | Custom CSV filename |
| `--body <DIR>` | Output directory for bodyfile format |
| `--bodyf <NAME>` | Custom bodyfile filename |
| `--bdl <DRIVE>` | Drive letter for bodyfile (required with --body) |
| `--blf` | Use LF instead of CRLF for newlines |
| `--de <ENTRY>` | Dump specific MFT entry details |
| `--ds <ID>` | Dump specific security descriptor |
| `--format <FORMAT>` | Console output format (table, json, csv, minimal) |
| `--debug` | Enable debug logging |
| `--trace` | Enable trace logging |
| `--progress` | Show progress bar |

## Output Formats

### CSV Format
Structured tabular data compatible with Excel and data analysis tools.

### JSON Format
Complete object serialization with full metadata preservation.

### Bodyfile Format
Timeline format compatible with forensic analysis tools like Sleuth Kit.

Format: `MD5|name|inode|mode_as_string|UID|GID|size|atime|mtime|ctime|crtime`

## Performance

mfte-rs is designed for high performance:

- **Memory-mapped I/O**: Efficient file access without loading entire files into memory
- **Zero-copy parsing**: Minimal memory allocations during parsing
- **Parallel processing**: Multi-threaded processing for large datasets (when applicable)
- **Optimized builds**: Release builds use aggressive optimization

## Cross-Platform Support

### Windows
- Native support for NTFS file system access
- Direct access to volume shadow copies (future feature)
- Windows-specific file path handling

### Linux/macOS/Unix
- Parse NTFS images and raw disk images
- Support for forensic disk images (dd, E01 via external tools)
- Cross-platform file handling

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build with all features
cargo build --release --all-features

# Build without progress bar feature
cargo build --release --no-default-features
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_mft_parsing
```

### Linting

```bash
# Check code formatting
cargo fmt --check

# Apply code formatting
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture

### Core Components

- **NTFS Parsers**: Low-level binary parsers for each file type
- **CLI Interface**: Command-line argument processing and validation
- **Output Modules**: Formatters for CSV, JSON, and bodyfile outputs
- **Error Handling**: Comprehensive error reporting and recovery

### Module Structure

```
src/
├── main.rs           # Main application entry point
├── cli/              # Command-line interface
│   └── mod.rs        # CLI argument parsing and validation
├── ntfs/             # NTFS parsing implementations
│   ├── mod.rs        # Module exports
│   ├── types.rs      # Common data structures
│   ├── mft.rs        # MFT parser
│   ├── usn_journal.rs # USN Journal parser
│   ├── boot.rs       # Boot sector parser
│   ├── sds.rs        # Security descriptor parser
│   └── i30.rs        # Index parser
└── output/           # Output format implementations
    ├── mod.rs        # Module exports
    ├── csv.rs        # CSV output
    ├── json.rs       # JSON output
    ├── bodyfile.rs   # Bodyfile output
    └── table.rs      # Console table output
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Run clippy lints (`cargo clippy`)
- Add tests for new features

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Comparison with Original MFTECmd

| Feature | MFTECmd (C#) | mfte-rs (Rust) |
|---------|-------------|----------------|
| Cross-platform | Windows only | Windows, Linux, macOS |
| Memory Safety | Managed (GC) | Memory safe (no GC) |
| Performance | Good | Excellent |
| Dependencies | .NET Framework/Core | None (static binary) |
| File Size | ~50MB with deps | ~5MB static binary |
| Build Time | Fast | Fast |
| Output Formats | CSV, JSON, Body | CSV, JSON, Body + Table |

## Acknowledgments

- Based on the original MFTECmd by Eric Zimmerman
- Inspired by the NTFS file system specifications
- Built with the Rust ecosystem

## Related Tools

- [MFTECmd](https://github.com/EricZimmerman/MFTECmd) - Original C# implementation
- [analyzeMFT](https://github.com/dkovar/analyzeMFT) - Python MFT parser
- [Sleuth Kit](https://www.sleuthkit.org/) - Digital forensics toolkit