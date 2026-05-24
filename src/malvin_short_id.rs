//! Shared short ids: `M` + five lowercase alphanumeric characters.

use crate::alnum_id::random_alnum;

pub const MALVIN_SHORT_ID_LEN: usize = 6;

#[must_use]
pub fn malvin_short_id() -> String {
    format!("M{}", random_alnum(5))
}

pub fn validate_malvin_short_id(id: &str) -> Result<(), String> {
    if is_valid_malvin_short_id(id) {
        Ok(())
    } else {
        Err(format!(
            "invalid id {id:?}: expected M followed by 5 lowercase letters or digits (example: Ma3bx9)"
        ))
    }
}

#[must_use]
pub fn is_valid_malvin_short_id(id: &str) -> bool {
    id.len() == MALVIN_SHORT_ID_LEN
        && id.as_bytes().first() == Some(&b'M')
        && id.as_bytes()[1..]
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::{
        is_valid_malvin_short_id, malvin_short_id, validate_malvin_short_id, MALVIN_SHORT_ID_LEN,
    };

    #[test]
    fn malvin_short_id_format() {
        let id = malvin_short_id();
        assert_eq!(id.len(), MALVIN_SHORT_ID_LEN);
        assert!(is_valid_malvin_short_id(&id));
    }

    #[test]
    fn validate_rejects_bad_ids() {
        assert!(validate_malvin_short_id("Ma3bx9").is_ok());
        assert!(validate_malvin_short_id("ma3bx9").is_err());
        assert!(validate_malvin_short_id("Ma3bx").is_err());
        assert!(validate_malvin_short_id("Ma3bx99").is_err());
    }
}
