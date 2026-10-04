use std::collections::BTreeMap;
use std::path::Path;

pub const WORKSPACE_TAR: &str = "/tmp/malvin-modal/workspace.tar.gz";
pub const LOGS_TAR: &str = "/tmp/malvin-modal/logs.tar.gz";
pub const RESULT_PATCH: &str = "/tmp/malvin-modal/result.patch";
pub const NEW_LOGS_TAR: &str = "/tmp/malvin-modal/new_logs.tar.gz";

pub const SETUP_SCRIPT: &str = r#"set -eu
S=/tmp/malvin-modal
mkdir -p "$MALVIN_MODAL_WORK" "$MALVIN_MODAL_LOGS"
tar --no-same-owner -xzf "$S/workspace.tar.gz" -C "$MALVIN_MODAL_WORK"
if [ -f "$S/logs.tar.gz" ]; then tar --no-same-owner -xzf "$S/logs.tar.gz" -C "$MALVIN_MODAL_LOGS"; fi
ls -1 "$MALVIN_MODAL_LOGS" | sort > "$S/logs_before"
cd "$MALVIN_MODAL_WORK"
git init -q
git add -A
git -c user.name=malvin -c user.email=malvin@localhost commit -q --allow-empty --no-verify -m "malvin --modal baseline"
git rev-parse HEAD > "$S/baseline"
"#;

pub const FINISH_SCRIPT: &str = r#"set -eu
S=/tmp/malvin-modal
cd "$MALVIN_MODAL_WORK"
git add -A
git diff --cached --binary "$(cat "$S/baseline")" > "$S/result.patch"
ls -1 "$MALVIN_MODAL_LOGS" | sort > "$S/logs_after"
comm -13 "$S/logs_before" "$S/logs_after" > "$S/new_logs"
tar -czf "$S/new_logs.tar.gz" -C "$MALVIN_MODAL_LOGS" -T "$S/new_logs"
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePaths {
    pub work_dir: String,
    pub home: String,
    pub logs_dir: String,
}

impl RemotePaths {
    #[must_use]
    pub fn new(work_dir: &Path, home: &Path) -> Self {
        let logs = home
            .join(crate::MALVIN_USER_HOME_DIR)
            .join("logs")
            .join(crate::workspace_logs_hash(work_dir));
        Self {
            work_dir: work_dir.display().to_string(),
            home: home.display().to_string(),
            logs_dir: logs.display().to_string(),
        }
    }

    #[must_use]
    pub fn config_path(&self) -> String {
        format!(
            "{}/{}/{}",
            self.home,
            crate::MALVIN_USER_HOME_DIR,
            crate::MALVIN_HOME_CONFIG_FILE
        )
    }

    #[must_use]
    pub fn script_env(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("MALVIN_MODAL_WORK".to_string(), self.work_dir.clone()),
            ("MALVIN_MODAL_LOGS".to_string(), self.logs_dir.clone()),
        ])
    }

    #[must_use]
    pub fn malvin_env(&self, credentials: BTreeMap<String, String>) -> BTreeMap<String, String> {
        let mut env = credentials;
        env.insert("HOME".to_string(), self.home.clone());
        env.insert("MALVIN_MODAL_REMOTE".to_string(), "1".to_string());
        env.insert("NPM_CONFIG_UPDATE_NOTIFIER".to_string(), "false".to_string());
        env
    }
}
