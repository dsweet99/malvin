use super::{MALVIN_OVERVIEW_DOC, ROUTER_DOC, command_doc_markdown, print_doc_to_writer};
use crate::cli::Cli;
use crate::cli::models_cmd::ModelsArgs;
use crate::cli::write_flow::WriteArgs;
use crate::cli::{AdminArgs, AdminCommand, Commands};
use clap::Parser;

fn capture_doc(command: Option<&Commands>) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    print_doc_to_writer(command, &mut buf)?;
    Ok(buf)
}

#[test]
fn subcommand_doc_embeds_have_malvin_heading() {
    let md = command_doc_markdown(&Commands::Admin(AdminArgs {
        command: AdminCommand::Models(ModelsArgs::default()),
    }));
    assert!(md.starts_with("# malvin "));
    let md = command_doc_markdown(&Commands::Write(WriteArgs {
        shared: crate::cli::SharedOpts::test_defaults(),
        request: None,
        out_path: "write.tex".to_string(),
        max_loops: 3,
        max_hypotheses: 5,
        tenacious: true,
        out_path_explicit: false,
    }));
    assert!(md.starts_with("# malvin write"));
    assert!(ROUTER_DOC.starts_with("# malvin"));
}

#[test]
fn print_doc_none_writes_overview_then_router() {
    let out = capture_doc(None).expect("capture");
    let expected = format!("{MALVIN_OVERVIEW_DOC}\n---\n\n{ROUTER_DOC}");
    assert_eq!(out.as_slice(), expected.as_bytes());
    let text = String::from_utf8(out).expect("utf8");
    assert!(text.starts_with(MALVIN_OVERVIEW_DOC));
    assert!(text.contains(ROUTER_DOC));
    assert!(text.contains("# malvin (default route)"));
}

#[test]
fn top_level_doc_parses_without_subcommand() {
    let cli = Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    assert!(cli.command.is_none());
}

#[test]
fn do_doc_parses_with_do_flag() {
    let cli = Cli::try_parse_from(["malvin", "--do", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    assert!(cli.do_workflow);
    assert!(cli.command.is_none());
    let mut buf = Vec::new();
    super::print_doc_for_cli_to_writer(&cli, &mut buf).expect("write");
    assert!(buf.starts_with(b"# malvin --do"));
}

#[test]
fn write_doc_parses_without_request_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "write", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Write(w)) => assert!(w.request.is_none()),
        _ => panic!("expected Write"),
    }
}

#[test]
fn write_doc_parses_with_request_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "write", "topic.md", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Write(e)) => {
            assert_eq!(e.request.as_deref(), Some("topic.md"));
            assert_eq!(e.out_path, "write.tex");
        }
        _ => panic!("expected Write"),
    }
}

#[test]
fn print_doc_write_writes_subcommand_md() {
    let cmd = Commands::Write(WriteArgs {
        shared: crate::cli::SharedOpts::test_defaults(),
        request: Some("topic".to_string()),
        out_path: "write.tex".to_string(),
        max_loops: 3,
        max_hypotheses: 5,
        tenacious: true,
        out_path_explicit: false,
    });
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert!(out.starts_with(b"# malvin write"));
}

#[test]
fn malvin_doc_embeds_name_section() {
    let out = capture_doc(None).expect("capture");
    let text = String::from_utf8(out).expect("utf8");
    assert!(
        text.contains("Session names") || text.contains(".malvin_home/names"),
        "doc must describe session names"
    );
    assert!(
        text.contains(".malvin_home/names") || text.contains("already holds"),
        "doc must describe registry or duplicate-name behavior"
    );
    assert!(
        !text.contains("### `--name"),
        "doc must not document removed --name option"
    );
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::super::{doc_text, print_doc_to_writer};

    #[test]
    fn kiss_cov_unit_names() {
        let _ = doc_text(None);
        let mut buf = Vec::new();
        print_doc_to_writer(None, &mut buf).expect("write");
        assert!(!buf.is_empty());
    }
}
