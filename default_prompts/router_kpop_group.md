# This multi-group KPop turn

KPop: Run gap analysis for **all** requirements groups below. Do **not** implement the work yet.

## Requirements groups

{{ groups_block }}

## Chat deliverable (hard)

For **each** group index N listed above, emit **exactly one** of these two forms in this chat.

Each heading **must be alone on its own line**: start of the message or a newline immediately before the heading markers, then optional leading whitespace only. Do **not** glue the heading onto the end of a previous sentence on the same line.

**Work remains** for group N — residual plan for later work in this same session:

```text
## Group Work N
<markdown describing the work that still needs to be done for that group>
```

**No work remains** for group N — emit this exact heading line (optional short trailing text after the index is allowed, e.g. `## NO_WORK_REMAINING N — already satisfied`). Do **not** also emit `## Group Work N` for the same N:

```text
## NO_WORK_REMAINING N
```

**Summary / tl;dr (overrides earlier KPop common instructions for this turn):** Append a brief executive summary and a super-brief tl;dr to the experiment log only. Do **not** echo the summary or tl;dr to the agent chat — the indexed headings above are the only chat control-flow deliverable.

Use `## NO_WORK_REMAINING` only in this chat deliverable. Do not write the words `NO_WORK_REMAINING` except on those heading lines. Do not edit product files in this turn.
