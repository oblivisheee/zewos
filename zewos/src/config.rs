use zewos_storage::{BackupConfig, CacheConfig};
#[derive(Clone, Copy)]
pub struct ZewosConfig {
    pub logging: bool,
    pub backup_config: BackupConfig,
    pub cache_config: CacheConfig,
}
impl ZewosConfig {
    pub fn new() -> Self {
        Self {
            logging: true,
            backup_config: BackupConfig::default(),
            cache_config: CacheConfig::default(),
        }
    }
    pub fn with_logging(mut self, logging: bool) -> Self {
        self.logging = logging;
        self
    }
    pub fn with_backup_config(mut self, backup_config: BackupConfig) -> Self {
        self.backup_config = backup_config;
        self
    }
    pub fn with_cache_config(mut self, cache_config: CacheConfig) -> Self {
        self.cache_config = cache_config;
        self
    }
}

impl Default for ZewosConfig {
    fn default() -> Self {
        Self::new()
    }
}
