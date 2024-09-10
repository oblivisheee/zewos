use anyhow::Result;
use chrono::{Local, NaiveTime};

#[derive(Clone)]
pub struct Log {
    details: String,
    action: String,
    additional_info: String,
    timestamp: NaiveTime,
}

impl Log {
    pub fn new(details: String, action: String, additional_info: String) -> Log {
        Log {
            details,
            action,
            additional_info,
            timestamp: Local::now().time(),
        }
    }

    pub fn details(&self) -> &str {
        &self.details
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn additional_info(&self) -> &str {
        &self.additional_info
    }

    pub fn timestamp(&self) -> &NaiveTime {
        &self.timestamp
    }
    pub fn serialize(&self) -> String {
        format!(
            "{} {}:{}:{}",
            self.timestamp.format("%H:%M:%S"),
            self.action,
            self.details,
            self.additional_info
        )
    }
}

#[derive(Clone)]
pub struct LogFileStruct {
    logs: Vec<Log>,
}

impl LogFileStruct {
    pub fn new() -> Self {
        Self { logs: vec![] }
    }
    pub fn insert(
        &mut self,
        details: String,
        action: String,
        additional_info: String,
    ) -> Result<()> {
        self.add(Log::new(details, action, additional_info))?;
        Ok(())
    }

    pub fn add(&mut self, log: Log) -> Result<()> {
        self.logs.push(log);
        Ok(())
    }

    pub fn serialize(&self) -> String {
        let mut result = String::new();
        for log in &self.logs {
            result.push_str(&log.serialize());
            result.push('\n');
        }
        result
    }
}
