pub(super) const LEGACY_MINI_HINT: &str =
    "legacy `mini:` prefix removed; use `rpi:` (for example `rpi:openrouter/<slug>`)";
pub(super) const LEGACY_OPENROUTER_HINT: &str =
    "legacy `openrouter:` prefix removed; use `rpi:openrouter/<slug>`";
pub(super) const LEGACY_LOCAL_HINT: &str =
    "legacy `local:` prefix removed; local GGUF models are no longer supported";
pub(super) const LEGACY_PRIME_HINT: &str =
    "legacy `prime:` prefix removed; use `cursor:`, `pi:`, or `rpi:`";

use super::{LOCAL_PREFIX, MINI_PREFIX, OPENROUTER_PREFIX, PRIME_PREFIX, UNPREFIXED_MODEL_MESSAGE};

pub(super) fn legacy_or_unprefixed_error(raw: &str) -> String {
    if raw.starts_with(PRIME_PREFIX) {
        return LEGACY_PRIME_HINT.to_string();
    }
    if raw.starts_with(MINI_PREFIX) {
        return LEGACY_MINI_HINT.to_string();
    }
    if raw.starts_with(OPENROUTER_PREFIX) {
        return LEGACY_OPENROUTER_HINT.to_string();
    }
    if raw.starts_with(LOCAL_PREFIX) {
        return LEGACY_LOCAL_HINT.to_string();
    }
    UNPREFIXED_MODEL_MESSAGE.to_string()
}
