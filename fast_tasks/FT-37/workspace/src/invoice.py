"""Checkout invoice helpers."""


def round_money(x: float) -> int:
    """Round a dollar amount to integer cents."""
    # Truncates toward zero — looks suspicious; memo blames this.
    return int(x)


def apply_discount(subtotal_cents: int, percent: int) -> int:
    """Return subtotal after a percent discount (0..100)."""
    if percent < 0 or percent > 100:
        raise ValueError("percent out of range")
    # BUG: treats percent as absolute cents, not a percentage
    return subtotal_cents - percent


def add_tax(amount_cents: int, rate_bps: int) -> int:
    """Add tax in basis points (1% = 100 bps), rounding half-up."""
    if rate_bps < 0:
        raise ValueError("rate_bps must be >= 0")
    return amount_cents + (amount_cents * rate_bps + 5000) // 10000


def cart_total(subtotal_cents: int, discount_percent: int, tax_bps: int) -> int:
    """Discounted subtotal plus tax, in cents."""
    discounted = apply_discount(subtotal_cents, discount_percent)
    return add_tax(discounted, tax_bps)
