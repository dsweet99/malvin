# KPOP experiment log: manual `len-3` stem vs `prompt_md_stem`

## Problem (restated)

`run_reviewer_pair_for_attempt` builds log filenames with:

```rust
&ctx.review_prompt[..ctx.review_prompt.len().saturating_sub(3)]
```

Elsewhere, log stems use `prompt_md_stem`, which is `strip_suffix(".md")`. If those two ever disagree, reviewer log basenames are inconsistent with coder logs and can be wrong for edge-case filenames.

---

## Hypothesis (H1)

**H1:** For every string that might be used as a review prompt filename, the legacy `[..len().saturating_sub(3)]` slice is identical to `prompt_md_stem`.

If **H1** is true, there is no behavioral gap between the two approaches (for those inputs).

---

## Predict / falsifying test

If **H1** is false, there exists at least one string `s` such that:

```text
&s[..s.len().saturating_sub(3)] != prompt_md_stem(s)
```

**Concrete prediction:** For `s = "readme.markdown"`, `prompt_md_stem` leaves the full string (no trailing `.md`), while the slice drops the last three bytes (`"own"`), producing a different stem.

---

## Falsify (command + outcome)

```text
cargo test legacy_slice_stem_diverges_from_prompt_md_stem -- --nocapture
```

**Result (2026-04-10):** Exit code **0**. The unit test `legacy_slice_stem_diverges_from_prompt_md_stem` **passed**, demonstrating:

- `"review_1.md"` — both stems match (`review_1`).
- `"readme.markdown"` — stems differ (`readme.marke` vs `readme.markdown`).
- `"review_1.MD"` — stems differ (`review_1` vs full string; `strip_suffix` is case-sensitive).

**Conclusion:** **H1 is rejected.** The legacy slice is not equivalent to `prompt_md_stem` for all plausible prompt names; unifying on `prompt_md_stem` is the correct fix for consistent, suffix-aware behavior.

---

## Follow-up hypothesis (H2)

**H2:** After replacing the slice with `prompt_md_stem(ctx.review_prompt)`, `cargo test` still passes.

**Predict:** Full suite green.

**Falsify:** `cargo test` (recorded after code change).

**Result (2026-04-10):** Exit code **0**. **113** lib tests + **1** main + **3** `cli_parity` + **1** `review_ops_order` — all passed.

**Conclusion:** **H2 is not rejected** (fix is consistent with the rest of the tree).

---

## Code change

`src/orchestrator/mod.rs`: `let stem = prompt_md_stem(ctx.review_prompt);` (replaces the `[..len-3]` slice).
