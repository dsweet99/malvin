# LLM style — malvin

When `.cursorrules` says so, read this file first before any search or file read.

## Index (topic files)

- `malvin_tooling.md`: command/quality gate matrix, Rust+Python checks, `malvin do`/`malvin code` behavior.
- `malvin_debugging.md`: KPOP HPF, falsify, `_malvin/**/plan.md`, ABORT handling, workspace `rg` fallback.
- `malvin_kpop_schedule.md`: KPOP cadence, review timing, exact `LGTM` / `is_lgtm_str`.
- `malvin_evaluations.md`: `evaluations/` harness rules, `HOME` isolation, max-loops control.
- `authoring_llm_style.md`: index maintenance and new TRIGGER/ADVICE additions.

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.  
CONFIDENCE: 0

TRIGGER: grounding.md never edit  
ADVICE: Never edit `grounding.md`.  
CONFIDENCE: 4

TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; keep root `review.md` in sync.  
CONFIDENCE: 7

TRIGGER: style md line budget  
ADVICE: Keep `./.llm_style/style.md` under 100 lines; move longer ADVICE into topic files.  
CONFIDENCE: 0

TRIGGER: no git commands  
ADVICE: Do not run git commands; user manages git state.  
CONFIDENCE: 1

TRIGGER: chat learnings  
ADVICE: When asked, return short bullets on new learnings: codebase structure, algorithms/methods/results, tooling constraints, user preferences, agent pacing, code quality.  
CONFIDENCE: 1

TRIGGER: all checks pre-commit  
ADVICE: If a behavior-affecting change is made, run at least: `kiss check .`, `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc`, `cargo test`, `ruff check`, `pytest -sv tests`.  
CONFIDENCE: 3

TRIGGER: behavioral tests first  
ADVICE: Prefer behavior-level assertions over structure-only checks. For `malvin do` and `malvin code`, validate exact stdout content and avoid leaked protocol chrome.  
CONFIDENCE: 1

TRIGGER: KPOP LGTM exact  
ADVICE: Maintain `review.md` as exactly `LGTM` (trimmed) for successful automation handoff.  
CONFIDENCE: 3

TRIGGER: plan grounding search  
ADVICE: Use root `plan.md` plus `_malvin/**/plan.md` and `grounding.md` as the workflow control path.  
CONFIDENCE: 4
