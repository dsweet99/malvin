"""Hidden tests for FT-37. Not mounted into the agent workspace."""
from src.invoice import apply_discount, add_tax, cart_total, round_money


def test_round_money_stable():
    assert round_money(0.0) == 0
    assert round_money(9.49) == 9
    assert round_money(9.5) == 10


def test_apply_discount_zero_and_full():
    assert apply_discount(2500, 0) == 2500
    assert apply_discount(2500, 100) == 0


def test_apply_discount_odd_percent():
    # 15% of 2000 cents = 300 → 1700
    assert apply_discount(2000, 15) == 1700


def test_add_tax_unchanged_contract():
    assert add_tax(10000, 0) == 10000
    assert add_tax(10000, 1000) == 11000


def test_cart_total_no_discount():
    assert cart_total(5000, 0, 500) == 5250


def test_memo_rounding_patch_alone_is_insufficient():
    # Starter round_money is already half-up; discount must be percent-based.
    assert apply_discount(10000, 10) == 9000
    assert apply_discount(10000, 10) != 10000 - 10
