#[cfg(not(unix))]
pub fn unix_bin_with_fallback(name: &str) -> String {
    name.to_string()
}

#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
pub fn unix_bin_with_fallback(name: &str) -> String {
    let bin = format!("/bin/{name}");
    if Path::new(&bin).is_file() {
        return bin;
    }
    let usr_bin = format!("/usr/bin/{name}");
    if Path::new(&usr_bin).is_file() {
        return usr_bin;
    }
    name.to_string()
}
