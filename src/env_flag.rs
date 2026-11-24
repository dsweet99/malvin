use std::ffi::OsStr;

#[must_use]
pub fn env_flag(key: &str) -> bool {
    flag_value_is_on(std::env::var_os(key).as_deref())
}

#[must_use]
pub fn flag_value_is_on(value: Option<&OsStr>) -> bool {
    value.is_some_and(|v| !v.is_empty() && v != "0")
}
