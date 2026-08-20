use std::io::Write;
use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::format_logs_dir;
use crate::mem_limit_config::format_host_resources_line;
use crate::output::{MALVIN_WHO, WHO_U, format_line, print_stdout_line, print_stdout_text};

pub fn emit_command_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    crate::init_from_env();
    let cmd = crate::command_line().expect("init_from_env populates argv via OnceLock");
    let line = format!("Command: {cmd}");
    if echo_stdout {
        print_stdout_line(WHO_U, &line);
        let _ = std::io::stdout().flush();
    }
    let log_path = run_dir.join("command.log");
    std::fs::write(&log_path, format!("{}\n", format_line(WHO_U, &line)))
        .map_err(|e| format!("command.log: {e}"))?;
    Ok(())
}

pub fn echo_primary_to_stdout(plan_path: &Path, echo_plain: bool) -> Result<(), String> {
    if !echo_plain {
        return Ok(());
    }
    let plan_text = std::fs::read_to_string(plan_path).map_err(|e| e.to_string())?;
    print_stdout_text(WHO_U, &plan_text);
    Ok(())
}

pub fn emit_host_resources_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    let line = format_host_resources_line();
    append_command_log_line(run_dir, echo_stdout, &line)
}

#[must_use]
pub fn format_model_line(model: &str) -> String {
    format!("Model: {model}")
}

fn append_command_log_line(run_dir: &Path, echo_stdout: bool, line: &str) -> Result<(), String> {
    if echo_stdout {
        print_stdout_line(WHO_U, line);
    }
    let log_path = run_dir.join("command.log");
    let formatted = format!("{}\n", format_line(WHO_U, line));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("command.log: {e}"))?;
    file.write_all(formatted.as_bytes())
        .map_err(|e| format!("command.log: {e}"))?;
    Ok(())
}

pub struct RunStartupEmitOpts {
    pub tee_stdout: bool,
    pub host_resources: bool,
    pub model: String,
}

impl RunStartupEmitOpts {
    #[must_use]
    pub fn from_shared(shared: &super::SharedOpts, host_resources: bool) -> Self {
        Self {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources,
            model: shared.model.canonical(),
        }
    }
}

pub fn emit_run_startup_banner(
    artifacts: &RunArtifacts,
    opts: RunStartupEmitOpts,
    _cli_request: &str,
) -> Result<(), String> {
    crate::agent_phase::reset_for_run();
    crate::agent_phase::note_orienting();
    emit_command_line(&artifacts.run_dir, opts.tee_stdout)?;
    if opts.host_resources && !crate::acp::test_no_real_agent_enabled() {
        emit_host_resources_line(&artifacts.run_dir, opts.tee_stdout)?;
    }
    append_command_log_line(
        &artifacts.run_dir,
        opts.tee_stdout,
        &format_model_line(&opts.model),
    )?;
    echo_primary_to_stdout(&artifacts.plan_path, opts.tee_stdout)?;
    Ok(())
}

pub fn emit_run_logs_line(artifacts: &RunArtifacts) -> Result<(), String> {
    print_stdout_line(
        MALVIN_WHO,
        &format!("Logs: {}", format_logs_dir(&artifacts.run_dir)?),
    );
    Ok(())
}

pub fn emit_run_startup_sequence(
    artifacts: &RunArtifacts,
    opts: RunStartupEmitOpts,
    cli_request: &str,
) -> Result<(), String> {
    emit_run_startup_banner(artifacts, opts, cli_request)?;
    emit_run_logs_line(artifacts)
}

#[cfg(test)]
mod tests {
    use super::{
        RunStartupEmitOpts, append_command_log_line, emit_host_resources_line, emit_run_logs_line,
        emit_run_startup_banner, emit_run_startup_sequence, format_model_line,
    };
    use crate::output::{WHO_U, format_who_tag_delim};

