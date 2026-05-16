//! `malvin plan` `@file` destination and source behavior.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::fs;

    use super::common::{
        PlanSpawn, acp_mock_code_streaming_update_js, spawn_malvin_plan, test_home_workspace,
        write_mock_executable,
    };

    #[test]
    fn malvin_plan_at_file_reads_source_not_literal() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-at");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        fs::write(workspace.join("source.md"), "real plan body\n").unwrap();
        let at_arg = format!("@{}", workspace.join("source.md").display());
        let out = spawn_malvin_plan(&sp, &[&at_arg]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        let body = fs::read_to_string(workspace.join("source.md")).unwrap();
        assert!(
            body.contains("real plan body"),
            "plan file should retain source content, not @ literal: {body}"
        );
        assert!(!body.trim().starts_with('@'));
        let default_plan = workspace.join("plan.md");
        assert!(
            !default_plan.exists(),
            "sole @path must review in place, not create {default_plan:?}"
        );
    }

    #[test]
    fn malvin_plan_literal_writes_default_plan_md() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-lit");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        let out = spawn_malvin_plan(&sp, &["literal body"]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            fs::read_to_string(workspace.join("plan.md")).unwrap(),
            "literal body\n"
        );
    }

    #[test]
    fn malvin_plan_plan_path_with_at_source_copies_then_reviews() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-copy");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        fs::write(workspace.join("in.md"), "copied plan\n").unwrap();
        let at_arg = format!("@{}", workspace.join("in.md").display());
        let out = spawn_malvin_plan(&sp, &["--plan-path", "out.md", &at_arg]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            fs::read_to_string(workspace.join("out.md")).unwrap(),
            "copied plan\n"
        );
    }

    #[test]
    fn malvin_plan_plan_path_with_literal_writes_destination() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-path-lit");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        let out = spawn_malvin_plan(&sp, &["--plan-path", "out.md", "literal body"]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            fs::read_to_string(workspace.join("out.md")).unwrap(),
            "literal body\n"
        );
    }

    #[test]
    fn malvin_plan_cross_dir_plan_path_run_dir_under_destination_parent() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-cross-dir");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        let src_side = workspace.join("src_side");
        let dst_side = workspace.join("dst_side");
        fs::create_dir_all(&src_side).unwrap();
        fs::create_dir_all(&dst_side).unwrap();
        fs::write(src_side.join("in.md"), "copied plan\n").unwrap();
        let at_arg = format!("@{}", src_side.join("in.md").display());
        let out = spawn_malvin_plan(&sp, &["--plan-path", "dst_side/out.md", &at_arg]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            fs::read_to_string(dst_side.join("out.md")).unwrap(),
            "copied plan\n"
        );
        assert!(
            dst_side.join("_malvin").is_dir(),
            "run artifacts must live under destination plan parent, not @ source"
        );
        assert!(
            !src_side.join("_malvin").exists(),
            "_malvin must not be created under @ source when --plan-path selects another tree"
        );
    }
}
