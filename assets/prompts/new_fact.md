# Find one new fact in your logs that is worth remembering

`malvin --advice facts` lists time-saving facts. Find one more fact that earlier sessions kept
rediscovering, support it with evidence, and add it to the list.

## Scope

Search every log on this machine, not only those of the current project. Logs live in
`~/.malvinconf/logs/<project-hash>/<YYYYMMDD_HHMMSS_id>/`. Each `<project-hash>` directory belongs
to one working directory, named by the file `work_dir` in any of its sessions.

## What counts as a fact worth remembering

A **fact** is a true, specific statement about this machine, a project, the operator's
preferences, or malvin's behavior that a session has needed in order to work and could learn directly from the
request. A **rediscovery** is a session that established the fact from its own direct evidence (a
tool call, measurement, failed command, etc.) when an earlier session already also had. [A
session that cites an earlier log (its messages contain `HISTORY-`) and uses that answer *reused*
the fact; reuse does not count as rediscovery.]

A candidate must meet requirements 1 and 2 and at least one of 3–5:

1. **Rediscovered in at least three independent sessions.** Sessions in different project
   directories are independent. Two sessions on the same request minutes apart count as one,
   unless each gathered the evidence itself.
2. **Cost at least three minutes in one rediscovery, or ten minutes in total.** Measure the gaps
   between consecutive `t|` timestamps in `main/stdout.log` during the detect–fail–diagnose–work-around
   steps, or use `wall_clock_ms` in `run_timing.json` when the whole session went to it.
3. **Sometimes went undiscovered.** At least one session lacked the fact and failed, gave a wrong
   result, or ended abruptly. Count these; they are stronger evidence than rediscoveries.
4. **Rediscovery was risky.** Learning it required something destructive, long, memory-hungry, or
   network-dependent, or a guess.
5. **Ignorance is risky.** Without it a session could damage files, exceed the sandbox memory
   limit, run up cost, or report something false.

For every candidate, record which requirements it meets and the number behind each.

Exclude:

- Facts already on the list, and restatements of them. Printing the list echoes its text into
  logs, so keywords from it match far more sessions than ever rediscovered anything.
- Facts worth re-checking because the answer changes, such as model prices.
- Anything established in only one session, however costly.

Prefer kinds of fact the list lacks: a project convention (which gate command, which environment,
which directory is the real one), a recurring operator preference (report format, typefaces,
tone), or a negative fact (something tried in several sessions that never works).

## How to search

Full commands are not in the logs: `t|` lines in `main/stdout.log` and `summary` fields of
`tool_call` events in `main/trace.jsonl` are cut to about sixty characters. Searchable full text:

- `m|` lines in `main/stdout.log`; `assistant`, `thinking`, and `run_done` events in `main/trace.jsonl`;
- `_run/exp_log_*.md` and `_run/understanding.md`;
- `plan_*.md` and `request_summary.txt` (the operator's request);
- `sandbox_oom.json`, `malvin_error.log`, and `router_*.log`.

Method: read every session's request and evidence log to collect candidates; grep the narrative
files across all sessions for each candidate's key terms; open each matching session's
`main/stdout.log` to classify it as rediscovery, reuse, or lacking, and to time it. Record the
sessions, quotations, measurements, and grep commands in your evidence log. Follow
`malvin --advice kpop` for the investigation.

## What to deliver

 **Add the best fact:**

       malvin admin facts add fact_id=<id> fact='<rule, then detail>' symptoms='<trigger>' symptoms='...'

   `fact_id`: lowercase letters, digits, `-`, `_`; at most 64 characters. `fact`: one line of at
   most 4,000 characters, starting with a one-sentence imperative rule ("Run ... with ...",
   "Do not ..."), then the scope (this machine, a named project, or malvin), a shell command that
   verifies it in seconds, and the verification date. Verify it yourself first; never add a fact
   that is no longer true. Give three to six `symptoms` a session can observe (a command printing
   nothing, an error message, a kind of request).

   Add exactly one fact. If no candidate qualifies, add nothing and say so.


