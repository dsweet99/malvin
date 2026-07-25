# FT-35 golden provenance (grader-only; not in agent workspace)

CODATA 2022 (NIST wallet / CODATA 2022 adjustment):

```text
α⁻¹ = 137.035 999 177(21)
alpha_inv_times_1e12 = 137035999177

c R_∞ = 3.289 841 960 2500(36) × 10^15 Hz
rydberg_frequency_hz = 3289841960250000
```

Near-misses:

- CODATA 2018 α⁻¹ → `137035999084`
- Scale error ×1000 → `137035999177000` (observed malvin failure mode)
- CODATA 2018 Rydberg → `3289841960355000`
- Rydberg constant in m⁻¹ instead of frequency

Designed failure modes:

- Edition mix-up (2018 vs 2022) — public magnitude windows still green
- Off-by-three-orders scaling on α⁻¹ encoding — public windows still green
- Wrong Rydberg integer while α is correct
