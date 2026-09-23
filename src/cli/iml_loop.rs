use std::future::Future;

pub(crate) async fn run_with_iml<F, Fut>(iml: bool, mut body: F) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    loop {
        body().await?;
        if !iml {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_with_iml;
    use std::cell::Cell;

    #[tokio::test]
    async fn without_iml_runs_body_once() {
        let n = Cell::new(0);
        run_with_iml(false, || {
            n.set(n.get() + 1);
            async { Ok(()) }
        })
        .await
        .expect("ok");
        assert_eq!(n.get(), 1);
    }

    #[tokio::test]
    async fn with_iml_repeats_until_body_errors() {
        let n = Cell::new(0);
        let err = run_with_iml(true, || {
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
}
