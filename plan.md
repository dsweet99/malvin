# Plan: Remove kiss and hardcoded gate tools from malvin

## User Request

I'd like to remove kiss from malvin.

Extended scope (same effort): **remove pytest, cargo, clippy, and all other malvin-owned linter/tester specifications.** Malvin and Modal/DeepSWE must not seed or mandate particular quality tools. Checks should come from **repo signals** (pre-commit, Makefile, existing `.malvin/checks`) or from **checks-discovery KPop** inferring what the repo already uses — not from malvin builtins.

(Summarized from prior discussion: kiss as an agent quality gate has been net harmful on DeepSWE — misaligned with Harbor grading, drove API refactors, consumed agent budget.)

## Current State

### External quality gates (agent-facing)

Malvin and DeepSWE today **specify** tools in several places:

| Area | Location | Runtime behavior |
|------|----------|------------------|
| Default gate command (Rust) | `src/repo_gates/mod.rs` | `builtin_gate_command_lines()` → **`kiss check` only**. Used by test helper `ensure_default_malvin_checks_file()` and `gate_restore_repair.rs` (replaces bare `kiss`-only checks). |
| Default gate commands (DeepSWE Python) | `ops/deepswe_run.py` | `builtin_gate_command_lines(root)` adds **`kiss check`**, then **`pytest -sv tests`**, **`stestr run`**, **`cargo clippy …`**, **`cargo test` / `cargo nextest run`** when repo heuristics match. Merged after pre-commit/Makefile scan. |
| Kiss clamp / dotfiles | `kiss_clamp.rs`, `session_dotfile_backup/` | `kiss clamp` at snapshot; backs up `.kissconfig` / `.kissignore`. |
| CLI kiss requirement | `entrypoint.rs` | `code` / `tidy` require kiss on PATH. |
| Checks discovery prompt | `init_constraints.md` | Agent discovers repo gates; **mandates `kiss check`**. |
| DeepSWE discovery | `discover_deepswe_check_lines()` | Pre-commit + Makefile scan **plus** builtin fallbacks + stestr/pytest swap + `ensure_kiss_check_first()`. |
| Sandbox prep | `ops/sandbox_prep.py` | `probe_check_tools()` — tool-specific probes for kiss, ruff, pytest, stestr, mypy, cargo. |
| Modal offline installs | `deepswe_modal.py` | `offline_check_tool_install_commands(checks)` preinstalls mypy/ruff packages when **checks text** mentions them (reactive, not a default list). |

**Malvin Rust core** has **no** pytest/cargo/clippy builtins today — only kiss. **DeepSWE ops** is where pytest/cargo/clippy/stestr defaults live.

**Gate flow today (code/tidy/do KPop):**

1. CLI requires kiss on PATH (code/tidy only).
2. Checks discovery KPop if `.malvin/checks` missing/empty (mandates kiss in prompt).
3. KPop snapshot may run `kiss clamp` (including `do` without CLI kiss check).
4. Gates run `.malvin/checks` lines (often kiss + repo or DeepSWE-seeded pytest/cargo).

### Internal malvin kiss coverage (developer-only)

`coverage_kiss/`, `*_kiss_cov_*` witnesses — malvin-repo CI only, not workspace agent gates. Optional Phase 4.

### Adjacent behavior after removal

- **No malvin-owned gate commands:** Rust `builtin_gate_command_lines()` → `[]`. DeepSWE deletes `builtin_gate_command_lines()`, `DEFAULT_*_CHECK` constants, kiss-first and stestr-append logic.
- **Checks discovery KPop stays:** Agent reads pre-commit, Makefile, CI, etc.; **no mandated tools** in `init_constraints.md`.
- **DeepSWE scan-only:** `discover_deepswe_check_lines()` keeps pre-commit + Makefile + existing `.malvin/checks` merge; **no builtin append loop**. Empty scan → empty checks file (or newline-only); workflows must tolerate empty or fail at gate time.
- **Existing user `.malvin/checks`:** Not rewritten by malvin.
- **`malvin do` without kiss:** Removing snapshot clamp fixes snapshot failure when kiss missing.

## Requested Changes

