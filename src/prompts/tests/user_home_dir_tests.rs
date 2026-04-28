#![allow(unsafe_code)]

#[test]
fn user_home_dir_prefers_home_then_userprofile() {
    let _lock = crate::test_utils::test_env_lock();
    let old_home = std::env::var_os("HOME");
    let old_profile = std::env::var_os("USERPROFILE");

    unsafe {
        std::env::set_var("HOME", "/tmp/custom-home");
        std::env::remove_var("USERPROFILE");
    }
    assert_eq!(super::super::user_home_dir(), std::path::PathBuf::from("/tmp/custom-home"));

    unsafe {
        std::env::set_var("USERPROFILE", "/tmp/fallback-userprofile");
        std::env::remove_var("HOME");
    }
    assert_eq!(
        super::super::user_home_dir(),
        std::path::PathBuf::from("/tmp/fallback-userprofile")
    );

    unsafe {
        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", "/tmp/ignored");
    }
    assert_eq!(
        super::super::user_home_dir(),
        std::path::PathBuf::from("/tmp/ignored")
    );

    unsafe {
        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match old_profile {
            Some(v) => std::env::set_var("USERPROFILE", v),
            None => std::env::remove_var("USERPROFILE"),
        }
    }
}

#[test]
fn user_home_dir_falls_back_to_temp_dir() {
    let _lock = crate::test_utils::test_env_lock();
    let old_home = std::env::var_os("HOME");
    let old_profile = std::env::var_os("USERPROFILE");

    unsafe {
        std::env::remove_var("HOME");
        std::env::remove_var("USERPROFILE");
    }
    let home = super::super::user_home_dir();
    assert_eq!(home, std::env::temp_dir());

    unsafe {
        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match old_profile {
            Some(v) => std::env::set_var("USERPROFILE", v),
            None => std::env::remove_var("USERPROFILE"),
        }
    }
}

