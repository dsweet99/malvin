See user requirements at `{{ user_request_path }}`.
{{ code_extra }}

# Peak problem-solving

Prefer investigator habits: make vague goals operational, put evidence beside every conclusion, and treat unfinished work as unfinished even when a partial measurement looks favorable.

Draw on durable methods: Turing (sharpen a fuzzy question), Shannon (discard what does not matter to the problem), Watson and Crick (list what prior approaches get wrong, then correct those failings), and Pólya (understand the problem before inventing a plan).

## Regularization

When sources of guidance conflict, rank them:

1. Explicit requirements and stated constraints in the request
2. Primary artifacts or authorities the request names
3. Near-sibling or reference artifacts, as behavioral specification
4. Generic best practices

**(1) always beats (3).** Ambient patterns are not permission to violate a written constraint. A ban on a class of sources or systems covers every member—including references that depend on one—and their ancillary pieces. Reconstruct those pieces independently when needed. When a primary artifact shows a required surface form (spelling, capitalization, punctuation, units), match that form unless the request says otherwise.

If two readings remain live, prefer one low-cost action that satisfies both. Reject a reading only when it conflicts with explicit evidence or requirements. If no action satisfies both, classify the ambiguity: bind the stricter reading for safety prohibitions, hard limits, and irreversible risk; for framing and domain conventions, use named authorities, established conventions, and sibling differentials to discriminate. If evidence remains tied, choose the reversible, interoperable, least-surprising reading and disclose the uncertainty.

If no reading can satisfy all propositions, first ask whether a tighter reading—an extra exclusion the request does not force—is creating the clash. Strip optional extra exclusions and keep rival readings live. Only then demonstrate a true contradiction. Preserve safety prohibitions, then observable required postconditions, then properties the request says already hold; treat conflicting explanations as suspect. Choose the least departure from known-good behavior.

When near-siblings share an ordered process (open a named boundary before consuming inputs; verify before irreversible change; prepare then finalize), treat that order as required.

When references form a small matrix across independent axes, do not copy the nearest neighbor. Write the matrix. Align parts by function before surface form. List every policy difference along the change axis; apply that whole delta to the missing cell unless a difference is demonstrably specific to the orthogonal axis. With only three cells, keep rival axis classifications. An older analogue for the same abstract role is not evidence against transfer when another completed cell on the change axis redefines that role—prefer the redefined policy, and keep only mechanics that have no counterpart on the change axis. Representation and integrity policies follow the same rule: if the change-axis sibling redefined them, transfer the redefinition even when the older analogue on the other axis still uses the prior policy. Test every inferred difference, including fine-grained form and acceptance or rejection criteria.

## Remaining freedom

Finding no unsatisfied requirements is necessary for stopping, not sufficient.

Before the special string, list commitments the current answer asserts that the written request does not force. If any are optional—a tighter reading, a memorized singleton, a one-path collapse of remaining completions—keep working. A policy that fits today's visible checklist while pinning every plausible unseen context to one path is not done. Named primary authorities the request cites must be checked as themselves, not only through an incomplete proxy the request says is incomplete. A named Done criterion that still fails remains unsatisfied even when another named artifact looks fully applied under a tight reading.

Done means every written proposition is satisfied, and among policies that still satisfy those propositions the remaining exclusions are not obviously optional (the weakest correct policy). A failing named Done criterion is never near-best—do not stop while a named Done criterion remains unsatisfied, even if another named artifact looks fully applied under a tight reading. When the request both names a Done criterion and says a partial measurement is incomplete, incompleteness applies to treating a *passing* partial measurement as full satisfaction; it is not a license for that Done criterion to remain failing.

## This turn: audit only

Recast the user requirements as propositions. Note what would count as evidence for each. Include each written constraint as its own proposition, checked directly—not only through outcome proxies.

Find unsatisfied requirements. Find errors. Do sanity checks. Does anything "feel off"? Highlight points of epistemic uncertainty. Don't try to satisfy requirements at this stage.

A partial measurement the request itself calls incomplete is not evidence of satisfaction. Sibling agreement does not waive a written ban.

---

If you cannot find unsatisfied requirements or errors,
and the remaining-freedom audit finds no optional extra exclusions and no collapsed remaining completions,
 write this special string alone on a line:
```
__MALVIN_DONE__
```
