use crate::self_worker::{Worker, invocation};

#[must_use]
pub fn run_if_worker() -> Option<crate::Exit> {
    let (worker, args) = invocation(std::env::args_os().skip(1))?;
    let result = match worker {
        Worker::Summary => crate::task_summary::run_worker(args),
        Worker::FactAgent => crate::fact_worker::run_worker(args),
        Worker::HerdrLinger => crate::herdr::run_linger_worker(args),
        Worker::LocalLlmManager => crate::local_llm::run_local_llm_manager(),
    };
    Some(match result {
        Ok(()) => crate::Exit::Success,
        Err(error) => {
            eprintln!("{}: {error}", worker.label());
            crate::Exit::Failure
        }
    })
}
