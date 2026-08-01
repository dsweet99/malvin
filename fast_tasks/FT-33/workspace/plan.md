# FT-33: Underspecified registry priors → answer.json

Edit only files in this workspace. Network research is allowed and expected.

## Situation

You must fill a shared fixture used by three unrelated tools. The fixture author left every field blank and wrote only domain descriptions. Each blank has **exactly one** correct fill-in from a current public primary source (a standards document or official statistics overview). Guessing a nearby or historically adjacent value fails.

Do **not** expect the plan to name RFC numbers, registry row labels, or the literal answers. Recover those details from the public web using the identification rules below. Prefer primary sources (RFC Editor / IANA registries / Eurostat) over blogs.

Write `answer.json` at the workspace root exactly in this shape:

```json
{
  "coap_documentation_id_low": <int>,
  "coap_documentation_id_high": <int>,
  "new_top_level_media_type": <string>,
  "nuts_level1_regions": <int>,
  "nuts_level2_regions": <int>,
  "nuts_level3_regions": <int>,
  "time_ordered_uuid_example": <string>,
  "answer": <int>
}
```

Field names are fixed for grading. Integer fields must be JSON integers (no quotes, no commas). The media-type and UUID fields are lowercase strings.

## Identification rules (authority-bound)

1. **CoAP documentation Content-Format IDs.** In the IANA CoAP Content-Formats registry, after the late-2025 update to registration procedures, a contiguous pair of identifiers is reserved specifically for use in documentation and examples. Put the smaller identifier in `coap_documentation_id_low` and the larger in `coap_documentation_id_high`. Do **not** use identifiers from the Experimental Use range.

2. **New top-level media type.** In 2025, IANA registered a new top-level Internet media type for content that drives tactile actuators / force-feedback rendering hardware (payloads that need that class of subsystem). Put that top-level type token alone (no subtype, no parameters) in `new_top_level_media_type`.

3. **NUTS region counts.** Using the NUTS classification that became valid for EU statistical data transmission on **1 January 2024** (not the 2021 edition and not the 2027 edition), report how many regions exist at NUTS levels 1, 2, and 3 in `nuts_level1_regions`, `nuts_level2_regions`, and `nuts_level3_regions`.

4. **Time-ordered UUID example.** Using the IETF UUID standard published in 2024 that supersedes RFC 4122, take the published appendix example for the Unix-millisecond time-ordered UUID version whose timestamp field is exactly `1645557742000` (the well-known demo instant 2022-02-22 14:22:22 −05:00). Put the canonical 8-4-4-4-12 lowercase hex string in `time_ordered_uuid_example`. Do not substitute a UUID version 1, version 6, or ULID example.

5. **Derived answer.** Compute
   `answer = (coap_documentation_id_high - coap_documentation_id_low + 1) * nuts_level1_regions`
   with integer arithmetic.

## Rules

- Overwrite or create `answer.json` at the workspace root.
- Do not special-case grader heuristics; recover the real registry / table values.
- Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).

## Done when

`answer.json` matches the hidden golden values exactly (all eight fields), including the derived `answer`.
