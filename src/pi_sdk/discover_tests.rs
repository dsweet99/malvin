use std::os::unix::fs::PermissionsExt;

use super::discover::{parse_pi_version, path_is_executable as is_executable, pi_missing_binary_message, pi_version_ok, resolve_pi_bin, PI_MIN_VERSION};
use super::models_list::{
    list_pi_models_sync, parse_list_models_table, pi_list_models_timeout,
    DEFAULT_PI_LIST_MODELS_TIMEOUT_MS,
};

#[test]
fn resolve_pi_bin_honors_malvin_pi_override() {
    let _ = is_executable;
    let missing_dir = tempfile::tempdir().expect("tmpdir");
    assert!(!is_executable(&missing_dir.path().join("missing")));
    let dir = tempfile::tempdir().expect("tmpdir");
    let fake = write_exec_script(dir.path(), "fake-pi", "#!/bin/sh\necho fake\n");
    crate::acp::with_env("MALVIN_PI", Some(fake.to_str().expect("utf8")), || {
        let got = resolve_pi_bin().expect("resolve");
        assert_eq!(got, fake);
    });
}

#[test]
fn resolve_pi_bin_rejects_missing_malvin_pi() {
    let missing = tempfile::tempdir()
        .expect("tmpdir")
        .path()
        .join("no-such-pi");
    crate::acp::with_env("MALVIN_PI", Some(missing.to_str().expect("utf8")), || {
        let err = resolve_pi_bin().expect_err("missing");
        assert!(err.contains("MALVIN_PI"));
        assert!(
            err.contains("does not bundle")
                || err.contains(pi_missing_binary_message().as_str())
                || err.contains("Install")
        );
    });
}

#[test]
fn resolve_pi_bin_rejects_non_executable_malvin_pi() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let fake = dir.path().join("pi-noexec");
    std::fs::write(&fake, "#!/bin/sh\n").expect("write");
    let mut perms = std::fs::metadata(&fake).expect("meta").permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&fake, perms).expect("chmod");
    assert!(!is_executable(&fake));
    crate::acp::with_env("MALVIN_PI", Some(fake.to_str().expect("utf8")), || {
        let err = resolve_pi_bin().expect_err("non-exec");
        assert!(err.contains("not executable"));
    });
}

#[test]
fn parse_list_models_skips_header() {
    let text = "\
provider    model                                          context
openai      gpt-4o                                         128K
openrouter  anthropic/claude-3-haiku                       200K
Showing 2 of 95 providers. Run `pi --list-providers` to see all.
";
    let rows = parse_list_models_table(text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "openai/gpt-4o");
    assert_eq!(rows[1].id, "openrouter/anthropic/claude-3-haiku");
    assert!(rows.iter().all(|r| r.thinking.is_none()));
    assert!(rows.iter().all(|r| !r.id.to_ascii_lowercase().contains("showing")));
}

#[test]
fn parse_list_models_fixed_width_keeps_spaces_in_model() {
    let text = "\
provider        model                                                     context  max-out
openai          my spaced model id                                        128K     4.1K
";
    let rows = parse_list_models_table(text);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "openai/my spaced model id");
    assert_eq!(rows[0].name, "my spaced model id");
    assert!(rows[0].thinking.is_none());
}

#[test]
fn parse_list_models_reads_thinking_column() {
    let text = "\
provider        model                                                     context  max-out  thinking  images
openai          gpt-4o                                                    128K     16.4K    no        yes
openrouter      qwen/qwen3-thinking                                       128K     16.4K    yes       no
";
    let rows = parse_list_models_table(text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].thinking, Some(false));
    assert_eq!(rows[1].thinking, Some(true));
}


#[test]
fn pi_version_ok_and_missing_hint() {
    assert!(pi_missing_binary_message().contains("MALVIN_PI"));
    assert_eq!(PI_MIN_VERSION, (0, 1, 23));
    assert_eq!(
        parse_pi_version("pi 0.1.23 (590d6189 2026-08-11T00:00:10Z)"),
        Some((0, 1, 23))
    );
    let dir = tempfile::tempdir().expect("tmpdir");
    let ok_bin = write_exec_script(dir.path(), "pi-ok", "#!/bin/sh\necho 'pi 0.1.23'\n");
    assert!(pi_version_ok(&ok_bin).is_ok());
    let old_bin = write_exec_script(dir.path(), "pi-old", "#!/bin/sh\necho 'pi 0.1.22'\n");
    assert!(pi_version_ok(&old_bin).expect_err("old").contains("too old"));
    let bad_bin = write_exec_script(dir.path(), "pi-bad", "#!/bin/sh\nexit 1\n");
    assert!(pi_version_ok(&bad_bin).is_err());
}

#[test]
fn pi_list_models_timeout_env_clamps_and_defaults() {
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let prior = std::env::var_os("MALVIN_PI_LIST_MODELS_TIMEOUT_MS");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "0");
    }
    assert_eq!(
        pi_list_models_timeout(),
        std::time::Duration::from_millis(1),
        "zero must clamp to at least 1ms"
    );
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "not-a-number");
    }
    assert_eq!(
        pi_list_models_timeout(),
        std::time::Duration::from_millis(DEFAULT_PI_LIST_MODELS_TIMEOUT_MS)
    );
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "250");
    }
    assert_eq!(
        pi_list_models_timeout(),
        std::time::Duration::from_millis(250)
    );
    #[allow(unsafe_code)]
    unsafe {
        match prior {
            Some(v) => std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", v),
            None => std::env::remove_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS"),
        }
    }
}

#[cfg(unix)]
#[test]
fn list_pi_models_sync_times_out_hanging_pi() {
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let hang = write_exec_script(dir.path(), "hang-pi", "#!/bin/sh\nsleep 30\n");
    let prior_pi = std::env::var_os("MALVIN_PI");
    let prior_to = std::env::var_os("MALVIN_PI_LIST_MODELS_TIMEOUT_MS");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI", hang.as_os_str());
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "200");
    }
    let started = std::time::Instant::now();
    let err = list_pi_models_sync().expect_err("hanging pi must time out");
    let elapsed = started.elapsed();
    #[allow(unsafe_code)]
    unsafe {
        match prior_pi {
            Some(v) => std::env::set_var("MALVIN_PI", v),
            None => std::env::remove_var("MALVIN_PI"),
        }
        match prior_to {
            Some(v) => std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", v),
            None => std::env::remove_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS"),
        }
    }
    assert!(
        err.contains("timed out"),
        "expected timeout error, got: {err}"
    );
    assert!(
        elapsed < std::time::Duration::from_millis(1500),
        "timeout path took too long: {elapsed:?}"
    );
}

#[cfg(unix)]
#[test]
fn list_pi_models_sync_parses_fake_pi_table() {
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let body = "#!/bin/sh\ncat <<'EOF'\nprovider model context\nopenai gpt-4o 128K\nShowing 1 of 1 providers.\nEOF\n";
    let fake = write_exec_script(dir.path(), "ok-pi", body);
    let prior_pi = std::env::var_os("MALVIN_PI");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI", fake.as_os_str());
    }
    let rows = list_pi_models_sync().expect("fake pi listing");
    #[allow(unsafe_code)]
    unsafe {
        match prior_pi {
            Some(v) => std::env::set_var("MALVIN_PI", v),
            None => std::env::remove_var("MALVIN_PI"),
        }
    }
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "openai/gpt-4o");
}

fn write_exec_script(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write");
    let mut perms = std::fs::metadata(&path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod");
    path
}
