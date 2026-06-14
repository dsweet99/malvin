c- Revise `{{ doc_path }}` in place. Edit that file directly; do not write to a separate output path.
- Apply the definitions and constraints below to the document at `{{ doc_path }}`.

## Definitions
- Mystifying synonymy: Confusing the reading by referring to the same thing by two different terms.
- Non-local reference: An offset (e.g., parenthetical) definition that isn't right next to the noun it defines.
- Misinterpretable "this" (or "that", "these", "those") are uses of these pronouns where it may be unclear what the antecedent is. It's better to reference it explicitly, "blah blah pretzels. These pretzels are making me thirsty." is better than "blah blah pretzels. These are making me thirsty."
- Unnecessary intro: It is not necessary to say "X matters because" or "X is important because". We can just tell the reader about the importance directly.
- Unsubstantiated throw-away: An extra phrase that sounds like "there's more to the story", that you're referring to something well-known that the reader should know, but there's no support in the text through reference or data. Like "and related settings".

## Constraints
- No cases of mystifying synonymy
- Use complete sentences most everywhere. Avoid choppy or "AI shorthand" writing.
  - It's ok to use phrases in bullet points, pseudocode comments, captions, etc.,
     but be clear. Clarity is paramount.
- No cases of non-local reference
- No cases of misinterpretable "this"
- No unnecessary intros
- No unsubstantiated throw-aways
- No vague, underprecise, wishy-washy, or hedgy language. Replace them with clear, precise, supported claims (whatever they may be) or just remove them.
- Don't use or discuss terms before introducing them.
- Claims should come with stated evidence or citation. Hypotheses should be labeled as such.
- Attempt to falsify every claim and hypothesis.
- Make sure sentences flow naturally from one to the next. Use good transitions.

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
