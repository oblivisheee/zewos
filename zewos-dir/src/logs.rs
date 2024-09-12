use super::{file::File, handlers::FolderHandler};
use chrono::Local;
use std::io;
use std::path::PathBuf;
pub use zewos_core::logging::Log;
use zewos_core::logging::LogFileStruct;

#[derive(Clone)]
pub struct LogsManager {
    handler: FolderHandler,
    logs: Vec<LogFile>,
    current_log: Option<LogFile>,
}

impl LogsManager {
    pub(crate) fn new(path: PathBuf) -> io::Result<Self> {
        let path = path.join("logs");
        let handler = FolderHandler::new(path)?;
        Ok(LogsManager {
            handler,
            logs: Vec::new(),
            current_log: None,
        })
    }

    pub fn start_session(&mut self) -> io::Result<()> {
        let file_name = format!("{}.zewos", Local::now().format("%Y-%m-%d_%H-%M-%S"));
        self.current_log = Some(LogFile::new(self.handler.path.join(file_name))?);
        Ok(())
    }

    pub fn end_session(&mut self) -> io::Result<()> {
        if let Some(log) = self.current_log.take() {
            log.save()?;
        }
        Ok(())
    }

    pub fn add_log(
        &mut self,
        details: &str,
        action: &str,
        additional_info: &str,
    ) -> io::Result<()> {
        if self.current_log.is_none() {
            // Session hasn't activated. Either start it or you have logs turned off.
            return Ok(());
        }
        if let Some(log) = self.current_log.as_mut() {
            log.add_log(details, action, additional_info)?;
        }
        self.current_log.as_ref().unwrap().save()?;
        Ok(())
    }
}
#[derive(Clone)]
pub struct LogFile {
    file: File,
    logs: LogFileStruct,
}

impl LogFile {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        Ok(LogFile {
            file: File::new(path),
            logs: LogFileStruct::new(),
        })
    }

    pub fn add_log(
        &mut self,
        details: &str,
        action: &str,
        additional_info: &str,
    ) -> io::Result<()> {
        let _ = self.logs.add(Log::new(
            details.to_string(),
            action.to_string(),
            additional_info.to_string(),
        ));
        Ok(())
    }

    pub fn save(&self) -> io::Result<()> {
        let serialized = self.logs.serialize();
        self.file.write_no_encrypt(serialized.as_bytes())
    }
}
