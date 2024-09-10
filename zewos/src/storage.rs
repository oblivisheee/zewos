use zewos_dir::dir::Directory;
use zewos_dir::logs::LogsManager;
use zewos_storage::{errors::StorageError, CacheConfig, StorageIndex};

pub struct Storage {
    index: StorageIndex,
    dir: Directory,
    logger: LogsManager,
}

impl Storage {
    pub fn init(origin: &str) -> Result<Self, StorageError> {
        let path = std::path::Path::new(origin).join(".zewos");
        if path.exists() {
            return Self::load(path.to_str().unwrap());
        }
        let config = CacheConfig::default();
        let index = StorageIndex::new(config)?;
        let dir = Directory::new(path.to_str().unwrap());
        let mut logger = dir.clone().logger();
        logger.start_session()?;
        logger.add_log("zewos_init", "init", "first_initialization")?;
        Ok(Self {
            index,
            dir: dir.clone(),
            logger,
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        let (data, metadata) = self.index.serialize_backup(Some(3)).unwrap();
        self.dir.objs_file().write(&data).unwrap();
        self.dir.backup_metadata_file().write(&metadata).unwrap();
        self.logger
            .add_log("zewos_storage", "save", "backup_created")?;
        Ok(())
    }

    pub fn load(origin: &str) -> Result<Self, StorageError> {
        let dir = Directory::new(origin);
        let data = dir.objs_file().read()?;
        let metadata = dir.backup_metadata_file().read()?;
        let index = StorageIndex::deserialize_backup(data, metadata)?;
        let mut logger = dir.clone().logger();
        logger.start_session()?;
        logger.add_log("zewos_init", "load", "storage_loaded")?;
        Ok(Self { index, dir, logger })
    }

    pub fn get(&mut self, key: &Vec<u8>) -> Result<Vec<u8>, StorageError> {
        self.logger.add_log(
            "zewos_request",
            "get",
            format!("key-\"{}\"", String::from_utf8(key.clone()).unwrap()).as_str(),
        )?;
        let result = self.index.get(key);
        match &result {
            Ok(_) => self.logger.add_log("zewos_request", "get", "success")?,
            Err(_) => self.logger.add_log("zewos_request", "get", "failed")?,
        }
        result
    }

    pub fn insert(
        &mut self,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        self.logger.add_log(
            "zewos_request",
            "insert",
            format!("key-\"{}\"", String::from_utf8(key.clone()).unwrap()).as_str(),
        )?;
        let result = self.index.insert(key, value);
        match &result {
            Ok(_) => self.logger.add_log("zewos_request", "insert", "success")?,
            Err(_) => self.logger.add_log("zewos_request", "insert", "failed")?,
        }
        self.save()?;
        result
    }

    pub fn remove(&mut self, key: &Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        self.logger.add_log(
            "zewos_request",
            "remove",
            format!("key-\"{}\"", String::from_utf8(key.clone()).unwrap()).as_str(),
        )?;
        let result = self.index.remove(key);
        match &result {
            Ok(_) => self.logger.add_log("zewos_request", "remove", "success")?,
            Err(_) => self.logger.add_log("zewos_request", "remove", "failed")?,
        }
        self.save()?;
        result
    }

    pub fn contains_key(&mut self, key: &Vec<u8>) -> Result<bool, StorageError> {
        self.logger.add_log(
            "zewos_request",
            "contains_key",
            format!("key-\"{}\"", String::from_utf8(key.clone()).unwrap()).as_str(),
        )?;
        self.index.contains_key(key)
    }

    pub fn len(&mut self) -> usize {
        self.logger
            .add_log("zewos_request", "len", "queried_length")
            .unwrap_or(());
        self.index.len()
    }

    pub fn is_empty(&mut self) -> bool {
        self.logger
            .add_log("zewos_request", "is_empty", "checked_emptiness")
            .unwrap_or(());
        self.index.is_empty()
    }

    pub fn get_all_keys(&mut self) -> Result<Vec<Vec<u8>>, StorageError> {
        self.logger
            .add_log("zewos_request", "get_all_keys", "retrieved_all_keys")?;
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
        let mut storage = Storage::init(origin).unwrap();
        assert!(storage.is_empty());
    }

    #[test]
    fn test_storage_load() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let mut storage = Storage::init(origin).unwrap();

        let key = b"key1".to_vec();
        let value = vec![1, 2, 3];
        storage.insert(key.clone(), value.clone()).unwrap();

        let mut loaded_storage = Storage::load(origin).unwrap();

        assert_eq!(loaded_storage.get(&key).unwrap(), value);
    }

    #[test]
    fn test_storage_insert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let mut storage = Storage::init(origin).unwrap();

        let key = b"key2".to_vec();
        let value = vec![4, 5, 6];
        storage.insert(key.clone(), value.clone()).unwrap();

        assert_eq!(storage.get(&key).unwrap(), value);
    }

    #[test]
    fn test_storage_remove() {
        let temp_dir = TempDir::new().unwrap();
        let origin = temp_dir.path().to_str().unwrap();
        let mut storage = Storage::init(origin).unwrap();

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
        let mut storage = Storage::init(origin).unwrap();

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
        let mut storage = Storage::init(origin).unwrap();

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
        let mut storage = Storage::init(origin).unwrap();

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
