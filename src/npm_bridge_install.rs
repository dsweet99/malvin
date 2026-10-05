use std::fs;
use std::path::{Path, PathBuf};

const LOCK_JSON: &str = "package-lock.json";
const NPM_STAMP: &str = ".malvin-npm-stamp";
const INSTALL_LOCK: &str = ".malvin-install.lock";

pub type PayloadFiles = &'static [(&'static str, &'static [u8])];

#[derive(Debug, Clone, Copy)]
pub struct NpmBridgePackage {
    pub dir_name: &'static str,
    pub sdk_marker: &'static str,
    pub npm_label: &'static str,
    pub payload: PayloadFiles,
}

pub(crate) fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

impl NpmBridgePackage {
    #[must_use]
    pub fn default_install_dir(&self) -> PathBuf {
        crate::user_home::user_home_dir()
            .join(crate::MALVIN_USER_HOME_DIR)
            .join("sdk-bridges")
            .join(self.dir_name)
    }

    pub fn install_into(&self, dest: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(dest).map_err(|e| format!("mkdir {}: {e}", dest.display()))?;
        let _lock = lock_install_dir(dest)?;
        write_payload(self.payload, dest)?;
        if !self.npm_deps_current(dest) {
            crate::cursor_sdk::bridge_install_npm::npm_ci(dest, self.npm_label)?;
            self.verify_sdk_installed(dest)?;
            self.write_npm_stamp(dest)?;
        }
        Ok(dest.join("dist").join("bridge.js"))
    }

    fn lock_stamp(&self) -> String {
        let lock = self
            .payload
            .iter()
            .find(|(rel, _)| *rel == LOCK_JSON)
            .map_or(&[][..], |(_, bytes)| *bytes);
        format!("{:x}", fnv1a64(lock))
    }

    pub(crate) fn npm_deps_current(&self, dest: &Path) -> bool {
        dest.join(self.sdk_marker).is_file()
            && fs::read_to_string(dest.join(NPM_STAMP))
                .unwrap_or_default()
                .trim()
                == self.lock_stamp()
    }

    fn verify_sdk_installed(&self, dest: &Path) -> Result<(), String> {
        if dest.join(self.sdk_marker).is_file() {
            return Ok(());
        }
        Err(format!(
            "npm ci finished but {} is missing",
            dest.join(self.sdk_marker).display()
        ))
    }

    fn write_npm_stamp(&self, dest: &Path) -> Result<(), String> {
        let path = dest.join(NPM_STAMP);
        fs::write(&path, format!("{}\n", self.lock_stamp()))
            .map_err(|e| format!("write {}: {e}", path.display()))
    }
}

fn lock_install_dir(dest: &Path) -> Result<fs::File, String> {
    let path = dest.join(INSTALL_LOCK);
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    file.lock()
        .map_err(|e| format!("lock {}: {e}", path.display()))?;
    Ok(file)
}

fn write_payload(payload: PayloadFiles, dest: &Path) -> Result<(), String> {
    for (rel, bytes) in payload {
        let path = dest.join(rel);
        if fs::read(&path).is_ok_and(|on_disk| on_disk == *bytes) {
            continue;
        }
        write_atomically(&path, bytes)?;
    }
    Ok(())
}

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension("malvin-tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))
}
