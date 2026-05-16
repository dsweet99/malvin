//! `malvin plan` `@file` edge cases (continued).

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
    fn malvin_plan_without_positional_errors_when_default_plan_missing() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-missing");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        let out = spawn_malvin_plan(&sp, &[]);
        assert!(!out.status.success());
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert!(
            combined.contains("plan file does not exist") || combined.contains("ERR:"),
            "expected missing-plan error: {combined}"
        );
    }

    #[test]
    fn malvin_plan_missing_at_file_fails() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-missing-at");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        let missing = workspace.join("no_such_plan.md");
        let at_arg = format!("@{}", missing.display());
        let out = spawn_malvin_plan(&sp, &[&at_arg]);
        assert!(!out.status.success());
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert!(
            combined.contains("does not exist") || combined.contains("ERR:"),
            "expected missing @file error: {combined}"
        );
    }

    fn assert_bare_at_rejected(sp: &PlanSpawn<'_>, args: &[&str]) {
        let out = spawn_malvin_plan(sp, args);
        assert!(!out.status.success());
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert!(
            combined.contains("Empty path after `@`") || combined.contains("ERR:"),
            "expected bare @ error for args={args:?}: {combined}"
        );
    }

    #[test]
    fn malvin_plan_bare_at_is_rejected_with_and_without_plan_path() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-bare-at");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        assert_bare_at_rejected(&sp, &["@"]);
        assert_bare_at_rejected(&sp, &["--plan-path", "out.md", "@"]);
    }

    #[test]
    fn malvin_plan_plan_path_only_reviews_existing_file() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-path-only");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        fs::write(workspace.join("custom.md"), "custom seed\n").unwrap();
        let out = spawn_malvin_plan(&sp, &["--plan-path", "custom.md"]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            fs::read_to_string(workspace.join("custom.md")).unwrap(),
            "custom seed\n"
        );
    }

    #[test]
    fn malvin_plan_without_positional_reviews_existing_plan_md() {
        let (root, home, workspace) = test_home_workspace();
        let mock = root.path().join("mock-agent-acp-plan-none");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", root.path().display());
        let sp = PlanSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path,
        };
        fs::write(workspace.join("plan.md"), "unchanged seed\n").unwrap();
        let out = spawn_malvin_plan(&sp, &[]);
        assert!(
            out.status.success(),
            "stderr={}\nstdout={}",
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        assert_eq!(
            fs::read_to_string(workspace.join("plan.md")).unwrap(),
            "unchanged seed\n"
        );
    }
}