    #[test]
    fn emit_command_line_uses_user_who_tag() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        super::emit_command_line(&run_dir, false).expect("emit");
        let text = std::fs::read_to_string(run_dir.join("command.log")).expect("read");
        let delim = format_who_tag_delim(WHO_U);
        assert!(
            text.contains(&format!(" {delim}Command: ")),
            "command.log must tag user startup with u|; got {text:?}"
        );
    }

    #[test]
    fn emit_host_resources_line_appends_to_command_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(run_dir.join("command.log"), "existing\n").expect("seed");
        emit_host_resources_line(&run_dir, false).expect("emit");
        let text = std::fs::read_to_string(run_dir.join("command.log")).expect("read");
        assert!(text.contains("existing") && text.contains("Memory:") && text.contains("CPUs:"));
    }

    #[test]
    fn format_model_line_shows_prefixed_model() {
        assert_eq!(
            format_model_line("cursor:composer-2"),
            "Model: cursor:composer-2"
        );
        assert_eq!(
            format_model_line("pi:openai/gpt-4o"),
            "Model: pi:openai/gpt-4o"
        );
    }

    #[test]
    fn append_model_line_uses_user_who_tag() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(run_dir.join("command.log"), "existing\n").expect("seed");
        append_command_log_line(&run_dir, false, &format_model_line("pi:openai/gpt-4o"))
            .expect("emit");
        let text = std::fs::read_to_string(run_dir.join("command.log")).expect("read");
        let delim = format_who_tag_delim(WHO_U);
        assert!(
            text.contains("existing") && text.contains(&format!(" {delim}Model: pi:openai/gpt-4o"))
        );
    }

    #[test]
    fn emit_run_startup_banner_writes_command_without_requiring_logs() {
        crate::test_utils::with_isolated_home(|_| {
            crate::test_utils::clear_test_no_real_agent_env();
            let tmp = tempfile::tempdir().expect("tempdir");
            let artifacts = crate::artifacts::create_run_artifacts_from_text("hi", Some(tmp.path()))
                .expect("art");
            emit_run_startup_banner(
                &artifacts,
                RunStartupEmitOpts {
                    tee_stdout: false,
                    host_resources: true,
                    model: "cursor:composer-2".into(),
                },
                "hi",
            )
            .expect("banner");
            let log = std::fs::read_to_string(artifacts.run_dir.join("command.log"))
                .expect("log");
            assert!(log.contains("Command:") && log.contains("Memory:") && log.contains("Model:"));
            emit_run_logs_line(&artifacts).expect("logs");
        });
    }

    #[test]
    fn emit_run_startup_sequence_includes_host_resources_when_requested() {
        crate::test_utils::with_isolated_home(|_| {
            crate::test_utils::clear_test_no_real_agent_env();
            let tmp = tempfile::tempdir().expect("tempdir");
            let artifacts = crate::artifacts::create_run_artifacts_from_text("hi", Some(tmp.path()))
                .expect("art");
            emit_run_startup_sequence(
                &artifacts,
                RunStartupEmitOpts {
                    tee_stdout: false,
                    host_resources: true,
                    model: "cursor:composer-2".into(),
                },
                "hi",
            )
            .expect("startup");
            let log = std::fs::read_to_string(artifacts.run_dir.join("command.log"))
                .expect("log");
            assert!(log.contains("Command:") && log.contains("Memory:") && log.contains("CPUs:"));
            assert!(log.contains("Model: cursor:composer-2"));
            assert!(log.find("Model:").expect("model") > log.find("Memory:").expect("memory"));
        });
    }

    #[test]
    fn emit_run_startup_sequence_omits_host_resources_when_disabled() {
        crate::test_utils::with_isolated_home(|_| {
            let tmp = tempfile::tempdir().expect("tempdir");
            let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
                .expect("art");
            emit_run_startup_sequence(
                &artifacts,
                RunStartupEmitOpts {
                    tee_stdout: false,
                    host_resources: false,
                    model: "pi:openai/gpt-4o".into(),
                },
                "code",
            )
            .expect("startup");
            let log = std::fs::read_to_string(artifacts.run_dir.join("command.log"))
                .expect("log");
            assert!(log.contains("Command:") && !log.contains("Memory:"));
            assert!(log.contains("Model: pi:openai/gpt-4o"));
        });
    }
}
