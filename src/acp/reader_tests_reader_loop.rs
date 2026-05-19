#[cfg(unix)]
mod unix {
    use crate::acp::reader_tests_helpers::reader_loop_eof_pending_error;

    #[tokio::test]
    async fn test_reader_loop_drains_pending_on_stdout_eof() {
        let err = reader_loop_eof_pending_error().await;
        assert!(err.contains("closed") || err.contains("acp"));
    }
}
