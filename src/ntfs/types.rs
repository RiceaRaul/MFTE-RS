use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Mft = 0,
    LogFile = 1,
    UsnJournal = 2,
    Boot = 3,
    Sds = 4,
    I30 = 5,
    Unknown = 99,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::Mft => write!(f, "MFT"),
            FileType::LogFile => write!(f, "LogFile"),
            FileType::UsnJournal => write!(f, "USN Journal"),
            FileType::Boot => write!(f, "Boot"),
            FileType::Sds => write!(f, "SDS"),
            FileType::I30 => write!(f, "I30"),
            FileType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MftRecord {
    pub entry_number: u32,
    pub sequence_number: u16,
    pub parent_entry_number: u32,
    pub parent_sequence_number: Option<u16>,
    pub in_use: bool,
    pub parent_path: String,
    pub file_name: String,
    pub extension: String,
    pub is_directory: bool,
    pub has_ads: bool,
    pub is_ads: bool,
    pub file_size: u64,
    pub created_0x10: Option<DateTime<Utc>>,
    pub created_0x30: Option<DateTime<Utc>>,
    pub last_modified_0x10: Option<DateTime<Utc>>,
    pub last_modified_0x30: Option<DateTime<Utc>>,
    pub last_record_change_0x10: Option<DateTime<Utc>>,
    pub last_record_change_0x30: Option<DateTime<Utc>>,
    pub last_access_0x10: Option<DateTime<Utc>>,
    pub last_access_0x30: Option<DateTime<Utc>>,
    pub update_sequence_number: i64,
    pub logfile_sequence_number: i64,
    pub security_id: i32,
    pub zone_id_contents: String,
    pub si_flags: u32,
    pub object_id_file_droid: String,
    pub reparse_target: String,
    pub reference_count: i32,
    pub name_type: u8,
    pub logged_util_stream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsnJournalEntry {
    pub offset: u64,
    pub timestamp: DateTime<Utc>,
    pub entry_number: u32,
    pub sequence_number: u16,
    pub parent_entry_number: u32,
    pub parent_sequence_number: u16,
    pub file_name: String,
    pub full_path: String,
    pub extension: String,
    pub reason: String,
    pub file_attributes: u32,
    pub usn: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub total_sectors: u64,
    pub mft_start_cluster: u64,
    pub mft_mirror_start_cluster: u64,
    pub clusters_per_mft_record: i8,
    pub clusters_per_index_buffer: i8,
    pub volume_serial_number: u64,
    pub oem_id: String,
    pub volume_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDescriptor {
    pub id: u32,
    pub hash: u32,
    pub offset: u64,
    pub length: u32,
    pub descriptor: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub entry_number: u32,
    pub sequence_number: u16,
    pub parent_entry_number: u32,
    pub parent_sequence_number: u16,
    pub file_name: String,
    pub full_path: String,
    pub file_size: u64,
    pub is_directory: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub attributes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListEntry {
    pub entry_number: u32,
    pub sequence_number: u16,
    pub file_name: String,
    pub full_path: String,
    pub extension: String,
    pub file_size: u64,
    pub is_directory: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub offset: Option<u64>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "Parse error at offset 0x{:x}: {}", offset, self.message),
            None => write!(f, "Parse error: {}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;