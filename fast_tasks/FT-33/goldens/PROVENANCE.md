# FT-33 golden provenance

Verified 2026-07-20 against primary sources:

| Field | Value | Source |
|---|---|---|
| `coap_documentation_id_low` / `_high` | 64998 / 64999 | RFC 9876 §4.3 / IANA CoAP Content-Formats (Reserved for Documentation) |
| `new_top_level_media_type` | `haptics` | RFC 9695 (March 2025); IANA Top-Level Media Types |
| `nuts_level*_regions` | 92 / 244 / 1165 | Eurostat NUTS 2024 overview (valid 1 Jan 2024); KS-GQ-23-010 |
| `time_ordered_uuid_example` | `017f22e2-79b0-7cc3-98c4-dc0c0c07398f` | RFC 9562 Appendix A.6 (unix_ts_ms = 1645557742000) |
| `answer` | 184 | `(64999 - 64998 + 1) * 92` |

Near-miss traps (must grade 0):

- CoAP Experimental Use 65000–65535 instead of documentation reservation
- NUTS 2021 counts (104 / 283 / 1345) or NUTS 2027 counts (91 / 242 / 1170)
- Typo `haptic` or invented top-levels (`sensor`, `touch`)
- UUID v1 / v6 / ULID examples instead of the A.6 UUIDv7 vector
