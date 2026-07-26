# malvin (default route)

One coder session: `header.md` + `router_requirements.md` + user request, then one KPop-shaped gap-analysis prompt per requirements group, then `router_work.md`. The agent is torn down after the work turn.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Styled stdout on a TTY (same startup chrome as `kpop` / `tidy`) |
| Logs | `router_1.log` under `~/.malvin_home/logs/<hash>/<run>/` (all turns in one file) |
| Contract file | `review_requirements.json` in the malvin run directory (`{{ review_requirements_path }}`) |
| Requires | No `.malvin/checks` at startup |

## Intention

Read the user request, invent a small set of grouped review requirements, gap-analyze each group with the KPop method (residual plan written to chat), then execute the planned work in the same session.

## Usage

```text
malvin [OPTIONS] <REQUEST>
```

There is no `router` subcommand. Bare `malvin REQUEST` is the default autonomous routing workflow.

## Arguments

### `<REQUEST>` (required)

Exactly **one shell argument**. Quote for internal spaces. Literal text, or an existing `.md` file path (same rules as `do`).

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for the default route:

| Flag | Effect |
|------|--------|
| `--gates` | Inject workspace check command text into the final `router_work.md` prompt (default: off). Does **not** restart the agent when checks fail. |
| `--no-tenacious` | Keep normal `--max-acp-retries` (default tenacious expands ACP retries only) |
| `--no-tee` | Disables live streaming |
| `--verbose` | Full prompt bodies in `prompts.log` |
| `--max-loops` | Legacy no-op on the default route (single session; kept for CLI compatibility) |

## Prompt workflow

Malvin opens one coder session and sends:

| Turn | Piece | Role |
|------|-------|------|
| 1 | `header.md` | Standard Malvin coding context |
| 1 | `router_requirements.md` | Write grouped review requirements JSON only |
| 1 | User request | Appended after headers |
| 2… | `kpop_common.md` + `router_kpop_group.md` | One turn per group: residual plan to chat; hypotheses to that group's `_kpop` exp log (`want` = `DEFAULT_MAX_HYPOTHESES`) |
| last | `router_work.md` | Execute the residual plans already in chat; optional `{{ code_extra }}` when `--gates` |

After turn 1, malvin loads and validates `review_requirements.json`. Missing, malformed, or over-limit files fail the run. Schema:

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

Validation: `groups.len() ∈ 0..=5`; each group's `requirements.len() ∈ 1..=5`; requirement strings non-empty after trim. Zero groups skips per-group KPop and still sends the final work prompt.

### Required template keys

| Key | Required by | Value source |
|-----|-------------|--------------|
| `logs_dir` | `header.md` | `malvin_logs_root(work_dir)` |
| `current_state` | `header.md` | `format_current_state(...)` |
| `git_extra` | `header.md` | `--git` → `You may run 'git commit'.`; otherwise `""` |
| `user_request_path` | `router_requirements.md` | `format_prompt_path(plan_path, work_dir)` |
| `review_requirements_path` | `router_requirements.md` | run-dir `review_requirements.json` |
| `malvin_command` | work / tools | `malvin --model=<active_model>` |
| `want` / `exp_log` / `group_*` | `router_kpop_group.md` | per-group prompt assembly |
| `code_extra` | `router_work.md` | `router_code_extra.md` when `--gates` |

## Session behavior

- Ensures `~/.malvin_home/config.toml` exists with defaults (same as `do`).
- Backs up `.gitignore`, `.malvin/checks`, `.malvin/config.toml`, and `~/.malvin_home/config.toml` at session start; restores session dotfiles at run end.
- Does **not** auto-run `malvin init` between turns.
- Checks `result.md` for `ABORT:` after the session completes.

## Related commands

| Command | When |
|---------|------|
| `malvin do` | One-turn direct answer without routing brief |
| `malvin kpop` | Hypothesis-driven investigation with `_kpop/` log |
| `malvin inspire` | Creative ideation without routing |

## Examples

```text
malvin "Figure out why tests fail and fix them"
malvin --gates notes/task.md
malvin --no-tenacious "Quick one-shot route"
```
