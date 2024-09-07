use zewos_dir::dir::Directory;
use zewos_storage::{errors::StorageError, CacheConfig, StorageIndex};

pub struct Storage {
    index: StorageIndex,
    dir: Directory,
}

impl Storage {
    pub fn init(origin: &str) -> Result<Self, StorageError> {
        //TODO: If path exists than load it.
        let path = std::path::Path::new(origin).join(".zewos");
        let config = CacheConfig::default();
        let index = StorageIndex::new(config)?;
        let dir = Directory::new(path.to_str().unwrap());

        Ok(Self { index, dir })
    }

    pub fn save(&self) -> std::io::Result<()> {
        let (data, metadata) = self.index.serialize_backup(Some(3)).unwrap();
        self.dir.objs_file().write(&data).unwrap();
        self.dir.metadata_file().write(&metadata).unwrap();
        Ok(())
    }

    pub fn load(origin: &str) -> Result<Self, StorageError> {
        let dir = Directory::new(origin);
        let data = dir.objs_file().read()?;
        let metadata = dir.metadata_file().read()?;
        let index = StorageIndex::deserialize_backup(data, metadata)?;
        Ok(Self { index, dir })
    }

    pub fn get(&self, key: &Vec<u8>) -> Result<Vec<u8>, StorageError> {
        self.index.get(key)
    }

    pub fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        let result = self.index.insert(key, value);
        self.save()?;
        result
    }

    pub fn remove(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        let result = self.index.remove(key);
        self.save()?;
        result
    }

    pub fn contains_key(&self, key: &Vec<u8>) -> Result<bool, StorageError> {
        self.index.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    pub fn get_all_keys(&self) -> Result<Vec<Vec<u8>>, StorageError> {
        self.index.get_all_keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_init() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_load() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();

        let key = b"key1".to_vec();
        let value = vec![1, 2, 3];
        storage.insert(key.clone(), value.clone()).unwrap();

        let loaded_storage = Storage::load(origin).unwrap();

        assert_eq!(loaded_storage.get(&key).unwrap(), value);
    }

    #[test]
    fn test_storage_insert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();

        let key = b"key2".to_vec();
        let value = vec![4, 5, 6];
        storage.insert(key.clone(), value.clone()).unwrap();

        assert_eq!(storage.get(&key).unwrap(), value);
    }

    #[test]
    fn test_storage_remove() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();

        let key = b"key3".to_vec();
        let value = vec![7, 8, 9];
        storage.insert(key.clone(), value.clone()).unwrap();

        assert_eq!(storage.remove(&key).unwrap(), Some(value));
        assert!(storage.get(&key).is_err());
    }

    #[test]
    fn test_storage_contains_key() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();

        let key = b"key4".to_vec();
        let value = vec![10, 11, 12];
        storage.insert(key.clone(), value).unwrap();

        assert!(storage.contains_key(&key).unwrap());
        assert!(!storage.contains_key(&b"non_existent_key".to_vec()).unwrap());
    }

    #[test]
    fn test_storage_len_and_is_empty() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();

        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);

        storage.insert(b"key5".to_vec(), vec![13, 14, 15]).unwrap();

        assert!(!storage.is_empty());
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_storage_get_all_keys() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let storage = Storage::init(origin).unwrap();

        let keys = vec![b"key6".to_vec(), b"key7".to_vec(), b"key8".to_vec()];
        for (i, key) in keys.iter().enumerate() {
            storage.insert(key.clone(), vec![i as u8]).unwrap();
        }

        let all_keys = storage.get_all_keys().unwrap();
        assert_eq!(all_keys.len(), 3);
        for key in keys {
            assert!(all_keys.contains(&key));
        }
    }
}
