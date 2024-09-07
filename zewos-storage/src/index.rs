use super::errors::StorageError;
use super::{
    backup::{Backup, BackupMetadata},
    cache::{CacheConfig, CacheManager},
    object::Object,
};
use std::sync::{Arc, RwLock};

pub struct StorageIndex {
    backup: Arc<RwLock<Backup>>,
    cache: Arc<RwLock<CacheManager>>,
}

impl StorageIndex {
    pub fn new(cache_config: CacheConfig) -> Result<Self, StorageError> {
        let backup = Arc::new(RwLock::new(Backup::new()));
        let cache = Arc::new(RwLock::new(CacheManager::new(cache_config)));

        Ok(Self { backup, cache })
    }

    pub fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        let object = Object::new(value)?;
        let result = self
            .backup
            .write()
            .unwrap()
            .insert(key.clone(), object.clone())?;
        self.cache.write().unwrap().insert(key, object)?;

        Ok(result.map(|opt_obj| opt_obj.to_bytes()))
    }

    pub fn get(&self, key: &Vec<u8>) -> Result<Vec<u8>, StorageError> {
        if let Some(object) = self.cache.read().unwrap().get(key) {
            return Ok(object.to_bytes());
        }

        if let Some(object) = self.backup.read().unwrap().get(key) {
            let _ = self
                .cache
                .write()
                .unwrap()
                .insert(key.clone(), object.clone());
            return Ok(object.to_bytes());
        }

        Err(StorageError::KeyNotFound)
    }

    pub fn remove(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        let result = self.backup.write().unwrap().remove(key)?;
        self.cache.write().unwrap().remove(key);
        Ok(result.map(|obj| obj.to_bytes()))
    }

    pub fn serialize_backup(
        &self,
        compression_level: Option<usize>,
    ) -> Result<(Vec<u8>, Vec<u8>), StorageError> {
        let backup = self.backup.read().unwrap();
        let (data, metadata) = backup.serialize(compression_level)?;
        Ok((data, metadata))
    }

    pub fn deserialize_backup(
        data: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Result<StorageIndex, StorageError> {
        let backup = Backup::deserialize(&metadata, &data)?;
        let cache = CacheManager::new(CacheConfig::default());
        cache.load_from_backup(&backup)?;
        Ok(Self {
            backup: Arc::new(RwLock::new(backup)),
            cache: Arc::new(RwLock::new(cache)),
        })
    }

    pub fn sync_cache(&self) -> Result<(), StorageError> {
        let backup = self.backup.read().unwrap();
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        for entry in backup.get_objects().iter() {
            let (key, object) = entry.pair();
            cache.insert(key.clone(), object.clone());
        }
        Ok(())
    }

    pub fn update_backup(&self, data: Vec<u8>, metadata: Vec<u8>) -> Result<(), StorageError> {
        let backup = Backup::deserialize(&metadata, &data)?;
        self.backup.write().unwrap().update(backup);
        Ok(())
    }

    pub fn get_metadata(&self) -> Result<BackupMetadata, StorageError> {
        self.backup
            .read()
            .unwrap()
            .get_metadata()
            .map_err(|e| StorageError::from(e))
    }

    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }

    pub fn evict_expired_cache(&self) {
        self.cache.write().unwrap().evict_expired();
    }

    pub fn get_object_count(&self) -> Result<usize, StorageError> {
        Ok(self.backup.read().unwrap().get_objects().len())
    }

    pub fn get_total_size(&self) -> Result<usize, StorageError> {
        Ok(self
            .backup
            .read()
            .unwrap()
            .get_objects()
            .iter()
            .map(|entry| entry.value().size())
            .sum())
    }

    pub fn contains_key(&self, key: &Vec<u8>) -> Result<bool, StorageError> {
        Ok(self.backup.read().unwrap().get(key).is_some())
    }

    pub fn get_all_keys(&self) -> Result<Vec<Vec<u8>>, StorageError> {
        Ok(self
            .backup
            .read()
            .unwrap()
            .get_objects()
            .iter()
            .map(|entry| entry.key().to_vec())
            .collect())
    }

    pub fn clear(&mut self) -> Result<(), StorageError> {
        self.cache.write().unwrap().clear();
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.backup.read().unwrap().get_objects().is_empty()
    }

    pub fn len(&self) -> usize {
        self.backup.read().unwrap().get_objects().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        assert!(index.insert(value.clone(), key.clone()).unwrap().is_none());
        assert_eq!(index.get(&key).unwrap(), value);

        // Test overwriting existing value
        let new_value = b"new_test_value".to_vec();
        assert_eq!(
            index
                .insert(new_value.clone(), key.clone())
                .unwrap()
                .unwrap(),
            value
        );
        assert_eq!(index.get(&key).unwrap(), new_value);
    }

    #[test]
    fn test_remove() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        index.insert(value.clone(), key.clone()).unwrap();
        assert_eq!(index.remove(&key).unwrap().unwrap(), value);
        assert!(index.get(&key).is_err());

        // Test removing non-existent key
        assert!(index.remove(&key).unwrap().is_none());
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key1 = b"test_key1".to_vec();
        let value1 = b"test_value1".to_vec();
        let key2 = b"test_key2".to_vec();
        let value2 = b"test_value2".to_vec();

        index.insert(value1.clone(), key1.clone()).unwrap();
        index.insert(value2.clone(), key2.clone()).unwrap();

        let (data, metadata) = index.serialize_backup(None).unwrap();
        let loaded_index = StorageIndex::deserialize_backup(data, metadata).unwrap();

        assert_eq!(loaded_index.get(&key1).unwrap(), value1);
        assert_eq!(loaded_index.get(&key2).unwrap(), value2);
    }

    #[test]
    fn test_clear_cache() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        index.insert(value.clone(), key.clone()).unwrap();
        index.clear_cache();

        // The value should still be retrievable from backup
        assert_eq!(index.get(&key).unwrap(), value);
    }

    #[test]
    fn test_sync_cache() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        index.insert(value.clone(), key.clone()).unwrap();
        index.clear_cache();
        index.sync_cache().unwrap();

        // The value should be in the cache after sync
        assert_eq!(
            index.cache.read().unwrap().get(&key).unwrap().to_bytes(),
            value
        );
    }

    #[test]
    fn test_get_metadata() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        index.insert(value.clone(), key.clone()).unwrap();

        let metadata = index.get_metadata().unwrap();
        assert_eq!(metadata.object_count(), 1);
        assert_eq!(metadata.total_size(), value.len());
    }

    #[test]
    fn test_evict_expired_cache() {
        let mut config = CacheConfig::default();
        config.ttl = std::time::Duration::from_secs(1);
        let index = StorageIndex::new(config).unwrap();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        index.insert(value.clone(), key.clone()).unwrap();

        // Sleep to allow cache entry to expire
        std::thread::sleep(std::time::Duration::from_secs(2));

        index.evict_expired_cache();

        // The value should still be retrievable from backup
        assert_eq!(
            index.backup.read().unwrap().get(&key).unwrap().to_bytes(),
            value
        );

        // But it should not be in the cache
        assert!(index.cache.read().unwrap().get(&key).is_none());
    }

    #[test]
    fn test_new_methods() {
        let index = StorageIndex::new(CacheConfig::default()).unwrap();
        let key1 = b"test_key1".to_vec();
        let value1 = b"test_value1".to_vec();
        let key2 = b"test_key2".to_vec();
        let value2 = b"test_value2".to_vec();

        index.insert(value1.clone(), key1.clone()).unwrap();
        index.insert(value2.clone(), key2.clone()).unwrap();

        assert_eq!(index.get_object_count().unwrap(), 2);
        assert_eq!(index.get_total_size().unwrap(), value1.len() + value2.len());
        assert!(index.contains_key(&key1).unwrap());
        assert!(!index.contains_key(&b"non_existent_key".to_vec()).unwrap());

        let all_keys = index.get_all_keys().unwrap();
        assert_eq!(all_keys.len(), 2);
        assert!(all_keys.contains(&key1));
        assert!(all_keys.contains(&key2));
    }
}
