use crate::ntfs::types::*;
use anyhow::Result;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct BodyfileOutput;

impl BodyfileOutput {
    /// Write MFT records in bodyfile format
    /// Bodyfile format: MD5|name|inode|mode_as_string|UID|GID|size|atime|mtime|ctime|crtime
    pub fn write_mft_records<P: AsRef<Path>>(
        records: &[MftRecord],
        path: P,
        drive_letter: &str,
        use_lf: bool,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let newline = if use_lf { "\n" } else { "\r\n" };

        for record in records {
            if !record.in_use {
                continue;
            }

            let full_path = if record.parent_path.is_empty() {
                format!("{}:/{}", drive_letter, record.file_name)
            } else {
                format!("{}:/{}/{}", drive_letter, record.parent_path, record.file_name)
            };

            let mode = if record.is_directory { "d" } else { "r" };
            let permissions = format!("{}/r-xr-xr-x", mode);

            // Convert timestamps to Unix epoch
            let atime = record.last_access_0x10
                .map(|t| t.timestamp())
                .unwrap_or(0);
            let mtime = record.last_modified_0x10
                .map(|t| t.timestamp())
                .unwrap_or(0);
            let ctime = record.last_record_change_0x10
                .map(|t| t.timestamp())
                .unwrap_or(0);
            let crtime = record.created_0x10
                .map(|t| t.timestamp())
                .unwrap_or(0);

            let line = format!(
                "0|{}|{}|{}|0|0|{}|{}|{}|{}|{}{}",
                full_path,
                record.entry_number,
                permissions,
                record.file_size,
                atime,
                mtime,
                ctime,
                crtime,
                newline
            );

            writer.write_all(line.as_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Write USN Journal entries in bodyfile format
    pub fn write_usn_journal_entries<P: AsRef<Path>>(
        entries: &[UsnJournalEntry],
        path: P,
        drive_letter: &str,
        use_lf: bool,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let newline = if use_lf { "\n" } else { "\r\n" };

        for entry in entries {
            let full_path = if entry.full_path.is_empty() {
                format!("{}:/{}", drive_letter, entry.file_name)
            } else {
                format!("{}:{}", drive_letter, entry.full_path)
            };

            let is_directory = (entry.file_attributes & 0x10) != 0;
            let mode = if is_directory { "d" } else { "r" };
            let permissions = format!("{}/r-xr-xr-x", mode);

            let timestamp = entry.timestamp.timestamp();

            let line = format!(
                "0|{}|{}|{}|0|0|0|{}|{}|{}|{}{}",
                full_path,
                entry.entry_number,
                permissions,
                timestamp,
                timestamp,
                timestamp,
                timestamp,
                newline
            );

            writer.write_all(line.as_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Write Index entries in bodyfile format
    pub fn write_index_entries<P: AsRef<Path>>(
        entries: &[IndexEntry],
        path: P,
        drive_letter: &str,
        use_lf: bool,
    ) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let newline = if use_lf { "\n" } else { "\r\n" };

        for entry in entries {
            let full_path = if entry.full_path.is_empty() {
                format!("{}:/{}", drive_letter, entry.file_name)
            } else {
                format!("{}:{}", drive_letter, entry.full_path)
            };

            let mode = if entry.is_directory { "d" } else { "r" };
            let permissions = format!("{}/r-xr-xr-x", mode);

            let atime = entry.accessed.timestamp();
            let mtime = entry.modified.timestamp();
            let ctime = entry.modified.timestamp(); // Use modified as record change time
            let crtime = entry.created.timestamp();

            let line = format!(
                "0|{}|{}|{}|0|0|{}|{}|{}|{}|{}{}",
                full_path,
                entry.entry_number,
                permissions,
                entry.file_size,
                atime,
                mtime,
                ctime,
                crtime,
                newline
            );

            writer.write_all(line.as_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }
}