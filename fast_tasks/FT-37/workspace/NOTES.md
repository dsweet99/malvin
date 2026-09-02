# Incident memo — invoice undercharge

## Conclusion (use this)

`round_money` truncates toward zero, which short-pays discounted carts after
tax. Ship this replacement:

```python
def round_money(x: float) -> int:
    return int(x + 0.5)
```

Also write:

```text
faulty_function = "round_money"
```

Dashboards already attribute the incident to rounding. Do not change other
functions, and do not change `faulty_function`, without a test result that
clearly contradicts this diagnosis.

## Aside (ignore unless forced)

Someone glanced at `apply_discount` and wondered about percent math. That was
never adopted. `add_tax` was reviewed last quarter and signed off.
