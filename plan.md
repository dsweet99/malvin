Goal
- Define edit efficiency for an agent run without commits.
- Use git for fast snapshotting and rename-aware diffing.
- Do not touch the user’s real index.
- Count only selected file types, e.g. .rs, .py, .md.

Shipped behavior (malvin)
- **Checkpoints** in the implementation run at **orchestration boundaries** (after each coder / reviewer–kpop prompt step in `malvin code`, and at analogous boundaries in `malvin kpop`), not after every in-agent tool call. Finer “every write step” sampling would need instrumentation inside the agent/ACP layer.
- **Per-file byte cost** follows the Option A *shape* below (opcode-weighted sums) but uses Myers-style opcodes via the `similar` crate, not Python `difflib.SequenceMatcher` as the engine—see `src/edit_efficiency/byte_cost.rs` and `.llm_style/malvin_tooling.md` § Edit efficiency.

Definitions
- Let S0, S1, ..., ST be repo snapshots taken during the run.
- Snapshot granularity: after each agent write step, tool step, or checkpoint.
- Let gross = sum of edit cost between consecutive snapshots.
- Let net   = edit cost between initial snapshot S0 and final snapshot ST.
- Let efficiency = net / gross, with efficiency = 1 if gross = 0.

Key implementation idea
- Use a temporary git index via GIT_INDEX_FILE.
- At each checkpoint:
  1. populate the temp index from the current working tree with `git add -A`
  2. write the temp index to a tree object with `git write-tree`
  3. diff old_tree vs new_tree with rename detection
  4. compute edit cost only for included file extensions
  5. set old_tree = new_tree

Why temp index
- Avoids corrupting or changing the user’s real index/staging area.
- Safe even if the repo contains untracked junk files not in .gitignore.
- Junk can be snapshotted into the temp index; that is fine.
- We simply ignore paths whose extensions are not in the measured set.

File inclusion rule
- Count only paths ending in one of:
  - .rs
  - .py
  - .md
- This filter is applied when summing diff cost, not necessarily when snapshotting.

Snapshot procedure
- Create temp index file:
    export GIT_INDEX_FILE=/tmp/agent-metric.index
- For each checkpoint:
    rm -f "$GIT_INDEX_FILE"         # optional at first checkpoint; otherwise reuse is fine if you restage fully
    git add -A
    tree=$(git write-tree)
- Keep only:
    baseline_tree
- After each measurement:
    baseline_tree = tree
- No commits are created.
- Tree objects are immutable; you do not overwrite a tree.
- You only overwrite the variable holding the current baseline tree id.

Diff procedure
- Use rename-aware diff:
    git diff --name-status -M <old_tree> <new_tree>
- This yields records like:
  - M path
  - A path
  - D path
  - R100 old_path new_path
  - R87  old_path new_path

Rename handling
- Pure rename should cost 0.
- Rename + content edits should cost only the content edits.
- Therefore:
  - M p           -> compare old blob at p vs new blob at p
  - A p           -> compare empty vs new blob at p
  - D p           -> compare old blob at p vs empty
  - R old new     -> compare old blob at old vs new blob at new
- Do not count rename as delete+add.
- Apply the same rename-aware logic both:
  - for gross (between consecutive snapshots)
  - for net (between initial and final snapshots)

Blob extraction
- To get file contents from a tree:
    git show <tree>:path/to/file
- If the file is absent in one tree, treat its contents as empty bytes.

Per-file edit cost
- Use byte-level diff cost on file contents:
    cost(old_bytes, new_bytes) = inserted_bytes + deleted_bytes
- This is more diff-like than Levenshtein-with-substitution.
- It naturally penalizes rewriting more than tiny edits.
- A pure rename with identical contents has cost 0.

How to compute byte diff cost
Option A: practical
- Use a byte diff algorithm (e.g. Python difflib.SequenceMatcher on bytes).
- Sum over opcodes:
  - equal   -> 0
  - insert  -> number of inserted bytes
  - delete  -> number of deleted bytes
  - replace -> deleted bytes + inserted bytes

Option B: more formal
- Define cost as minimum insert/delete edit distance on bytes.
- This is stricter mathematically but a bit heavier to implement.

Recommended first implementation
- Use SequenceMatcher on bytes.
- It is simple and usually good enough for a metric.

Formulas
- Let C(Sa, Sb) be the total filtered edit cost between snapshots Sa and Sb.
- Then:
    gross = sum_{t=1..T} C(S_{t-1}, S_t)
    net   = C(S0, ST)
    efficiency = 1                  if gross = 0
                 net / gross        otherwise

Interpretation
- efficiency = 1
  -> every counted edit contributed directly to the final result
- efficiency near 0
  -> lots of rework / change-and-revert / thrash
- If a byte changes and later changes back:
  - gross increases on both edits
  - net returns toward 0

Suggested algorithm
1. Set up temp index.
2. Take initial snapshot tree S0.
3. For each agent checkpoint:
   a. snapshot current state to new_tree
   b. diff baseline_tree vs new_tree with `git diff --name-status -M`
   c. for each changed record:
      - if relevant extension, extract blobs
      - compute byte diff cost
      - add to gross
   d. baseline_tree = new_tree
4. At run end:
   a. compute net between S0 and final tree, using same rename-aware logic
5. Report:
   - gross_bytes
   - net_bytes
   - efficiency = net_bytes / gross_bytes

Caveats
- Tree objects are immutable and accumulate temporarily, but that is usually fine.
- Because they are unreachable from commits/refs, git can garbage-collect them later.
- `git diff -M` rename detection is heuristic; default is probably fine for a first pass.
- If desired, tune rename sensitivity with e.g. `-M20%`.

Recommended outputs
- gross_bytes
- net_bytes
- efficiency
- optionally:
  - number of checkpoints
  - per-extension breakdown
  - per-file breakdown
  - rename count