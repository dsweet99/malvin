//! Fuzz test for init-discovery ruff detection (mixed-case pre-commit hook entries).
//!
//! Production code identifies ruff via `canonical_tool` (ASCII-lowercased first token),
//! consistent with pytest and dedupe policy.
//!
//! Replay with an explicit seed from failure output:
//! ```text
//! BRITTLE_FUZZ_SEED=123 cargo nextest run brittle_precommit_ruff_case_fuzz
//! ```
//! `brittle_precommit_ruff_case_fuzz` runs in default nextest (`canonical_tool` ruff fix).

use malvin::repo_gates::discover_init_checks::augment_init_checks_with_precommit_python_gates;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

const RUFF_BASELINE: &str = "ruff check .";

/// Random independent ASCII case flip per letter (deterministic from `rng`).
fn random_ascii_case_permutation(cmd: &str, rng: &mut StdRng) -> String {
    cmd.chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            }
            if rng.gen_bool(0.5) {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

fn write_precommit_ruff_entry(root: &std::path::Path, entry: &str) {
    let yaml = format!(
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: {entry}\n",
    );
    std::fs::write(root.join(".pre-commit-config.yaml"), yaml).unwrap();
}

fn seed_kiss_only_checks(root: &std::path::Path) {
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(root)
            .status()
            .expect("git init")
            .success()
    );
    let path = malvin::malvin_checks_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, "kiss check\n").unwrap();
}

fn checks_lines(root: &std::path::Path) -> Vec<String> {
    let text = std::fs::read_to_string(malvin::resolve_malvin_checks_path(root)).unwrap();
    text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

/// Same first-token policy as `canonical_tool` (lowercased), without importing private API.
fn checks_cover_ruff_semantically(lines: &[String]) -> bool {
    lines.iter().any(|l| {
        l.split_whitespace()
            .next()
            .is_some_and(|t| t.eq_ignore_ascii_case("ruff"))
    })
}

const FUZZ_ACTIVE_MARKER: &str =
    "`brittle_precommit_ruff_case_fuzz` runs in default nextest (`canonical_tool` ruff fix)";

#[test]
fn brittle_gate_signals_fuzz_runs_in_default_suite() {
    let src = include_str!("brittle_gate_signals_fuzz.rs");
    assert!(
        src.contains(FUZZ_ACTIVE_MARKER),
        "brittle fuzz must stay active in default nextest"
    );
    let prefix = src
        .split("fn brittle_precommit_ruff_case_fuzz")
        .next()
        .expect("fuzz fn");
    assert!(
        !prefix.contains("#[ignore"),
        "brittle_precommit_ruff_case_fuzz must not be #[ignore]"
    );
    assert!(
        src.contains("brittle_gate_signals_fuzz seed:"),
        "fuzz must print seed for replay"
    );
    assert!(src.contains("rand::random"), "fuzz must use unseeded randomness");
}

#[test]
fn augment_injects_lowercase_ruff_from_precommit_baseline() {
    let tmp = tempfile::tempdir().unwrap();
    write_precommit_ruff_entry(tmp.path(), RUFF_BASELINE);
    seed_kiss_only_checks(tmp.path());
    augment_init_checks_with_precommit_python_gates(tmp.path()).unwrap();
    let lines = checks_lines(tmp.path());
    assert!(
        checks_cover_ruff_semantically(&lines),
        "baseline precommit should augment ruff; got {lines:?}"
    );
}

#[test]
fn augment_injects_mixed_case_ruff_from_precommit() {
    let tmp = tempfile::tempdir().unwrap();
    write_precommit_ruff_entry(tmp.path(), "rUfF cheCk .");
    seed_kiss_only_checks(tmp.path());
    augment_init_checks_with_precommit_python_gates(tmp.path()).unwrap();
    let lines = checks_lines(tmp.path());
    assert!(
        checks_cover_ruff_semantically(&lines),
        "mixed-case precommit should augment ruff; got {lines:?}"
    );
}

#[test]
fn brittle_precommit_ruff_case_fuzz() {
    let seed: u64 = std::env::var("BRITTLE_FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(rand::random);
    eprintln!("brittle_gate_signals_fuzz seed: {seed}");
    let mut rng = StdRng::seed_from_u64(seed);

    for _ in 0..32 {
        let cmd = random_ascii_case_permutation(RUFF_BASELINE, &mut rng);
        if cmd == RUFF_BASELINE {
            continue;
        }
        let tmp = tempfile::tempdir().unwrap();
        write_precommit_ruff_entry(tmp.path(), &cmd);
        seed_kiss_only_checks(tmp.path());
        augment_init_checks_with_precommit_python_gates(tmp.path()).unwrap();
        let lines = checks_lines(tmp.path());
        assert!(
            checks_cover_ruff_semantically(&lines),
            "precommit entry {cmd:?} should augment ruff (seed={seed}); got {lines:?}"
        );
    }
}
