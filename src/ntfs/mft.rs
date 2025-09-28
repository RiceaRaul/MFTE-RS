use super::types::{MftRecord, ParseError, ParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

const MFT_RECORD_SIZE: usize = 1024;
const MFT_SIGNATURE: u32 = 0x454c4946; // "FILE"

pub struct MftParser {
    data: Vec<u8>,
    records: Vec<MftRecord>,
    entry_map: HashMap<u32, usize>, // Maps entry number to record index
}

impl MftParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            records: Vec::new(),
            entry_map: HashMap::new(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<()> {
        let mut offset = 0;

        // First pass: Parse all records and build entry map
        while offset + MFT_RECORD_SIZE <= self.data.len() {
            match self.parse_record(&self.data[offset..offset + MFT_RECORD_SIZE], offset) {
                Ok(Some(record)) => {
                    let entry_number = record.entry_number;
                    let record_index = self.records.len();
                    self.entry_map.insert(entry_number, record_index);
                    self.records.push(record);
                },
                Ok(None) => {}, // Skip invalid/unused records
                Err(e) => {
                    log::warn!("Failed to parse MFT record at offset 0x{:x}: {}", offset, e);
                }
            }
            offset += MFT_RECORD_SIZE;
        }

        // Second pass: Resolve parent paths
        self.resolve_parent_paths();

        log::info!("Parsed {} MFT records", self.records.len());
        Ok(())
    }

    fn parse_record(&self, data: &[u8], offset: usize) -> ParseResult<Option<MftRecord>> {
        let mut cursor = Cursor::new(data);

        // Read MFT record header
        let signature = cursor.read_u32::<LittleEndian>()
            .map_err(|_| ParseError {
                message: "Failed to read MFT signature".to_string(),
                offset: Some(offset as u64),
            })?;

        if signature != MFT_SIGNATURE {
            return Ok(None); // Not a valid MFT record
        }

        let _fixup_offset = cursor.read_u16::<LittleEndian>().unwrap();
        let _fixup_count = cursor.read_u16::<LittleEndian>().unwrap();
        let _lsn = cursor.read_u64::<LittleEndian>().unwrap();
        let sequence_number = cursor.read_u16::<LittleEndian>().unwrap();
        let _link_count = cursor.read_u16::<LittleEndian>().unwrap();
        let first_attribute_offset = cursor.read_u16::<LittleEndian>().unwrap();
        let flags = cursor.read_u16::<LittleEndian>().unwrap();
        let _used_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _allocated_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _base_record = cursor.read_u64::<LittleEndian>().unwrap();
        let _next_attribute_id = cursor.read_u16::<LittleEndian>().unwrap();

        let in_use = (flags & 0x01) != 0;
        let is_directory = (flags & 0x02) != 0;

        let entry_number = (offset / MFT_RECORD_SIZE) as u32;

        // Create a basic MFT record
        let mut record = MftRecord {
            entry_number,
            sequence_number,
            parent_entry_number: 0,
            parent_sequence_number: None,
            in_use,
            parent_path: String::new(),
            file_name: String::new(),
            extension: String::new(),
            is_directory,
            has_ads: false,
            is_ads: false,
            file_size: 0,
            created_0x10: None,
            created_0x30: None,
            last_modified_0x10: None,
            last_modified_0x30: None,
            last_record_change_0x10: None,
            last_record_change_0x30: None,
            last_access_0x10: None,
            last_access_0x30: None,
            update_sequence_number: 0,
            logfile_sequence_number: 0,
            security_id: 0,
            zone_id_contents: String::new(),
            si_flags: 0,
            object_id_file_droid: String::new(),
            reparse_target: String::new(),
            reference_count: 0,
            name_type: 0,
            logged_util_stream: String::new(),
        };

        // Parse attributes
        cursor.seek(SeekFrom::Start(first_attribute_offset as u64)).unwrap();
        self.parse_attributes(&mut cursor, &mut record)?;

        Ok(Some(record))
    }

    fn parse_attributes(&self, cursor: &mut Cursor<&[u8]>, record: &mut MftRecord) -> ParseResult<()> {
        loop {
            let pos = cursor.position();
            if pos + 4 > cursor.get_ref().len() as u64 {
                break;
            }

            let attr_type = cursor.read_u32::<LittleEndian>().unwrap();

            if attr_type == 0xFFFFFFFF {
                break; // End of attributes
            }

            let attr_length = cursor.read_u32::<LittleEndian>().unwrap();
            let _non_resident = cursor.read_u8().unwrap();
            let _name_length = cursor.read_u8().unwrap();
            let _name_offset = cursor.read_u16::<LittleEndian>().unwrap();
            let _flags = cursor.read_u16::<LittleEndian>().unwrap();
            let _attribute_id = cursor.read_u16::<LittleEndian>().unwrap();

            match attr_type {
                0x10 => self.parse_standard_info(cursor, record)?,
                0x30 => self.parse_file_name(cursor, record)?,
                0x80 => self.parse_data_attribute(cursor, record)?,
                _ => {
                    // Skip unknown attributes
                }
            }

            // Move to next attribute
            cursor.seek(SeekFrom::Start(pos + attr_length as u64)).unwrap();
        }

        Ok(())
    }

    fn parse_standard_info(&self, cursor: &mut Cursor<&[u8]>, record: &mut MftRecord) -> ParseResult<()> {
        let _resident_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _resident_offset = cursor.read_u16::<LittleEndian>().unwrap();
        cursor.seek(SeekFrom::Current(2)).unwrap(); // Reserved

        let created = cursor.read_u64::<LittleEndian>().unwrap();
        let modified = cursor.read_u64::<LittleEndian>().unwrap();
        let record_changed = cursor.read_u64::<LittleEndian>().unwrap();
        let accessed = cursor.read_u64::<LittleEndian>().unwrap();

        // Convert Windows FILETIME to DateTime<Utc>
        record.created_0x10 = Some(windows_filetime_to_datetime(created));
        record.last_modified_0x10 = Some(windows_filetime_to_datetime(modified));
        record.last_record_change_0x10 = Some(windows_filetime_to_datetime(record_changed));
        record.last_access_0x10 = Some(windows_filetime_to_datetime(accessed));

        record.si_flags = cursor.read_u32::<LittleEndian>().unwrap();

        Ok(())
    }

    fn parse_file_name(&self, cursor: &mut Cursor<&[u8]>, record: &mut MftRecord) -> ParseResult<()> {
        let _resident_size = cursor.read_u32::<LittleEndian>().unwrap();
        let _resident_offset = cursor.read_u16::<LittleEndian>().unwrap();
        cursor.seek(SeekFrom::Current(2)).unwrap(); // Reserved

        let parent_reference = cursor.read_u64::<LittleEndian>().unwrap();
        record.parent_entry_number = (parent_reference & 0xFFFFFFFFFFFF) as u32;
        record.parent_sequence_number = Some((parent_reference >> 48) as u16);

        let created = cursor.read_u64::<LittleEndian>().unwrap();
        let modified = cursor.read_u64::<LittleEndian>().unwrap();
        let record_changed = cursor.read_u64::<LittleEndian>().unwrap();
        let accessed = cursor.read_u64::<LittleEndian>().unwrap();

        // Set 0x30 timestamps
        record.created_0x30 = Some(windows_filetime_to_datetime(created));
        record.last_modified_0x30 = Some(windows_filetime_to_datetime(modified));
        record.last_record_change_0x30 = Some(windows_filetime_to_datetime(record_changed));
        record.last_access_0x30 = Some(windows_filetime_to_datetime(accessed));

        let _allocated_size = cursor.read_u64::<LittleEndian>().unwrap();
        let real_size = cursor.read_u64::<LittleEndian>().unwrap();
        record.file_size = real_size;

        let _flags = cursor.read_u32::<LittleEndian>().unwrap();
        let _reparse_value = cursor.read_u32::<LittleEndian>().unwrap();

        let name_length = cursor.read_u8().unwrap();
        record.name_type = cursor.read_u8().unwrap();

        // Read filename (UTF-16)
        let mut name_bytes = vec![0u8; (name_length as usize) * 2];
        cursor.read_exact(&mut name_bytes).unwrap();

        let name = string_from_utf16le(&name_bytes)
            .unwrap_or_else(|_| String::from("INVALID_NAME"));

        record.file_name = name.clone();

        // Extract extension
        if let Some(dot_pos) = name.rfind('.') {
            record.extension = name[dot_pos + 1..].to_string();
        }

        Ok(())
    }

    fn parse_data_attribute(&self, _cursor: &mut Cursor<&[u8]>, _record: &mut MftRecord) -> ParseResult<()> {
        // Data attribute parsing - for now just skip
        Ok(())
    }

    pub fn get_records(&self) -> &[MftRecord] {
        &self.records
    }

    fn resolve_parent_paths(&mut self) {
        // Clone the entry map for borrowing purposes
        let entry_map = self.entry_map.clone();

        for i in 0..self.records.len() {
            let parent_entry = self.records[i].parent_entry_number;
            if parent_entry == 5 {
                // Entry 5 is the root directory
                self.records[i].parent_path = String::new();
            } else if parent_entry != self.records[i].entry_number {
                // Build path by following parent chain
                let path = self.build_path(parent_entry, &entry_map, 0);
                self.records[i].parent_path = path;
            }
        }
    }

    fn build_path(&self, entry_number: u32, entry_map: &HashMap<u32, usize>, depth: usize) -> String {
        // Prevent infinite recursion
        if depth > 100 {
            return String::from("...[path too deep]");
        }

        if entry_number == 5 {
            return String::new(); // Root directory
        }

        if let Some(&record_index) = entry_map.get(&entry_number) {
            if record_index < self.records.len() {
                let record = &self.records[record_index];
                let parent_path = if record.parent_entry_number == 5 {
                    String::new()
                } else {
                    self.build_path(record.parent_entry_number, entry_map, depth + 1)
                };

                if parent_path.is_empty() {
                    record.file_name.clone()
                } else {
                    format!("{}/{}", parent_path, record.file_name)
                }
            } else {
                String::from("...[invalid index]")
            }
        } else {
            String::from("...[parent not found]")
        }
    }
}

// Helper function to convert UTF-16LE bytes to String
fn string_from_utf16le(bytes: &[u8]) -> Result<String, std::string::FromUtf16Error> {
    let utf16_chars: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&utf16_chars)
}

trait StringFromUtf16Le {
    fn from_utf16le(&self) -> Result<String, std::string::FromUtf16Error>;
}

impl StringFromUtf16Le for [u8] {
    fn from_utf16le(&self) -> Result<String, std::string::FromUtf16Error> {
        string_from_utf16le(self)
    }
}

fn windows_filetime_to_datetime(filetime: u64) -> DateTime<Utc> {
    // Windows FILETIME is 100-nanosecond intervals since January 1, 1601
    // Unix timestamp is seconds since January 1, 1970
    const FILETIME_UNIX_DIFF: u64 = 11644473600; // seconds between 1601 and 1970

    if filetime == 0 {
        return DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    }

    let seconds = filetime / 10_000_000;
    if seconds < FILETIME_UNIX_DIFF {
        return DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    }

    let unix_seconds = seconds - FILETIME_UNIX_DIFF;
    let nanos = ((filetime % 10_000_000) * 100) as u32;

    DateTime::<Utc>::from_timestamp(unix_seconds as i64, nanos)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
}