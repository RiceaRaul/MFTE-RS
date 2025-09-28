use crate::ntfs::types::*;
use std::io::{self, Write};

pub struct TableOutput;

impl TableOutput {
    pub fn print_mft_records(records: &[MftRecord], limit: Option<usize>) {
        let records_to_show = match limit {
            Some(n) => &records[..n.min(records.len())],
            None => records,
        };

        println!("{:<8} {:<6} {:<50} {:<10} {:<20} {:<20}",
                 "Entry", "Seq", "File Name", "Size", "Created", "Modified");
        println!("{}", "-".repeat(120));

        for record in records_to_show {
            let created = record.created_0x10
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let modified = record.last_modified_0x10
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let file_name = if record.file_name.len() > 48 {
                format!("{}...", &record.file_name[..45])
            } else {
                record.file_name.clone()
            };

            println!("{:<8} {:<6} {:<50} {:<10} {:<20} {:<20}",
                     record.entry_number,
                     record.sequence_number,
                     file_name,
                     record.file_size,
                     created,
                     modified);
        }

        if let Some(limit) = limit {
            if records.len() > limit {
                println!("\n... and {} more records", records.len() - limit);
            }
        }
    }

    pub fn print_usn_journal_entries(entries: &[UsnJournalEntry], limit: Option<usize>) {
        let entries_to_show = match limit {
            Some(n) => &entries[..n.min(entries.len())],
            None => entries,
        };

        println!("{:<8} {:<6} {:<40} {:<20} {:<30}",
                 "Entry", "Seq", "File Name", "Timestamp", "Reason");
        println!("{}", "-".repeat(110));

        for entry in entries_to_show {
            let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

            let file_name = if entry.file_name.len() > 38 {
                format!("{}...", &entry.file_name[..35])
            } else {
                entry.file_name.clone()
            };

            let reason = if entry.reason.len() > 28 {
                format!("{}...", &entry.reason[..25])
            } else {
                entry.reason.clone()
            };

            println!("{:<8} {:<6} {:<40} {:<20} {:<30}",
                     entry.entry_number,
                     entry.sequence_number,
                     file_name,
                     timestamp,
                     reason);
        }

        if let Some(limit) = limit {
            if entries.len() > limit {
                println!("\n... and {} more entries", entries.len() - limit);
            }
        }
    }

    pub fn print_boot_sector(boot: &BootSector) {
        println!("Boot Sector Information:");
        println!("{}", "-".repeat(50));
        println!("OEM ID:                    {}", boot.oem_id);
        println!("Bytes per Sector:          {}", boot.bytes_per_sector);
        println!("Sectors per Cluster:       {}", boot.sectors_per_cluster);
        println!("Total Sectors:             {}", boot.total_sectors);
        println!("MFT Start Cluster:         {}", boot.mft_start_cluster);
        println!("MFT Mirror Start Cluster:  {}", boot.mft_mirror_start_cluster);
        println!("Clusters per MFT Record:   {}", boot.clusters_per_mft_record);
        println!("Clusters per Index Buffer: {}", boot.clusters_per_index_buffer);
        println!("Volume Serial Number:      0x{:016X}", boot.volume_serial_number);

        if !boot.volume_label.is_empty() {
            println!("Volume Label:              {}", boot.volume_label);
        }
    }

    pub fn print_security_descriptors(descriptors: &[SecurityDescriptor], limit: Option<usize>) {
        let descriptors_to_show = match limit {
            Some(n) => &descriptors[..n.min(descriptors.len())],
            None => descriptors,
        };

        println!("{:<8} {:<12} {:<16} {:<8} {:<20}",
                 "ID", "Hash", "Offset", "Length", "Descriptor (hex)");
        println!("{}", "-".repeat(70));

        for desc in descriptors_to_show {
            let descriptor_preview = if desc.descriptor.len() > 16 {
                format!("{}...", hex::encode(&desc.descriptor[..16]))
            } else {
                hex::encode(&desc.descriptor)
            };

            println!("{:<8} {:<12} 0x{:<14X} {:<8} {}",
                     desc.id,
                     desc.hash,
                     desc.offset,
                     desc.length,
                     descriptor_preview);
        }

        if let Some(limit) = limit {
            if descriptors.len() > limit {
                println!("\n... and {} more descriptors", descriptors.len() - limit);
            }
        }
    }

    pub fn print_index_entries(entries: &[IndexEntry], limit: Option<usize>) {
        let entries_to_show = match limit {
            Some(n) => &entries[..n.min(entries.len())],
            None => entries,
        };

        println!("{:<8} {:<6} {:<40} {:<10} {:<20} {:<20}",
                 "Entry", "Seq", "File Name", "Size", "Created", "Modified");
        println!("{}", "-".repeat(110));

        for entry in entries_to_show {
            let created = entry.created.format("%Y-%m-%d %H:%M:%S").to_string();
            let modified = entry.modified.format("%Y-%m-%d %H:%M:%S").to_string();

            let file_name = if entry.file_name.len() > 38 {
                format!("{}...", &entry.file_name[..35])
            } else {
                entry.file_name.clone()
            };

            println!("{:<8} {:<6} {:<40} {:<10} {:<20} {:<20}",
                     entry.entry_number,
                     entry.sequence_number,
                     file_name,
                     entry.file_size,
                     created,
                     modified);
        }

        if let Some(limit) = limit {
            if entries.len() > limit {
                println!("\n... and {} more entries", entries.len() - limit);
            }
        }
    }

    pub fn print_summary(file_type: &str, record_count: usize, processing_time: u128) {
        println!("\nProcessing Summary:");
        println!("{}", "-".repeat(30));
        println!("File Type:         {}", file_type);
        println!("Records Processed: {}", record_count);
        println!("Processing Time:   {} ms", processing_time);
    }

    pub fn print_progress_bar(current: usize, total: usize, width: usize) {
        let progress = (current as f64 / total as f64 * width as f64) as usize;
        let percentage = (current as f64 / total as f64 * 100.0) as usize;

        print!("\r[{}{}] {}% ({}/{})",
               "=".repeat(progress),
               " ".repeat(width - progress),
               percentage,
               current,
               total);

        io::stdout().flush().unwrap();

        if current == total {
            println!(); // New line when complete
        }
    }
}