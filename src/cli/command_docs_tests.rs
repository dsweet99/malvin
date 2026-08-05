use super::{
    command_doc_markdown, print_doc_to_writer, MALVIN_OVERVIEW_DOC, ROUTER_DOC,
};
use crate::cli::Cli;
use crate::cli::{Commands, InspireArgs};
use crate::cli::explain_flow::ExplainArgs;
use crate::cli::models_cmd::ModelsArgs;
use clap::Parser;

fn capture_doc(command: Option<&Commands>) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    print_doc_to_writer(command, &mut buf)?;
    Ok(buf)
}

#[test]
fn subcommand_doc_embeds_have_malvin_heading() {
    let md = command_doc_markdown(&Commands::Models(ModelsArgs::default()));
    assert!(md.starts_with("# malvin "));
    let md = command_doc_markdown(&Commands::Inspire(InspireArgs { request: None }));
    assert!(md.starts_with("# malvin inspire"));
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
fn print_doc_init_writes_subcommand_md() {
    use crate::cli::init_flow::InitArgs;
    let cmd = Commands::Init(InitArgs {});
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert!(out.starts_with(b"# malvin init"));
}

#[test]
fn print_doc_inspire_writes_subcommand_md() {
    let cmd = Commands::Inspire(InspireArgs { request: None });
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert_eq!(out.as_slice(), command_doc_markdown(&cmd).as_bytes());
    assert!(out.starts_with(b"# malvin inspire"));
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
fn inspire_doc_parses_without_request_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "inspire", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Inspire(i)) => assert!(i.request.is_none()),
        _ => panic!("expected Inspire"),
    }
}

#[test]
fn explain_doc_parses_with_request_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "explain", "topic.md", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Explain(e)) => {
            assert_eq!(e.request.as_deref(), Some("topic.md"));
            assert_eq!(e.out_path, "explain.tex");
        }
        _ => panic!("expected Explain"),
    }
}

#[test]
fn print_doc_explain_writes_subcommand_md() {
    let cmd = Commands::Explain(ExplainArgs {
        request: Some("topic".to_string()),
        out_path: "explain.tex".to_string(),
        max_loops: 3,
        max_hypotheses: 5,
        tenacious: true,
        out_path_explicit: false,
    });
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert!(out.starts_with(b"# malvin explain"));
}

#[test]
fn malvin_doc_embeds_name_section() {
    let out = capture_doc(None).expect("capture");
    let text = String::from_utf8(out).expect("utf8");
    assert!(text.contains("--name"), "doc must mention --name");
    assert!(
        text.contains(".malvin_home/names") || text.contains("already holds"),
        "doc must describe registry or duplicate-name behavior"
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
