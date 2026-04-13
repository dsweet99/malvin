//! Non-Linux / non-macOS: no OS backend; delegates to `ChildHealth::cannot_sample`.

use super::ChildHealth;

#[must_use]
pub(super) fn sample_child_health(_pid: u32) -> ChildHealth {
    ChildHealth::cannot_sample()
}
