use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mfte-rs")]
#[command(about = "Cross-platform NTFS file system artifact parser")]
#[command(version = "0.1.0")]
#[command(author = "Claude Code")]
pub struct Cli {
    /// File to process ($MFT | $J | $LogFile | $Boot | $SDS | $I30). Required
    #[arg(short = 'f', long = "file", required = true)]
    pub file: PathBuf,

    /// $MFT file to use when -f points to a $J file (Use this to resolve parent path in $J CSV output)
    #[arg(short = 'm', long = "mft")]
    pub mft_file: Option<PathBuf>,

    /// Directory to save JSON formatted results to. This or --csv required unless --de or --body is specified
    #[arg(long = "json")]
    pub json_dir: Option<PathBuf>,

    /// File name to save JSON formatted results to. When present, overrides default name
    #[arg(long = "jsonf")]
    pub json_filename: Option<String>,

    /// Directory to save CSV formatted results to. This or --json required unless --de or --body is specified
    #[arg(long = "csv")]
    pub csv_dir: Option<PathBuf>,

    /// File name to save CSV formatted results to. When present, overrides default name
    #[arg(long = "csvf")]
    pub csv_filename: Option<String>,

    /// Directory to save bodyfile formatted results to. --bdl is also required when using this option
    #[arg(long = "body")]
    pub body_dir: Option<PathBuf>,

    /// File name to save body formatted results to. When present, overrides default name
    #[arg(long = "bodyf")]
    pub body_filename: Option<String>,

    /// Drive letter (C, D, etc.) to use with bodyfile. Only the drive letter itself should be provided
    #[arg(long = "bdl")]
    pub body_drive_letter: Option<String>,

    /// When true, use LF vs CRLF for newlines. Default is FALSE
    #[arg(long = "blf")]
    pub body_lf: bool,

    /// Directory to save exported FILE record. --do is also required when using this option
    #[arg(long = "dd")]
    pub dump_dir: Option<PathBuf>,

    /// Offset of the FILE record to dump as decimal or hex. Ex: 5120 or 0x1400 Use --de or --debug to see offsets
    #[arg(long = "do")]
    pub dump_offset: Option<String>,

    /// Dump full details for entry/sequence #. Format is 'Entry' or 'Entry-Seq' as decimal or hex. Example: 5, 624-5 or 0x270-0x5.
    #[arg(long = "de")]
    pub dump_entry: Option<String>,

    /// When true, dump resident files to dir specified by --csv or --json, in 'Resident' subdirectory
    #[arg(long = "dr")]
    pub dump_resident: bool,

    /// When true, displays contents of directory from specified by --de. Ignored when --de points to a file
    #[arg(long = "fls")]
    pub file_list: bool,

    /// Dump full details for Security Id as decimal or hex. Example: 624 or 0x270
    #[arg(long = "ds")]
    pub dump_security: Option<String>,

    /// The custom date/time format to use when displaying time stamps. Default is: %Y-%m-%d %H:%M:%S%.f
    #[arg(long = "dt")]
    pub datetime_format: Option<String>,

    /// Include DOS file name types. Default is FALSE
    #[arg(long = "sn")]
    pub include_short_names: bool,

    /// Generate condensed file listing. Requires --csv. Default is FALSE
    #[arg(long = "fl")]
    pub file_listing: bool,

    /// When true, include all timestamps from 0x30 attribute vs only when they differ from 0x10. Default is FALSE
    #[arg(long = "at")]
    pub all_timestamps: bool,

    /// Process all Volume Shadow Copies that exist on drive specified by -f. Default is FALSE
    #[arg(long = "vss")]
    pub volume_shadow_copies: bool,

    /// Deduplicate -f & VSCs based on SHA-1. First file found wins. Default is FALSE
    #[arg(long = "dedupe")]
    pub deduplicate: bool,

    /// Show debug information during processing
    #[arg(long = "debug")]
    pub debug: bool,

    /// Show trace information during processing
    #[arg(long = "trace")]
    pub trace: bool,

    /// Output format for console display
    #[arg(long = "format", value_enum, default_value_t = OutputFormat::Table)]
    pub output_format: OutputFormat,

    /// Show progress bar for large files
    #[arg(long = "progress")]
    pub show_progress: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Tabular output
    Table,
    /// JSON output to console
    Json,
    /// CSV output to console
    Csv,
    /// Minimal output
    Minimal,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        // Check that at least one output format is specified
        if self.json_dir.is_none()
            && self.csv_dir.is_none()
            && self.body_dir.is_none()
            && self.dump_entry.is_none()
            && self.dump_security.is_none() {
            return Err("At least one output option must be specified (--json, --csv, --body, --de, or --ds)".to_string());
        }

        // Check bodyfile requirements
        if self.body_dir.is_some() && self.body_drive_letter.is_none() {
            return Err("--bdl is required when using --body".to_string());
        }

        // Check dump requirements
        if self.dump_dir.is_some() && self.dump_offset.is_none() {
            return Err("--do is required when using --dd".to_string());
        }

        // Check file listing requirements
        if self.file_listing && self.csv_dir.is_none() {
            return Err("--fl requires --csv".to_string());
        }

        // Validate file exists
        if !self.file.exists() {
            return Err(format!("Input file does not exist: {}", self.file.display()));
        }

        // Validate MFT file if provided
        if let Some(ref mft_file) = self.mft_file {
            if !mft_file.exists() {
                return Err(format!("MFT file does not exist: {}", mft_file.display()));
            }
        }

        Ok(())
    }

    pub fn get_default_filename(&self, extension: &str, file_type: &str) -> String {
        let input_name = self.file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        format!("{}_{}.{}", input_name, file_type, extension)
    }
}