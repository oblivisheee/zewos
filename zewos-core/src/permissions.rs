#[cfg(not(windows))]
use nix;
#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
#[cfg(windows)]
use windows::Win32::Security::{GetTokenInformation, TokenUser, TOKEN_QUERY};
#[cfg(windows)]
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use std::fs::{self, File};
use std::io;
use std::path::Path;
#[derive(Clone)]
pub struct PermissionsManager {
    storage_path: String,
    #[cfg(not(windows))]
    app_uid: u32,
    #[cfg(not(windows))]
    app_gid: u32,
}

impl PermissionsManager {
    #[cfg(windows)]
    pub fn new(storage_path: String, _app_uid: u32, _app_gid: u32) -> Self {
        PermissionsManager { storage_path }
    }

    #[cfg(not(windows))]
    pub fn new(storage_path: String) -> Self {
        let app_uid = nix::unistd::geteuid().as_raw();
        let app_gid = nix::unistd::getegid().as_raw();
        PermissionsManager {
            storage_path,
            app_uid,
            app_gid,
        }
    }

    #[cfg(windows)]
    pub fn set_file_permissions(&self, file_path: &str) -> io::Result<()> {
        let full_path = Path::new(&self.storage_path).join(file_path);
        let mut options = OpenOptions::new();
        options.write(true);
        options
            .custom_flags(windows::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS as u32);
        let file = options.open(&full_path)?;

        // Set permissions to read and write only for the current user
        let mut security_info = windows::Win32::Security::SECURITY_INFORMATION(
            windows::Win32::Security::OWNER_SECURITY_INFORMATION.0
                | windows::Win32::Security::DACL_SECURITY_INFORMATION.0,
        );
        let mut token = windows::Win32::Foundation::HANDLE::default();
        unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;
            let mut token_user = TokenUser::default();
            let mut return_length = 0;
            GetTokenInformation(
                token,
                windows::Win32::Security::TokenUser,
                &mut token_user as *mut _ as *mut _,
                std::mem::size_of::<TokenUser>() as u32,
                &mut return_length,
            )?;
            windows::Win32::Security::SetNamedSecurityInfoW(
                full_path
                    .as_os_str()
                    .encode_wide()
                    .collect::<Vec<_>>()
                    .as_ptr(),
                windows::Win32::Security::SE_FILE_OBJECT,
                security_info,
                Some(&token_user.User.Sid),
                None,
                None,
                None,
            )?;
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_file_permissions(&self, file_path: &str) -> io::Result<()> {
        let full_path = Path::new(&self.storage_path).join(file_path);
        let file = File::open(&full_path)?;

        let permissions = fs::Permissions::from_mode(0o600);
        file.set_permissions(permissions)?;

        // Set ownership to the app
        nix::unistd::chown(
            &full_path,
            Some(nix::unistd::Uid::from_raw(self.app_uid)),
            Some(nix::unistd::Gid::from_raw(self.app_gid)),
        )?;

        Ok(())
    }

    pub fn create_file_with_permissions(&self, file_path: &str) -> io::Result<()> {
        let full_path = Path::new(&self.storage_path).join(file_path);
        let _file = File::create(&full_path)?;

        self.set_file_permissions(file_path)?;

        Ok(())
    }

    pub fn create_folder_with_permissions(&self, folder_path: &str) -> io::Result<()> {
        let full_path = Path::new(&self.storage_path).join(folder_path);
        fs::create_dir_all(&full_path)?;

        #[cfg(not(windows))]
        {
            let permissions = fs::Permissions::from_mode(0o700);
            fs::set_permissions(&full_path, permissions)?;

            nix::unistd::chown(
                &full_path,
                Some(nix::unistd::Uid::from_raw(self.app_uid)),
                Some(nix::unistd::Gid::from_raw(self.app_gid)),
            )?;
        }

        #[cfg(windows)]
        {
            self.set_file_permissions(folder_path)?;
        }

        Ok(())
    }

    #[cfg(windows)]
    pub fn check_file_permissions(&self, file_path: &str) -> io::Result<bool> {
        let full_path = Path::new(&self.storage_path).join(file_path);
        let metadata = fs::metadata(&full_path)?;

        Ok(metadata.permissions().readonly() == false)
    }

    #[cfg(not(windows))]
    pub fn check_file_permissions(&self, file_path: &str) -> io::Result<bool> {
        use std::os::unix::fs::MetadataExt;

        let full_path = Path::new(&self.storage_path).join(file_path);
        let metadata = fs::metadata(&full_path)?;

        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check if the file is only readable and writable by the owner
        let correct_permissions = mode & 0o777 == 0o600;

        // Check if the file is owned by the app
        let correct_ownership = metadata.uid() == self.app_uid && metadata.gid() == self.app_gid;

        Ok(correct_permissions && correct_ownership)
    }
}

#[cfg(test)]
#[cfg(not(windows))]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[cfg(not(windows))]
    #[test]
    fn test_set_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap().to_string();

        let permissions_manager = PermissionsManager::new(storage_path.clone());

        let file_path = "test_file.txt";
        let full_path = temp_dir.path().join(file_path);
        let mut file = File::create(&full_path).unwrap();
        file.write_all(b"test content").unwrap();

        permissions_manager.set_file_permissions(file_path).unwrap();

        assert!(permissions_manager
            .check_file_permissions(file_path)
            .unwrap());
    }
    #[cfg(not(windows))]
    #[test]
    fn test_create_file_with_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap().to_string();
        let permissions_manager = PermissionsManager::new(storage_path.clone());

        let file_path = "new_test_file.txt";
        permissions_manager
            .create_file_with_permissions(file_path)
            .unwrap();

        assert!(permissions_manager
            .check_file_permissions(file_path)
            .unwrap());
    }
}
