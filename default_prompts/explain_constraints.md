- Explain the topic in the user request: `{{ explain_request }}`.

- NB: **The user request takes precedence over the constraints below.**

- Use complete sentences.
- Make it 1–2 pages long.
- Include a tl;dr at the top with 2-4 bullets, each a short phrase containing one complete thought
- After the tl;dr, write a brief summary.
- Write "malvin" as the author, and attach a footnote that reads "https://github.com/dsweet99/malvin".

{{ explain_output_instruction }}


- Write connected exposition, not a fact sheet. Each section should read like a short technical blog post: one main idea per paragraph, with transitions that explain why you’re moving on. Prefer a few linked paragraphs over lists of standalone sentences, semicolon-separated fact dumps, or stacks of parenthetical abbreviations. If you must cut for length, drop facts rather than compressing many into telegraphic fragments.
- Start with something concrete, then grow more abstract.
- Follow one narrative thread per section. Open with what the reader should understand next (the problem, the invariant, or the design choice), then explain mechanism, then tradeoffs or deviations. End paragraphs with a bridge to what follows when the connection isn’t obvious. Avoid cataloging features in arbitrary order.
- Assume the reader has not and will not read the underlying code. The reader is likely not interested in names of variables, functions, etc. but
   rather in how things work algorithmically, mathematically, etc.
- State one thesis up front. Before definitions or mechanism, give the reader the single design problem this answers and the one architectural response. Every section should support that thesis; cut or demote detail that doesn't.
- Outline by argument, not by topic. Sections follow dependency order (problem → architecture → how each part works → limitations), not a catalog of features. Each section title should answer one question; subsections deepen that answer. Consolidate paper comparisons in one place rather than scattering them. Once in a while it's ok to explicitly use a question as a section heading or an introductory sentence, but you should do this rarely. It gets old.
- Introduce terms when they earn their place. Define a concept when the narrative first needs it, not in a front-loaded definition block. The summary states the thesis; the tl;dr previews the argument, not a vocabulary list.

- Speculate internally about the reader's potential state of mind while reading, but do not write about it. Do not tell the reader they will "often confuse this or that" or that something is "obvious". Assume the reader is intelligent and interested in what you have to tell them.
- Use equations where appropriate, where they make it easier to convey a point precisely.
- You may use brief algobox/pseudocode.
- Highlight areas of concern or confusion that the user might need to address, even if only to clarify the intention behind the code. Put the highlights/callouts throughout the document near where they are naturally discussed.
- References to papers and urls are welcome.
  - Ensure that a cited reference supports the claim to which the citation is attached.
- You might want to use tikz (if available) to create a diagram for a figure.
- Get figures, equations, etc to fit inside their hboxes.
- Make figures vector art (.pdf or .eps) not rasterized bitmaps.
- Get rid of all TeX warnings
- Make the document visually pleasing.
- Good title:
  - A good academic or technical title must be informative and concise.
  - 3 Core Rules
   - Be Specific: Keep it to 10–12 words. Cut filler like "A study of."
   - Avoid heavy jargon.
   - Match Guidelines: Follow journal rules. Avoid puns, jokes, and over-promising conclusions.
