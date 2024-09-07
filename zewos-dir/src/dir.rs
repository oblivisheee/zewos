use super::file::File;
use super::handlers::FolderHandler;
use std::path::PathBuf;

pub struct Directory {
    handler: FolderHandler,
    files: Vec<File>,
}

impl Directory {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut dir = Directory {
            handler: FolderHandler::new(path.clone()).unwrap(),
            files: Vec::new(),
        };
        dir.create().unwrap();
        dir.files = Self::generate_files(&path);
        dir
    }

    fn generate_files(origin: &PathBuf) -> Vec<File> {
        ["objs.ze", "meta.ze"]
            .iter()
            .map(|entry| File::new(origin.join(entry)))
            .collect()
    }

    pub fn get_handler(&self) -> &FolderHandler {
        &self.handler
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
