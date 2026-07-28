Generate grouped review requirements for the user request at `{{ user_request_path }}`.

## Hard constraints

- Write **only** the JSON file at `{{ review_requirements_path }}`. Do not edit other files.
- Do **not** start implementing, fixing, or investigating beyond what is needed to list requirements.
- Use at most **3** groups. Use fewer when that is enough.
- Each group must have between **1** and **3** requirements (inclusive). Use fewer when that is enough.
- Where the user specifies something precisely, follow them precisely.
- Where there is ambiguity, resolve it with prior domain knowledge (either that you already possess or that you acquire through appropriate research).
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