1. Remove kiss from agent pipeline (gates, clamp, dotfiles, CLI requirement, docs).
2. Remove **all malvin/Modal hardcoded linter and tester commands** (pytest, cargo clippy, cargo test, stestr, kiss, etc.) from builtins and discovery fallbacks.
3. **Keep checks-discovery KPop** — agent infers commands from repo files; malvin does not name specific tools in prompts or defaults.
4. **DeepSWE:** keep pre-commit/Makefile scanning only; drop builtin fallbacks and tool-specific sandbox probing.
5. Keep malvin functional when `.malvin/checks` is populated by discovery or the user.

## Q&A

### Q1. Does “remove kiss” include malvin’s internal `coverage_kiss/` harness?

**Answer:** **No** for Phases 1–3. Optional Phase 4. `.pre-commit-config.yaml` malvin dev hooks are also out of scope unless Phase 4.

### Q2. What replaces missing `.malvin/checks`?

**Answer:**

- **Malvin (`code`/`tidy`/`do`):** checks-discovery KPop runs when file missing/empty; agent writes commands found in repo config. No malvin default commands. Fail clearly if discovery finishes with no commands.
- **DeepSWE:** `discover_deepswe_check_lines()` returns pre-commit + Makefile + existing checks only — **no builtins**. Empty repo → empty checks (not `kiss check\n` or pytest).
- **Rust `builtin_gate_command_lines()` → `[]`**. Delete kiss repair paths in `gate_restore_repair.rs`; do not auto-fill checks.

### Q3. Should malvin strip tools from existing `.malvin/checks`?

**Answer:** **No.** Stop seeding/mandating; do not rewrite user files.

### Q4. What happens to `.kissconfig` / `.kissignore`?

**Answer:** Drop from session backup/restore and kiss merge/repair logic. Files may remain on disk unmanaged.

### Q5. DeepSWE Docker/Modal image?

**Answer:** Remove kiss install and kiss-first discovery. Remove builtin pytest/cargo/stestr seeding. Keep scanning repo config files only.

### Q6. `sandbox_prep` and Modal offline tool installs?

**Answer:** **Remove `probe_check_tools()` entirely** (user decision). Review `offline_check_tool_install_commands()` in `deepswe_modal.py` — it reacts to checks text, not malvin defaults; keep only if still needed for Harbor offline sandboxes, or simplify in Phase 3.

## Plan

### Phase 1 — Remove kiss from core gate pipeline (Rust)

- [x] **`src/repo_gates/mod.rs`:** Remove kiss constants; `builtin_gate_command_lines()` → `[]`.
- [x] **Delete:** `kiss_clamp.rs`, `kissconfig_warn.rs`; simplify `gate_run.rs`.
- [x] **`entrypoint.rs` / `support_paths.rs`:** Remove kiss CLI requirement.
- [x] **`session_dotfile_backup/slots.rs`:** Remove kiss rows; renumber slots. Update `mod.rs`, `wrappers.rs`, merge/restore.
- [x] **`artifacts/mod.rs`:** Drop kiss re-exports.
- [x] **Gate restore:** Delete kiss merge/repair in `gate_restore_merge.rs`, `gate_restore_checks.rs`, `gate_restore_repair.rs` (including `default_malvin_checks_bytes` kiss replacement).
- [x] **Tests:** repo_gates, gate_run, session_dotfile_backup, do_stdout/clamp, kiss_*_gate_path, code_kpop_contract, acp_do_dotfiles, cli_parity, kpop_bridge, review_prep/gate_error regression.

**Validation:**

- `cargo test repo_gates`
- `cargo test gate_run_tests`
- `cargo test session_dotfile_backup`
- `cargo test checks_discovery`
- `cargo build --release`
- `rg 'require_kiss|kiss_clamp|KISS_CHECK' src tests` — no matches

### Phase 2 — Prompts, docs, checks discovery (no mandated tools)

