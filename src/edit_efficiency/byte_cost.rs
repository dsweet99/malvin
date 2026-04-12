//! Byte-level edit cost from diff opcodes (aligned with `plan.md` Option A: opcode sums, not raw LCS distance).
//!
//! We use Myers diff on byte slices ([`similar::capture_diff_slices`]) and sum insert/delete/replace
//! weights like Python `difflib` opcode *style* accounting (equal → 0; replace → old + new length).
//! Python’s [`difflib.SequenceMatcher`](https://docs.python.org/3/library/difflib.html) uses a different
//! underlying algorithm; byte totals may **differ** from Myers on the same inputs—expect approximate
//! alignment for typical edits, not bit-identical parity with `CPython`.

use similar::{Algorithm, DiffOp, capture_diff_slices};

/// Sum of inserted bytes + deleted bytes between `old` and `new` from Myers diff opcodes.
#[must_use]
pub fn byte_edit_cost(old: &[u8], new: &[u8]) -> u64 {
    let ops = capture_diff_slices(Algorithm::Myers, old, new);
    ops.iter()
        .map(|op| match *op {
            DiffOp::Equal { .. } => 0,
            DiffOp::Delete { old_len, .. } => old_len as u64,
            DiffOp::Insert { new_len, .. } => new_len as u64,
            DiffOp::Replace {
                old_len,
                new_len,
                ..
            } => old_len as u64 + new_len as u64,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::byte_edit_cost;

    #[test]
    fn kiss_stringify_byte_cost() {
        let _ = stringify!(byte_edit_cost);
    }

    #[test]
    fn identical_is_zero() {
        assert_eq!(byte_edit_cost(b"abc", b"abc"), 0);
    }

    #[test]
    fn replace_counts_both_sides() {
        let c = byte_edit_cost(b"aaa", b"bbb");
        assert_eq!(c, 6);
    }
}
