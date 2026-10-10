#![allow(clippy::multiple_crate_versions)]

fn main() -> malvin::Exit {
    malvin::run_timing::note_process_start();
    let exit = malvin::entrypoint();
    malvin::run_timing::emit_process_footnotes_if_armed();
    exit
}
