//! Shared assertions for deprecated `malvin code` CLI invocations.

pub fn assert_code_deprecated(out: &std::process::Output) {
    assert_eq!(
        out.status.code(),
        Some(1),
        "malvin code must exit 1; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("deprecated"),
        "malvin code stderr must mention deprecation: {stderr}"
    );
}
