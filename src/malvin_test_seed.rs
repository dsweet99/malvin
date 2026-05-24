//! Test helpers for `.malvin/checks` (keeps `test_utils` under kiss line limits).

#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub fn seed_malvin_checks(work: &Path, content: &str) {
    std::fs::create_dir_all(work.join(crate::MALVIN_DIR)).expect("mkdir .malvin");
    std::fs::write(crate::malvin_checks_path(work), content).expect("write .malvin/checks");
}
