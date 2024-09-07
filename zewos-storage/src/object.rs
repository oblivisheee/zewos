use super::errors::ObjectError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    size: usize,
    #[serde(with = "chrono::serde::ts_microseconds")]
    created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_microseconds")]
    last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Object {
    data: Vec<u8>,
    metadata: Metadata,
}

impl Metadata {
    pub fn new(name: String, size: usize) -> Result<Self, ObjectError> {
        if name.is_empty() {
            return Err(ObjectError::InvalidName("Name cannot be empty".to_string()));
        }
        if size == 0 {
            return Err(ObjectError::InvalidSize(size));
        }
        let now = Utc::now();
        Ok(Metadata {
            name,
            size,
            created_at: now,
            last_updated: now,
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn get_last_updated(&self) -> &DateTime<Utc> {
        &self.last_updated
    }

    pub fn update(&mut self) {
        self.last_updated = Utc::now();
    }
}

impl Object {
    pub fn new(data: Vec<u8>) -> Result<Self, ObjectError> {
        if data.is_empty() {
            return Err(ObjectError::InvalidData);
        }
        let metadata = Metadata::new(format!("Object_{}", Utc::now().timestamp()), data.len())?;
        Ok(Object { data, metadata })
    }

    pub fn get_metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn update_name(&mut self, name: String) -> Result<(), ObjectError> {
        if name.is_empty() {
            return Err(ObjectError::InvalidName("Name cannot be empty".to_string()));
        }
        self.metadata.name = name;
        self.metadata.update();
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.metadata.size
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}
