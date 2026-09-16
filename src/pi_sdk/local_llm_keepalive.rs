use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::local_llm_client::touch_via_manager;
use super::local_llm_paths::idle_duration;

const SLEEP_SLICE: Duration = Duration::from_millis(50);

struct Keepalive {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

static KEEPALIVE: OnceLock<Mutex<Option<Keepalive>>> = OnceLock::new();

fn slot() -> &'static Mutex<Option<Keepalive>> {
    KEEPALIVE.get_or_init(|| Mutex::new(None))
}

fn keepalive_period() -> Duration {
    let idle = idle_duration();
    let third = idle / 3;
    if third.is_zero() {
        Duration::from_millis(200)
    } else {
        third.min(Duration::from_mins(1))
    }
}

fn already_running() -> bool {
    slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some()
}

fn sleep_until_stop_or(stop: &AtomicBool, budget: Duration) -> bool {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if stop.load(Ordering::SeqCst) {
            return true;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        thread::sleep(remaining.min(SLEEP_SLICE));
    }
    stop.load(Ordering::SeqCst)
}

fn spawn_worker(stop: Arc<AtomicBool>, period: Duration) -> Option<JoinHandle<()>> {
    thread::Builder::new()
        .name("malvin-local-llm-keepalive".into())
        .spawn(move || {
            while !sleep_until_stop_or(&stop, period) {
                let _ = touch_via_manager();
            }
        })
        .ok()
}

fn install_or_abort(stop: Arc<AtomicBool>, join: Option<JoinHandle<()>>) {
    let mut guard = slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.is_some() {
        stop.store(true, Ordering::SeqCst);
        if let Some(j) = join {
            let _ = j.join();
        }
        return;
    }
    *guard = Some(Keepalive { stop, join });
}

pub(crate) fn start_keepalive() {
    if already_running() {
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let join = spawn_worker(Arc::clone(&stop), keepalive_period());
    install_or_abort(stop, join);
}

pub(crate) fn stop_keepalive() {
    let Some(mut ka) = slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
    else {
        return;
    };
    ka.stop.store(true, Ordering::SeqCst);
    if let Some(join) = ka.join.take() {
        let _ = join.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::with_env;

    #[test]
    fn keepalive_period_is_positive() {
        assert!(keepalive_period() > Duration::ZERO);
    }

    #[test]
    fn stop_keepalive_returns_quickly() {
        let _guard = crate::pi_sdk::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        with_env("MALVIN_TIME_SINCE_LAST_CALL_SECONDS", Some("60"), || {
            start_keepalive();
            thread::sleep(Duration::from_millis(20));
            let t0 = Instant::now();
            stop_keepalive();
            assert!(
                t0.elapsed() < Duration::from_millis(500),
                "stop must not wait out the full period"
            );
        });
    }
}
