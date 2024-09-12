use super::errors::CacheError;
use super::{backup::Backup, object::Object};

use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct CacheEntry {
    object: Object,
    last_accessed: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CacheConfig {
    pub max_size: usize,
    pub ttl: Duration,
    pub eviction_strategy: EvictionStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1024 * 1024 * 1024,
            ttl: Duration::from_secs(300),
            eviction_strategy: EvictionStrategy::LeastRecentlyUsed,
        }
    }
}

impl CacheConfig {
    pub fn new(max_size: usize, ttl: Duration, eviction_strategy: EvictionStrategy) -> Self {
        Self {
            max_size,
            ttl,
            eviction_strategy,
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_eviction_strategy(mut self, eviction_strategy: EvictionStrategy) -> Self {
        self.eviction_strategy = eviction_strategy;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum EvictionStrategy {
    LeastRecentlyUsed,
    FirstInFirstOut,
}

pub struct CacheManager {
    cache: DashMap<Vec<u8>, CacheEntry>,
    config: DashMap<(), CacheConfig>,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let config_map = DashMap::new();
        config_map.insert((), config);
        Self {
            cache: DashMap::new(),
            config: config_map,
        }
    }

    pub fn get(&self, key: &Vec<u8>) -> Option<Object> {
        self.cache.get_mut(key).map(|mut entry| {
            entry.value_mut().last_accessed = Instant::now();
            entry.object.clone()
        })
    }

    pub fn insert(&self, k: Vec<u8>, v: Object) -> Result<(), CacheError> {
        let entry = CacheEntry {
            object: v,
            last_accessed: Instant::now(),
        };

        if self.cache.len() >= self.config.get(&()).unwrap().max_size {
            self.evict()?;
        }

        self.cache.insert(k, entry);
        Ok(())
    }

    pub fn remove(&self, k: &Vec<u8>) -> Option<Object> {
        self.cache.remove(k).map(|(_, entry)| entry.object)
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn evict_expired(&self) {
        let now = Instant::now();
        let ttl = self.config.get(&()).unwrap().ttl;
        self.cache
            .retain(|_, entry| now.duration_since(entry.last_accessed) <= ttl);
    }

    fn evict(&self) -> Result<(), CacheError> {
        match self.config.get(&()).unwrap().eviction_strategy {
            EvictionStrategy::LeastRecentlyUsed => self.evict_lru(),
            EvictionStrategy::FirstInFirstOut => self.evict_fifo(),
        }
    }

    fn evict_lru(&self) -> Result<(), CacheError> {
        if let Some(entry) = self
            .cache
            .iter()
            .min_by_key(|entry| entry.value().last_accessed)
        {
            self.cache.remove(entry.key());
            Ok(())
        } else {
            Err(CacheError::InsertionError(
                "Failed to evict LRU item".to_string(),
            ))
        }
    }

    fn evict_fifo(&self) -> Result<(), CacheError> {
        if let Some(entry) = self.cache.iter().next() {
            let key = entry.key().clone();
            self.cache.remove(&key);
            Ok(())
        } else {
            Err(CacheError::InsertionError(
                "Failed to evict FIFO item".to_string(),
            ))
        }
    }

    pub fn load_from_backup(&self, backup: &Backup) -> Result<(), CacheError> {
        for item in backup.get_objects() {
            let (k, v) = item.pair();
            self.insert(k.clone(), v.clone())?;
        }
        Ok(())
    }

    pub fn get_size(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn contains_key(&self, k: &Vec<u8>) -> bool {
        self.cache.contains_key(k)
    }

    pub fn update_config(&self, config: CacheConfig) {
        self.config.insert((), config);
    }
}
