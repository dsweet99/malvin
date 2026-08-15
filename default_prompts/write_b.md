[NB: The user request may override any direction below, but these are the defaults.]

Write a short technical LaTeX paper for an intelligent nonspecialist on the topic below. Teach as you would a college freshman.

Put the LaTeX source in `{{ tex_display }}` and a compiled PDF in `{{ pdf_display }}`. Both files must be non-empty.
Do not overwrite unrelated workspace files.

Draw from `notes.tex` in `{{ workspace_dir }}`. Do not add information. Your job now is to transform the notes into good writing, not to do further research or to generate new ideas.

Write in complete sentences throughout—including the abstract, section openings, and captions. Prefer separate sentences or a short list over a colon-led comma chain that packs several ideas into one line.

If the writing is related to coding, assume the reader will not read the underlying source code. Explain the algorithms, mathematics, or design ideas in plain English. Introduce field terms at first use. Use vocabulary natural to the topic’s field. Avoid process jargon, checklist words, and review slang. Consider using algpseudocode for minimal pseudocode.

Name output files with a lowercase snakecase stem derived from the paper’s title. Keep that stem to five words or fewer; shorter is usually better.


# How good papers sound

Aim for prose like these openings and turns.

Shannon (*A Mathematical Theory of Communication*) names the problem, then cuts away what does not matter:

> The fundamental problem of communication is that of reproducing at one point either exactly or approximately a message selected at another point. Frequently the messages have meaning… These semantic aspects of communication are irrelevant to the engineering problem. The significant aspect is that the actual message is one selected from a set of possible messages.

Turing (*Computing Machinery and Intelligence*) replaces a fuzzy question with a sharper one:

> I propose to consider the question, ‘Can machines think?’ … Instead of attempting such a definition I shall replace the question by another, which is closely related to it and is expressed in relatively unambiguous words. … These questions replace our original, ‘Can machines think?’

Watson and Crick (*Molecular Structure of Nucleic Acids*) state the proposal, then say what prior work gets wrong:

> We wish to suggest a structure for the salt of deoxyribose nucleic acid (D.N.A.). … A structure for nucleic acid has already been proposed by Pauling and Corey. … In our opinion, this structure is unsatisfactory for two reasons: (1) … (2) …

Weiser (*The Computer for the 21st Century*) opens with a claim, then contrasts today’s machines:

> The most profound technologies are those that disappear. They weave themselves into the fabric of everyday life until they are indistinguishable from it. … Silicon-based information technology, in contrast, is far from having become part of the environment.

Vaswani et al. (*Attention Is All You Need*) state the proposal and put the measured outcome next to it:

> We propose a new simple network architecture, the Transformer, based solely on attention mechanisms, dispensing with recurrence and convolutions entirely. … Our model achieves 28.4 BLEU on the WMT 2014 English-to-German translation task, improving over the existing best results, including ensembles, by over 2 BLEU.

# How to write this paper

1. **Open with the claim or problem.** Lead with what you will argue or propose. Shannon opens on the fundamental problem of communication. Watson and Crick open on the structure they wish to suggest. A short abstract may name the proposal and its measured outcome, as Attention Is All You Need does. Do not bury the thesis under background. If the abstract needs several steps or sources, put them in a short list of complete sentences or items rather than one long comma-chained sentence.

2. **Make vague words operational.** When a natural question is fuzzy, rewrite it into a form the rest of the paper can test. Turing replaces “Can machines think?” with the imitation game. Shannon sets “meaning” aside for the engineering problem. Define each important term by what a reader could check.

3. **Give a minimal picture before the machinery.** Before equations or implementation detail, give the reader one simple picture that can carry the argument. Shannon’s diagram of source, channel, and destination is one such picture. PageRank’s random surfer is another. Formalism should elaborate that picture; it should not replace it.

4. **Contrast prior art with short, numbered failings—then give yours.** Say what earlier approaches get wrong in short, checkable points. Watson and Crick do this with Pauling’s model. Then introduce your alternative as the fix those failings imply. Prefer numbered or bulleted points written as complete sentences over a single run-on sentence.

5. **Put evidence next to the claim.** Put numbers, comparisons, and scope in the same paragraph as the claim they support. The Transformer abstract places BLEU scores beside the architectural claim. When you idealize, say so, as Turing does for morphogenesis, so the reader knows what would falsify the story.

# Clarity faults to avoid

Do not use two names for the same thing without need. Do not place a definition far from the noun it defines. Do not write “this,” “that,” “these,” or “those” when the referent is unclear; name the referent. Do not cheerlead with “X matters because…” when you could state the problem or claim directly. Do not hint at more with phrases such as “and related settings” unless the text supports them. Do not use vague or hedgy wording unless you label it as a hypothesis. Do not leave a major claim without evidence or citation beside it. Label hypotheses, and try to falsify each. Do not label claims.

# Page and figure craft

Keep the page quiet and readable. Use a clear hierarchy, alignment, and whitespace. Use at most two or three typefaces. Keep a dominant and accent color split near 70/30. Keep content inside the margins. Beware of large blank regions; they are unnecessary, though TeX files (especially with figures) can produce them.

Prefer TikZ or other vector figures. If you need a `.png` draft to inspect layout, still ship vector (`.pdf` or `.eps`) in the document when you can. Avoid overlapping text, clipped arrows, wasted empty regions, and labels too small to read in the PDF. Wrap long labels inside fixed-width nodes. Route arrows around the main content.
