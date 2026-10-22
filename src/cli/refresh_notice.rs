use malvin::output::{MALVIN_WHO, print_stdout_line};

pub(crate) const REFRESHING_STDOUT: &str = "Refreshing...";

pub(crate) fn print_refreshing() {
    print_stdout_line(MALVIN_WHO, REFRESHING_STDOUT);
}
