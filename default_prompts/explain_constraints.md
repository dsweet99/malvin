- Explain the topic in the user request: `{{ explain_request }}`.

- NB: **The user request takes precedence over the constraints below.**

{{ explain_locate_instruction }}

- Write a short technical paper that a careful peer could publish for an intelligent nonspecialist. Treat the reader as a peer—never condescend or narrate their confusion. Prefer plain English; introduce field terms at first use. Back claims with evidence or citation; label hypotheses. Prefer calm typography and content that stays inside the page margins; prefer vector figures (TikZ when drawing). Assume the reader will not read underlying source code; explain the algorithms, mathematics, or design ideas. Use math or figures only when they tighten the claim.

## Definitions (author/review checks — not paper vocabulary)
- Mystifying synonymy: referring to the same thing by two different terms without need.
- Non-local reference: an offset definition that is not next to the noun it defines.
- Misinterpretable pronoun: “this/that/these/those” with an unclear antecedent; name the referent explicitly.
- Unnecessary intro: “X matters because…” cheerleading instead of stating the forcing problem directly.
- Unsubstantiated throw-away: a phrase that hints at more (“and related settings”) without support in the text.
- Cold entry: the first sentence of the abstract or of any section opens on a definition, mechanism, notation dump, or toy case before situating the topic in its working setting and naming the concrete obstacle, bound, or open question that forces the next move. A warm earlier stretch does not license a cold later opening—even if it promised that toy. Classic situating is allowed; cheerleading is not.
- Unpaid debt: a non-final paragraph or section that does not leave a claim or question the next opening sentence takes as its subject or premise and continues or answers, or a next opening that ignores that claim and starts a new locally complete topic, or that introduces a fresh symbol before paying that claim. Apply between adjacent paragraphs as well as between sections.
- Settle-and-stop: adjacent paragraphs or sections that each finish one point and could be reordered or independently deleted without harming a named argumentative obligation. Parallel supporting reasons under one claim are fine; a sequence of independent notes is not.
- Topic-adjacent join: a clause after a colon or semicolon that stays on-topic but does not continue or cash the same move as the clause before the punctuation (classic failure: a list of spectral facts, then a stapled “and the finite-dimensional case is the same theorem”). If deleting the trailing clause leaves no hanging obligation in what precedes the punctuation, the join fails: split into its own sentence with an explicit hinge, or rewrite so the second half continues or cashes the first. Ordinary next sentences that open under the claim-chain rule are not topic-adjacent failures. Continuity is ordinary peer argumentation—not an engineered grammatically incomplete first half written only to pass a deletion test.
- Review metalanguage: naming the review checks inside the paper’s prose or section titles, or using review-lattice surface words as if they were ordinary scientific English—including “pressure”, “settle”, “debt”, “through-line”, “landscape”, “co-reason”, “co-reasons”, close paraphrases used as continuity scaffolding (for example “budget pressure”, “ranking pressure”, “what remains to settle”, “working landscape”, “three co-reasons”), and labeled check names (“cold entry”, “unpaid debt”, “settle-and-stop”, “the debt is paid”). Continuity must read as ordinary peer argumentation. Exception: a coinciding field term (for example a defined “loss landscape”) may appear only after it is introduced as that domain object—never as situating or continuity scaffolding.

## Authoring moves
- Opening order: the first sentence of the abstract and of every section situates the working setting and names, in ordinary scientific English, the concrete obstacle, bound, or open question that forces the next move; only after that sentence may a definition, mechanism, notation, or toy appear. “X matters because…” is not situating. Do not use banned review-lattice words to perform this situating.
- Claim chain: end each non-final paragraph on a claim or question; open the next paragraph by taking that same claim or question as its subject or premise (same referent, reused or paraphrased) and continuing or answering it in ordinary prose. Parallel supporting reasons under one claim are allowed; never label them “co-reasons” in the paper.
- Same-move joins: every colon or semicolon must continue or cash the preceding clause in ordinary peer prose; reject topic-adjacent staples; prefer an explicit hinge sentence over a forced dangling clause.
- Independence self-check: if swapping or deleting an adjacent non-final pair leaves the argument intact, rewrite until those edits would break a load-bearing claim.

## Pass/fail lattice
Pass only when all of the following hold (fail the review if any fails):
1. **No cold entry** (as defined above); opening order must situate setting and forcing problem before definition, mechanism, notation, or toy.
2. **No unpaid debt** between adjacent paragraphs and between sections; the claim-chain move must hold.
3. **No settle-and-stop** (as defined above); the independence self-check must fail to find reorderable or independently deletable adjacent stretches.
4. **No topic-adjacent joins** (as defined above); same-move continuity holds across colon/semicolon joins.
5. **No review metalanguage** in prose, section titles, captions, or abstracts (as defined above)—including banned surface words used as continuity scaffolding.
6. **No mystifying synonymy, non-local reference, misinterpretable pronoun, unnecessary intro, or unsubstantiated throw-away.**
7. **No vague, underprecise, wishy-washy, or hedgy language** unless labeled as a hypothesis.
8. **Claims carry stated evidence or citation**; hypotheses are labeled; attempt to falsify each.
9. **Typography and figures:** apply visual hierarchy, balance, contrast, alignment, proximity, whitespace, repetition, rule of thirds for focal placement, a 70/30 dominant/accent split, at most two or three typefaces, and less-is-more clutter control; content inside margins; classic ratios. Prefer TikZ/vector. If a figure needs visual self-check, draft a `.png` for inspection, but ship vector (`.pdf`/`.eps`) in the document when available. Figures: no text overlap, no clipped nodes or arrows, no wasted space, readable labels at PDF scale, long labels wrap inside fixed-width nodes, arrows route around primary content (optional/dashed paths must not cross labels).

- After situating, claim early; build from a concrete case to the general rule when that tightens the claim. Carry one continuous argument so each non-final paragraph ends on a claim the next opening continues in ordinary prose. Write in complete sentences most everywhere; short phrases are fine in bullets, captions, and pseudocode comments when clarity remains. Avoid invented shorthand; introduce terms at first use; use good transitions; attempt to falsify every claim and hypothesis.
