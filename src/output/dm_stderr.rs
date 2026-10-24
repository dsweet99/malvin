use std::io::Read;
use std::process::Child;

pub(crate) fn stderr_snippet(child: &mut Child) -> String {
    let Some(mut stderr) = child.stderr.take() else {
        return String::new();
    };
    let mut bytes = [0_u8; 180];
    let Ok(n) = stderr.read(&mut bytes) else {
        return String::new();
    };
    let text = String::from_utf8_lossy(&bytes[..n]);
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        String::new()
    } else {
        format!(": {line}")
    }
}
