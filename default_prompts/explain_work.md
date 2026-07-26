# Role: explain work

Execute this plan. Improve the explanation work product until the review failures are addressed.

## Review

{{ review }}

## Plan

{{ plan }}

## Authoring / output

{{ explain_output_instruction }}

- Produce LaTeX and compile a PDF with no TeX warnings. Default length is 1–2 pages unless the request says otherwise. Prefer calm typography and content that stays inside the page margins; prefer vector figures. Prefer TikZ for figures.
- Author from the user request and cited primary sources; do not copy another local draft in as the work product.
- Author: malvin, with a footnote that reads `https://github.com/dsweet99/malvin`. Assume the reader will not read underlying source code; explain the algorithms, mathematics, or design ideas.
- Satisfy the review style lattice in the finished prose (not by naming the lattice):
  - opening order: first sentence of the abstract and of every section situates the working setting and names, in ordinary scientific English, the concrete obstacle, bound, or open question that forces the next move; only then may definition, mechanism, notation, or toy appear—even when a prior stretch promised that toy;
  - claim chain: each non-final paragraph ends on a claim or question; the next opening takes that same referent as subject or premise and continues or answers it in ordinary prose;
  - same-move joins: every colon or semicolon continues or cashes the preceding clause in ordinary peer prose; no topic-adjacent staples; prefer an explicit hinge sentence over manufacturing a dangling first half; ordinary claim-chain next sentences are fine;
  - finish the draft; do not thrash on micro-joins once a careful peer would read the argument as continuous;
  - independence: adjacent stretches must not be reorderable or independently deletable without breaking a load-bearing claim; rewrite until a swap-or-delete self-check would break the argument;
  - no review metalanguage in prose, section titles, captions, or abstracts: do not write “pressure”, “settle”, “debt”, “through-line”, “landscape”, “co-reason”, “co-reasons”, close paraphrases used as continuity scaffolding, or labeled checks (“cold entry”, “unpaid debt”, “settle-and-stop”, “the debt is paid”); a coinciding field term may appear only after it is introduced as that domain object;
  - no mystifying synonymy, non-local reference, misinterpretable pronoun, unnecessary intro, or unsubstantiated throw-away;
  - no vague or hedgy language unless labeled as a hypothesis; introduce terms at first use; claims backed by evidence or citation; attempt to falsify each;
  - figures obey overlap/clip/whitespace/readable-label rules; prefer TikZ; apply hierarchy, balance, contrast, alignment, proximity, whitespace, repetition, rule of thirds, 70/30 dominant/accent, at most two or three typefaces;
  - after situating, claim early; concrete case then general rule when that helps; peer audience; plain English; complete sentences (short phrases ok in bullets, captions, and pseudocode comments).
