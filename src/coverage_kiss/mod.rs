//! Behavioral smoke tests for modules referenced by `kiss check` coverage wiring.

use std::collections::HashMap;
use std::path::Path;

include!("artifacts.inc");
include!("helpers.inc");
include!("stringify_refs.inc");
include!("kpop.inc");
include!("orch.inc");
include!("paths.inc");
include!("prompts_run.inc");
include!("repo.inc");
include!("sync.inc");
include!("behaviors.inc");

#[path = "../multiturn_prompt_tests.rs"]
mod multiturn_prompt_tests;

#[path = "../kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;

#[path = "../kpop_test_stubs_tests.rs"]
mod kpop_test_stubs_tests;

#[path = "../kpop_multiturn_prompts_tests.rs"]
mod kpop_multiturn_prompts_tests;

#[path = "../run_id_tests.rs"]
mod run_id_tests;

#[path = "../stdout_log_path_tests.rs"]
mod stdout_log_path_tests;
