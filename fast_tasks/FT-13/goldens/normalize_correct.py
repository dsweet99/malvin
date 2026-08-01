import re

def normalize_ws(s: str) -> str:
    return re.sub(r"\s+", " ", s.strip())
