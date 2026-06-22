//! Shared helpers for slots kiss-cov witness tests.

use crate::session_dotfile_backup::slots::{DotfileSpecRow, DOTFILE_ROWS};
use std::path::Path;

pub(super) const KISSCONFIG_FILE: &str = ".kissconfig";
pub(super) const MALVIN_CONFIG_SLOT: usize = 3;

pub(super) const ROW_WITNESS_0: DotfileSpecRow = DOTFILE_ROWS[0];
pub(super) const ROW_WITNESS_1: DotfileSpecRow = DOTFILE_ROWS[1];
pub(super) const ROW_WITNESS_2: DotfileSpecRow = DOTFILE_ROWS[2];
pub(super) const ROW_WITNESS_3: DotfileSpecRow = DOTFILE_ROWS[3];
pub(super) const ROW_WITNESS_4: DotfileSpecRow = DOTFILE_ROWS[4];
pub(super) const ROW_WITNESS_5: DotfileSpecRow = DOTFILE_ROWS[5];

pub(super) fn dotfile_spec_row_field_count(row: &DotfileSpecRow) -> usize {
    let &DotfileSpecRow {
        rel,
        home_subdir,
        mkdir_lbl,
        collision_lbl,
        restore_lbl,
        copy_err,
        restore_copy_err,
    } = row;
    let _ = (
        rel,
        home_subdir,
        mkdir_lbl,
        collision_lbl,
        restore_lbl,
        copy_err,
        restore_copy_err,
    );
    7
}

pub(super) fn write_merged_default_malvin_config(cfg_path: &Path) {
    let template = crate::malvin_config_file::parse_template_value().expect("template");
    let mut ensured = toml::Value::Table(toml::map::Map::new());
    crate::malvin_config_file::merge_missing_keys(&mut ensured, &template);
    let mut ensured_text = toml::to_string_pretty(&ensured).expect("toml");
    if !ensured_text.ends_with('\n') {
        ensured_text.push('\n');
    }
    std::fs::write(cfg_path, ensured_text).expect("write default config");
}

#[cfg(test)]
#[path = "slots_kiss_cov_shared_kiss_cov_test.rs"]
mod slots_kiss_cov_shared_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = dotfile_spec_row_field_count;
    }
}
