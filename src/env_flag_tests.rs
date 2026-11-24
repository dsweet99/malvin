use std::ffi::OsStr;

use crate::env_flag::{env_flag, flag_value_is_on};

pub(crate) fn flags_are_on_only_for_non_empty_values_other_than_zero() {
    for (value, on) in [
        (None, false),
        (Some(""), false),
        (Some("0"), false),
        (Some("1"), true),
        (Some("yes"), true),
    ] {
        assert_eq!(flag_value_is_on(value.map(OsStr::new)), on, "{value:?}");
    }
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    let key = "MALVIN_TEST_ENV_FLAG_PROBE";
    for (value, on) in [(Some("0"), false), (Some("1"), true), (None, false)] {
        crate::agent_process::with_env(key, value, || {
            assert_eq!(env_flag(key), on, "{key}={value:?}");
        });
    }
}
