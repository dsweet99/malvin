# FT-23: offline protocol under a dense constraint lattice

Edit only files in this workspace.

## Task

Produce a valid offline A/B measurement protocol as **`protocol.json`**. The JSON must satisfy Draft-07 schema `schema/protocol.schema.json` **and** every extra rule below. Schema validity alone is not enough. The grader compares **file bytes**, not only parsed equality.

### Exact field rules (conjunction; all required)

1. `experiment_mode` must be exactly the string `offline_ab`.
2. `max_runtime_minutes` must be the integer `12` (exact equality, not merely ≤ some cap).
3. `max_memory_mb` must be the integer `256` (exact).
4. `n_seeds` must be the integer `5` (exact).
5. `primary_metric` must be exactly `f1` (not `auroc`).
6. `steps` must be an array of **exactly four** objects, in this **exact id order**: `load`, then `mask`, then `fit`, then `report`.
7. Each step object has only `id` and `budget_ms` (integers). The four `budget_ms` values must sum to **exactly** `1000`.
8. `meta.author` must be exactly `offline`.
9. `meta.schema_version` must be exactly `2026.07.15`.
10. No forbidden keys anywhere in the tree: `download_url`, `api_key`, `endpoint`.
11. No string value anywhere in the tree may contain the substring `http` (case-sensitive).
12. Top-level keys must be exactly the schema-required set (no extras). Nested objects likewise must not carry extra properties.

### Fingerprint rule

`fingerprint` must be the lowercase hex SHA-256 digest of the UTF-8 bytes of:

```text
json.dumps(payload, sort_keys=True, separators=(",", ":"))
```

where `payload` is the full protocol object **with the `fingerprint` key removed**. Recompute after you finish editing other fields.

### Exact file encoding (byte identity)

Write `protocol.json` as **one line** of UTF-8 JSON produced by:

```text
json.dumps(full_object, sort_keys=True, separators=(",", ":")) + "\n"
```

That means: sorted keys at every object level, no spaces after `:` or `,`, no pretty-print indentation, and exactly one trailing newline. A semantically correct object written with `indent=2` (or any other spacing) **fails**.

### Constraints

- Stdlib only; no network; do not rename the deliverable.
- Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).

## Done when

`protocol.json` bytes match the canonical encoding of an object that satisfies every rule above.
