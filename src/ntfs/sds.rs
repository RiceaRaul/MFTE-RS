use super::types::{SecurityDescriptor, ParseError, ParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub struct SdsParser {
    data: Vec<u8>,
    descriptors: Vec<SecurityDescriptor>,
}

impl SdsParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            descriptors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> ParseResult<()> {
        let mut cursor = Cursor::new(&self.data);

        while (cursor.position() as usize) < self.data.len() {
            match self.parse_descriptor(&mut cursor) {
                Ok(Some(descriptor)) => self.descriptors.push(descriptor),
                Ok(None) => break, // End of valid descriptors
                Err(e) => {
                    log::warn!("Failed to parse SDS descriptor at offset 0x{:x}: {}", cursor.position(), e);
                    break;
                }
            }
        }

        log::info!("Parsed {} security descriptors", self.descriptors.len());
        Ok(())
    }

    fn parse_descriptor(&self, cursor: &mut Cursor<&Vec<u8>>) -> ParseResult<Option<SecurityDescriptor>> {
        let start_pos = cursor.position();

        if start_pos + 20 > self.data.len() as u64 {
            return Ok(None); // Not enough data for SDS header
        }

        let hash = cursor.read_u32::<LittleEndian>()
            .map_err(|_| ParseError {
                message: "Failed to read SDS hash".to_string(),
                offset: Some(start_pos),
            })?;

        let id = cursor.read_u32::<LittleEndian>().unwrap();
        let offset = cursor.read_u64::<LittleEndian>().unwrap();
        let length = cursor.read_u32::<LittleEndian>().unwrap();

        if length == 0 || length > 0x10000 { // Sanity check
            return Ok(None);
        }

        // Read the security descriptor data
        let mut descriptor_data = vec![0u8; length as usize];
        cursor.read_exact(&mut descriptor_data)
            .map_err(|_| ParseError {
                message: "Failed to read security descriptor data".to_string(),
                offset: Some(start_pos),
            })?;

        let descriptor = SecurityDescriptor {
            id,
            hash,
            offset: start_pos,
            length,
            descriptor: descriptor_data,
        };

        Ok(Some(descriptor))
    }

    pub fn get_descriptors(&self) -> &[SecurityDescriptor] {
        &self.descriptors
    }

    pub fn find_by_id(&self, id: u32) -> Option<&SecurityDescriptor> {
        self.descriptors.iter().find(|desc| desc.id == id)
    }
}