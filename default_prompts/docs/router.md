# malvin (default route)

Outer agent sessions (`effective_max_loops(--max-loops)`): each session writes `review_requirements.json`, runs **one** multi-group KPop gap-analysis turn, then optionally `router_work.md`. KPop chat headings decide whether work runs and whether the outer loop can stop.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as `tidy` / `inspire`); with `--quiet` / `-q`, only `MALVIN_DM_*` bodies |
| Logs | `router_N.log` under `~/.malvin_home/logs/<hash>/<run>/` (one file per outer session) |
| Contract file | `review_requirements.json` in the malvin run directory (`{{ review_requirements_path }}`) |
| Requires | No `.malvin/checks` at startup (unless `--gates` later needs them) |

## Intention

Read the user request, invent a small set of grouped review requirements, gap-analyze **all** groups in one KPop turn (residual plans or no-work markers in chat), then execute residual work when needed. When the outer loop decides to exit, send `router_summarize.md` once on that final already-open coder session before teardown (not after intermediate sessions that will continue). Repeat for another outer agent lifetime when `--max-loops` allows and stop conditions are not met.

## Usage

```text
malvin [OPTIONS] <REQUEST>
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow.

## Arguments

### `<REQUEST>` (required)

Exactly **one shell argument**. Quote for internal spaces. Literal text, or an existing `.md` file path (same rules as `--do`).

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `--max-loops` | Outer agent-session budget (`effective_max_loops`; default 1). Tenacious expands to 9999 unless this flag is set on the command line. |
| `--gates` | After each outer session’s turns (while the coder session is still open), run workspace `.malvin/checks`. Pass stops success; fail continues (even when chat said no work remaining). Exhausted budget with failing gates fails the run. Also injects check text into `router_work.md` when work runs. |
| `--no-tenacious` | Keep normal `--max-loops` / `--max-acp-retries` (default tenacious expands both) |
| `--no-tee` | Disables live streaming |
| `--quiet` / `-q` | Stdout shows only `MALVIN_DM_*` bodies (not `-b`; `--do` is always DM-only without this flag) |
| `--verbose` | Full prompt bodies in `prompts.log` |

## Prompt workflow

Each outer session opens one coder session and sends:

| Turn | Piece | Role |
|------|-------|------|
| 1 | `header.md` | Standard Malvin coding context |
| 1 | `router_requirements.md` | Write grouped review requirements JSON only |
| 1 | User request | Appended after headers |
| 2 | `kpop_common.md` + `router_kpop_group.md` | **One** turn covering all groups; residual plan or `## NO_WORK_REMAINING N` per index; hypotheses to `_kpop` exp log (`max_hypotheses` from `[default_workflow].max_hypotheses`, default 5) |
| 3 (optional) | `router_work.md` | Run only when any group has `## Group Work N` or a missing/unclear deliverable; optional `{{ code_extra }}` when `--gates` |
| Exit only | `router_summarize.md` | **Once per run**, when exiting the outer loop: pass to the same already-open final coder session before teardown; first-person DM summary of the session |

Before turn 1, malvin clears any stale `review_requirements.json`. After turn 1, it loads and validates the file. Missing, malformed, or over-limit files fail the run. Schema:

```json
{
  "groups": [
    {
      "title": "optional short label",
      "requirements": ["...", "..."]
    }
  ]
}
```

Validation: `groups.len() ∈ 1..=3`; each group's `requirements.len() ∈ 1..=3`; requirement strings non-empty after trim. Empty `groups` is rejected (stop signal is KPop chat, not empty JSON).

### Stop / continue (without `--gates`)

After the KPop turn, malvin parses chat for indexed headings. **`all_no_work`** when every `N` in `1..=groups.len()` has a `NO_WORK_REMAINING N` marker and no `Group Work N` marker. Canonical form is a whole-line `## TOKEN N`; the parser also accepts mid-line glued `##` forms, alternate `#` depth, and bare `TOKEN N` with a non-identifier boundary (wrong index still fails). Then skip work and stop success. Otherwise run work and, if outer budget remains, tear down **without** summarize and start another session. Exhausting the budget without `--gates` is success (with the single exit summarize on that final session).

### Stop / continue (with `--gates`)

After each outer session’s turns (whether work ran or was skipped), run workspace gates while the coder session is still open:

| Condition | Action |
|-----------|--------|
| Gates pass | Send exit summarize on the open session, tear down, stop success |
| Gates fail, loops remain | Tear down **without** summarize; restart (even if chat said all `NO_WORK_REMAINING`) |
| Gates fail, budget exhausted | Send exit summarize on the open session, tear down, fail with a workspace gate error |

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `review_requirements_path` | `router_requirements.md` | run artifacts |
| `groups_block` / `max_hypotheses` / `exp_log` | `router_kpop_group.md` | multi-group assembly |
| `code_extra` | `router_work.md` | `router_code_extra.md` when `--gates` |

When the outer loop decides to exit, malvin sends `router_summarize.md` on the same final coder session, then ends the session. Intermediate sessions that continue do not receive summarize.

## Config

`~/.malvin_home/config.toml`:

```toml
[default_workflow]
max_hypotheses = 5
```

Missing section falls back to 5. This path does **not** use `[agent].max_hypotheses`.

## Examples

```text
malvin "add a CLI flag for dry-run"
malvin --max-loops 3 notes/task.md
malvin --gates --max-loops 5 notes/task.md
malvin --no-tenacious --max-loops 1 "small fix"
```
