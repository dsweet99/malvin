# FT-05: Extract and cite membrane potential threshold

Edit only files in this workspace.

## Task
Read only files under `corpus/` (five Markdown abstracts + methods snippets). Answer: which cited author string first states that resting membrane potential is approximately −70 mV?

Write `answer.json` exactly:

```json
{"claim":"<verbatim ≤120 char quote>","source_path":"corpus/<file>","author":"<lastname>"}
```

The claim field must contain the substring `-70` and the characters `mV`. Paths must be relative as shown.

## Rules
- Corpus only; no network.
- Verbatim quote from the corpus (grader requires exact claim match).

## Done when
`answer.json` matches the golden claim, path, and author.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
