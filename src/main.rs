mod cli;
mod ntfs;
mod output;

fn get_filename_with_default(provided: Option<&str>, default: String) -> String {
    provided.map(|s| s.to_string()).unwrap_or(default)
}

use cli::{Cli, OutputFormat};
use ntfs::{FileType, *};
use output::*;

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use std::time::Instant;


fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    let log_level = if cli.trace {
        "trace"
    } else if cli.debug {
        "debug"
    } else {
        "info"
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();

    // Validate command line arguments
    if let Err(e) = cli.validate() {
        error!("Validation error: {}", e);
        std::process::exit(1);
    }

    let start_time = Instant::now();

    // Determine file type
    let file_type = detect_file_type(&cli.file)?;
    info!("Detected file type: {}", file_type);

    // Process file based on type
    let result = match file_type {
        FileType::Mft => process_mft(&cli),
        FileType::UsnJournal => process_usn_journal(&cli),
        FileType::Boot => process_boot(&cli),
        FileType::Sds => process_sds(&cli),
        FileType::I30 => process_i30(&cli),
        FileType::LogFile => {
            warn!("LogFile processing not yet implemented");
            Ok(())
        }
        FileType::Unknown => {
            error!("Unknown file type for: {}", cli.file.display());
            std::process::exit(1);
        }
    };

    let processing_time = start_time.elapsed();

    match result {
        Ok(()) => {
            info!("Processing completed successfully in {} ms", processing_time.as_millis());
        }
        Err(e) => {
            error!("Processing failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn detect_file_type(path: &Path) -> Result<FileType> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mmap = unsafe { Mmap::map(&file)? };

    if mmap.len() < 4 {
        return Ok(FileType::Unknown);
    }

    // Check first 4 bytes for signatures
    let signature = u32::from_le_bytes([mmap[0], mmap[1], mmap[2], mmap[3]]);

    match signature {
        0x454c4946 => Ok(FileType::Mft), // "FILE"
        0x58444e49 => Ok(FileType::I30), // "INDX"
        _ => {
            // Check for other patterns
            if mmap.len() >= 512 {
                // Check for NTFS boot sector
                if mmap[3..11] == *b"NTFS    " {
                    return Ok(FileType::Boot);
                }
            }

            // Check for USN Journal (starts with record length)
            if mmap.len() >= 60 {
                let record_length = u32::from_le_bytes([mmap[0], mmap[1], mmap[2], mmap[3]]);
                if record_length > 60 && record_length < 0x10000 {
                    return Ok(FileType::UsnJournal);
                }
            }

            // Default to unknown
            Ok(FileType::Unknown)
        }
    }
}

fn process_mft(cli: &Cli) -> Result<()> {
    info!("Processing MFT file: {}", cli.file.display());

    let file = File::open(&cli.file)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut parser = mft::MftParser::new(mmap.to_vec());
    parser.parse()?;

    let records = parser.get_records();
    info!("Parsed {} MFT records", records.len());

    // Handle specific entry dump if requested
    if let Some(ref entry_spec) = cli.dump_entry {
        dump_specific_entry(records, entry_spec)?;
        return Ok(());
    }

    // Output results
    output_results(cli, records, "mft")?;

    // Show console output if requested
    match cli.output_format {
        OutputFormat::Table => table::TableOutput::print_mft_records(records, Some(20)),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(records)?),
        OutputFormat::Csv => {
            // Print CSV headers and first few records
            println!("entry_number,sequence_number,file_name,file_size,in_use,is_directory");
            for record in records.iter().take(10) {
                println!("{},{},{},{},{},{}",
                    record.entry_number,
                    record.sequence_number,
                    record.file_name,
                    record.file_size,
                    record.in_use,
                    record.is_directory);
            }
        }
        OutputFormat::Minimal => {
            println!("Processed {} MFT records", records.len());
        }
    }

    Ok(())
}

fn process_usn_journal(cli: &Cli) -> Result<()> {
    info!("Processing USN Journal file: {}", cli.file.display());

    let file = File::open(&cli.file)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut parser = usn_journal::UsnJournalParser::new(mmap.to_vec());
    parser.parse()?;

    let entries = parser.get_entries();
    info!("Parsed {} USN Journal entries", entries.len());

    // Output results
    if let Some(ref json_dir) = cli.json_dir {
        let default_filename = cli.get_default_filename("json", "usn");
        let filename = cli.json_filename.as_deref().unwrap_or(&default_filename);
        let output_path = json_dir.join(filename);
        json::JsonOutput::write_usn_journal_entries(entries, &output_path)?;
        info!("JSON output written to: {}", output_path.display());
    }

    if let Some(ref csv_dir) = cli.csv_dir {
        let filename = get_filename_with_default(
            cli.csv_filename.as_deref(),
            cli.get_default_filename("csv", "usn")
        );
        let output_path = csv_dir.join(&filename);
        csv::CsvOutput::write_usn_journal_entries(entries, &output_path)?;
        info!("CSV output written to: {}", output_path.display());
    }

    if let Some(ref body_dir) = cli.body_dir {
        let filename = get_filename_with_default(
            cli.body_filename.as_deref(),
            cli.get_default_filename("body", "usn")
        );
        let output_path = body_dir.join(&filename);
        let drive_letter = cli.body_drive_letter.as_deref().unwrap_or("C");
        bodyfile::BodyfileOutput::write_usn_journal_entries(entries, &output_path, drive_letter, cli.body_lf)?;
        info!("Bodyfile output written to: {}", output_path.display());
    }

    // Console output
    match cli.output_format {
        OutputFormat::Table => table::TableOutput::print_usn_journal_entries(entries, Some(20)),
        _ => println!("Processed {} USN Journal entries", entries.len()),
    }

    Ok(())
}

fn process_boot(cli: &Cli) -> Result<()> {
    info!("Processing Boot sector file: {}", cli.file.display());

    let file = File::open(&cli.file)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let boot_sector = boot::BootParser::parse(&mmap)?;
    info!("Parsed boot sector information");

    // Output results
    if let Some(ref json_dir) = cli.json_dir {
        let filename = get_filename_with_default(
            cli.json_filename.as_deref(),
            cli.get_default_filename("json", "boot")
        );
        let output_path = json_dir.join(&filename);
        json::JsonOutput::write_boot_sector(&boot_sector, &output_path)?;
        info!("JSON output written to: {}", output_path.display());
    }

    if let Some(ref csv_dir) = cli.csv_dir {
        let filename = get_filename_with_default(
            cli.csv_filename.as_deref(),
            cli.get_default_filename("csv", "boot")
        );
        let output_path = csv_dir.join(&filename);
        csv::CsvOutput::write_boot_sector(&boot_sector, &output_path)?;
        info!("CSV output written to: {}", output_path.display());
    }

    // Console output
    table::TableOutput::print_boot_sector(&boot_sector);

    Ok(())
}

fn process_sds(cli: &Cli) -> Result<()> {
    info!("Processing SDS file: {}", cli.file.display());

    let file = File::open(&cli.file)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut parser = sds::SdsParser::new(mmap.to_vec());
    parser.parse()?;

    let descriptors = parser.get_descriptors();
    info!("Parsed {} security descriptors", descriptors.len());

    // Handle specific security descriptor dump if requested
    if let Some(ref security_id) = cli.dump_security {
        dump_specific_security_descriptor(descriptors, security_id)?;
        return Ok(());
    }

    // Output results
    if let Some(ref json_dir) = cli.json_dir {
        let filename = get_filename_with_default(
            cli.json_filename.as_deref(),
            cli.get_default_filename("json", "sds")
        );
        let output_path = json_dir.join(&filename);
        json::JsonOutput::write_security_descriptors(descriptors, &output_path)?;
        info!("JSON output written to: {}", output_path.display());
    }

    if let Some(ref csv_dir) = cli.csv_dir {
        let filename = get_filename_with_default(
            cli.csv_filename.as_deref(),
            cli.get_default_filename("csv", "sds")
        );
        let output_path = csv_dir.join(&filename);
        csv::CsvOutput::write_security_descriptors(descriptors, &output_path)?;
        info!("CSV output written to: {}", output_path.display());
    }

    // Console output
    match cli.output_format {
        OutputFormat::Table => table::TableOutput::print_security_descriptors(descriptors, Some(20)),
        _ => println!("Processed {} security descriptors", descriptors.len()),
    }

    Ok(())
}

fn process_i30(cli: &Cli) -> Result<()> {
    info!("Processing I30 index file: {}", cli.file.display());

    let file = File::open(&cli.file)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut parser = i30::I30Parser::new(mmap.to_vec());
    parser.parse()?;

    let entries = parser.get_entries();
    info!("Parsed {} index entries", entries.len());

    // Output results
    if let Some(ref json_dir) = cli.json_dir {
        let filename = get_filename_with_default(
            cli.json_filename.as_deref(),
            cli.get_default_filename("json", "i30")
        );
        let output_path = json_dir.join(&filename);
        json::JsonOutput::write_index_entries(entries, &output_path)?;
        info!("JSON output written to: {}", output_path.display());
    }

    if let Some(ref csv_dir) = cli.csv_dir {
        let filename = get_filename_with_default(
            cli.csv_filename.as_deref(),
            cli.get_default_filename("csv", "i30")
        );
        let output_path = csv_dir.join(&filename);
        csv::CsvOutput::write_index_entries(entries, &output_path)?;
        info!("CSV output written to: {}", output_path.display());
    }

    if let Some(ref body_dir) = cli.body_dir {
        let filename = get_filename_with_default(
            cli.body_filename.as_deref(),
            cli.get_default_filename("body", "i30")
        );
        let output_path = body_dir.join(&filename);
        let drive_letter = cli.body_drive_letter.as_deref().unwrap_or("C");
        bodyfile::BodyfileOutput::write_index_entries(entries, &output_path, drive_letter, cli.body_lf)?;
        info!("Bodyfile output written to: {}", output_path.display());
    }

    // Console output
    match cli.output_format {
        OutputFormat::Table => table::TableOutput::print_index_entries(entries, Some(20)),
        _ => println!("Processed {} index entries", entries.len()),
    }

    Ok(())
}

fn output_results(cli: &Cli, records: &[ntfs::types::MftRecord], file_type: &str) -> Result<()> {
    // JSON output
    if let Some(ref json_dir) = cli.json_dir {
        let filename = get_filename_with_default(
            cli.json_filename.as_deref(),
            cli.get_default_filename("json", file_type)
        );
        let output_path = json_dir.join(&filename);
        json::JsonOutput::write_mft_records(records, &output_path)?;
        info!("JSON output written to: {}", output_path.display());
    }

    // CSV output
    if let Some(ref csv_dir) = cli.csv_dir {
        let filename = get_filename_with_default(
            cli.csv_filename.as_deref(),
            cli.get_default_filename("csv", file_type)
        );
        let output_path = csv_dir.join(&filename);
        csv::CsvOutput::write_mft_records(records, &output_path)?;
        info!("CSV output written to: {}", output_path.display());
    }

    // Bodyfile output
    if let Some(ref body_dir) = cli.body_dir {
        let filename = get_filename_with_default(
            cli.body_filename.as_deref(),
            cli.get_default_filename("body", file_type)
        );
        let output_path = body_dir.join(&filename);
        let drive_letter = cli.body_drive_letter.as_deref().unwrap_or("C");
        bodyfile::BodyfileOutput::write_mft_records(records, &output_path, drive_letter, cli.body_lf)?;
        info!("Bodyfile output written to: {}", output_path.display());
    }

    Ok(())
}

fn dump_specific_entry(records: &[ntfs::types::MftRecord], entry_spec: &str) -> Result<()> {
    // Parse entry specification (e.g., "5", "624-5", "0x270-0x5")
    let (entry_num, seq_num) = parse_entry_spec(entry_spec)?;

    let record = records.iter()
        .find(|r| r.entry_number == entry_num &&
                  (seq_num.is_none() || Some(r.sequence_number) == seq_num))
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", entry_spec))?;

    println!("MFT Entry Details:");
    println!("{}", "-".repeat(50));
    println!("Entry Number:       {}", record.entry_number);
    println!("Sequence Number:    {}", record.sequence_number);
    println!("In Use:             {}", record.in_use);
    println!("File Name:          {}", record.file_name);
    println!("Parent Path:        {}", record.parent_path);
    println!("File Size:          {}", record.file_size);
    println!("Is Directory:       {}", record.is_directory);
    println!("Has ADS:            {}", record.has_ads);

    if let Some(created) = record.created_0x10 {
        println!("Created (0x10):     {}", created.format("%Y-%m-%d %H:%M:%S%.6f"));
    }
    if let Some(modified) = record.last_modified_0x10 {
        println!("Modified (0x10):    {}", modified.format("%Y-%m-%d %H:%M:%S%.6f"));
    }

    Ok(())
}

fn dump_specific_security_descriptor(descriptors: &[ntfs::types::SecurityDescriptor], security_id: &str) -> Result<()> {
    let id = parse_numeric_value(security_id)? as u32;

    let descriptor = descriptors.iter()
        .find(|d| d.id == id)
        .ok_or_else(|| anyhow::anyhow!("Security descriptor not found: {}", security_id))?;

    println!("Security Descriptor Details:");
    println!("{}", "-".repeat(50));
    println!("ID:                 {}", descriptor.id);
    println!("Hash:               0x{:08X}", descriptor.hash);
    println!("Offset:             0x{:016X}", descriptor.offset);
    println!("Length:             {}", descriptor.length);
    println!("Descriptor (hex):   {}", hex::encode(&descriptor.descriptor));

    Ok(())
}

fn parse_entry_spec(spec: &str) -> Result<(u32, Option<u16>)> {
    if let Some(dash_pos) = spec.find('-') {
        let entry_str = &spec[..dash_pos];
        let seq_str = &spec[dash_pos + 1..];

        let entry_num = parse_numeric_value(entry_str)? as u32;
        let seq_num = parse_numeric_value(seq_str)? as u16;

        Ok((entry_num, Some(seq_num)))
    } else {
        let entry_num = parse_numeric_value(spec)? as u32;
        Ok((entry_num, None))
    }
}

fn parse_numeric_value(value: &str) -> Result<u64> {
    if value.starts_with("0x") || value.starts_with("0X") {
        u64::from_str_radix(&value[2..], 16)
            .with_context(|| format!("Invalid hex value: {}", value))
    } else {
        value.parse::<u64>()
            .with_context(|| format!("Invalid decimal value: {}", value))
    }
}
