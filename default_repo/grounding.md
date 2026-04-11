# Project grounding (template)

Replace this file after copying `default_repo/` into a new repository. It should describe **your** project, not abstract placeholders.

## What belongs here

- **Purpose**: What the codebase is for and who it serves.
- **Long-lived constraints**: Policies and invariants that should survive refactors.
- **Stable behavioral contracts**: User-visible I/O, CLI flags, and logging semantics when they are part of the product promise—these are *not* “implementation trivia”; they are externally observable behavior you intend to keep stable.

## What does not belong here

- Ephemeral implementation details (specific internal file names, temporary workarounds). Put those in code comments, commits, or design notes.

## Malvin

In the **Malvin** repository, the canonical grounding document is `grounding.md` at the **repository root**. This `default_repo/` copy exists for bootstrapping other projects; do not treat it as Malvin’s source of truth when working in-tree.
