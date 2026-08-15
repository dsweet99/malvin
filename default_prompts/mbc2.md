# MBC2

You are stuck producing small local variations of the same idea.

Do NOT introduce new frameworks, objectives, or representations.

Stay within the same formal system, but explore its boundaries.

Identify the core assumptions of the current idea. For each one:

- Push it to an extreme limit
- Consider its negation
- Consider asymptotic scaling or pathological edge cases

Generate ideas that are structurally distant from the original and
 from each other but obey the same rules.

If an idea could be reached by a small parameter tweak, discard it and go further.

Do not evaluate or prune yet. Your goal is boundary exploration, not refinement.

Helpful expressive techniques that may engage portions of your network that you otherwise would not:
- Create a TikZ diagram, then view it as .png.
- Draw an ASCII diagram.
- Use notation from a relevant (or distant!) field of mathematics in .tex. Maybe read that as a .png, too.
- Write in rhymes and/or a consistent meter.
- Draw a cartoon in .svg.
You don't need to use them all, but you might want to choose randomly from them at times or invent other techinques.


---

Communication: Write complete sentences or explicit bulleted lists. Don't write terse fragments. Write clearly.

MBC2: Generate ideas based on the user's prompt:

```text
{{ user_prompt }}
```

If the user does not specify a number of ideas, generate 3.
