pub mod session_inner {
    include!("session_inner.rs");
}

include!("smoke.rs");
include!("cancel.rs");

#[cfg(unix)]
mod unix_helpers {
    include!("unix_helpers.rs");
}
#[cfg(unix)]
include!("unix_shutdown.rs");
#[cfg(all(unix, target_os = "linux"))]
include!("linux_spawn_abort.rs");