- [x] **`init_constraints.md`:** Remove “Always include `kiss check`…”. Keep “discover how the repo runs quality gates” with pre-commit/Makefile/CI examples — **do not name pytest, cargo, clippy, kiss, or other specific tools** as requirements or examples of what to always include.
- [x] **Docs:** Remove kiss PATH requirement, kiss dotfiles from backup lists, kiss-metric success criteria (`code.md`, `tidy.md`, `malvin.md`, `kpop.md`, `do.md`, `inspire.md`, `kpop_program.md`, `malvin_post.md`).
- [x] **`README.md`:** Remove `cargo install kiss-ai` agent prerequisite.
- [x] **Discovery tests:** `checks_discovery_flow.rs`, `tests/checks_discovery.rs` — assert discovery produces **repo-derived** commands (e.g. from seeded Makefile/pre-commit fixture), not `kiss check` or hardcoded pytest/cargo.
- [x] **Prep/workflow tests:** Replace `seed_malvin_checks(..., "kiss check\n")` with neutral fixtures (`make lint\n`, `true\n`, or repo-specific lines).

**Validation:**

- `cargo test checks_discovery`
- `grep -rE 'kiss check|kiss-ai|Always include.*kiss|pytest -sv|cargo clippy' default_prompts README.md` — no agent-facing mandated-tool references
- Discovery prompt text contains no tool mandates

### Phase 3 — DeepSWE and ops (scan-only, no builtins, no probe_check_tools)

- [x] **`ops/deepswe_run.py`:**
  - Delete `KISS_CHECK_COMMAND`, `DEFAULT_PYTEST_CHECK`, `DEFAULT_STESTR_CHECK`, `DEFAULT_RUST_*`, `builtin_gate_command_lines()`, `ensure_kiss_check_first()`, kiss install in Docker build, `.kiss` markers.
  - **`discover_deepswe_check_lines()`:** Keep `precommit_hook_entries`, `makefile_gate_targets`, `existing_malvin_checks_lines`, `dedupe_check_lines` — **remove** builtin fallback loop, stestr/pytest swap that appends `DEFAULT_STESTR_CHECK`, and kiss-first wrapper.
  - **`discover_deepswe_checks()`:** Empty/missing workspace → `""` or `"\n"`, not kiss/pytest.
  - Remove helpers only used by deleted builtins (`python_ruff_and_pytest_flags`, `repo_uses_stestr` gate-append paths) if unused after scan-only discovery.
  - Rewrite `_test_discover_deepswe_checks_*` — minimal repo expects **empty** lines; python repo with tests dir but no pre-commit expects **empty** (no auto-pytest); pre-commit fixture still discovers `ruff check`.
- [x] **`ops/deepswe_modal.py`:** Remove kiss image install and kiss self-tests; align with scan-only discovery; review `offline_check_tool_install_commands` / `offline_agent_checks` (keep reactive installs or trim in same PR).
- [x] **`ops/sandbox_prep.py`:** **Remove `probe_check_tools()`** and its call site in `prepare_task_sandbox`; delete probe self-tests.
- [x] **`ops/toolchain_repos.py`:** Remove kiss helpers when unused.
- [x] **`tests/test_deepswe_run_selftest_discover.py`:** Update for scan-only behavior; drop `test_deepswe_kiss_repo_root` if applicable.
- [x] **Optional:** Delete `ops/kiss_triage/`, `tests/test_kiss_admin_tooling_contract.py`.

**Validation:**

- `python -m pytest tests/test_deepswe_run_selftest_discover.py -q`
- `python -m pytest tests/test_ops_selftest.py -q` (sandbox_prep self-tests)
- `rg -E 'DEFAULT_PYTEST|DEFAULT_RUST|DEFAULT_STESTR|KISS_CHECK|ensure_kiss_check_first|builtin_gate_command_lines|probe_check_tools' ops/` — no matches
- Fixture repo with only `tests/test_foo.py` and no pre-commit → discovered checks **empty**
- Fixture repo with `.pre-commit-config.yaml` ruff hook → discovered checks contain ruff line only

### Phase 4 (optional) — Remove internal kiss dependency entirely

- [ ] `coverage_kiss/`, `*_kiss_cov_*`, `.pre-commit-config.yaml` kiss hook.

**Validation:** `cargo test` without kiss on PATH.

**Note:** Independent of agent behavior; defer unless requested.
