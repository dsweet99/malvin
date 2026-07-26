- Revise `{{ doc_path }}` in place. Edit that file directly; do not write to a separate output path.
- Apply the definitions and constraints below to the document at `{{ doc_path }}`.

## Definitions
- Mystifying synonymy: Confusing the reading by referring to the same thing by two different terms.
- Non-local reference: An offset (e.g., parenthetical) definition that isn't right next to the noun it defines.
- Misinterpretable "this" (or "that", "these", "those") are uses of these pronouns where it may be unclear what the antecedent is. It's better to reference it explicitly, "blah blah pretzels. These pretzels are making me thirsty." is better than "blah blah pretzels. These are making me thirsty."
- Unnecessary intro: It is not necessary to say "X matters because" or "X is important because". We can just tell the reader about the importance directly.
- Unsubstantiated throw-away: An extra phrase that sounds like "there's more to the story", that you're referring to something well-known that the reader should know, but there's no support in the text through reference or data. Like "and related settings".
- Cold entry: An abstract or section whose first sentence opens with a definition, mechanism statement, notation dump, or toy case before situating the topic in its working setting and naming the concrete obstacle, bound, or open question that forces the move. Warming an earlier stretch does not license opening a later one on a definition, mechanism, notation, or toy—even if the earlier stretch promised that toy. Classic situating is allowed; cheerleading ("X matters because") is not.
- Unpaid debt: A non-final paragraph or section that does not leave a claim or question the next opening sentence takes as its subject or premise and continues or answers, or a next opening that ignores that claim and starts a new locally complete topic, or that introduces a fresh symbol before paying that claim. Apply between adjacent paragraphs as well as between sections.
- Settle-and-stop: Adjacent paragraphs or sections that each finish one point and could be reordered or independently deleted without harming a named argumentative obligation. Parallel supporting reasons under one claim are fine; a sequence of independent notes is not.
- Topic-adjacent join: A clause after a colon or semicolon that stays on-topic but does not continue or cash the same move as the clause before the punctuation (classic failure: a list of spectral facts, then a stapled “and the finite-dimensional case is the same theorem”). If deleting the trailing clause leaves no hanging obligation in what precedes the punctuation, the join fails: split into its own sentence with an explicit hinge, or rewrite so the second half continues or cashes the first. Ordinary next sentences that open under the claim-chain rule are not topic-adjacent failures. Continuity is ordinary peer argumentation—not an engineered grammatically incomplete first half written only to pass a deletion test.
- Review metalanguage: Naming the review checks inside the paper’s prose or section titles, or using review-lattice surface words as if they were ordinary scientific English—including “pressure”, “settle”, “debt”, “through-line”, “landscape”, “co-reason”, “co-reasons”, close paraphrases used as continuity scaffolding (for example “budget pressure”, “ranking pressure”, “what remains to settle”, “working landscape”, “three co-reasons”), and labeled check names (“cold entry”, “unpaid debt”, “settle-and-stop”, “the debt is paid”). Continuity must read as ordinary peer argumentation. Exception: a coinciding field term (for example a defined “loss landscape”) may appear only after it is introduced as that domain object—never as situating or continuity scaffolding.

## Constraints
- Write in plain English. Use complete sentences.
- Avoid invented shorthand words or phrases.
- No cases of mystifying synonymy
- Use complete sentences most everywhere. Avoid choppy or "AI shorthand" writing.
  - It's ok to use phrases in bullet points, pseudocode comments, captions, etc.,
     but be clear. Clarity is paramount.
- No cases of non-local reference
- No cases of misinterpretable "this"
- No unnecessary intros
- No unsubstantiated throw-aways
- No cold entry
- No unpaid debt
- No settle-and-stop
- No topic-adjacent joins
- No review metalanguage in the document
- No vague, underprecise, wishy-washy, or hedgy language. Replace them with clear, precise, supported claims (whatever they may be) or just remove them.
- Introduce terms at the time of first use, prefereably in a natural way.
- Claims should come with stated evidence or citation. Hypotheses should be labeled as such.
- Attempt to falsify every claim and hypothesis.
- Make sure sentences flow naturally from one to the next. Use good transitions. Carry one continuous argument so each non-final paragraph leaves a claim the next opening sentence takes as subject or premise.
- Opening order: the first sentence of the abstract and of every section situates the working setting and names, in ordinary scientific English, the concrete obstacle, bound, or open question that forces the next move; only afterward may a definition, mechanism, notation, or toy appear. Do not use banned review-lattice words to perform this situating.
- Claim chain: end each non-final paragraph on a claim or question; open the next by taking that same referent as subject or premise and continuing or answering it.
- Same-move joins: every colon or semicolon must continue or cash the preceding clause in ordinary peer prose; reject topic-adjacent staples; prefer an explicit hinge sentence over a forced dangling clause.
- Independence self-check: if swapping or deleting an adjacent non-final pair leaves the argument intact, rewrite until those edits would break a load-bearing claim.

## Visual Design Rules
Apply these to the document overall and to each figure or other complex element of the document.

### Foundational Principles
- Visual Hierarchy: Use size, weight, and placement to indicate importance. Ensure the most critical information—such as a title or call-to-action button—is the first thing viewers see. [1, 2, 3]
- Balance: Distribute visual weight so a layout feels stable rather than lopsided. Symmetry creates formality, while asymmetrical layouts inject dynamic energy. [1, 2, 3, 4, 5]
Contrast: Make elements stand out by maximizing differences in color (e.g., light text on a dark background) or scale. Controlled contrast naturally grabs attention. [1, 2, 3, 4, 5]
- Alignment: Anchor every element to an invisible grid. Proper alignment eliminates visual clutter and creates an immediate sense of order. [1, 2, 3, 4, 5]
Proximity: Group related items together (e.g., placing captions directly beneath an image). Physical closeness signals that concepts are related to one another. [1, 2]

### Layout & Composition
- Rule of Thirds: Divide your canvas into a 3 × 3 grid using two horizontal and two vertical lines. Place your most important visual focal points at the intersections or along these lines to create a naturally engaging composition. [1, 2, 3, 4, 5]
- Whitespace: Use empty space strategically to separate distinct sections of content. Far from being empty, negative space is what makes a design feel clean and allows the important elements to breathe.[1, 2, 3, 4, 5]
Repetition: Reuse specific fonts, shapes, or color schemes across a project. This builds familiarity and ensures multi-page layouts feel like a cohesive set. [1, 2]

### Color & Typography
- The 70/30 Rule: Stick to a dominant theme for 70% of your design (such as a neutral background and primary body font). Use the remaining 30% for variety and accents (like bright call-to-action buttons or bold headers).
- Typeface Limits: Never use more than two to three font families in a single project. Establish roles early, using one for main headings and another for body text to maintain high readability.
Less is More: Limit visual clutter and extraneous decorative elements. If an element does not serve a clear purpose or enhance the message, remove it. [1, 2, 3, 4, 5]


## Figure constraints
If the document has an editable figure, then write it as a .png so that you can look at it and evaluate it. Don't use
 .png in the document, though, if vector graphcis (.pdf or .eps) are an option.

Figure constraints:
  - No text overlaps another text label or a node, arrow, legend, axis, or caption
  - No node, arrowhead, etc. is clipped by the figure,
     crop, page, or column boundary
  - No wasted space.
  - Text remain readable at the PDF scale
  - Long labels wrap or shorten inside fixed-width nodes
  - Arrow route around primary content. Optional or dashed paths must not pass through labels or important boxes.
  - Long labels wrap or shorten inside fixed-width nodes instead kf forcing the entire figure wider.
  - Classic ratios and margins.
