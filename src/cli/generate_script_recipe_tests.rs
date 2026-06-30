use super::*;

#[test]
fn parse_ok_run_10_steps() {
    let recipe = parse_run_n_steps_recipe("run-10-steps: startup.sh, task.sh, exit.sh").expect("parse");
    assert_eq!(recipe.n, 10);
    assert_eq!(recipe.startup, "startup.sh");
}

#[test]
fn parse_err_missing_colon() {
    assert!(parse_run_n_steps_recipe("run-3-steps").is_err());
}

#[test]
fn parse_err_path_traversal() {
    assert!(parse_run_n_steps_recipe("run-3-steps: ../a.sh,b.sh,c.sh").is_err());
}

#[test]
fn expand_steps_n3() {
    let recipe = parse_run_n_steps_recipe("run-3-steps: s.sh,t.sh,e.sh").expect("parse");
    let steps = expand_steps(&recipe);
    assert_eq!(
        steps,
        vec![
            ScriptStep {
                index: 1,
                script: "s.sh".to_string(),
                phase: "startup",
            },
            ScriptStep {
                index: 2,
                script: "t.sh".to_string(),
                phase: "task",
            },
            ScriptStep {
                index: 3,
                script: "e.sh".to_string(),
                phase: "exit",
            },
        ]
    );
}

#[test]
fn expand_steps_n10() {
    let recipe = parse_run_n_steps_recipe("run-10-steps: s.sh,t.sh,e.sh").expect("parse");
    let steps = expand_steps(&recipe);
    assert_eq!(steps.len(), 10);
    assert_eq!(steps[9].phase, "exit");
}

#[test]
fn parse_err_bad_prefix() {
    assert!(parse_run_n_steps_recipe("steps-3: a.sh,b.sh,c.sh").is_err());
}

#[test]
fn recipe_structs_round_trip() {
    let recipe = RunNStepsRecipe {
        n: 4,
        startup: "s.sh".to_string(),
        task: "t.sh".to_string(),
        exit: "e.sh".to_string(),
    };
    let steps = expand_steps(&recipe);
    assert_eq!(steps.len(), 4);
    assert_eq!(steps[0].phase, "startup");
}

#[test]
fn recipe_label_formats_run_n_steps() {
    assert_eq!(recipe_label(10), "run-10-steps");
}
