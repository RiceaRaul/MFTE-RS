use super::types::{UsnJournalEntry, ParseError, ParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, Utc};
use std::io::{Cursor, Read};

pub struct UsnJournalParser {
    data: Vec<u8>,
    entries: Vec<UsnJournalEntry>,
}

impl UsnJournalParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            entries: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<()> {
        let mut cursor = Cursor::new(&self.data);
        let mut offset = 0u64;

        while (cursor.position() as usize) < self.data.len() {
            match self.parse_entry(&mut cursor, offset) {
                Ok(Some(entry)) => {
                    offset += entry.offset;
                    self.entries.push(entry);
                }
                Ok(None) => break, // End of valid entries
                Err(e) => {
                    log::warn!("Failed to parse USN Journal entry at offset 0x{:x}: {}", offset, e);
                    break;
                }
            }
        }

        log::info!("Parsed {} USN Journal entries", self.entries.len());
        Ok(())
    }

    fn parse_entry(&self, cursor: &mut Cursor<&Vec<u8>>, base_offset: u64) -> ParseResult<Option<UsnJournalEntry>> {
        let start_pos = cursor.position();

        if start_pos + 60 > self.data.len() as u64 {
            return Ok(None); // Not enough data for minimum USN record
        }

        let record_length = cursor.read_u32::<LittleEndian>()
            .map_err(|_| ParseError {
                message: "Failed to read USN record length".to_string(),
                offset: Some(base_offset + start_pos),
            })?;

        if record_length == 0 {
            return Ok(None); // End of records
        }

        let _major_version = cursor.read_u16::<LittleEndian>().unwrap();
        let _minor_version = cursor.read_u16::<LittleEndian>().unwrap();

        let file_reference = cursor.read_u64::<LittleEndian>().unwrap();
        let entry_number = (file_reference & 0xFFFFFFFFFFFF) as u32;
        let sequence_number = (file_reference >> 48) as u16;

        let parent_file_reference = cursor.read_u64::<LittleEndian>().unwrap();
        let parent_entry_number = (parent_file_reference & 0xFFFFFFFFFFFF) as u32;
        let parent_sequence_number = (parent_file_reference >> 48) as u16;

        let usn = cursor.read_u64::<LittleEndian>().unwrap();
        let timestamp = cursor.read_u64::<LittleEndian>().unwrap();
        let reason = cursor.read_u32::<LittleEndian>().unwrap();
        let _source_info = cursor.read_u32::<LittleEndian>().unwrap();
        let _security_id = cursor.read_u32::<LittleEndian>().unwrap();
        let file_attributes = cursor.read_u32::<LittleEndian>().unwrap();
        let file_name_length = cursor.read_u16::<LittleEndian>().unwrap();
        let file_name_offset = cursor.read_u16::<LittleEndian>().unwrap();

        // Read filename
        let current_pos = cursor.position();
        cursor.set_position(start_pos + file_name_offset as u64);

        let mut name_bytes = vec![0u8; file_name_length as usize];
        cursor.read_exact(&mut name_bytes).unwrap();

        let file_name = string_from_utf16le(&name_bytes)
            .unwrap_or_else(|_| String::from("INVALID_NAME"));

        // Extract extension
        let extension = if let Some(dot_pos) = file_name.rfind('.') {
            file_name[dot_pos + 1..].to_string()
        } else {
            String::new()
        };

        // Convert Windows FILETIME to DateTime<Utc>
        let datetime = windows_filetime_to_datetime(timestamp);

        let entry = UsnJournalEntry {
            offset: record_length as u64,
            timestamp: datetime,
            entry_number,
            sequence_number,
            parent_entry_number,
            parent_sequence_number,
            file_name,
            full_path: String::new(), // Will be resolved later if MFT is available
            extension,
            reason: format_usn_reason(reason),
            file_attributes,
            usn,
        };

        // Move to next record
        cursor.set_position(start_pos + record_length as u64);

        Ok(Some(entry))
    }

    pub fn get_entries(&self) -> &[UsnJournalEntry] {
        &self.entries
    }
}

fn windows_filetime_to_datetime(filetime: u64) -> DateTime<Utc> {
    // Windows FILETIME is 100-nanosecond intervals since January 1, 1601
    // Unix timestamp is seconds since January 1, 1970
    const FILETIME_UNIX_DIFF: u64 = 11644473600; // seconds between 1601 and 1970

    let seconds = filetime / 10_000_000 - FILETIME_UNIX_DIFF;
    let nanos = ((filetime % 10_000_000) * 100) as u32;

    DateTime::<Utc>::from_timestamp(seconds as i64, nanos)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
}

fn format_usn_reason(reason: u32) -> String {
    let mut reasons = Vec::new();

    if reason & 0x00000001 != 0 { reasons.push("DATA_OVERWRITE"); }
    if reason & 0x00000002 != 0 { reasons.push("DATA_EXTEND"); }
    if reason & 0x00000004 != 0 { reasons.push("DATA_TRUNCATION"); }
    if reason & 0x00000010 != 0 { reasons.push("NAMED_DATA_OVERWRITE"); }
    if reason & 0x00000020 != 0 { reasons.push("NAMED_DATA_EXTEND"); }
    if reason & 0x00000040 != 0 { reasons.push("NAMED_DATA_TRUNCATION"); }
    if reason & 0x00000100 != 0 { reasons.push("FILE_CREATE"); }
    if reason & 0x00000200 != 0 { reasons.push("FILE_DELETE"); }
    if reason & 0x00000400 != 0 { reasons.push("EA_CHANGE"); }
    if reason & 0x00000800 != 0 { reasons.push("SECURITY_CHANGE"); }
    if reason & 0x00001000 != 0 { reasons.push("RENAME_OLD_NAME"); }
    if reason & 0x00002000 != 0 { reasons.push("RENAME_NEW_NAME"); }
    if reason & 0x00004000 != 0 { reasons.push("INDEXABLE_CHANGE"); }
    if reason & 0x00008000 != 0 { reasons.push("BASIC_INFO_CHANGE"); }
    if reason & 0x00010000 != 0 { reasons.push("HARD_LINK_CHANGE"); }
    if reason & 0x00020000 != 0 { reasons.push("COMPRESSION_CHANGE"); }
    if reason & 0x00040000 != 0 { reasons.push("ENCRYPTION_CHANGE"); }
    if reason & 0x00080000 != 0 { reasons.push("OBJECT_ID_CHANGE"); }
    if reason & 0x00100000 != 0 { reasons.push("REPARSE_POINT_CHANGE"); }
    if reason & 0x00200000 != 0 { reasons.push("STREAM_CHANGE"); }
    if reason & 0x80000000 != 0 { reasons.push("CLOSE"); }

    if reasons.is_empty() {
        format!("UNKNOWN(0x{:08x})", reason)
    } else {
        reasons.join(" | ")
    }
}

// Helper trait for UTF-16LE string conversion
fn string_from_utf16le(bytes: &[u8]) -> Result<String, std::string::FromUtf16Error> {
    let utf16_chars: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&utf16_chars)
}