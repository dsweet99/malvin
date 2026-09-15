pub fn format_workspace_gate_failure(command: &str, detail: &str) -> String {
    format!(
        "ERR: Workspace checks did not pass; the next step did not run.\n\
Run `malvin -g`, then retry `{command}`.\n\
\n\
{detail}"
    )
}
