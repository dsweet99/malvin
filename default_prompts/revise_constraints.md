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

## Figure constraints
If the document has an editable figure, then
write it as a .png so that you can look at it
and evaluate it.

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
