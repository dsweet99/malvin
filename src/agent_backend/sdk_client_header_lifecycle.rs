use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct CoderSessionHeader {
    pub prompt: String,
    pub log_path: PathBuf,
    pub stdout_label: String,
    pub log_who: String,
}

#[derive(Clone, Debug)]
pub(crate) enum SessionHeaderLifecycle {
    Unbound,
    Pending(CoderSessionHeader),
    Satisfied(CoderSessionHeader),
}

impl SessionHeaderLifecycle {
    pub(crate) fn bind(&mut self, header: CoderSessionHeader) {
        *self = Self::Pending(header);
    }

    pub(crate) fn mark_satisfied_keeping_header(&mut self) {
        *self = match std::mem::replace(self, Self::Unbound) {
            Self::Pending(h) | Self::Satisfied(h) => Self::Satisfied(h),
            Self::Unbound => Self::Unbound,
        };
    }

    pub(crate) fn mark_fresh_spawn(&mut self) {
        *self = match std::mem::replace(self, Self::Unbound) {
            Self::Pending(h) | Self::Satisfied(h) => Self::Pending(h),
            Self::Unbound => Self::Unbound,
        };
    }

    pub(crate) fn require_redelivery(&mut self) {
        self.mark_fresh_spawn();
    }

    #[must_use]
    pub(crate) const fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied(_))
    }

    #[must_use]
    pub(crate) const fn is_unbound(&self) -> bool {
        matches!(self, Self::Unbound)
    }

    #[must_use]
    pub(crate) const fn pending_header(&self) -> Option<&CoderSessionHeader> {
        match self {
            Self::Pending(h) => Some(h),
            Self::Satisfied(_) | Self::Unbound => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CoderSessionHeader, SessionHeaderLifecycle};
    use std::path::PathBuf;

    fn sample_header(label: &str) -> CoderSessionHeader {
        CoderSessionHeader {
            prompt: format!("prompt-{label}"),
            log_path: PathBuf::from(format!("/tmp/{label}.log")),
            stdout_label: label.to_string(),
            log_who: "header".into(),
        }
    }

    #[test]
    fn bind_after_satisfied_reenters_pending() {
        let mut life = SessionHeaderLifecycle::Unbound;
        life.bind(sample_header("first"));
        life.mark_satisfied_keeping_header();
        assert!(life.is_satisfied());
        life.bind(sample_header("second"));
        assert!(!life.is_satisfied());
        assert_eq!(
            life.pending_header().map(|h| h.stdout_label.as_str()),
            Some("second")
        );
    }

    #[test]
    fn require_redelivery_promotes_satisfied_header_to_pending() {
        let mut life = SessionHeaderLifecycle::Unbound;
        life.bind(sample_header("keep"));
        life.mark_satisfied_keeping_header();
        life.require_redelivery();
        assert_eq!(
            life.pending_header().map(|h| h.stdout_label.as_str()),
            Some("keep")
        );
    }

    #[test]
    fn mark_satisfied_from_unbound_stays_unbound() {
        let mut life = SessionHeaderLifecycle::Unbound;
        life.mark_satisfied_keeping_header();
        assert!(life.is_unbound());
        assert!(!life.is_satisfied());
    }

    #[test]
    fn fresh_spawn_from_satisfied_reenters_pending_with_same_header() {
        let mut life = SessionHeaderLifecycle::Unbound;
        life.bind(sample_header("spawn"));
        life.mark_satisfied_keeping_header();
        life.mark_fresh_spawn();
        assert_eq!(
            life.pending_header().map(|h| h.stdout_label.as_str()),
            Some("spawn")
        );
    }
}
