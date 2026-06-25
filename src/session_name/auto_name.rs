use super::{acquire_session_name, assert_no_peer_name_lock, generate_auto_name, generate_auto_name_with, name_path, names_registry_root, sleep_child, with_isolated_names};

#[test]
fn auto_name_is_five_lowercase_alnum() {
    with_isolated_names(|_| {
        let names: Vec<String> = (0..8).map(|i| format!("a{i:04}")).collect();
        let (name, guard) = generate_auto_name_with(|i| names[i].clone()).expect("auto name");
        assert!(name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
        assert_eq!(name.len(), 5);
        drop(guard);
    });
}

#[cfg(unix)]
#[test]
fn auto_name_retries_on_live_collision() {
    with_isolated_names(|_| {
        let mut child = sleep_child("120");
        let holder_pid = child.id();
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("aaaaa"), format!("{holder_pid}\n")).expect("live peer");
        let (name, guard) = generate_auto_name_with(|i| {
            if i == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .expect("second draw succeeds");
        assert_eq!(name, "bbbbb");
        let _ = child.kill();
        let _ = child.wait();
        drop(guard);
    });
}

#[test]
fn auto_name_reclaims_stale_file_on_draw() {
    with_isolated_names(|_| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("aaaaa"), "424242\n").expect("stale");
        let (name, guard) =
            generate_auto_name_with(|_| "aaaaa".to_string()).expect("reclaim stale draw");
        assert_eq!(name, "aaaaa");
        drop(guard);
    });
}

#[cfg(unix)]
#[test]
fn auto_name_fails_after_sixteen_live_collisions() {
    with_isolated_names(|_| {
        let mut child = sleep_child("120");
        let holder_pid = child.id();
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("aaaaa"), format!("{holder_pid}\n")).expect("live peer");
        let err = generate_auto_name_with(|_| "aaaaa".to_string()).expect_err("16 collisions");
        assert!(
            err.contains("failed to allocate a unique auto-generated session name"),
            "err={err}"
        );
        let _ = child.kill();
        let _ = child.wait();
    });
}

#[test]
fn generate_auto_name_allocates_five_char_lock() {
    with_isolated_names(|_| {
        let (name, guard) = generate_auto_name().expect("auto");
        assert_eq!(guard.name(), name);
        assert_eq!(name.len(), 5);
        drop(guard);
    });
}

#[test]
fn acquire_session_name_supports_explicit_and_auto() {
    with_isolated_names(|_| {
        let (explicit, guard) = acquire_session_name(Some("probe")).expect("explicit");
        assert_eq!(explicit, "probe");
        drop(guard);
        let (auto, guard2) = acquire_session_name(None).expect("auto");
        assert_eq!(auto.len(), 5);
        drop(guard2);
    });
}

#[test]
fn assert_no_peer_name_lock_clears_stale_file() {
    with_isolated_names(|_| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), "424242\n").expect("stale pid");
        assert_no_peer_name_lock("probe").expect("stale cleared");
        assert!(!name_path("probe").exists());
    });
}
