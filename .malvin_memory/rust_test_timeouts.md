# Rust unit test duration and hangs

TRIGGER: nested fake command, set_fake_command_dir, mutex hang
ADVICE: `set_fake_command_dir` in `src/cli/repo_checks/command_support.rs` locks `FAKE_COMMAND_DIR_MUTEX`. Nested guards on one thread deadlock if each call locks again—acquire the mutex only when `TEST_FAKE_COMMAND_DIR` is `None` (outermost guard); store the guard in `FakeCommandDirGuard._process_lock`. Regression: `stale_fake_command_path_tests::nested_fake_command_dir_guards_restore_stack`.
CONFIDENCE: 0

TRIGGER: test hang zombie, nextest stuck, malvin deps
ADVICE: Before debugging a hang, `ps` for stale `target/debug/deps/malvin-* --exact <test>` or old `cargo-nextest` from prior runs; kill them. Mutex self-deadlocks and poisoned waits can leave zombies that block new runs on the same lock.
CONFIDENCE: 0

TRIGGER: nextest timeout, slow-timeout, test 10s
ADVICE: Enforce per-test ceiling in `.config/nextest.toml`: `[profile.default] slow-timeout = { period = "10s", terminate-after = 1 }` (key is `slow-timeout`, not `test-timeout`). After `cargo nextest run`, grep `PASS [ …s]` in `quality_gates.log` for the slowest durations.
CONFIDENCE: 0

TRIGGER: captive cat, test_captive_session, rpc_timeout slow
ADVICE: `captive_cat_acp_session_for_tests` (`src/acp/test_captive_session.rs`) uses `default_test_spawn_args` `rpc_timeout` (was 30s). Failed tidy/ACP RPC on `cat` waits the full timeout—e.g. `post_run_gate_tidy_skips_second_begin_when_coder_session_open` ~31s; use ≤5s unless a test needs longer.
CONFIDENCE: 0

TRIGGER: macOS timeout command, shell timeout
ADVICE: macOS often has no `timeout` in PATH. Prefer `cargo nextest` `slow-timeout`, `tokio::time::timeout`, or `tests/common/child_wait.rs::wait_child_with_timeout` for subprocess bounds—not bare shell `timeout`.
CONFIDENCE: 0
