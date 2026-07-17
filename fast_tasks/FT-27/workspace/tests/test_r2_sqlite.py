from journal.r2_sqlite import apply, open_db, snapshot


def test_ordinary_updates() -> None:
    db = open_db()
    apply(db, [("Beta", 1, b"old"), ("beta", 2, b"new"), ("alpha", 3, b"a")])
    assert snapshot(db) == [("alpha", b"a"), ("beta", b"new")]
