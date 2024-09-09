use dashmap::DashMap;

use super::errors::BackupError;
use super::hash::Sha256;
use super::{
    compression::{compress_bytes, decompress_bytes},
    object::Object,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupMetadata {
    creation_date: SystemTime,
    last_modified: SystemTime,
    object_count: usize,
    total_size: usize,
}

impl BackupMetadata {
    pub fn creation_date(&self) -> SystemTime {
        self.creation_date
    }

    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }

    pub fn object_count(&self) -> usize {
        self.object_count
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }
}

pub struct Backup {
    metadata: BackupMetadata,
    objects: Box<DashMap<Vec<u8>, Object>>,
    hash: Sha256,
    backup_path: Option<PathBuf>,
}

impl Backup {
    pub fn new() -> Self {
        let metadata = BackupMetadata {
            creation_date: SystemTime::now(),
            last_modified: SystemTime::now(),
            object_count: 0,
            total_size: 0,
        };

        Self {
            metadata,
            objects: Box::new(DashMap::new()),
            hash: Sha256::new(&[]),
            backup_path: None,
        }
    }

    pub fn insert(&mut self, k: Vec<u8>, v: Object) -> Result<Option<Object>, BackupError> {
        let result = self.objects.insert(k, v.clone());
        self.metadata.object_count += 1;
        self.metadata.total_size += v.len();
        self.metadata.last_modified = SystemTime::now();
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
            self.metadata.last_modified = SystemTime::now();
            self.update_hash()?;
        }
        Ok(removed.map(|(_, obj)| obj))
    }
    pub(crate) fn update(&mut self, backup: Backup) {
        self.metadata = backup.metadata;
        self.objects = backup.objects;
        self.hash = backup.hash;
    }

    pub fn serialize(&self, level: Option<usize>) -> Result<(Vec<u8>, Vec<u8>), BackupError> {
        let metadata_json = serde_json::to_vec(&self.metadata)?;

        let object_data = bincode::serialize(&self.objects)?;
        let level_compression = level.unwrap_or(3);
        let compressed = compress_bytes(&object_data, level_compression.try_into().unwrap())?;

        Ok((compressed, metadata_json))
    }

    pub fn deserialize(metadata: &[u8], data: &[u8]) -> Result<Self, BackupError> {
        let metadata: BackupMetadata = serde_json::from_slice(metadata)?;
        let decompressed = decompress_bytes(data)?;
        let objects: Box<DashMap<Vec<u8>, Object>> = bincode::deserialize(&decompressed)?;
        let mut backup = Self {
            metadata,
            objects,
            hash: Sha256::new(&[]),
            backup_path: None,
        };
        backup.update_hash()?;
        Ok(backup)
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

        let (data, metadata) = backup.serialize(None).unwrap();
        let deserialized = Backup::deserialize(&metadata, &data).unwrap();

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
        assert_eq!(metadata.creation_date(), creation_time);
        assert!(metadata.last_modified() > creation_time);
        assert_eq!(metadata.object_count(), 1);
        assert_eq!(metadata.total_size(), 3);
    }
}
