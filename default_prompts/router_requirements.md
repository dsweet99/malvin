Generate grouped review requirements for the user request at `{{ user_request_path }}`.

## Hard constraints

- Write **only** the JSON file at `{{ review_requirements_path }}`. Do not edit other files.
- Do **not** start implementing, fixing, or investigating beyond what is needed to list requirements.
- Use at most **5** groups. Use fewer when that is enough; zero groups is allowed when there is nothing to review.
- Each group must have between **1** and **5** requirements (inclusive). Use fewer when that is enough.
- Requirement strings must be non-empty after trimming.
- Optional short `title` per group; omit or use `""` when a label adds nothing.

## Schema

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

After writing that file, output nothing else of substance — Pause.
