//! Recipe parsing for `malvin generate-script`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunNStepsRecipe {
    pub n: usize,
    pub startup: String,
    pub task: String,
    pub exit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptStep {
    pub index: usize,
    pub script: String,
    pub phase: &'static str,
}

pub fn parse_run_n_steps_recipe(recipe: &str) -> Result<RunNStepsRecipe, String> {
    let (prefix, scripts_part) = recipe
        .split_once(':')
        .ok_or_else(|| "recipe must contain ':' separating prefix from scripts".to_string())?;
    let n = parse_run_n_steps_count(prefix.trim())?;
    let [startup, task, exit] = parse_run_n_steps_scripts(scripts_part)?;
    Ok(RunNStepsRecipe {
        n,
        startup,
        task,
        exit,
    })
}

fn parse_run_n_steps_count(prefix: &str) -> Result<usize, String> {
    let Some(steps_suffix) = prefix.strip_prefix("run-") else {
        return Err(format!("recipe prefix must be run-{{N}}-steps, got {prefix:?}"));
    };
    let Some(n_str) = steps_suffix.strip_suffix("-steps") else {
        return Err(format!("recipe prefix must be run-{{N}}-steps, got {prefix:?}"));
    };
    let n: usize = n_str
        .parse()
        .map_err(|_| format!("step count must be a positive integer, got {n_str:?}"))?;
    if n < 3 {
        return Err(format!("step count must be at least 3, got {n}"));
    }
    Ok(n)
}

fn parse_run_n_steps_scripts(scripts_part: &str) -> Result<[String; 3], String> {
    let scripts: Vec<String> = scripts_part
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if scripts.len() != 3 {
        return Err(format!(
            "run-N-steps recipes require exactly 3 scripts (startup, task, exit), got {}",
            scripts.len()
        ));
    }
    for name in &scripts {
        if name.contains('/') || name.contains("..") {
            return Err(format!("script basename must not contain '/' or '..': {name:?}"));
        }
    }
    Ok([scripts[0].clone(), scripts[1].clone(), scripts[2].clone()])
}

pub fn expand_steps(recipe: &RunNStepsRecipe) -> Vec<ScriptStep> {
    let mut steps = Vec::with_capacity(recipe.n);
    steps.push(ScriptStep {
        index: 1,
        script: recipe.startup.clone(),
        phase: "startup",
    });
    for index in 2..recipe.n {
        steps.push(ScriptStep {
            index,
            script: recipe.task.clone(),
            phase: "task",
        });
    }
    steps.push(ScriptStep {
        index: recipe.n,
        script: recipe.exit.clone(),
        phase: "exit",
    });
    steps
}

pub fn recipe_label(n: usize) -> String {
    format!("run-{n}-steps")
}

#[cfg(test)]
mod tests {
    include!("generate_script_recipe_tests.rs");
}
