use super::types::{BootSector, ParseError, ParseResult};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

pub struct BootParser;

impl BootParser {
    pub fn parse(data: &[u8]) -> ParseResult<BootSector> {
        if data.len() < 512 {
            return Err(ParseError {
                message: "Boot sector data too small".to_string(),
                offset: None,
            });
        }

        let mut cursor = Cursor::new(data);

        // Read BPB (BIOS Parameter Block)
        cursor.set_position(11); // Skip jump instruction and OEM ID
        let bytes_per_sector = cursor.read_u16::<LittleEndian>().unwrap();
        let sectors_per_cluster = cursor.read_u8().unwrap();
        let _reserved_sectors = cursor.read_u16::<LittleEndian>().unwrap();
        let _fat_count = cursor.read_u8().unwrap();
        let _root_entries = cursor.read_u16::<LittleEndian>().unwrap();
        let _total_sectors_16 = cursor.read_u16::<LittleEndian>().unwrap();
        let _media_descriptor = cursor.read_u8().unwrap();
        let _sectors_per_fat = cursor.read_u16::<LittleEndian>().unwrap();
        let _sectors_per_track = cursor.read_u16::<LittleEndian>().unwrap();
        let _heads = cursor.read_u16::<LittleEndian>().unwrap();
        let _hidden_sectors = cursor.read_u32::<LittleEndian>().unwrap();
        let _total_sectors_32 = cursor.read_u32::<LittleEndian>().unwrap();

        // NTFS-specific fields
        cursor.set_position(40);
        let total_sectors = cursor.read_u64::<LittleEndian>().unwrap();
        let mft_start_cluster = cursor.read_u64::<LittleEndian>().unwrap();
        let mft_mirror_start_cluster = cursor.read_u64::<LittleEndian>().unwrap();
        let clusters_per_mft_record = cursor.read_i8().unwrap();
        cursor.read_u8().unwrap(); // Reserved
        cursor.read_u8().unwrap(); // Reserved
        cursor.read_u8().unwrap(); // Reserved
        let clusters_per_index_buffer = cursor.read_i8().unwrap();
        cursor.read_u8().unwrap(); // Reserved
        cursor.read_u8().unwrap(); // Reserved
        cursor.read_u8().unwrap(); // Reserved
        let volume_serial_number = cursor.read_u64::<LittleEndian>().unwrap();

        // Read OEM ID
        cursor.set_position(3);
        let mut oem_bytes = [0u8; 8];
        cursor.read_exact(&mut oem_bytes).unwrap();
        let oem_id = String::from_utf8_lossy(&oem_bytes).trim_end_matches('\0').to_string();

        Ok(BootSector {
            bytes_per_sector,
            sectors_per_cluster,
            total_sectors,
            mft_start_cluster,
            mft_mirror_start_cluster,
            clusters_per_mft_record,
            clusters_per_index_buffer,
            volume_serial_number,
            oem_id,
            volume_label: String::new(), // Volume label is typically in MFT, not boot sector
        })
    }
}