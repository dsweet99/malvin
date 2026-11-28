pub fn print_outgoing_prompt_log(_trace_who: &str, bracket_label: &str) {
    for name in prompt_file_stems(bracket_label) {
        super::stdout_display::print_stdout_line(super::who_tag::WHO_P, name);
    }
}

fn prompt_file_stems(label: &str) -> impl Iterator<Item = &str> {
    label.split('+').filter_map(prompt_file_stem)
}

fn prompt_file_stem(part: &str) -> Option<&str> {
    let stem = part.strip_suffix(".md").unwrap_or(part);
    (!stem.is_empty()).then_some(stem)
}

pub fn append_outgoing_prompt_log_lines(body: &str) {
    let ts = crate::time_format::timestamp_now_string();
    for line in super::stdout_format::logical_lines(body) {
        super::style::append_stdout_log_line(&super::stdout_log_pair::tagged_log_line(
            &ts,
            super::who_tag::WHO_U,
            line,
        ));
    }
}

pub fn init_stdout_style() {
    apply_stdout_style(std::env::var_os("NO_COLOR").is_none());
}

fn apply_stdout_style(use_color: bool) {
    super::style::set_log_use_color(use_color);
    super::stdout_heartbeat::spawn_wall_clock_poller_if_needed();
}

#[cfg(test)]
pub(crate) fn init_stdout_style_for_test(use_color: bool) {
    apply_stdout_style(use_color);
}
