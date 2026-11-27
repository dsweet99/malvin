use std::time::{Duration, Instant};

const OPEN_LINE_HOLD: Duration = Duration::from_secs(10);

#[cfg(not(test))]
static OPEN_LINE_SINCE: std::sync::Mutex<Option<Instant>> = std::sync::Mutex::new(None);

#[cfg(not(test))]
fn with_open_line<R>(f: impl FnOnce(&mut Option<Instant>) -> R) -> R {
    let mut guard = OPEN_LINE_SINCE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&mut guard)
}

#[cfg(test)]
thread_local! {
    static OPEN_LINE_SINCE: std::cell::RefCell<Option<Instant>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn with_open_line<R>(f: impl FnOnce(&mut Option<Instant>) -> R) -> R {
    OPEN_LINE_SINCE.with(|cell| f(&mut cell.borrow_mut()))
}

pub(crate) fn set_agent_text_line_open(open: bool) {
    with_open_line(|since| *since = open.then(Instant::now));
}

pub(crate) fn agent_text_line_open_at(now: Instant) -> bool {
    with_open_line(|since| since.is_some_and(|t| now.saturating_duration_since(t) < OPEN_LINE_HOLD))
}

#[cfg(test)]
mod tests {
    use super::{OPEN_LINE_HOLD, agent_text_line_open_at, set_agent_text_line_open};
    use std::time::Instant;

    #[test]
    fn open_line_expires_after_hold() {
        set_agent_text_line_open(true);
        let now = Instant::now();
        assert!(agent_text_line_open_at(now));
        assert!(!agent_text_line_open_at(now + OPEN_LINE_HOLD));
        set_agent_text_line_open(false);
        assert!(!agent_text_line_open_at(Instant::now()));
    }
}
