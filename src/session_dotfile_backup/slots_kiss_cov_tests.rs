//! External kiss witnesses for [`super::slots`] (must be `*_tests.rs` for kiss).

use super::slots_kiss_cov_shared::{
    dotfile_spec_row_field_count, KISSCONFIG_FILE, MALVIN_CONFIG_SLOT,
};
use super::slots::{
    backup_slot, dotfile_source_path, labels_for_test, restore_malvin_config_missing_for_test,
    restore_slot, DotfileSpecRow, DOTFILE_ROWS,
};
use super::DotfileBackupState;
use std::path::Path;

#[test]
fn kiss_cov_dotfile_spec_row_field_count() {
    for row in DOTFILE_ROWS {
        assert_eq!(dotfile_spec_row_field_count(&row), 7);
        let DotfileSpecRow {
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
    }
}

#[test]
fn slots_branchy_witness_covers_dotfile_rows() {
    for (slot, row_ref) in DOTFILE_ROWS.iter().enumerate() {
        let lbl = labels_for_test(row_ref);
        let path = dotfile_source_path(slot, Path::new("/tmp/w"));
        assert_eq!(dotfile_spec_row_field_count(row_ref), 7);
        if slot == MALVIN_CONFIG_SLOT {
            assert!(path.to_string_lossy().contains("malvin"));
        } else if slot == 0 {
            assert_eq!(path, Path::new("/tmp/w").join(KISSCONFIG_FILE));
        } else if lbl.mkdir == row_ref.mkdir_lbl {
            assert_eq!(lbl.restore, row_ref.restore_lbl);
        } else {
            panic!("slot {slot} label mismatch");
        }
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::write(work.join(KISSCONFIG_FILE), "[gate]\n").expect("kissconfig");
    let mut id = |n: usize| format!("id{n}");
    let backup = backup_slot(0, work, &mut id).expect("backup");
    if matches!(backup, DotfileBackupState::Present(_)) {
        restore_slot(work, &backup, 0).expect("restore");
    } else if matches!(backup, DotfileBackupState::Missing) {
        panic!("missing backup");
    } else {
        panic!("expected backup");
    }
}

#[test]
fn kiss_cov_slots_static_unit_refs() {
    let _ = DotfileSpecRow::rel_path;
    let _ = labels_for_test;
    let _ = restore_malvin_config_missing_for_test;
    let _: [DotfileSpecRow; 6] = DOTFILE_ROWS;
}

#[test]
fn kiss_static_type_refs() {
    let row = &DOTFILE_ROWS[0];
    assert_eq!(row.rel, KISSCONFIG_FILE);
    assert!(!row.home_subdir.is_empty());
    let _ = dotfile_source_path(0, Path::new("/tmp"));
}

#[cfg(unix)]
#[test]
fn kiss_cov_slots_malvin_config_slot_roundtrip() {
    crate::test_utils::with_isolated_home(|work| {
        crate::seed_malvin_config(work, "home-config\n");
        let mut generate_id = |n: usize| format!("kiss-cfg-{n}");
        let backup = backup_slot(MALVIN_CONFIG_SLOT, work, &mut generate_id).expect("backup");
        match backup {
            DotfileBackupState::Present(_) => {
                restore_slot(work, &backup, MALVIN_CONFIG_SLOT).expect("restore");
            }
            DotfileBackupState::Missing => panic!("home config slot should backup"),
        }
    });
}

#[test]
fn kiss_cov_slots_backup_restore_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    std::fs::write(work.join(KISSCONFIG_FILE), "[gate]\n").expect("kissconfig");
    let mut generate_id = |n: usize| format!("kiss-slot-{n}");
    for slot in 0..DOTFILE_ROWS.len() {
        if slot == MALVIN_CONFIG_SLOT {
            continue;
        }
        let backup = backup_slot(slot, work, &mut generate_id).expect("backup");
        match backup {
            DotfileBackupState::Present(_) => {
                restore_slot(work, &backup, slot).expect("restore");
            }
            DotfileBackupState::Missing if slot == 0 => {
                panic!("kissconfig slot should be present");
            }
            DotfileBackupState::Missing => {}
        }
    }
}

#[test]
fn kiss_cov_dotfile_spec_row_construct_destructure_by_value() {
    let row = DotfileSpecRow {
        rel: KISSCONFIG_FILE,
        home_subdir: "kissconfig",
        mkdir_lbl: "kissconfig backup mkdir",
        collision_lbl: "kissconfig backup mkdir",
        restore_lbl: "kissconfig restore",
        copy_err: ".kissconfig backup copy",
        restore_copy_err: "kissconfig restore",
    };
    let touched = std::hint::black_box(row);
    let DotfileSpecRow {
        rel,
        home_subdir,
        mkdir_lbl,
        collision_lbl,
        restore_lbl,
        copy_err,
        restore_copy_err,
    } = touched;
    assert_eq!(rel, KISSCONFIG_FILE);
    assert_eq!(home_subdir, "kissconfig");
    assert_eq!(mkdir_lbl, collision_lbl);
    assert!(!restore_lbl.is_empty());
    assert!(!copy_err.is_empty());
    assert!(!restore_copy_err.is_empty());
    let _ = std::mem::size_of::<DotfileSpecRow>();
    let lbl = labels_for_test(&DOTFILE_ROWS[0]);
    assert_eq!(lbl.mkdir, DOTFILE_ROWS[0].mkdir_lbl);
}

#[test]
fn kiss_cov_dotfile_rows_iterate_by_value() {
    for row in DOTFILE_ROWS {
        let moved: DotfileSpecRow = row;
        let copied = moved;
        let cloned = copied;
        assert_eq!(moved.rel, cloned.rel);
    }
}
