# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mfte-rs is a cross-platform NTFS file system artifact parser written in Rust, designed as a modern alternative to the C# MFTECmd tool. It provides high-performance parsing of MFT, USN Journal, Boot sectors, SDS security descriptors, and I30 directory index files with support for multiple output formats.

## Build and Development Commands

### Building the Project
```bash
# Debug build for development
cargo build

# Release build for production (optimized)
cargo build --release

# Build with all features enabled
cargo build --release --all-features

# Build without default features (no progress bar)
cargo build --release --no-default-features

# Check compilation without building
cargo check
```

### Running the Application
```bash
# Run with cargo (development)
cargo run -- -f "/path/to/mft" --csv "/output/dir"

# Run release binary directly
./target/release/mfte-rs -f "/path/to/mft" --csv "/output/dir"

# Example commands for different file types
cargo run -- -f sample_mft --json output --debug
cargo run -- -f sample_usn_journal -m sample_mft --csv output
cargo run -- -f sample_boot --format table
```

### Testing and Quality
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test ntfs::mft

# Code formatting
cargo fmt
cargo fmt --check

# Linting with clippy
cargo clippy
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit
```

## Architecture Overview

### Core Module Structure

**main.rs** (Application Entry Point)
- Command-line argument parsing and validation
- File type auto-detection based on signatures
- Processing pipeline coordination
- Cross-platform file handling with memory mapping
- Error handling and logging setup

**cli/mod.rs** (Command-Line Interface)
- Comprehensive argument parsing using clap
- Input validation and file existence checks
- Output format and filename handling
- Cross-platform path processing

**ntfs/** (NTFS Parsing Core)
- `types.rs`: Common data structures and enums for all NTFS artifacts
- `mft.rs`: Master File Table parser with attribute processing
- `usn_journal.rs`: USN Journal change tracking parser
- `boot.rs`: NTFS boot sector parser
- `sds.rs`: Security descriptor parser
- `i30.rs`: Directory index parser

**output/** (Output Format Handlers)
- `csv.rs`: CSV serialization using csv crate
- `json.rs`: JSON serialization using serde_json
- `bodyfile.rs`: Timeline format for forensic analysis
- `table.rs`: Console table formatting and progress display

### Key Design Patterns

**Memory-Mapped I/O**: Uses memmap2 for efficient file access without loading entire files into memory

**Zero-Copy Parsing**: Minimal memory allocations during binary parsing operations

**Error Propagation**: Uses anyhow for comprehensive error handling and context

**Type Safety**: Leverages Rust's type system for memory-safe binary parsing

**Modular Architecture**: Clean separation between parsing, CLI, and output concerns

### File Type Detection

The application auto-detects NTFS artifact types using signature-based detection:
- MFT files: "FILE" signature (0x454c4946)
- I30 indexes: "INDX" signature (0x58444e49)
- Boot sectors: "NTFS    " OEM identifier
- USN Journal: Heuristic based on record length patterns
- SDS: Structure-based detection

### Cross-Platform Considerations

**File Path Handling**: Uses std::path::PathBuf for cross-platform path operations

**Memory Mapping**: memmap2 crate handles platform-specific memory mapping

**Newline Handling**: Configurable CRLF vs LF for bodyfile output

**Binary Parsing**: Little-endian byte order (NTFS standard) using byteorder crate

## Development Workflow

### Adding New NTFS Artifact Support
1. Define data structures in `ntfs/types.rs`
2. Implement parser in new module (e.g., `ntfs/logfile.rs`)
3. Add file type detection logic in `main.rs`
4. Update CLI options in `cli/mod.rs`
5. Add output format support in `output/` modules
6. Update documentation and examples

### Parser Implementation Pattern
```rust
// Standard parser structure
pub struct NewParser {
    data: Vec<u8>,
    records: Vec<NewRecord>,
}

impl NewParser {
    pub fn new(data: Vec<u8>) -> Self { ... }
    pub fn parse(&mut self) -> ParseResult<()> { ... }
    pub fn get_records(&self) -> &[NewRecord] { ... }
}
```

### Error Handling Strategy
- Use `ParseResult<T>` for parser operations
- Provide detailed error messages with byte offsets
- Log warnings for recoverable parsing issues
- Use `anyhow::Context` for error context preservation

### Performance Optimization
- Prefer `&[u8]` slices over `Vec<u8>` copies where possible
- Use `nom` combinator library for complex binary parsing
- Implement streaming parsers for very large files
- Add progress reporting for long-running operations

## Dependencies and Features

### Core Dependencies
- `clap`: Command-line argument parsing with derive macros
- `serde`: Serialization framework for JSON/CSV output
- `memmap2`: Memory-mapped file I/O
- `anyhow`/`thiserror`: Error handling
- `chrono`: Date/time handling for NTFS timestamps

### Optional Features
- `progress`: Progress bar support using indicatif
- Default features can be disabled for minimal builds

### Binary Size Optimization
- Release profile uses LTO and aggressive optimization
- Single codegen unit for maximum optimization
- Panic abort for smaller binary size

## Testing Strategy

### Unit Tests
- Parser correctness with known test vectors
- Error handling for malformed input
- Edge cases and boundary conditions

### Integration Tests
- End-to-end CLI testing with sample files
- Output format validation
- Cross-platform file handling

### Sample Data
- Include sanitized NTFS artifacts for testing
- Test files should cover various NTFS versions
- Edge cases: empty files, corrupted headers, large files

## Common Development Tasks

### Adding CLI Options
1. Update `Cli` struct in `cli/mod.rs`
2. Add validation in `Cli::validate()`
3. Handle new option in processing functions
4. Update help text and documentation

### Supporting New Output Formats
1. Create new module in `output/`
2. Implement serialization for all data types
3. Add format variant to `OutputFormat` enum
4. Update processing pipeline in `main.rs`

### Debugging Parsing Issues
1. Enable trace logging: `RUST_LOG=trace cargo run`
2. Use hex dump tools to examine binary structure
3. Add debug prints with byte offsets
4. Validate against known good parsers

### Performance Profiling
```bash
# Build with debug symbols
cargo build --release --debug

# Profile with perf (Linux)
perf record target/release/mfte-rs -f large_mft --csv output
perf report

# Profile with Instruments (macOS)
# Use Xcode Instruments for detailed profiling
```

## Security Considerations

### Input Validation
- All binary parsing includes bounds checking
- Rust's memory safety prevents buffer overflows
- Validate file signatures before processing
- Limit maximum allocations to prevent DoS

### Safe Parsing Practices
- Never trust input data lengths
- Use saturating arithmetic for size calculations
- Implement timeouts for large file processing
- Validate UTF-16 strings from NTFS structures

## Release Process

### Pre-release Checklist
1. Run full test suite: `cargo test`
2. Check formatting: `cargo fmt --check`
3. Run clippy: `cargo clippy -- -D warnings`
4. Test on all target platforms
5. Update version in Cargo.toml
6. Update CHANGELOG.md

### Cross-Platform Builds
```bash
# Add targets for cross-compilation
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin

# Build for multiple targets
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
```