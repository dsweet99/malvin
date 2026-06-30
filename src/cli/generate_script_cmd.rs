//! `malvin generate-script` — materialize auto-script JSON and stub shell scripts from a recipe.

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::json;

use crate::cli::cli_request::require_cli_request;
use crate::output::{MALVIN_WHO, print_stdout_line};

pub use crate::cli::generate_script_recipe::{
    expand_steps, parse_run_n_steps_recipe, recipe_label, RunNStepsRecipe, ScriptStep,
};

#[derive(Args, Debug)]
pub struct GenerateScriptArgs {
    /// Recipe string, e.g. `run-10-steps: startup.sh, task.sh, exit.sh`.
    #[arg(value_name = "RECIPE")]
    pub recipe: Option<String>,
    /// JSON output path.
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
    /// Directory for stub shell scripts.
    #[arg(long = "scripts-dir")]
    pub scripts_dir: Option<PathBuf>,
}

fn write_auto_script_json(
    output_path: &Path,
    recipe: &RunNStepsRecipe,
    steps: &[ScriptStep],
) -> Result<(), String> {
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("create output parent {}: {e}", parent.display())
            })?;
        }
    }

    let steps_json: Vec<_> = steps
        .iter()
        .map(|s| {
            json!({
                "index": s.index,
                "script": s.script,
                "phase": s.phase,
            })
        })
        .collect();

    let doc = json!({
        "version": 1,
        "recipe": recipe_label(recipe.n),
        "atomic": true,
        "work_dir": ".",
        "steps": steps_json,
    });

    let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(output_path, text).map_err(|e| format!("write {}: {e}", output_path.display()))
}

fn stub_body(script: &str, phase: &str) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\necho \"malvin-script-stub: {script} phase={phase}\"\n"
    )
}

fn set_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .map_err(|e| format!("metadata {}: {e}", path.display()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)
            .map_err(|e| format!("chmod {}: {e}", path.display()))?;
    }
    Ok(())
}

fn materialize_stub_scripts(
    recipe: &RunNStepsRecipe,
    scripts_dir: &Path,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(scripts_dir)
        .map_err(|e| format!("create scripts dir {}: {e}", scripts_dir.display()))?;

    let stubs = [
        (&recipe.startup, "startup"),
        (&recipe.task, "task"),
        (&recipe.exit, "exit"),
    ];

    let mut created = Vec::new();
    for (basename, phase) in stubs {
        let path = scripts_dir.join(basename);
        if path.exists() {
            continue;
        }
        std::fs::write(&path, stub_body(basename, phase))
            .map_err(|e| format!("write {}: {e}", path.display()))?;
        set_executable(&path)?;
        created.push(basename.clone());
    }
    Ok(created)
}

pub fn run_generate_script(args: GenerateScriptArgs) -> Result<(), String> {
    let recipe_str = require_cli_request(args.recipe.as_ref(), "generate-script")?;
    let parsed = parse_run_n_steps_recipe(&recipe_str)?;
    let steps = expand_steps(&parsed);

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from("auto-script.json"));
    write_auto_script_json(&output_path, &parsed, &steps)?;

    let scripts_dir = match args.scripts_dir {
        Some(dir) => dir,
        None => std::env::current_dir().map_err(|e| format!("current directory: {e}"))?,
    };
    let created = materialize_stub_scripts(&parsed, &scripts_dir)?;

    let output_name = output_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("auto-script.json");
    print_stdout_line(
        MALVIN_WHO,
        &format!(
            "wrote {output_name} ({} steps, recipe {})",
            steps.len(),
            recipe_label(parsed.n)
        ),
    );
    for name in created {
        print_stdout_line(MALVIN_WHO, &format!("created {name}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_generate_script_requires_recipe() {
        let err = run_generate_script(GenerateScriptArgs {
            recipe: None,
            output: None,
            scripts_dir: None,
        })
        .expect_err("missing recipe");
        assert!(err.contains("REQUEST"), "{err}");
    }

    #[test]
    fn run_generate_script_writes_output_and_stubs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output = dir.path().join("auto-script.json");
        run_generate_script(GenerateScriptArgs {
            recipe: Some("run-3-steps: s.sh,t.sh,e.sh".to_string()),
            output: Some(output.clone()),
            scripts_dir: Some(dir.path().to_path_buf()),
        })
        .expect("run");
        assert!(output.is_file());
        assert!(dir.path().join("s.sh").is_file());
    }
}
