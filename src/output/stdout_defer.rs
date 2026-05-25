use std::sync::OnceLock;

type DeferTaggedStdoutFn = fn(&str, &str) -> bool;
type DeferPushFn = fn(String) -> bool;

static DEFER_TAGGED_STDOUT: OnceLock<DeferTaggedStdoutFn> = OnceLock::new();
static DEFER_PUSH: OnceLock<DeferPushFn> = OnceLock::new();

pub(crate) fn register_defer_stdout_hooks(tagged: DeferTaggedStdoutFn, push: DeferPushFn) {
    let _ = DEFER_TAGGED_STDOUT.set(tagged);
    let _ = DEFER_PUSH.set(push);
}

pub(crate) fn try_defer_tagged_stdout(display: &str, log: &str) -> bool {
    DEFER_TAGGED_STDOUT
        .get()
        .is_some_and(|hook| hook(display, log))
}

pub(crate) fn try_defer_push_line(line: String) -> bool {
    DEFER_PUSH.get().is_some_and(|hook| hook(line))
}
