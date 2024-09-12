use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub file_size: u64,
    pub file_path: PathBuf,
    pub backup_info: BackupMetadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupMetadata {
    pub creation_date: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub total_size: usize,
    pub object_count: u64,
    pub compression_level: Option<usize>,
}

pub fn calculate_total_backup_size(metadatas: &[Metadata]) -> usize {
    metadatas.iter().map(|m| m.backup_info.total_size).sum()
}

pub fn find_largest_backup(metadatas: &[Metadata]) -> Option<&Metadata> {
    metadatas.iter().max_by_key(|m| m.backup_info.total_size)
}

pub fn get_average_compression_ratio(metadatas: &[Metadata]) -> f32 {
    if metadatas.is_empty() {
        return 0.0;
    }
    let total_ratio: f32 = metadatas
        .iter()
        .filter_map(|m| m.backup_info.compression_level)
        .map(|level| level as f32)
        .sum();
    total_ratio / metadatas.len() as f32
}

impl BackupMetadata {
    pub fn new(total_size: usize, compression_level: Option<usize>) -> Self {
        BackupMetadata {
            creation_date: Utc::now(),
            total_size,
            compression_level,
            object_count: 0,
            last_modified: Utc::now(),
        }
    }

    pub fn update_compression_ratio(&mut self, original_size: u64) {
        if original_size > 0 {
            self.compression_level =
                Some((self.total_size as f64 / original_size as f64 * 100.0) as usize);
        }
    }
}

impl Default for BackupMetadata {
    fn default() -> Self {
        BackupMetadata {
            creation_date: Utc::now(),
            total_size: 0,
            compression_level: Some(3),
            object_count: 0,
            last_modified: Utc::now(),
        }
    }
}

impl Metadata {
    pub fn new(file_path: PathBuf, file_size: u64) -> Self {
        let now = Utc::now();
        Metadata {
            created_at: now,
            last_modified: now,
            file_size,
            file_path,
            backup_info: BackupMetadata::default(),
        }
    }

    pub fn update_last_modified(&mut self) {
        self.last_modified = Utc::now();
    }

    pub fn is_recently_modified(&self, threshold: chrono::Duration) -> bool {
        Utc::now().signed_duration_since(self.last_modified) < threshold
    }
}
