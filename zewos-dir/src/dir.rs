use super::file::File;
use super::handlers::FolderHandler;
use super::logs::LogsManager;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Directory {
    handler: FolderHandler,
    subfolders: Vec<FolderHandler>,
    files: Vec<File>,
    logger: LogsManager,
}

impl Directory {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut dir = Directory {
            handler: FolderHandler::new(path.clone()).unwrap(),
            subfolders: Vec::new(),
            files: Vec::new(),
            logger: LogsManager::new(path.clone()).unwrap(),
        };
        dir.create().unwrap();
        dir.subfolders = Self::generate_folders(&path);
        dir.files = Self::generate_files(&path);
        dir
    }
    fn generate_folders(origin: &PathBuf) -> Vec<FolderHandler> {
        ["objects"]
            .iter()
            .map(|entry| FolderHandler::new(origin.join(entry)).unwrap())
            .collect()
    }
    fn generate_files(origin: &PathBuf) -> Vec<File> {
        let objects = PathBuf::from("objects").join("objects.bin");
        [
            objects,
            PathBuf::from("metadata.zewos"),
            PathBuf::from("config.zewos"),
        ]
        .iter()
        .map(|entry| File::new(origin.join(entry)))
        .collect()
    }

    pub fn get_handler(&self) -> &FolderHandler {
        &self.handler
    }
    pub fn logger(&self) -> LogsManager {
        self.logger.clone()
    }

    pub fn get_files(&self) -> &[File] {
        &self.files
    }

    pub fn objs_file(&self) -> &File {
        self.files.get(0).unwrap()
    }

    pub fn metadata_file(&self) -> &File {
        self.files.get(1).unwrap()
    }
    pub fn config_file(&self) -> &File {
        self.files.get(2).unwrap()
    }

    pub fn exists(&self) -> bool {
        self.handler.exists()
    }

    pub fn create(&self) -> std::io::Result<()> {
        self.handler.create()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn list_contents(&self) -> std::io::Result<Vec<PathBuf>> {
        self.handler.list_contents()
    }
}
