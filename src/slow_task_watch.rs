use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const SLOW_TASK_THRESHOLD: Duration = Duration::from_mins(5);
pub const SLOW_TASK_POLL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Default)]
pub struct WatchedPromptFlag(Arc<AtomicBool>);

impl WatchedPromptFlag {
    pub fn set(&self, active: bool) {
        self.0.store(active, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub struct SlowTaskWatch(tokio::task::JoinHandle<()>);

impl Drop for SlowTaskWatch {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub fn start<F>(timing: (Duration, Duration), flag: WatchedPromptFlag, launch: F) -> SlowTaskWatch
where
    F: FnOnce() + Send + 'static,
{
    let (threshold, poll) = timing;
    SlowTaskWatch(tokio::spawn(async move {
        tokio::time::sleep(threshold).await;
        while !flag.is_set() {
            tokio::time::sleep(poll).await;
        }
        launch();
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    const TICK: Duration = Duration::from_millis(5);

    fn counter_launch(count: &Arc<AtomicUsize>) -> impl FnOnce() + Send + 'static + use<> {
        let count = Arc::clone(count);
        move || {
            count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn launches_once_after_threshold_when_flag_set() {
        let count = Arc::new(AtomicUsize::new(0));
        let flag = WatchedPromptFlag::default();
        flag.set(true);
        let _watch = start((TICK * 4, TICK), flag, counter_launch(&count));
        tokio::time::sleep(TICK * 2).await;
        assert_eq!(count.load(Ordering::SeqCst), 0, "before threshold");
        tokio::time::sleep(TICK * 20).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn waits_for_flag_after_threshold() {
        let count = Arc::new(AtomicUsize::new(0));
        let flag = WatchedPromptFlag::default();
        let _watch = start((TICK, TICK), flag.clone(), counter_launch(&count));
        tokio::time::sleep(TICK * 10).await;
        assert_eq!(count.load(Ordering::SeqCst), 0, "flag not set");
        flag.set(true);
        tokio::time::sleep(TICK * 10).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn dropping_the_watch_cancels_it() {
        let count = Arc::new(AtomicUsize::new(0));
        let flag = WatchedPromptFlag::default();
        flag.set(true);
        drop(start((TICK * 4, TICK), flag, counter_launch(&count)));
        tokio::time::sleep(TICK * 20).await;
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn threshold_is_five_minutes() {
        assert_eq!(SLOW_TASK_THRESHOLD, Duration::from_secs(300));
        assert!(SLOW_TASK_POLL < SLOW_TASK_THRESHOLD);
    }
}
