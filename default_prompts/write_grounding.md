{{ coding_rules }}

Read the existing codebase and write a new grounding file to `{{ grounding_path }}`.

Your job is to produce a concise project contract for future agent runs.

Read broadly enough to ground the document in evidence:
- source files
- tests
- build/config files
- CLI/help text
- existing docs, if they match the codebase

Use up to 4 parallel subagents.

Write a concrete grounding file that captures the stable, high-value truths of this repository:
- what the project is for
- which languages and major tools are actually used
- key workflows, commands, artifacts, and interfaces
- hard constraints, invariants, and non-goals
- required checks, tests, and review expectations
- important operational assumptions

Rules:
- Prefer facts supported by code, tests, configs, and docs that match the codebase.
- Do not invent roadmap items, aspirations, or behavior you cannot support.
- If you are unsure, omit the point or phrase it conservatively.
- Keep it concise and scannable. Prefer headings and bullets over long prose.
- Describe stable contracts, not incidental implementation details.
- Include exact command lines when they are clearly established by the repo.
- Only create `{{ grounding_path }}` when it does not already exist.
- Do not modify any file other than `{{ grounding_path }}`.
