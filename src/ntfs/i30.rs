use super::types::{IndexEntry, ParseError, ParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, Utc};
use std::io::{Cursor, Read};

pub struct I30Parser {
    data: Vec<u8>,
    entries: Vec<IndexEntry>,
}

impl I30Parser {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            entries: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<()> {
        let mut cursor = Cursor::new(&self.data);

        // Parse INDX header
        let signature = cursor.read_u32::<LittleEndian>()
            .map_err(|_| ParseError {
                message: "Failed to read INDX signature".to_string(),
                offset: Some(0),
            })?;

        if signature != 0x58444e49 { // "INDX"
            return Err(ParseError {
                message: "Invalid INDX signature".to_string(),
                offset: Some(0),
            });
        }

        let _fixup_offset = cursor.read_u16::<LittleEndian>().unwrap();
        let _fixup_count = cursor.read_u16::<LittleEndian>().unwrap();
        let _lsn = cursor.read_u64::<LittleEndian>().unwrap();
        let _vcn = cursor.read_u64::<LittleEndian>().unwrap();

        // Parse index header
        let entries_offset = cursor.read_u32::<LittleEndian>().unwrap();
        let _total_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _allocated_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _flags = cursor.read_u32::<LittleEndian>().unwrap();

        // Jump to entries
        cursor.set_position(24 + entries_offset as u64);

        while (cursor.position() as usize) < self.data.len() {
            match self.parse_entry(&mut cursor) {
                Ok(Some(entry)) => self.entries.push(entry),
                Ok(None) => break, // End of entries
                Err(e) => {
                    log::warn!("Failed to parse I30 entry at offset 0x{:x}: {}", cursor.position(), e);
                    break;
                }
            }
        }

        log::info!("Parsed {} I30 index entries", self.entries.len());
        Ok(())
    }

    fn parse_entry(&self, cursor: &mut Cursor<&Vec<u8>>) -> ParseResult<Option<IndexEntry>> {
        let start_pos = cursor.position();

        if start_pos + 16 > self.data.len() as u64 {
            return Ok(None); // Not enough data for index entry header
        }

        let file_reference = cursor.read_u64::<LittleEndian>()
            .map_err(|_| ParseError {
                message: "Failed to read file reference".to_string(),
                offset: Some(start_pos),
            })?;

        let entry_number = (file_reference & 0xFFFFFFFFFFFF) as u32;
        let sequence_number = (file_reference >> 48) as u16;

        let entry_length = cursor.read_u16::<LittleEndian>().unwrap();
        let filename_length = cursor.read_u16::<LittleEndian>().unwrap();
        let flags = cursor.read_u32::<LittleEndian>().unwrap();

        if entry_length == 0 || (flags & 0x02) != 0 {
            return Ok(None); // End entry or invalid entry
        }

        // Parse filename attribute
        let parent_file_reference = cursor.read_u64::<LittleEndian>().unwrap();
        let parent_entry_number = (parent_file_reference & 0xFFFFFFFFFFFF) as u32;
        let parent_sequence_number = (parent_file_reference >> 48) as u16;

        let created = cursor.read_u64::<LittleEndian>().unwrap();
        let modified = cursor.read_u64::<LittleEndian>().unwrap();
        let _record_changed = cursor.read_u64::<LittleEndian>().unwrap();
        let accessed = cursor.read_u64::<LittleEndian>().unwrap();

        let _allocated_size = cursor.read_u64::<LittleEndian>().unwrap();
        let file_size = cursor.read_u64::<LittleEndian>().unwrap();
        let attributes = cursor.read_u32::<LittleEndian>().unwrap();
        let _reparse_value = cursor.read_u32::<LittleEndian>().unwrap();

        let name_length = cursor.read_u8().unwrap();
        let _name_type = cursor.read_u8().unwrap();

        // Read filename (UTF-16)
        let mut name_bytes = vec![0u8; (name_length as usize) * 2];
        cursor.read_exact(&mut name_bytes)
            .map_err(|_| ParseError {
                message: "Failed to read filename".to_string(),
                offset: Some(start_pos),
            })?;

        let file_name = string_from_utf16le(&name_bytes)
            .unwrap_or_else(|_| String::from("INVALID_NAME"));

        let entry = IndexEntry {
            entry_number,
            sequence_number,
            parent_entry_number,
            parent_sequence_number,
            file_name,
            full_path: String::new(), // Will be resolved later
            file_size,
            is_directory: (attributes & 0x10) != 0,
            created: windows_filetime_to_datetime(created),
            modified: windows_filetime_to_datetime(modified),
            accessed: windows_filetime_to_datetime(accessed),
            attributes,
        };

        // Move to next entry
        cursor.set_position(start_pos + entry_length as u64);

        Ok(Some(entry))
    }

    pub fn get_entries(&self) -> &[IndexEntry] {
        &self.entries
    }
}

fn windows_filetime_to_datetime(filetime: u64) -> DateTime<Utc> {
    const FILETIME_UNIX_DIFF: u64 = 11644473600;
    let seconds = filetime / 10_000_000 - FILETIME_UNIX_DIFF;
    let nanos = ((filetime % 10_000_000) * 100) as u32;

    DateTime::<Utc>::from_timestamp(seconds as i64, nanos)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
}

fn string_from_utf16le(bytes: &[u8]) -> Result<String, std::string::FromUtf16Error> {
    let utf16_chars: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&utf16_chars)
}