use chrono::Utc;
use md5;

use sha3::{Digest, Sha3_256};

use std::sync::Once;
use sysinfo::{Components, Disks, System};
use whoami;

#[derive(Clone)]
pub struct SystemFingerprint {
    machine_id: String,
    cpu_info: String,
    os_info: String,
    disk_size: u64,
    components: String,
}

impl SystemFingerprint {
    pub fn new() -> Self {
        static INIT: Once = Once::new();
        static mut SYSTEM: Option<System> = None;

        INIT.call_once(|| {
            let mut sys = System::new_all();
            sys.refresh_all();
            unsafe {
                SYSTEM = Some(sys);
            }
        });

        let system = unsafe { SYSTEM.as_ref().unwrap() };

        SystemFingerprint {
            machine_id: Self::get_machine_id(),
            cpu_info: Self::get_cpu_info(&system),
            os_info: Self::get_os_info(),
            disk_size: Self::get_disk_info(),
            components: Self::get_components_info(),
        }
    }

    fn get_machine_id() -> String {
        hex::encode(
            md5::compute(format!(
                "{:?}:{:?}:{:?}:{:?}",
                whoami::fallible::hostname(),
                whoami::realname(),
                whoami::username(),
                whoami::distro()
            ))
            .to_vec(),
        )
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
    pub fn generate_fingerprint(&self) -> zeroize::Zeroizing<Vec<u8>> {
        let fingerprint = format!(
            "{}:{}:{}:{}:{}",
            self.machine_id, self.cpu_info, self.os_info, self.disk_size, self.components
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
