//! Regression guards for CLI kiss path strings (kept separate for line budget).

const SRC: &str = concat!(
    include_str!("cli_cross_cov.rs"),
    include_str!("cli_cross_cov_kiss.rs"),
);

const CROSS_REFS_VALUE_SYMBOLS: &[&str] = &[
    "entrypoint",
    "run_code",
    "run_do",
    "run_tidy",
    "run_plan",
];

const DUPLICATE_SUPER_AND_CRATE_CLI_SYMBOLS: &[(&str, &str)] = &[
    ("stringify!(super::args::KpopArgs)", "stringify!(crate::cli::KpopArgs)"),
    (
        "stringify!(super::prepare_prompt_store)",
        "stringify!(crate::cli::prepare_prompt_store)",
    ),
    (
        "stringify!(super::run_emit::emit_run_startup_sequence)",
        "stringify!(crate::cli::run_emit::emit_run_startup_sequence)",
    ),
    (
        "stringify!(super::shared_opts::GlobalOpts)",
        "stringify!(crate::cli::shared_opts::GlobalOpts)",
    ),
    ("stringify!(super::exit::Exit)", "stringify!(crate::cli::Exit)"),
    ("stringify!(super::PlanArgs)", "stringify!(crate::cli::PlanArgs)"),
    (
        "stringify!(super::kpop_flow::kpop_run_acp_multiturn)",
        "stringify!(crate::cli::kpop_flow::kpop_run_acp_multiturn)",
    ),
    (
        "stringify!(crate::cli::shared_opts::SharedOpts)",
        "stringify!(crate::cli::SharedOpts)",
    ),
];

const INVALID_STRINGIFY_PATH_SUBSTRINGS: &[&str] = &[
    "stringify!(super::do_flow_prompt::",
    "stringify!(super::do_flow::",
    "stringify!(super::kpop_flow::into_bug_followup_artifacts)",
    "stringify!(crate::repo_checks::gate_run::append_quality_gates_log_text)",
    "stringify!(crate::repo_checks::gate_run::append_quality_gates_log_line)",
    "stringify!(super::mid_session_gates::mid_session_post_run_tidy::prepare)",
    "stringify!(crate::repo_checks::tests_gates_unix::scan_for_extension_handles_symlink_cycles)",
    "stringify!(crate::repo_checks::tests_gates_common::",
];

#[test]
fn cli_cross_cov_must_not_reference_phantom_prepare_code_run() {
    for line in SRC.lines() {
        assert!(
            !(line.contains("stringify!") && line.contains("prepare_code_run")),
            "kiss coverage refs must reference real symbols, not phantom prepare_code_run: {line}"
        );
    }
}

#[test]
fn cli_cross_cov_must_stringify_gate_run_scan_symlink_cycles() {
    assert!(
        SRC.contains("gate_run::scan_for_extension_handles_symlink_cycles"),
        "cli_cross_cov must reference production gate_run::scan_for_extension_handles_symlink_cycles"
    );
}

fn line_stringifies_exact_symbol(line: &str, sym: &str) -> bool {
    line.contains("stringify!") && line.contains(&format!("::{sym})"))
}

#[test]
fn cli_cross_cov_must_not_stringify_symbols_already_in_cross_refs() {
    for line in SRC.lines() {
        for sym in CROSS_REFS_VALUE_SYMBOLS {
            assert!(
                !line_stringifies_exact_symbol(line, sym),
                "cli_cross_cov must not stringify {sym} when smoke_cov_cli_cross_refs already uses let _ ="
            );
        }
    }
}

#[test]
fn cli_cross_cov_must_not_duplicate_super_and_crate_cli_stringify_paths() {
    for (super_path, crate_path) in DUPLICATE_SUPER_AND_CRATE_CLI_SYMBOLS {
        assert!(
            !SRC.contains(crate_path),
            "cli_cross_cov must use {super_path} only, not also {crate_path}"
        );
    }
}

#[test]
fn cli_cross_cov_must_not_stringify_invalid_cli_module_paths() {
    for line in SRC.lines() {
        if !line.contains("stringify!") {
            continue;
        }
        for bad in INVALID_STRINGIFY_PATH_SUBSTRINGS {
            assert!(
                !line.contains(bad),
                "cli_cross_cov must not stringify unresolvable path: found {bad} in {line}"
            );
        }
    }
}
