use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupMetadata {
    pub creation_date: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub total_size: usize,
    pub object_count: u64,
    pub compression_level: Option<usize>,
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
