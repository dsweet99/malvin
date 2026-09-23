## ripwire

Ripwire is a CLI that builds a deterministic, ranked call-graph map of a repository and emits a compact, token-budgeted context bundle (XML or JSON). Use it on demand via shell to orient before blind search, expand one symbol instead of whole files, and estimate change blast radius.

Example commands:

```bash
# Orient on a task: ranked signatures for the request
ripwire . --for="agent session spawn and quality gates"

# Expand one symbol body instead of reading a whole file
ripwire . --expand=run_router --top-k=0

# Blast radius before a risky edit
ripwire . --impact=run_router
```

Do not pass `--top-k` with `--impact` (`--top-k=0` exits 1). To page impact results, use `--limit` instead. Prefer keeping stderr visible when learning flags so failures are not hidden by `2>/dev/null`.

# Acronyms
PWS: Problems Worth Solving. See ~/problems_worth_solving.md
