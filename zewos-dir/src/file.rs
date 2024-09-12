use super::handlers::FileHandler;
use std::fs::{self, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Clone)]
pub struct File {
    handler: FileHandler,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File {
            handler: FileHandler::new(path).unwrap(),
        }
    }

    pub fn read(&self) -> io::Result<Vec<u8>> {
        self.handler.read()
    }

    pub fn write(&self, contents: &[u8]) -> io::Result<()> {
        self.handler.write(contents)
    }
    pub fn write_no_encrypt(&self, contents: &[u8]) -> io::Result<()> {
        self.handler.write_no_encrypt(contents)
    }
    pub fn read_no_decrypt(&self) -> io::Result<Vec<u8>> {
        self.handler.read_no_decrypt()
    }

    pub fn append(&self, contents: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.handler.path)?;
        file.write_all(contents.as_bytes())
    }

    pub fn exists(&self) -> bool {
        self.handler.path.exists()
    }

    pub fn delete(&self) -> io::Result<()> {
        fs::remove_file(&self.handler.path)
    }

    pub fn rename(&self, new_name: &str) -> io::Result<()> {
        fs::rename(&self.handler.path, &new_name)
    }

    pub fn size(&self) -> io::Result<u64> {
        let metadata = fs::metadata(&self.handler.path)?;
        Ok(metadata.len())
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        let file = OpenOptions::new().write(true).open(&self.handler.path)?;
        file.set_len(size)
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.handler.path)?;
        file.seek(pos)
    }
    pub fn path(&self) -> &PathBuf {
        &self.handler.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        assert!(file.exists());
    }

    #[test]
    fn test_read_write() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        let content = b"Hello, World!";
        file.write(content).unwrap();
        assert_eq!(file.read().unwrap(), content);
    }

    #[test]
    fn test_append() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        file.write(b"Hello").unwrap();
        file.append(", World!").unwrap();
        assert_eq!(file.read().unwrap(), b"Hello, World!");
    }

    #[test]
    fn test_delete() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        assert!(file.exists());
        file.delete().unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_rename() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        let new_name = temp_file.path().with_file_name("new_name");
        file.rename(new_name.to_str().unwrap()).unwrap();
        assert!(new_name.exists());
    }

    #[test]
    fn test_size() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        let content = b"Hello, World!";
        file.write(content).unwrap();
        assert_eq!(file.size().unwrap(), content.len() as u64);
    }

    #[test]
    fn test_truncate() {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::new(temp_file.path().to_path_buf());
        file.write(b"Hello, World!").unwrap();
        file.truncate(5).unwrap();
        assert_eq!(file.read().unwrap(), b"Hello");
    }
}
