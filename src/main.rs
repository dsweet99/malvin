#![allow(clippy::multiple_crate_versions)]

fn main() -> malvin::Exit {
    if let Some(exit) = malvin::task_summary::run_if_worker() {
        return exit;
    }
    malvin::run_timing::note_process_start();
    let exit = malvin::entrypoint();
    malvin::run_timing::emit_process_footnotes_if_armed();
    exit
}
