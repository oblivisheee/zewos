use dashmap::DashMap;

use super::errors::BackupError;
use super::hash::Sha256;
use super::{
    compression::{compress_bytes, decompress_bytes},
    object::Object,
};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
pub use zewos_core::metadata::BackupMetadata;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BackupConfig {
    compression_level: Option<usize>,
}

impl BackupConfig {
    pub fn new() -> Self {
        Self {
            compression_level: Some(3),
        }
    }

    pub fn with_compression_level(mut self, level: usize) -> Self {
        self.compression_level = Some(level);
        self
    }
}
impl Default for BackupConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Backup {
    metadata: BackupMetadata,
    objects: Box<DashMap<Vec<u8>, Object>>,
    hash: Sha256,

    config: BackupConfig,
}

impl Backup {
    pub fn new() -> Self {
        Self::with_config(BackupConfig::new())
    }

    pub fn with_config(config: BackupConfig) -> Self {
        let metadata = BackupMetadata::new(0, config.compression_level);

        Self {
            metadata,
            objects: Box::new(DashMap::new()),
            hash: Sha256::new(&[]),

            config,
        }
    }

    pub fn insert(&mut self, k: Vec<u8>, v: Object) -> Result<Option<Object>, BackupError> {
        let result = self.objects.insert(k, v.clone());
        self.metadata.object_count += 1;
        self.metadata.total_size += v.len();
        self.metadata.last_modified = chrono::Utc::now();
        self.update_hash()?;
        Ok(result)
    }

    pub fn get(&self, k: &[u8]) -> Option<Object> {
        self.objects.get(k).map(|ref_obj| ref_obj.clone())
    }

    pub fn get_objects(&self) -> &DashMap<Vec<u8>, Object> {
        &self.objects
    }

    pub fn remove(&mut self, k: &[u8]) -> Result<Option<Object>, BackupError> {
        let removed = self.objects.remove(k);
        if let Some((_, obj)) = removed.clone() {
            self.metadata.object_count -= 1;
            self.metadata.total_size -= obj.len();
            self.metadata.last_modified = chrono::Utc::now();
            self.update_hash()?;
        }
        Ok(removed.map(|(_, obj)| obj))
    }
    pub(crate) fn update(&mut self, backup: Backup) {
        self.metadata = backup.metadata;
        self.objects = backup.objects;
        self.hash = backup.hash;
    }

    pub fn serialize(&self) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), BackupError> {
        let (compressed, metadata_json, config_json) =
            self.serialize_custom(self.config.compression_level)?;
        Ok((compressed, metadata_json, config_json))
    }

    pub fn deserialize(metadata: &[u8], data: &[u8], config: &[u8]) -> Result<Self, BackupError> {
        let metadata: BackupMetadata = serde_json::from_slice(metadata)?;
        let config: BackupConfig = serde_json::from_slice(config)?;
        let decompressed = decompress_bytes(data)?;
        let objects: Box<DashMap<Vec<u8>, Object>> = bincode::deserialize(&decompressed)?;
        let mut backup = Self {
            metadata,
            objects,
            hash: Sha256::new(&[]),

            config,
        };
        backup.update_hash()?;
        Ok(backup)
    }
    pub fn serialize_custom(
        &self,
        level: Option<usize>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), BackupError> {
        let metadata_json = serde_json::to_vec(&self.metadata)?;
        let config_json = serde_json::to_vec(&self.config)?;
        let object_data = bincode::serialize(&self.objects)?;

        let compressed = compress_bytes(&object_data, level.unwrap_or(3).try_into().unwrap())?;

        Ok((compressed, metadata_json, config_json))
    }

    pub fn get_metadata(&self) -> Result<BackupMetadata, BackupError> {
        Ok(self.metadata.clone())
    }

    fn update_hash(&mut self) -> Result<(), BackupError> {
        self.hash = Sha256::new(&bincode::serialize(&self.objects)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_backup_new() {
        let backup = Backup::new();
        assert_eq!(backup.metadata.object_count, 0);
        assert_eq!(backup.metadata.total_size, 0);
    }

    #[test]
    fn test_backup_with_config() {
        let config = BackupConfig::new().with_compression_level(5);

        let backup = Backup::with_config(config);
        assert_eq!(backup.config.compression_level, Some(5));
    }

    #[test]
    fn test_backup_insert() {
        let mut backup = Backup::new();
        let obj = Object::new(vec![1, 2, 3]).unwrap();
        let result = backup.insert(vec![0], obj.clone());
        assert!(result.is_ok());
        assert_eq!(backup.metadata.object_count, 1);
        assert_eq!(backup.metadata.total_size, 3);
        assert_eq!(backup.get(&[0]), Some(obj));
    }

    #[test]
    fn test_backup_remove() {
        let mut backup = Backup::new();
        let obj = Object::new(vec![1, 2, 3]).unwrap();
        backup.insert(vec![0], obj.clone()).unwrap();
        let result = backup.remove(&[0]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(obj));
        assert_eq!(backup.metadata.object_count, 0);
        assert_eq!(backup.metadata.total_size, 0);
    }

    #[test]
    fn test_backup_serialize_deserialize() {
        let mut backup = Backup::new();
        backup
            .insert(vec![0], Object::new(vec![1, 2, 3]).unwrap())
            .unwrap();
        backup
            .insert(vec![1], Object::new(vec![4, 5, 6]).unwrap())
            .unwrap();

        let (data, metadata, config) = backup.serialize_custom(None).unwrap();
        let deserialized = Backup::deserialize(&metadata, &data, &config).unwrap();

        assert_eq!(
            backup.metadata.object_count,
            deserialized.metadata.object_count
        );
        assert_eq!(backup.metadata.total_size, deserialized.metadata.total_size);

        // Compare objects without considering metadata
        let original_obj = backup.get(&[0]).unwrap();
        let deserialized_obj = deserialized.get(&[0]).unwrap();
        assert_eq!(original_obj.to_bytes(), deserialized_obj.to_bytes());

        let original_obj = backup.get(&[1]).unwrap();
        let deserialized_obj = deserialized.get(&[1]).unwrap();
        assert_eq!(original_obj.to_bytes(), deserialized_obj.to_bytes());
    }

    #[test]
    fn test_backup_metadata() {
        let mut backup = Backup::new();
        let creation_time = backup.metadata.creation_date;
        std::thread::sleep(Duration::from_secs(1));
        backup
            .insert(vec![0], Object::new(vec![1, 2, 3]).unwrap())
            .unwrap();

        let metadata = backup.get_metadata().unwrap();
        assert_eq!(metadata.creation_date, creation_time);
        assert!(metadata.last_modified > creation_time);
        assert_eq!(metadata.object_count, 1);
        assert_eq!(metadata.total_size, 3);
    }
}
