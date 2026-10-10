use std::future::Future;

use super::shared_opts::MetaLoopCount;

pub(crate) async fn run_meta_loop<F, Fut>(count: MetaLoopCount, mut body: F) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let mut done = 0u64;
    loop {
        body().await?;
        done = done.saturating_add(1);
        if count.is_complete(done) {
            return Ok(());
        }
    }
}

impl MetaLoopCount {
    const fn is_complete(self, done: u64) -> bool {
        match self {
            Self::Times(n) => done >= n,
            Self::Forever => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_meta_loop;
    use crate::cli::shared_opts::MetaLoopCount;
    use std::cell::Cell;

    #[tokio::test]
    async fn default_count_runs_body_once() {
        let n = Cell::new(0);
        run_meta_loop(MetaLoopCount::Times(1), || {
            n.set(n.get() + 1);
            async { Ok(()) }
        })
        .await
        .expect("ok");
        assert_eq!(n.get(), 1);
    }

    #[tokio::test]
    async fn finite_count_runs_body_n_times() {
        let n = Cell::new(0);
        run_meta_loop(MetaLoopCount::Times(3), || {
            n.set(n.get() + 1);
            async { Ok(()) }
        })
        .await
        .expect("ok");
        assert_eq!(n.get(), 3);
    }

    #[tokio::test]
    async fn forever_repeats_until_body_errors() {
        let n = Cell::new(0);
        let err = run_meta_loop(MetaLoopCount::Forever, || {
            let i = n.get() + 1;
            n.set(i);
            async move {
                if i >= 3 {
                    Err("stop".to_string())
                } else {
                    Ok(())
                }
            }
        })
        .await;
        assert_eq!(err, Err("stop".to_string()));
        assert_eq!(n.get(), 3);
    }

    #[tokio::test]
    async fn error_stops_before_the_count_is_reached() {
        let n = Cell::new(0);
        let err = run_meta_loop(MetaLoopCount::Times(5), || {
            let i = n.get() + 1;
            n.set(i);
            async move {
                if i >= 2 {
                    Err("stop".to_string())
                } else {
                    Ok(())
                }
            }
        })
        .await;
        assert_eq!(err, Err("stop".to_string()));
        assert_eq!(n.get(), 2);
    }
}
