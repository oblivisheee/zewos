use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zewos_core::permissions::PermissionsManager;

pub struct FileHandler {
    pub path: PathBuf,
    permissions: PermissionsManager,
}

impl FileHandler {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let permissions = PermissionsManager::new(path.to_str().unwrap_or_default().to_string());

        if path.exists() {
            if path.is_file() {
                permissions.check_file_permissions(path.to_str().unwrap_or_default())?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path exists but is not a file",
                ));
            }
        } else {
            permissions.create_file_with_permissions(path.to_str().unwrap_or_default())?;
        }

        Ok(FileHandler { path, permissions })
    }

    pub fn read(&self) -> io::Result<Vec<u8>> {
        let mut file = File::open(&self.path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    pub fn write(&self, contents: &[u8]) -> io::Result<()> {
        let mut file = File::create(&self.path)?;
        file.write_all(contents)
    }
}

pub struct FolderHandler {
    path: PathBuf,
    permissions: PermissionsManager,
}

impl FolderHandler {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let permissions = PermissionsManager::new(path.to_str().unwrap_or_default().to_string());

        if path.exists() {
            if !path.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Path exists but is not a directory",
                ));
            }
        } else {
            permissions.create_folder_with_permissions(path.to_str().unwrap_or_default())?;
        }

        Ok(FolderHandler { path, permissions })
    }

    pub fn create(&self) -> io::Result<()> {
        self.permissions
            .create_folder_with_permissions(self.path.to_str().unwrap_or_default())
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    pub fn list_contents(&self) -> io::Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.path)?;
        let mut contents = Vec::new();
        for entry in entries {
            let entry = entry?;
            contents.push(entry.path());
        }
        Ok(contents)
    }
}

pub fn join_paths(base: &Path, path: &str) -> PathBuf {
    base.join(path)
}
