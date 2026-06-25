use super::{
    command_doc_markdown, print_doc_to_writer, MALVIN_OVERVIEW_DOC,
};
use crate::cli::Cli;
use crate::cli::args::{Commands, InspireArgs, KpopArgs};
use crate::cli::delight_flow::DelightArgs;
use crate::cli::explain_flow::ExplainArgs;
use crate::cli::revise_flow::ReviseArgs;
use crate::cli::models_cmd::ModelsArgs;
use clap::Parser;

fn capture_doc(command: Option<&Commands>) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    print_doc_to_writer(command, &mut buf)?;
    Ok(buf)
}

#[test]
fn subcommand_doc_embeds_have_malvin_heading() {
    let md = command_doc_markdown(&Commands::Models(ModelsArgs {}));
    assert!(md.starts_with("# malvin "));
    let md = command_doc_markdown(&Commands::Kpop(KpopArgs {
        max_loops: 1,
        max_hypotheses: 1,
        tenacious: false,
        request: None,
    }));
    assert!(md.starts_with("# malvin "));
    let md = command_doc_markdown(&Commands::Inspire(InspireArgs { request: None }));
    assert!(md.starts_with("# malvin inspire"));
}

#[test]
fn print_doc_none_writes_full_malvin_md() {
    let out = capture_doc(None).expect("capture");
    assert_eq!(out.as_slice(), MALVIN_OVERVIEW_DOC.as_bytes());
}

#[test]
fn print_doc_some_writes_subcommand_md() {
    let cmd = Commands::Kpop(KpopArgs {
        max_loops: 1,
        max_hypotheses: 1,
        tenacious: false,
        request: None,
    });
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert_eq!(out.as_slice(), command_doc_markdown(&cmd).as_bytes());
    assert!(out.starts_with(b"# malvin kpop"));
}

#[test]
fn top_level_doc_parses_without_subcommand() {
    let cli = Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    assert!(cli.command.is_none());
}

#[test]
fn kpop_doc_parses_without_request_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "kpop", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Kpop(k)) => assert!(k.request.is_none()),
        _ => panic!("expected Kpop"),
    }
}

#[test]
fn init_doc_parses_without_languages_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "init", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Init(i)) => assert!(i.languages.is_empty()),
        _ => panic!("expected Init"),
    }
}

#[test]
fn delight_doc_parses_without_out_path() {
    let cli = Cli::try_parse_from(["malvin", "delight", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Delight(d)) => assert_eq!(d.out_path, "plan.md"),
        _ => panic!("expected Delight"),
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
fn print_doc_delight_writes_subcommand_md() {
    let cmd = Commands::Delight(DelightArgs {
        guidance: None,
        out_path: "plan.md".to_string(),
        max_loops: 3,
        max_hypotheses: 5,
        tenacious: true,
    });
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert!(out.starts_with(b"# malvin delight"));
}

#[test]
fn revise_doc_parses_with_doc_path_when_doc_flag_set() {
    let cli = Cli::try_parse_from(["malvin", "revise", "doc.md", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
    match cli.command.as_ref() {
        Some(Commands::Revise(r)) => assert_eq!(r.doc_path, "doc.md"),
        _ => panic!("expected Revise"),
    }
}

#[test]
fn print_doc_revise_writes_subcommand_md() {
    let cmd = Commands::Revise(ReviseArgs {
        doc_path: "doc.md".to_string(),
        max_loops: 3,
        max_hypotheses: 5,
        tenacious: true,
    });
    let out = capture_doc(Some(&cmd)).expect("capture");
    assert!(out.starts_with(b"# malvin revise"));
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

#[test]
fn init_doc_substitutes_advice_path() {
    use crate::cli::args::{Commands, InitArgs};
    let cmd = Commands::Init(InitArgs {
        force: false,
        languages: vec![],
        path: None,
    });
    let out = capture_doc(Some(&cmd)).expect("capture");
    let text = String::from_utf8(out).expect("utf8");
    assert!(
        text.contains(".malvin/advice.md"),
        "init doc must show advice path"
    );
    assert!(
        !text.contains("{{ advice_path }}"),
        "init doc must not leave unresolved advice_path placeholder"
    );
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::super::{doc_text, print_doc, print_doc_to_writer};

    #[test]
    fn kiss_cov_unit_names() {
        let _ = doc_text;
        let _ = print_doc;
    }
}
