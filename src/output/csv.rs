use crate::ntfs::types::*;
use anyhow::Result;
use csv::Writer;
use std::fs::{create_dir_all, File};
use std::path::Path;

pub struct CsvOutput;

impl CsvOutput {
    pub fn write_mft_records<P: AsRef<Path>>(
        records: &[MftRecord],
        path: P,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;;
        let mut writer = Writer::from_writer(file);

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
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
        let mut writer = Writer::from_writer(file);

        for entry in entries {
            writer.serialize(entry)?;
        }

        writer.flush()?;
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
        let mut writer = Writer::from_writer(file);

        writer.serialize(boot)?;
        writer.flush()?;
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
        let mut writer = Writer::from_writer(file);

        for descriptor in descriptors {
            // Convert binary data to hex string for CSV
            let descriptor_csv = SecurityDescriptorCsv {
                id: descriptor.id,
                hash: descriptor.hash,
                offset: descriptor.offset,
                length: descriptor.length,
                descriptor_hex: hex::encode(&descriptor.descriptor),
            };
            writer.serialize(&descriptor_csv)?;
        }

        writer.flush()?;
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
        let mut writer = Writer::from_writer(file);

        for entry in entries {
            writer.serialize(entry)?;
        }

        writer.flush()?;
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
        let mut writer = Writer::from_writer(file);

        for entry in entries {
            writer.serialize(entry)?;
        }

        writer.flush()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SecurityDescriptorCsv {
    id: u32,
    hash: u32,
    offset: u64,
    length: u32,
    descriptor_hex: String,
}