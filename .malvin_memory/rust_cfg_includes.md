# Rust cfg and include! (platform splits)

TRIGGER: include!, cfg linux, macOS build fail
ADVICE: `#[cfg(target_os = "linux")]` applies only to the **next** item. In `src/acp_memory_containment/mod.rs`, put `#[cfg(target_os = "linux")]` on **each** `include!("*.inc")` (`cgroup_memory`, `linux_fs`, `linux_parent_death`, `linux_spawn`). One attribute on the first include does not gate the rest.
CONFIDENCE: 1

TRIGGER: acp_memory_containment, cgroup, macOS compile
ADVICE: Linux cgroup code lives in `src/acp_memory_containment/*.inc`. Non-Linux uses `stub.rs` for `memory_limit_oom_baseline_at` and `memory_limit_exceeded_since_baseline` (re-exported from `mod.rs`). macOS build failures here are usually cfg/stub gaps, not a missing checkout. Non-Linux also uses `#[cfg_attr(not(target_os = "linux"), allow(dead_code))]` and `allow(clippy::missing_const_for_fn)` on stub paths—grep those when auditing linter cheats.
CONFIDENCE: 1

TRIGGER: include!, kiss dependency, acp mod split
ADVICE: `src/acp/mod.rs` documents that ACP is split into `include!("*.inc")` fragments so `kiss check` dependency depth stays within limits—not just line-count. Before adding new ACP code to `mod.rs`, check existing `.inc` layout and `kiss stats` indirect_dependencies.
CONFIDENCE: 2

TRIGGER: memory.events, cgroup test, macOS test fail
ADVICE: Tests that write fake cgroup files and assert `memory_limit_exceeded()` or OOM exit messages need `#[cfg(target_os = "linux")]` on the test or a `linux_*` submodule—non-Linux stubs never read those files. See `tests/regression_bugs.rs` (`linux_regression` mod) and `rpc_part2_health.inc`.
CONFIDENCE: 0

TRIGGER: child_health macos test, sample_child_health, TaskAllInfo
ADVICE: In `src/child_health/macos.rs` unit tests, call `sample_child_health_macos`, not `sample_child_health`. For mapping tests, use `SampledTaskPidInfo` + `child_health_from_pid_info_parts` instead of `TaskAllInfo::default()` (no `Default`; zeroed `TaskAllInfo` trips `-D unsafe-code`).
CONFIDENCE: 0
