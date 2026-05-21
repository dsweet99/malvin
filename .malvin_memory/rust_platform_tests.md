# Rust platform-specific tests

TRIGGER: linux test, cfg_attr test, integration test macOS
ADVICE: For Linux-only integration tests, use `#[cfg(target_os = "linux")] mod linux_* { ... #[test] fn ... }` so bodies and imports are not compiled on macOS. Avoid `#[cfg_attr(..., test)]` on free functions: the body still compiles and will error on unresolved Linux-only symbols (`memory_limit_exceeded_at`, `remove_cgroup_dir`, etc.).
CONFIDENCE: 3

TRIGGER: kiss, two attributes, cfg test
ADVICE: Kiss `annotations_per_function` counts each attribute separately. Prefer one `#[test]` inside a `#[cfg(linux)] mod` over `#[cfg(...)]` plus `#[test]` on the same function.
CONFIDENCE: 3

TRIGGER: /bin/true, /bin/sleep, macOS, transport test
ADVICE: Hardcoding `/bin/true` or `/bin/sleep` fails on macOS (`/usr/bin/true`, `/usr/bin/sleep`). Reuse `unix_bin_with_fallback` from `src/acp/reader_tests.rs` (checks `/bin/{name}` then `/usr/bin/{name}`) in other ACP tests such as `transport_tests.rs`.
CONFIDENCE: 3

TRIGGER: AcpSession spawn test, spawn stderr, handshake timeout
ADVICE: For stderr emitted before ACP handshake (e.g. containment warn at spawn), use `bin_override: Some("/bin/cat")`, clear/take `capture_stderr_output` lines, and `tokio::time::timeout` (~500ms) on `AcpSession::spawn`—handshake blocks but the warn already printed. On Linux when `writable_cgroups_on_host()`, containment may be active (no warn); use `emit_containment_unavailable_warn_after_spawn` unit tests or skip spawn integration.
CONFIDENCE: 0
