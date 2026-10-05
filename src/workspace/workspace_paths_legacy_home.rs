use std::path::Path;

use super::MALVIN_USER_HOME_DIR;

pub const LEGACY_MALVIN_USER_HOME_DIR: &str = ".malvin_home";

#[derive(Debug, PartialEq, Eq)]
pub enum LegacyHomeMigration {
    NotNeeded,
    Moved,
    Merged,
}

pub fn migrate_legacy_malvin_user_home() {
    let home = crate::user_home_dir();
    let (old, new) = (LEGACY_MALVIN_USER_HOME_DIR, MALVIN_USER_HOME_DIR);
    match migrate_legacy_malvin_user_home_in(&home) {
        Ok(LegacyHomeMigration::Moved) => {
            eprintln!("malvin: moved ~/{old} to ~/{new} (the old path is now a symlink)");
        }
        Ok(LegacyHomeMigration::Merged) => {
            eprintln!("malvin: merged ~/{old} into ~/{new} (the old path is now a symlink)");
        }
        Ok(LegacyHomeMigration::NotNeeded) => {}
        Err(e) => eprintln!("malvin: could not move ~/{old} to ~/{new}: {e}"),
    }
}

fn is_real_dir(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|m| m.is_dir())
}

pub fn migrate_legacy_malvin_user_home_in(home: &Path) -> std::io::Result<LegacyHomeMigration> {
    let legacy = home.join(LEGACY_MALVIN_USER_HOME_DIR);
    let current = home.join(MALVIN_USER_HOME_DIR);
    if !is_real_dir(&legacy) {
        return Ok(LegacyHomeMigration::NotNeeded);
    }
    let outcome = if current.symlink_metadata().is_err() {
        if let Err(e) = std::fs::rename(&legacy, &current) {
            return if is_real_dir(&legacy) {
                Err(e)
            } else {
                Ok(LegacyHomeMigration::NotNeeded)
            };
        }
        LegacyHomeMigration::Moved
    } else {
        merge_dir_into(&legacy, &current)?;
        remove_empty_dir_tree(&legacy)?;
        LegacyHomeMigration::Merged
    };
    #[cfg(unix)]
    std::os::unix::fs::symlink(MALVIN_USER_HOME_DIR, &legacy)?;
    Ok(outcome)
}

fn merge_dir_into(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let from = entry?.path();
        let to = dst.join(from.file_name().unwrap_or_default());
        if is_real_dir(&from) && is_real_dir(&to) {
            merge_dir_into(&from, &to)?;
        } else {
            std::fs::rename(&from, &to)?;
        }
    }
    Ok(())
}

fn remove_empty_dir_tree(dir: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if is_real_dir(&path) {
            remove_empty_dir_tree(&path)?;
        }
    }
    std::fs::remove_dir(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_legacy(home: &Path) -> std::path::PathBuf {
        let legacy = home.join(LEGACY_MALVIN_USER_HOME_DIR);
        std::fs::create_dir_all(legacy.join("logs").join("hash_a").join("run_1")).unwrap();
        std::fs::write(legacy.join("config.toml"), "x = 1\n").unwrap();
        legacy
    }

    #[cfg(unix)]
    fn assert_old_path_links_to_new(home: &Path) {
        let legacy = home.join(LEGACY_MALVIN_USER_HOME_DIR);
        assert!(legacy.symlink_metadata().unwrap().file_type().is_symlink());
        std::fs::write(legacy.join("logs").join("via_old_path"), "").unwrap();
        assert!(
            home.join(MALVIN_USER_HOME_DIR)
                .join("logs")
                .join("via_old_path")
                .is_file()
        );
    }

    fn moves_when_new_dir_is_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        assert_eq!(
            migrate_legacy_malvin_user_home_in(home).unwrap(),
            LegacyHomeMigration::NotNeeded
        );
        assert!(!home.join(MALVIN_USER_HOME_DIR).exists());
        seed_legacy(home);
        assert_eq!(
            migrate_legacy_malvin_user_home_in(home).unwrap(),
            LegacyHomeMigration::Moved
        );
        let current = home.join(MALVIN_USER_HOME_DIR);
        assert_eq!(
            std::fs::read_to_string(current.join("config.toml")).unwrap(),
            "x = 1\n"
        );
        assert!(current.join("logs").join("hash_a").join("run_1").is_dir());
        #[cfg(unix)]
        assert_old_path_links_to_new(home);
        assert_eq!(
            migrate_legacy_malvin_user_home_in(home).unwrap(),
            LegacyHomeMigration::NotNeeded
        );
    }

    fn merges_into_existing_new_dir_with_legacy_files_winning() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        seed_legacy(home);
        let current = home.join(MALVIN_USER_HOME_DIR);
        std::fs::create_dir_all(current.join("logs").join("hash_a").join("run_2")).unwrap();
        std::fs::write(current.join("config.toml"), "x = 0\n").unwrap();
        assert_eq!(
            migrate_legacy_malvin_user_home_in(home).unwrap(),
            LegacyHomeMigration::Merged
        );
        assert_eq!(
            std::fs::read_to_string(current.join("config.toml")).unwrap(),
            "x = 1\n"
        );
        assert!(current.join("logs").join("hash_a").join("run_1").is_dir());
        assert!(current.join("logs").join("hash_a").join("run_2").is_dir());
        #[cfg(unix)]
        assert_old_path_links_to_new(home);
    }

    fn remove_empty_dir_tree_keeps_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("d");
        std::fs::create_dir_all(dir.join("a").join("b")).unwrap();
        std::fs::write(dir.join("a").join("late_write"), "").unwrap();
        assert!(remove_empty_dir_tree(&dir).is_err());
        assert!(dir.join("a").join("late_write").is_file());
        assert!(!dir.join("a").join("b").exists());
    }

    #[test]
    fn kiss_bundled_legacy_home_migration_tests() {
        moves_when_new_dir_is_absent();
        merges_into_existing_new_dir_with_legacy_files_winning();
        remove_empty_dir_tree_keeps_files();
        let _ = migrate_legacy_malvin_user_home;
    }
}
