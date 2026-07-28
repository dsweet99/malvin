# This multi-group KPop turn

Apply the KPop method to gap analysis for **all** requirements groups below. Do **not** implement the work yet.

Complete up to `{{ want }}` KPOP iterations in this turn. Before beginning each iteration, write a section header `## Step K — KPOP …` to `{{ exp_log }}`; use the next integer K that is not already used in that file.

If you finish gap analysis for all groups before using the iterations budget, append `## KPOP_SOLVED` **only** to `{{ exp_log }}` (with one short paragraph), and stop adding new steps there. Do **not** put `## KPOP_SOLVED` in this chat.

## Requirements groups

{{ groups_block }}

## Chat deliverable (hard)

For **each** group index N listed above, emit **exactly one** of these two forms in this chat:

**Work remains** for group N — residual plan for later work in this same session:

```text
## Group Work N
<markdown describing the work that still needs to be done for that group>
```

**No work remains** for group N — emit this exact heading line (optional short trailing text after the index is allowed, e.g. `## NO_WORK_REMAINING N — already satisfied`). Do **not** also emit `## Group Work N` for the same N:

```text
## NO_WORK_REMAINING N
```

Use `## NO_WORK_REMAINING` only in this chat deliverable, never in `{{ exp_log }}`. Use `## KPOP_SOLVED` only in `{{ exp_log }}`, never in this chat. Do not write the words `NO_WORK_REMAINING` except on those heading lines. Do not edit product files in this turn. Log hypotheses and results to `{{ exp_log }}` as they become available.
