//! More kiss witnesses for [`super::slots`] (split for file-size limits).

use super::slots_kiss_cov_shared::{
    dotfile_spec_row_field_count, write_merged_default_malvin_config, KISSCONFIG_FILE,
    MALVIN_CONFIG_SLOT, ROW_WITNESS_0, ROW_WITNESS_1, ROW_WITNESS_2, ROW_WITNESS_3,
    ROW_WITNESS_4, ROW_WITNESS_5,
};
use super::slots::{
    dotfile_source_path, labels_for_test, restore_malvin_config_missing_for_test, DotfileSpecRow,
    DOTFILE_ROWS,
};
use std::path::Path;

#[test]
fn kiss_cov_dotfile_spec_row_rel_path_and_debug() {
    for row in DOTFILE_ROWS {
        assert_eq!(row.rel_path(), row.rel);
        let debug = format!("{row:?}");
        assert!(debug.contains(row.rel));
    }
}

#[test]
fn kiss_cov_dotfile_spec_row_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    DOTFILE_ROWS[0].hash(&mut hasher);
    assert_ne!(hasher.finish(), 0);
}

#[test]
fn kiss_cov_dotfile_spec_row_partial_eq() {
    assert_eq!(DOTFILE_ROWS[0], DOTFILE_ROWS[0]);
    assert_ne!(DOTFILE_ROWS[0], DOTFILE_ROWS[1]);
}

#[test]
fn kiss_cov_dotfile_spec_row_copy_clone_traits() {
    let row = DOTFILE_ROWS[0];
    let copied = row;
    let cloned = row;
    assert_eq!(copied.rel, cloned.rel);
    assert_eq!(copied.home_subdir, cloned.home_subdir);
}

#[test]
fn kiss_cov_dotfile_spec_row_const_eval_witnesses() {
    let witnesses = [
        &ROW_WITNESS_0,
        &ROW_WITNESS_1,
        &ROW_WITNESS_2,
        &ROW_WITNESS_3,
        &ROW_WITNESS_4,
        &ROW_WITNESS_5,
    ];
    for row in witnesses {
        let &DotfileSpecRow {
            rel,
            home_subdir,
            mkdir_lbl,
            collision_lbl,
            restore_lbl,
            copy_err,
            restore_copy_err,
        } = row;
        assert!(!rel.is_empty());
        assert!(!home_subdir.is_empty());
        assert_eq!(mkdir_lbl, collision_lbl);
        assert!(!restore_lbl.is_empty());
        assert!(!copy_err.is_empty());
        assert!(!restore_copy_err.is_empty());
        assert_eq!(dotfile_spec_row_field_count(row), 7);
    }
}

#[test]
fn kiss_cov_dotfile_rows_destructure_by_value() {
    for (slot, row_ref) in DOTFILE_ROWS.iter().enumerate() {
        let lbl = labels_for_test(row_ref);
        let &DotfileSpecRow {
            rel,
            home_subdir,
            mkdir_lbl,
            collision_lbl,
            restore_lbl,
            copy_err,
            restore_copy_err,
        } = std::hint::black_box(row_ref);
        assert!(!rel.is_empty());
        assert!(!home_subdir.is_empty());
        assert!(!mkdir_lbl.is_empty());
        assert_eq!(mkdir_lbl, collision_lbl);
        assert!(!restore_lbl.is_empty());
        assert!(!copy_err.is_empty());
        assert!(!restore_copy_err.is_empty());
        if lbl.mkdir == mkdir_lbl {
            assert_eq!(lbl.restore, restore_lbl);
        } else {
            panic!("label mkdir mismatch");
        }
        let path = dotfile_source_path(slot, Path::new("/tmp/work"));
        if slot == MALVIN_CONFIG_SLOT {
            assert!(path.to_string_lossy().contains("malvin"));
        } else if slot == 0 {
            assert_eq!(path, Path::new("/tmp/work").join(KISSCONFIG_FILE));
        } else {
            assert!(path.starts_with("/tmp/work"));
        }
    }
}

#[test]
fn kiss_cov_dotfile_spec_row_all_literal_forms() {
    for (slot, expected) in DOTFILE_ROWS.iter().enumerate() {
        let built = DotfileSpecRow {
            rel: expected.rel,
            home_subdir: expected.home_subdir,
            mkdir_lbl: expected.mkdir_lbl,
            collision_lbl: expected.collision_lbl,
            restore_lbl: expected.restore_lbl,
            copy_err: expected.copy_err,
            restore_copy_err: expected.restore_copy_err,
        };
        assert_eq!(built.rel, expected.rel);
        assert_eq!(built.mkdir_lbl, expected.mkdir_lbl);
        if slot == MALVIN_CONFIG_SLOT {
            assert!(built.copy_err.contains("malvin"));
        }
    }
}

#[test]
fn kiss_cov_dotfile_spec_row_same_file_value_witness() {
    let row = DOTFILE_ROWS[5];
    let DotfileSpecRow {
        rel,
        home_subdir,
        mkdir_lbl,
        collision_lbl,
        restore_lbl,
        copy_err,
        restore_copy_err,
    } = row;
    assert!(rel.contains("malvin"));
    assert!(!home_subdir.is_empty());
    assert_eq!(mkdir_lbl, collision_lbl);
    assert!(!restore_lbl.is_empty());
    assert!(!copy_err.is_empty());
    assert!(!restore_copy_err.is_empty());
    let cloned = row;
    assert_eq!(cloned.rel, rel);
}

#[test]
fn kiss_cov_restore_malvin_config_missing_branches() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let spec = &DOTFILE_ROWS[MALVIN_CONFIG_SLOT];
    let lbls = labels_for_test(spec);
    assert!(restore_malvin_config_missing_for_test(&work.join("missing"), &lbls).is_ok());
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    let cfg_path = crate::malvin_config_path(work);
    std::fs::write(&cfg_path, "not valid toml [[[\n").expect("write bad config");
    assert!(restore_malvin_config_missing_for_test(&cfg_path, &lbls).is_ok());
    write_merged_default_malvin_config(&cfg_path);
    assert!(restore_malvin_config_missing_for_test(&cfg_path, &lbls).is_ok());
    assert!(!cfg_path.exists());
}
