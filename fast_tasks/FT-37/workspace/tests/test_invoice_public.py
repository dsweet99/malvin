from src.invoice import apply_discount, cart_total, round_money


def test_round_money_half_up():
    assert round_money(1.4) == 1
    assert round_money(1.5) == 2
    assert round_money(2.5) == 3


def test_apply_discount_ten_percent():
    assert apply_discount(10000, 10) == 9000


def test_cart_total_with_discount_and_tax():
    # $100.00, 10% off → $90.00; +8.25% tax → $97.43
    assert cart_total(10000, 10, 825) == 9743
