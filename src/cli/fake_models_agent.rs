use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub(crate) const MODELS_LISTING_SCRIPT: &str = "#!/bin/sh\nif [ \"$1\" = models ]; then printf 'composer-2 — Fast\\nTip: upgrade\\n'; exit 0; fi\nexit 1\n";

pub(crate) const FAILING_AGENT_SCRIPT: &str = "#!/bin/sh\nexit 1\n";

pub(crate) fn install_agent(dir: &Path, script: &str) -> PathBuf {
    let agent = dir.join("agent");
    std::fs::write(&agent, script).expect("write fake agent");
    let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&agent, perms).expect("chmod fake agent");
    agent
}
