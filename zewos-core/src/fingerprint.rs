use super::hash::Blake3;
use chrono::Utc;
use lazy_static::lazy_static;
use sha3::{Digest, Sha3_256};
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use sysinfo::{Components, Disks, System};
use whoami;

#[derive(Debug, Clone)]
pub struct SystemFingerprint {
    machine_id: String,
    cpu_info: String,
    os_info: String,
    disk_size: u64,
    components: String,
    admin_data: String,
}

impl SystemFingerprint {
    pub fn new() -> Self {
        lazy_static! {
            static ref SYSTEM: Arc<Mutex<System>> = Arc::new(Mutex::new({
                let mut sys = System::new_all();
                sys.refresh_all();
                sys
            }));
        }

        let system = SYSTEM.lock().unwrap();

        SystemFingerprint {
            machine_id: Self::get_machine_id(),
            cpu_info: Self::get_cpu_info(&system),
            os_info: Self::get_os_info(),
            disk_size: Self::get_disk_info(),
            components: Self::get_components_info(),
            admin_data: Self::get_admin_data(),
        }
    }

    fn get_machine_id() -> String {
        let input = format!(
            "{:?}:{:?}:{:?}:{:?}",
            whoami::fallible::hostname(),
            whoami::realname(),
            whoami::username(),
            whoami::distro()
        );
        let hash = Blake3::new(input.as_bytes());
        hex::encode(hash.as_bytes())
    }

    fn get_cpu_info(system: &System) -> String {
        let cpu = system.cpus().first().unwrap();
        format!("{}", cpu.brand())
    }

    fn get_components_info() -> String {
        let components = Components::new();
        let mut hasher = Sha3_256::new();
        for component in components.iter() {
            hasher.update(component.label().as_bytes());
        }
        let result = hasher.finalize();
        hex::encode(result.to_vec())
    }

    fn get_os_info() -> String {
        format!("{} {}", whoami::distro(), whoami::arch())
    }

    fn get_disk_info() -> u64 {
        let disks = Disks::new();
        disks.iter().map(|disk| disk.total_space()).sum()
    }

    fn get_admin_data() -> String {
        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(&[
                    "-Command",
                    "Get-WmiObject -Class Win32_BIOS | Select-Object -ExpandProperty SerialNumber",
                ])
                .output()
        } else if cfg!(target_os = "linux") {
            Command::new("sh")
                .arg("-c")
                .arg("sudo dmidecode -s system-serial-number")
                .output()
        } else if cfg!(target_os = "macos") {
            Command::new("sh")
                .arg("-c")
                .arg("ioreg -l | grep IOPlatformSerialNumber")
                .output()
        } else {
            return String::from("Unsupported OS");
        };

        match output {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
            Err(_) => String::from("Failed to get admin data"),
        }
    }

    pub fn generate_fingerprint(&self) -> zeroize::Zeroizing<Vec<u8>> {
        let fingerprint = format!(
            "{}:{}:{}:{}:{}:{}",
            self.machine_id,
            self.cpu_info,
            self.os_info,
            self.disk_size,
            self.components,
            self.admin_data
        );
        let mut hasher = Sha3_256::new();
        hasher.update(fingerprint.as_bytes());
        let result = hasher.finalize();
        zeroize::Zeroizing::new(result.to_vec())
    }

    fn print_info(&self) {
        println!("Machine ID: {}", self.machine_id);
        println!("CPU Info: {}", self.cpu_info);
        println!("OS Info: {}", self.os_info);
        println!("Disk Size: {}", self.disk_size);
        println!("Components: {}", self.components);
        println!("Admin Data: {}", self.admin_data);
    }
}

impl Default for SystemFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fingerprint = SystemFingerprint::new();
        assert!(!fingerprint.machine_id.is_empty());
        assert!(!fingerprint.cpu_info.is_empty());
        assert!(!fingerprint.os_info.is_empty());
        assert!(!fingerprint.admin_data.is_empty());
    }

    #[test]
    fn test_get_machine_id() {
        let id1 = SystemFingerprint::get_machine_id();
        let id2 = SystemFingerprint::get_machine_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_fingerprint() {
        let fingerprint = SystemFingerprint::new();
        let hash1 = fingerprint.generate_fingerprint();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let fingerprint2 = SystemFingerprint::new();
        let hash2 = fingerprint2.generate_fingerprint();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_print_info() {
        let fingerprint = SystemFingerprint::new();
        fingerprint.print_info(); // This will print to stdout, we're just ensuring it doesn't panic
    }
}
