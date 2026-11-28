use std::collections::BTreeSet;

use crate::output::print_log_warning;

pub(crate) fn warn_config_once(line: &str) {
    static SHOWN: std::sync::Mutex<BTreeSet<String>> = std::sync::Mutex::new(BTreeSet::new());
    let first = SHOWN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(line.to_string());
    if first {
        print_log_warning(line);
    }
}

pub(crate) fn read_string(value: Option<&toml::Value>) -> Option<String> {
    value?.as_str().map(str::to_string)
}

fn parse_toml_integer(value: Option<&toml::Value>) -> Option<i64> {
    let v = value?;
    if let Some(i) = v.as_integer() {
        return Some(i);
    }
    v.as_str()?.parse().ok()
}

pub(crate) fn read_u32(value: Option<&toml::Value>) -> Option<u32> {
    parse_toml_integer(value).and_then(|i| u32::try_from(i).ok())
}

pub(crate) fn read_u64(value: Option<&toml::Value>) -> Option<u64> {
    parse_toml_integer(value).and_then(|i| u64::try_from(i).ok())
}

pub(crate) fn read_f64(value: Option<&toml::Value>) -> Option<f64> {
    let v = value?;
    if let Some(f) = v.as_float() {
        return Some(f);
    }
    if let Some(i) = v.as_integer() {
        #[allow(clippy::cast_precision_loss)]
        return Some(i as f64);
    }
    v.as_str()?.parse().ok()
}
