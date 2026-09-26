description: Write a technical report or scholarly paper

Use when drafting a scholarly paper. Prefer structure, clarity, and verifiable claims over human-process ceremony (authorship politics, social promo, Overleaf UI).

## Canonical structure (XYZ+1)

Every major front section answers the same spine:

- **X** — What are we trying to do, and why does it matter?
- **Y** — Why is it hard?
- **Z** — How do we solve it (our contribution)?
- **1** — How do we verify? Experiments/results and/or theory.

**Abstract:** Short XYZ+1.
**Introduction:** Longer XYZ+1; list contributions as bullets; aim for Figure 1 on page 1; optional future-work teaser if space.
**Related Work:** Academic siblings—compare and contrast assumptions/methods. Description alone is not enough. If a method applies to your setting, compare it in experiments; if not, say why.
**Background:** Academic ancestors needed to understand the method; usually includes formalism and unusual assumptions.
**Problem Setting:** Separate section only if the setting itself is a contribution.
**Method:** What you do and why, in the shared formalism.
**Experimental Setup:** Instantiation of the setting plus implementation details that make results reproducible.
**Results and Discussion:** Outcomes, baselines from Related Work, statistics/CIs, fairness of hyperparameters, ablations, limitations.
**Conclusion:** Brief recap plus future work (academic “offspring”).

## Process (agent-friendly)

1. Outline first: one idea per line → one paragraph later. Easier to reshape than full prose.
2. Expand with a LaTeX comment TL;DR above each paragraph (`% summary`), then write the paragraph to match. No fluff.
3. Draft the Abstract early—even before final results—to force a coherent story; revise later.
4. After a full draft, cut about one third of the words.
5. Stay inside the page limit; avoid sparse “lazy” whitespace and orphaned short lines.

## Writing and LaTeX checklist

- Prefer active voice; name who did what.
- Keep contributions sharp: never blur prior work vs yours.
- Consistent tense; avoid needless future tense (“we will show”).
- Cut fillers (“can”, “in order to”, “shall”). Prefer “is” over hedging when the claim is definitional.
- Guide the reader: why this paragraph exists and how it fits.
- Quotes in LaTeX: ``like this'' or `\enquote{...}` (csquotes).
- Equations are part of sentences: punctuation after display math; avoid useless colons before equations.
- `\citet` when authors are grammatical subjects; `~\citep` otherwise.
- `\usepackage[backref=page]{hyperref}`; cleveref for figures/tables.
- Acronym + cite: `name~\citep[ACRO]{key}`. Introduce acronyms before use; only define symbols/acronyms you use.
- Cite claims not backed by your experiments; prefer published versions over default arXiv Scholar hits; fix `??` broken refs in the PDF.
- No copy-paste from other papers except verbatim quotes.
- Consistent EN-US or EN-GB; consistent bold/italic convention; no random Capitalization; no AI anthropomorphism; no subjective adjectives as claims.
- Avoid synonym drift for paper-specific terms; if a term collides with common usage, disambiguate once with an example.

