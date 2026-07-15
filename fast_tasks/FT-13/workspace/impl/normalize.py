def normalize_ws(s: str) -> str:
    # BUG: tabs are not treated as whitespace (only spaces).
    out = []
    buf = ""
    for ch in s.strip():
        if ch == " ":
            if buf:
                out.append(buf)
                buf = ""
        else:
            buf += ch
    if buf:
        out.append(buf)
    return " ".join(out)
