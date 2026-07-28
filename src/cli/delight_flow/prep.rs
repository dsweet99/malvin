use std::path::{Path, PathBuf};

use crate::artifacts::resolve_user_md_request;
use crate::cli::default_output_path::{
    allocate_default_sibling_file, path_relative_to_cwd, DELIGHT_DEFAULT_OUT_PATH,
};

pub(crate) fn resolve_delight_guidance(guidance: Option<&String>) -> Result<Option<String>, String> {
    let Some(raw) = guidance else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let (text, _) = resolve_user_md_request(trimmed)?;
    Ok(Some(text))
}

/// Compose the router REQUEST for `malvin delight` (embeds guidance when present).
pub(crate) fn compose_delight_router_request(
    out_path_display: &str,
    guidance: Option<&str>,
) -> String {
    let mut body = format!(
        "Author a user-delighting feature pitch as a new markdown file.\n\
         Write the pitch to `{out_path_display}`.\n\
         Keep the pitch concise and actionable for a later implementation run. No implementation details. \
         Format as problem-solution-benefit or what-so what-now what.\n\
         Make the pitch different from previously-rejected or previously-accepted pitches when those exist. \
         Aim to delight the user with novelty while staying grounded in this repository.\n\
         Do not overwrite unrelated workspace files.\n"
    );
    if let Some(g) = guidance.map(str::trim).filter(|s| !s.is_empty()) {
        body.push_str("\nUser guidance for the pitch:\n\n");
        body.push_str(g);
        body.push('\n');
    }
    body
}

pub(crate) fn delight_preflight(out_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let resolved_out_path = if out_path == DELIGHT_DEFAULT_OUT_PATH {
        let default = cwd.join(DELIGHT_DEFAULT_OUT_PATH);
        allocate_default_sibling_file(&default, "pitch", ".md")?
    } else {
        let resolved = cwd.join(out_path);
        if resolved.exists() {
            return Err(format!(
                "malvin delight: `{}` already exists; refusing to overwrite",
                resolved.display()
            ));
        }
        resolved
    };
    if let Some(parent) = resolved_out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let rel_out = path_relative_to_cwd(&resolved_out_path)?;
    let work_dir = crate::artifacts::work_dir_for_path(Path::new(&rel_out));
    Ok((resolved_out_path, work_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_embeds_out_path_and_optional_guidance() {
        let no_g = compose_delight_router_request("pitch.md", None);
        assert!(no_g.contains("`pitch.md`"));
        assert!(!no_g.contains("User guidance"));
        let with_g = compose_delight_router_request("plans/x.md", Some("focus on UX"));
        assert!(with_g.contains("`plans/x.md`"));
        assert!(with_g.contains("User guidance for the pitch:"));
        assert!(with_g.contains("focus on UX"));
    }

    #[test]
    fn resolve_guidance_none_and_empty() {
        assert!(resolve_delight_guidance(None).unwrap().is_none());
        let empty = String::from("  ");
        assert!(resolve_delight_guidance(Some(&empty)).unwrap().is_none());
    }

    #[test]
    fn resolve_guidance_literal() {
        let g = String::from("steer toward latency");
        let got = resolve_delight_guidance(Some(&g)).unwrap().unwrap();
        assert_eq!(got, "steer toward latency");
    }
}

#[cfg(test)]
#[path = "../delight_flow_prep_preflight_tests.rs"]
mod delight_flow_prep_preflight_tests;
