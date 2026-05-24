use std::path::Path;

use super::BugLogMatch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MalvinRunLogKind {
    Bug,
    Kpop,
}

impl MalvinRunLogKind {
    const fn log_tag(self) -> &'static str {
        match self {
            Self::Bug => "BUG_LOG",
            Self::Kpop => "KPOP_LOG",
        }
    }

    const fn id_only_tag(self) -> Option<&'static str> {
        match self {
            Self::Bug => Some("BUG_ID"),
            Self::Kpop => None,
        }
    }

    pub(super) const fn missing_log_err_label(self) -> &'static str {
        match self {
            Self::Bug => "BUG_LOG",
            Self::Kpop => "KPOP_LOG",
        }
    }

    pub(super) const fn fallback_err_label(self) -> &'static str {
        match self {
            Self::Bug => "BUG_ID",
            Self::Kpop => "KPOP",
        }
    }
}

pub(super) fn malvin_log_tag_marker() -> String {
    format!(
        "[{}] ",
        crate::output::format_log_tag_inner(crate::output::MALVIN_WHO)
    )
}

pub(super) fn malvin_tagged_line_payload(line: &str) -> Option<&str> {
    let marker = malvin_log_tag_marker();
    let idx = line.find(marker.as_str())?;
    Some(line[idx + marker.len()..].trim_start())
}

pub(super) fn parse_log_line(text: &str, id: &str, kind: MalvinRunLogKind) -> Option<String> {
    let needle = format!("{}: {id} ", kind.log_tag());
    for line in text.lines() {
        let Some(payload) = malvin_tagged_line_payload(line) else {
            continue;
        };
        let Some(idx) = payload.find(needle.as_str()) else {
            continue;
        };
        let rest = payload[idx + needle.len()..].trim();
        if rest.is_empty() || rest.contains(' ') {
            continue;
        }
        return Some(rest.to_string());
    }
    None
}

pub(super) fn line_has_id_only(text: &str, id: &str, kind: MalvinRunLogKind) -> bool {
    kind.id_only_tag().is_some_and(|id_only_tag| {
        let log_line = format!("{}: {id} ", kind.log_tag());
        let id_line = format!("{id_only_tag}: {id}");
        for line in text.lines() {
            let Some(payload) = malvin_tagged_line_payload(line) else {
                continue;
            };
            if payload.contains(log_line.as_str()) {
                return false;
            }
            if payload.contains(id_line.as_str()) {
                return true;
            }
        }
        false
    })
}

pub(super) fn match_run_logs(
    run_dir: &Path,
    id: &str,
    kind: MalvinRunLogKind,
) -> Option<BugLogMatch> {
    for name in ["stdout.log", "command.log"] {
        let log_path = run_dir.join(name);
        let Ok(text) = std::fs::read_to_string(&log_path) else {
            continue;
        };
        if let Some(exp_log_rel) = parse_log_line(&text, id, kind) {
            return Some(BugLogMatch {
                run_dir: run_dir.to_path_buf(),
                log_file: log_path,
                exp_log_rel: Some(exp_log_rel),
            });
        }
        if line_has_id_only(&text, id, kind) {
            return Some(BugLogMatch {
                run_dir: run_dir.to_path_buf(),
                log_file: log_path,
                exp_log_rel: None,
            });
        }
    }
    None
}

#[test]
fn match_run_logs_finds_bug_id_only_line() {
    use crate::output::{format_log_tag_inner, MALVIN_WHO};
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let id = "Ma1b2c";
    let line = format!(
        "20260101.000000.000 [{}] BUG_ID: {id}\n",
        format_log_tag_inner(MALVIN_WHO)
    );
    std::fs::write(run_dir.join("stdout.log"), line).expect("write");
    let m = match_run_logs(&run_dir, id, MalvinRunLogKind::Bug).expect("match");
    assert_eq!(m.run_dir, run_dir);
}

#[test]
fn match_run_logs_reads_command_log_for_kpop() {
    use crate::output::{format_log_tag_inner, MALVIN_WHO};
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let id = "Mcmd01";
    let line = format!(
        "20260101.000000.000 [{}] KPOP_LOG: {id} ./exp.md\n",
        format_log_tag_inner(MALVIN_WHO)
    );
    std::fs::write(run_dir.join("command.log"), line).expect("write");
    let m = match_run_logs(&run_dir, id, MalvinRunLogKind::Kpop).expect("match");
    assert_eq!(m.exp_log_rel.as_deref(), Some("./exp.md"));
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{
        line_has_id_only, malvin_log_tag_marker, malvin_tagged_line_payload, match_run_logs,
        parse_log_line, MalvinRunLogKind,
    };

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = MalvinRunLogKind::Bug;
        let _ = malvin_log_tag_marker;
        let _ = malvin_tagged_line_payload;
        let _ = parse_log_line;
        let _ = line_has_id_only;
        let _ = match_run_logs;
    }
}
