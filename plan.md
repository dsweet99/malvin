# Malvin — product plan

This document tracks planned work for the `malvin` CLI and related behavior. Decisions below supersede older drafts.

---

## 1. `malvin init` — initialize a repository

Add a subcommand **`init`** that sets up a new or existing repo with sensible defaults (run in the target directory or with an explicit path).

### Sources

Templates live under **`default_repo/`** in the malvin repository:

| Template in `default_repo/` | Installed as |
|-----------------------------|--------------|
| `gitignore`                 | `.gitignore` |
| `kissignore`                | `.kissignore` |
| `pre-commit-config.yaml`    | `.pre-commit-config.yaml` |
| `grounding.md`              | `grounding.md` |

Also copy **`admin/check_untracked.sh`** from the malvin repo into the target repo at **`admin/check_untracked.sh`** (same relative path), for use by pre-commit hooks.

### Behavior

- Create **`.pre-commit-config.yaml`** from the template if missing, then run **`pre-commit install`**. If `pre-commit` is missing, fail with a clear message (or document a soft skip—pick one behavior and test it).
- Run **`kiss init`** (creates `.kissconfig`). Do not hand-edit `.kissconfig` in consumers; follow project rules.
- Run **`git lfs install`** for Git LFS setup (not `git install lfs`). If `git-lfs` is not on `PATH`, fail fast with an actionable error.
- Add **`grounding.md`**, **`.gitignore`**, **`.kissignore`** from templates when missing, using the table above.
- **Idempotency:** By default, do **not** overwrite existing files. Support **`--force`** to refresh listed files from `default_repo/` (and re-copy `admin/check_untracked.sh` when forced, if we include it in the force set—document which paths `--force` touches).

---

## 2. ACP — retries with backoff on RPC failures

When the agent fails with errors such as:

`agent acp (coder prompt) failed after retries. Last error: acp RPC timed out`

…apply a **bounded retry policy** so transient timeouts are less likely to fail the whole run.

Also, if you get the message "Upgrade your plan to continue", just stop. Always show that to the user on stderr, irresepective of tee on or off.

### Policy (decided)

- **Up to 3 attempts** per retriable failure (initial try plus up to two retries after failure).
- **Waits between attempts:** **1 s** before the 2nd attempt, **3 s** before the 3rd attempt (after failures).
- **Scope (initial implementation):** focus on failures during **`session/prompt`** and other JSON-RPC calls that surface as RPC timeouts. Handshake / `session/new` may use the same policy or a simpler single-retry—implement consistently and document in code comments.

---

## 3. Logging — tee and stdout

### Current behavior (baseline)

Today the CLI uses **`--no-tee`** to disable tee; default is tee **on**. When tee is on, ACP trace content is printed to stdout (see agent tee path and `SharedOpts`). Run-directory logs (e.g. trace files, `command.log`) are still written regardless.

### Plan

- Keep **tee on by default** and **`--no-tee`** to disable (align help text with `grounding.md`).
- When tee is on, ensure **every** ACP-driven phase (coder, review, kpop, learn, etc.) is covered—no silent subsets.
- **Stretch (optional):** true **streaming** of RPC/trace lines to stdout *during* an in-flight prompt, if the transport allows it without breaking the existing trace file contract. If not feasible short-term, document that tee follows “after prompt” or incremental reads—match actual implementation.

---

## 4. Default model — `composer-2`

**Change** the CLI default model from the current **`opus-4.5`** to **`composer-2`** (`SharedOpts` / spawn args), unless the user passes **`--model`**.

Update tests and any docs that assert the old default.

---

## 5. `malvin models` — list models from Cursor agent

Add **`malvin models`** that:

- Runs **`cursor-agent models`** (or the supported equivalent) to obtain the list.
- **Parse** stable fields for **name** and **description** (strip ANSI, drop trailing “tip”/banner lines). If parsing fails, fall back to a stripped pass-through rather than crashing—unless we prefer fail-fast; prefer robust display.
- Print a **plain** list: names and descriptions.
- At the **bottom**, state that **`composer-2`** is the default model in `malvin`.

### When `cursor-agent` is missing

If the binary is not found or execution fails: exit **non-zero** with a short, actionable message (no generic “toast” without detail). The tool is unusable without the agent; make that explicit.

---

## Open questions

1. **`pre-commit` missing during `init`:** Fail hard, or skip hook installation with a warning?
2. **`--force` scope:** Should `--force` overwrite only `default_repo` files, or also always refresh `admin/check_untracked.sh`?
3. **Stretch streaming (§3):** Ship “full coverage of existing tee” in v1 and defer true streaming to a follow-up?
