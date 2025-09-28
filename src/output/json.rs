use crate::ntfs::types::*;
use anyhow::Result;
use serde_json;
use std::fs::{create_dir_all, File};
use std::path::Path;

pub struct JsonOutput;

impl JsonOutput {
    pub fn write_mft_records<P: AsRef<Path>>(
        records: &[MftRecord],
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, records)?;
        Ok(())
    }

    pub fn write_usn_journal_entries<P: AsRef<Path>>(
        entries: &[UsnJournalEntry],
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, entries)?;
        Ok(())
    }

    pub fn write_boot_sector<P: AsRef<Path>>(
        boot: &BootSector,
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, boot)?;
        Ok(())
    }

    pub fn write_security_descriptors<P: AsRef<Path>>(
        descriptors: &[SecurityDescriptor],
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;

        // Convert binary data to hex for JSON serialization
        let descriptors_json: Vec<_> = descriptors
            .iter()
            .map(|desc| SecurityDescriptorJson {
                id: desc.id,
                hash: desc.hash,
                offset: desc.offset,
                length: desc.length,
                descriptor_hex: hex::encode(&desc.descriptor),
            })
            .collect();

        serde_json::to_writer_pretty(file, &descriptors_json)?;
        Ok(())
    }

    pub fn write_index_entries<P: AsRef<Path>>(
        entries: &[IndexEntry],
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, entries)?;
        Ok(())
    }

    pub fn write_file_listing<P: AsRef<Path>>(
        entries: &[FileListEntry],
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, entries)?;
        Ok(())
    }

    pub fn write_analysis_summary<P: AsRef<Path>>(
        summary: &AnalysisSummary,
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, summary)?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SecurityDescriptorJson {
    id: u32,
    hash: u32,
    offset: u64,
    length: u32,
    descriptor_hex: String,
}

#[derive(serde::Serialize)]
pub struct AnalysisSummary {
    pub file_type: String,
    pub file_size: u64,
    pub records_processed: usize,
    pub processing_time_ms: u128,
    pub errors_encountered: usize,
    pub warnings: Vec<String>,
}