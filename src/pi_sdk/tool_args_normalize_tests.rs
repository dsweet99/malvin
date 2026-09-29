use super::{coerce_string_scalars, normalize_path_alias, wrap_tool_args};
use crate::pi_sdk::isolated_bash::IsolatedToolFactory;
use pi::sdk::{Config, Tool, ToolFactory, default_tool_registry};
use serde_json::json;

fn builtin(name: &str, cwd: &std::path::Path) -> Box<dyn Tool> {
    default_tool_registry(&[name], cwd, &Config::default())
        .into_tools()
        .into_iter()
        .find(|t| t.name() == name)
        .expect("builtin tool")
}

#[test]
fn aliases_are_renamed_to_path() {
    for alias in ["file", "file_path", "filePath"] {
        let v = normalize_path_alias(json!({ alias: "a.txt", "limit": 3 }));
        assert_eq!(v, json!({ "path": "a.txt", "limit": 3 }), "alias {alias}");
    }
}

#[test]
fn log_summary_shows_the_file_alias() {
    let summary =
        crate::pi_sdk::tool_summary_from_pi(Some("read"), Some(&json!({ "file": "a.txt" })));
    assert_eq!(summary.as_deref(), Some("Read a.txt"));
}

#[test]
fn existing_path_and_non_objects_are_left_alone() {
    let v = normalize_path_alias(json!({ "path": "p.txt", "file": "f.txt" }));
    assert_eq!(v, json!({ "path": "p.txt", "file": "f.txt" }));
    assert_eq!(normalize_path_alias(json!("x")), json!("x"));
}

#[test]
fn wrapper_keeps_name_and_schema() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let read = wrap_tool_args(builtin("read", dir.path()));
    assert_eq!(read.name(), "read");
    assert_eq!(read.parameters(), builtin("read", dir.path()).parameters());
}

#[test]
fn string_scalars_follow_the_declared_schema_type() {
    let schema = json!({ "properties": {
        "ignoreCase": { "type": "boolean" },
        "limit": { "type": "integer" },
        "pattern": { "type": "string" },
    }});
    let v = coerce_string_scalars(
        json!({ "ignoreCase": "True", "limit": " 100 ", "pattern": "42", "extra": "false" }),
        &schema,
    );
    assert_eq!(
        v,
        json!({ "ignoreCase": true, "limit": 100, "pattern": "42", "extra": "false" })
    );
    let bad = json!({ "ignoreCase": "yes", "limit": "ten" });
    assert_eq!(coerce_string_scalars(bad.clone(), &schema), bad);
}

fn notes_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tmpdir");
    std::fs::write(dir.path().join("notes.txt"), "alias-marker\n").expect("write");
    dir
}

async fn run_factory_tool(dir: &std::path::Path, name: &str, input: serde_json::Value) -> String {
    let tools = IsolatedToolFactory
        .create_tool_registry(&[name], dir, &Config::default())
        .into_tools();
    let tool = tools.iter().find(|t| t.name() == name).expect("tool");
    let out = tool.execute("c1", input, None).await.expect("alias call");
    assert!(!out.is_error);
    format!("{:?}", out.content)
}

#[tokio::test]
async fn pi_rejects_alias_and_string_scalars_without_the_shim() {
    let dir = notes_dir();
    let raw = builtin("read", dir.path());
    for input in [
        json!({ "file": "notes.txt" }),
        json!({ "path": "notes.txt", "limit": "1" }),
        json!({ "path": "notes.txt", "hashline": "false" }),
    ] {
        assert!(raw.execute("c0", input, None).await.is_err());
    }
}

#[tokio::test]
async fn factory_read_accepts_string_scalars() {
    let dir = notes_dir();
    let input = json!({ "file": "notes.txt", "limit": "1", "hashline": "false" });
    let read = run_factory_tool(dir.path(), "read", input).await;
    assert!(read.contains("alias-marker"));
}

#[tokio::test]
async fn factory_read_and_ls_accept_aliases() {
    let dir = notes_dir();
    let read = run_factory_tool(dir.path(), "read", json!({ "file": "notes.txt" })).await;
    assert!(read.contains("alias-marker"));
    let ls = run_factory_tool(dir.path(), "ls", json!({ "filePath": "." })).await;
    assert!(ls.contains("notes.txt"));
}
