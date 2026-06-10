# Summarize the activity

Malvin finished a multi-loop KPop session (`--max-loops` > 1). Read the session logs below—especially the KPop experiment logs—and summarize what happened for the user.

## Logs to read

- KPop transcript: `{{ kpop_log }}`
- Stdout log: `{{ stdout_log }}`
- Command log: `{{ command_log }}`
- Plan: `{{ plan_path }}`
- KPop experiment logs (under `{{ kpop_log_dir }}`):
{{ exp_log_paths }}


## Output format

Respond in this order:

1. **Executive summary**: 1–7 complete sentences covering goals, what was tried, outcomes, and remaining issues.
2. **TLDR**: One complete sentence summarizing the session, then up to three bullet points (each at most seven words).

Use clear, plain language. Be sure to capture all of the information in the KPop summaries from the KPop logs. Do not modify repository files.