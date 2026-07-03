- Discover how the repo in `{{ repo_root_path }}` runs quality gates (linters and unit tests; not integration tests or other slow or complex tests) today and write the result to `{{ repo_root_path }}/.malvin/checks` (one shell command per non-empty line, in run order).
- Always include `kiss check` unless the repo explicitly excludes kiss.
- Examples of places to look for linter and tester invocations:
  - `.pre-commit-config.yaml`
  - `Makefile` / `justfile`
  - `pyproject.toml` / `package.json` scripts
  - `.github/workflows/*.{yml,yaml}` CI steps
  
