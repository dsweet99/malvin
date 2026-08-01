# FT-31 golden provenance (grader-only; not in agent workspace)

Primary sources (2020 Census Apportionment Results):

- Table 1: https://www2.census.gov/programs-surveys/decennial/2020/data/apportionment/apportionment-2020-table01.pdf
  - California apportionment population 39,576,757; House seats 52 (largest seat count among 50 states)
  - Wyoming apportionment population 577,719; House seats 1 (smallest apportionment population among 50 states)
- Table 2: https://www2.census.gov/programs-surveys/decennial/2020/data/apportionment/apportionment-2020-table02.pdf
  - California resident population 39,538,223
  - Wyoming resident population 576,851
- Table A: https://www2.census.gov/programs-surveys/decennial/2020/data/apportionment/apportionment-2020-tableA.pdf
  - California overseas 38,534
  - Wyoming overseas 868

Relational identification (agent-facing): S_max = state with most 2020 House seats → California; S_min = state with smallest 2020 apportionment population → Wyoming.

Formula: answer = floor(38534 / 868) * 52 = 44 * 52 = 2288

Designed failure modes (≠ 2288):
- 2010 overseas × 2010 CA seats (53) → 954
- resident populations × 52 → 3536
- 2020 overseas × 53 → 2332
- floor(A/B) only → 44
- TX as “largest” (38 seats) with WY overseas → floor(37785/868)*38 = 43*38 = 1634
- AK as “smallest apportionment” confusion → floor(38534/2690)*52 = 14*52 = 728
