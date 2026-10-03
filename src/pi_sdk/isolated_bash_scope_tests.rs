use super::IsolatedToolFactory;
use pi::sdk::{Config, ToolFactory, ToolOutput};
use serde_json::{Value, json};
use std::path::Path;

async fn run_tool(cwd: &Path, name: &str, input: Value) -> ToolOutput {
    let tools = IsolatedToolFactory
        .create_tool_registry(&[name], cwd, &Config::default())
        .into_tools();
    let tool = tools.iter().find(|t| t.name() == name).expect("tool");
    let out = tool.execute("c1", input, None).await;
    let out = out.unwrap_or_else(|e| panic!("{name} outside cwd failed: {e}"));
    assert!(!out.is_error, "{name} outside cwd errored: {:?}", out.content);
    out
}

fn text(out: &ToolOutput) -> String {
    format!("{:?}", out.content)
}

fn dirs() -> (tempfile::TempDir, tempfile::TempDir) {
    let cwd = tempfile::tempdir().expect("cwd");
    let outside = tempfile::tempdir().expect("outside");
    (cwd, outside)
}

#[tokio::test]
async fn factory_write_read_edit_outside_cwd() {
    let (cwd, outside) = dirs();
    let target = outside.path().join("note.txt");
    let path = target.to_string_lossy().to_string();
    run_tool(
        cwd.path(),
        "write",
        json!({ "path": path, "content": "outside-marker\n" }),
    )
    .await;
    assert_eq!(
        std::fs::read_to_string(&target).expect("written"),
        "outside-marker\n"
    );
    let read = run_tool(cwd.path(), "read", json!({ "path": path })).await;
    assert!(format!("{:?}", read.content).contains("outside-marker"));
    run_tool(
        cwd.path(),
        "edit",
        json!({ "path": path, "oldText": "outside-marker", "newText": "edited-marker" }),
    )
    .await;
    assert!(
        std::fs::read_to_string(&target)
            .expect("edited")
            .contains("edited-marker")
    );
}

#[tokio::test]
async fn factory_ls_grep_find_outside_cwd() {
    let (cwd, outside) = dirs();
    std::fs::write(outside.path().join("probe.txt"), "probe-marker\n").expect("seed");
    let dir = outside.path().to_string_lossy().to_string();
    let ls = run_tool(cwd.path(), "ls", json!({ "path": dir })).await;
    assert!(format!("{:?}", ls.content).contains("probe.txt"));
    let grep = run_tool(
        cwd.path(),
        "grep",
        json!({ "pattern": "probe-marker", "path": dir }),
    )
    .await;
    let probe = outside.path().join("probe.txt");
    let canonical = std::fs::canonicalize(&probe).expect("canonical");
    assert!(
        text(&grep).contains(&format!("\"{}:1: probe-marker", canonical.display())),
        "grep paths must be absolute: {}",
        text(&grep)
    );
    let find = run_tool(cwd.path(), "find", json!({ "pattern": "*.txt", "path": dir })).await;
    assert!(text(&find).contains("probe.txt"));
}

#[tokio::test]
async fn factory_relative_parent_path_reaches_outside_cwd() {
    let parent = tempfile::tempdir().expect("parent");
    let cwd = parent.path().join("work");
    std::fs::create_dir(&cwd).expect("cwd");
    std::fs::write(parent.path().join("sib.txt"), "sibling-marker\n").expect("seed");
    let ls = run_tool(&cwd, "ls", json!({ "path": ".." })).await;
    assert!(text(&ls).contains("sib.txt"));
    let read = run_tool(&cwd, "read", json!({ "path": "../sib.txt" })).await;
    assert!(text(&read).contains("sibling-marker"));
}

#[tokio::test]
async fn factory_tools_inside_cwd_still_use_relative_paths() {
    let (cwd, _outside) = dirs();
    std::fs::create_dir(cwd.path().join("sub")).expect("sub");
    std::fs::write(cwd.path().join("sub/in.txt"), "inside-marker\n").expect("seed");
    let grep = run_tool(
        cwd.path(),
        "grep",
        json!({ "pattern": "inside-marker", "path": "sub" }),
    )
    .await;
    assert!(text(&grep).contains("sub/in.txt:1: inside-marker"));
    let ls = run_tool(cwd.path(), "ls", json!({})).await;
    assert!(text(&ls).contains("sub"));
}

#[test]
fn outside_cwd_path_classifies_targets() {
    use super::outside_cwd::outside_cwd_path;
    let (cwd, outside) = dirs();
    let outside_str = outside.path().to_string_lossy().to_string();
    assert!(outside_cwd_path(&json!({ "path": outside_str }), cwd.path()).is_some());
    assert!(outside_cwd_path(&json!({ "path": "a/b.txt" }), cwd.path()).is_none());
    assert!(outside_cwd_path(&json!({ "path": "" }), cwd.path()).is_none());
    assert!(outside_cwd_path(&json!({}), cwd.path()).is_none());
    assert!(outside_cwd_path(&json!({ "path": "../x" }), cwd.path()).is_some());
    if let Some(home) = std::env::var_os("HOME") {
        let got = outside_cwd_path(&json!({ "path": "~/x" }), cwd.path());
        let home = std::fs::canonicalize(home).expect("home");
        assert_eq!(got, Some(home.join("x").to_string_lossy().into_owned()));
    }
}
