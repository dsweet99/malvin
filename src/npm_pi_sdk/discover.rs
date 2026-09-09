use std::path::PathBuf;

pub const NPM_PI_MISSING_HINT: &str = "pi backend requires the TypeScript/npm Pi agent \
(@earendil-works/pi-coding-agent). Set MALVIN_PI to its cli.js or rpc-entry.js, or install the package.";

pub fn resolve_npm_pi_entry() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MALVIN_PI").filter(|v| !v.is_empty()) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "MALVIN_PI points to a missing file ({}); {NPM_PI_MISSING_HINT}",
            path.display()
        ));
    }
    first_entry_candidate().ok_or_else(|| NPM_PI_MISSING_HINT.to_string())
}

fn first_entry_candidate() -> Option<PathBuf> {
    for root in candidate_package_roots() {
        for rel in [
            "dist/bundle/rpc-entry.js",
            "dist/rpc-entry.js",
            "dist/bundle/cli.js",
            "dist/cli.js",
        ] {
            let candidate = root.join(rel);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn candidate_package_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    push_scoped(&mut roots, PathBuf::from("node_modules"));
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        push_scoped(
            &mut roots,
            home.join(".malvin_home/sdk-bridges/node_modules"),
        );
        push_npx_cache(&mut roots, &home.join(".npm/_npx"));
    }
    roots
}

fn push_scoped(out: &mut Vec<PathBuf>, modules: PathBuf) {
    for scope in ["@earendil-works", "@mariozechner"] {
        let pkg = modules.join(scope).join("pi-coding-agent");
        if pkg.is_dir() {
            out.push(pkg);
        }
    }
}

fn push_npx_cache(out: &mut Vec<PathBuf>, npx_root: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(npx_root) else {
        return;
    };
    for entry in entries.flatten().take(24) {
        push_scoped(out, entry.path().join("node_modules"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malvin_pi_missing_file_errors() {
        let _lock = crate::test_utils::test_env_lock();
        crate::acp::with_env("MALVIN_PI", Some("/no/such/pi-entry.js"), || {
            let err = resolve_npm_pi_entry().expect_err("missing");
            assert!(err.contains("MALVIN_PI") && err.contains("missing file"));
        });
    }

    #[test]
    fn hint_mentions_package() {
        assert!(NPM_PI_MISSING_HINT.contains("@earendil-works/pi-coding-agent"));
    }
}
