# KPOP experiment log: why most of the Rust port is untracked

## Problem (restatement)

The working tree contains a full Rust port (`src/lib.rs`, `src/acp/`, `src/agent/`, `default_prompts/`, `tests/`, etc.), but **git only tracks `src/main.rs` under `src/`**. A clone of the branch without those files would fail to build, and `git diff main` understates the real change set. We want a **falsifiable** explanation for why those paths show as `??` (untracked).

---

## Round 1

### Hypothesis H1

**Cause:** Untracked files match a **`.gitignore` (or exclude) rule**, so git deliberately keeps them out of the index.

### Predict

If H1 is true, then:

```bash
git check-ignore -v src/lib.rs src/acp/mod.rs default_prompts/implement.md tests/cli_parity.rs
```

exits **0** and prints the matching rule and path.

### Falsify (test)

**Command:**

```bash
git check-ignore -v src/lib.rs src/acp/mod.rs default_prompts/implement.md tests/cli_parity.rs; echo exit:$?
```

**Result:** Exit code **1**, **no output** (no path is ignored by ignore rules).

**Conclusion:** **Reject H1.** Ignore rules are not why these files are untracked.

---

## Round 2

### Hypothesis H2

**Cause:** **Sparse checkout** (or similar) limits which paths are checked out or tracked, so library sources appear only as local files outside git’s view.

### Predict

If H2 is true, then `.git/info/sparse-checkout` exists and/or `core.sparseCheckout` is true, and relevant paths are excluded from the sparse set.

### Falsify (test)

**Commands:**

```bash
test -f .git/info/sparse-checkout && cat .git/info/sparse-checkout || echo 'no sparse-checkout file'
git config --get core.sparseCheckout
```

**Result:** No `sparse-checkout` file; `core.sparseCheckout` unset.

**Conclusion:** **Reject H2.** Not a sparse-checkout artifact.

---

## Round 3

### Hypothesis H3

**Cause:** The library and asset files were **never added to the index** (never `git add`’d) after creation, while `src/main.rs` was committed at least once.

### Predict

If H3 is true, then:

- `git ls-files src/` lists **only** `src/main.rs`.
- `git log --oneline --all -- src/lib.rs` is **empty** (no commit has ever recorded that path).

### Falsify (test)

**Commands:**

```bash
git ls-files src/
git log --oneline --all -- src/lib.rs
```

**Result:**

- `git ls-files src/` → **`src/main.rs` only**.
- `git log --oneline --all -- src/lib.rs` → **empty** (no lines).

**Conclusion:** **H3 is consistent with observations and is not falsified** by these tests. The simplest remaining explanation is **operator/process omission**: port files exist on disk but were not staged or committed.

---

## Resolution (problem “solved” in root-cause sense)

**Accepted explanation:** Untracked status is due to **missing `git add` / commits** for the port tree and bundled prompts/tests, **not** due to ignore rules or sparse checkout.

**Practical fix (outside this log):** Stage and commit `src/lib.rs`, `src/**` (except build artifacts), `default_prompts/`, `tests/`, `pyproject.toml`, `.gitignore`, etc., as appropriate for the project’s release policy.

---

## Session metadata

- Repo: `/home/dsweet/Projects/malvin`
- Branch context: `dsweet/port` (from prior `git status` snapshot)
- Log path: `_malvin/20260410_130102_3b8d7e7b/_kpop/exp_log_git_untracked_root_cause.md`
