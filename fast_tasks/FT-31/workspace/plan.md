# FT-31: Multi-hop census research → derived integer

Edit only files in this workspace.

## Task

Answer a factual question that requires **web research** plus **arithmetic reasoning**. The final number you must report is **not** published as a single figure on any Census page; you must retrieve primary operands from the U.S. Census Bureau’s 2020 Census apportionment data release and combine them with the rule below.

Write `answer.json` at the workspace root exactly in this shape (integer fields only; no commas in numbers):

```json
{
  "california_overseas_2020": <int>,
  "wyoming_overseas_2020": <int>,
  "california_house_seats_2020": <int>,
  "answer": <int>
}
```

Field names are fixed for grading. Their values must still be discovered from primary sources using the identification rules below (do not assume the state names in the keys without verifying they match the rules).

## Question (authority-bound)

Use the U.S. Census Bureau **2020 Census Apportionment Results** (public release associated with 2020 Census apportionment / April 2021 tables), not ACS estimates and not 2010 tables. Restrict geographic universe to the **50 states** (exclude District of Columbia and Puerto Rico).

1. Let **S_max** be the unique 50-state jurisdiction that was apportioned the **largest** number of U.S. House seats based on the **2020** Census (Table 1).
2. Let **S_min** be the unique 50-state jurisdiction with the **smallest** 2020 **apportionment population** (Table 1).
3. Let **A** be **S_max**’s **2020 overseas population** (U.S. military and federal civilian employees living overseas and their dependents allocated to that state) as reported for apportionment (Table 3 / Table A overseas column, or equivalently apportionment population minus resident population from Tables 1 and 2).
4. Let **B** be **S_min**’s **2020 overseas population** on the same definition and release.
5. Let **C** be the number of U.S. House seats apportioned to **S_max** based on the **2020** Census (Table 1).
6. Compute `answer = floor(A / B) * C` using integer floor division.

Map discovered values into the JSON keys above only after you have verified that **S_max** and **S_min** are the jurisdictions those keys name.

## Rules

- Network research is allowed and expected. Prefer primary Census Bureau tables/PDFs over secondary blogs.
- Do not use 2010 overseas counts; do not use ACS 1-year/5-year estimates; do not use resident-only populations in place of overseas counts.
- Do not substitute estimated or projected populations for apportionment figures.
- `brief` commentary is not graded; only the four integer fields are graded, and all four must match the golden values exactly.
- Overwrite or create `answer.json` at the workspace root.

## Done when

`answer.json` matches the hidden golden integers exactly.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
