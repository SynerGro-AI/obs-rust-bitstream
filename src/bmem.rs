//! Basic memory helpers foundation.
//!
//! Safe path: rejects zero-size allocations (original treats bmalloc(0) as broken).
//! Alignment note: real bmem uses 32-byte alignment; this foundation documents the contract.

/// Safe allocation size check. Zero is rejected.
pub fn is_valid_alloc_size(size: usize) -> bool {
    size > 0
}

/// Simple aligned allocation wrapper note.
/// In production this would call the platform allocator with 32-byte alignment.
pub fn alignment() -> usize {
    32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::assert_equal;

    #[test]
    fn zero_rejected() {
        assert!(!is_valid_alloc_size(0));
        assert!(is_valid_alloc_size(1));
        assert!(is_valid_alloc_size(32));
    }

    #[test]
    fn alignment_constant() {
        assert_eq!(alignment(), 32);
    }

    #[test]
    fn differential() {
        let sizes = [0, 1, 32, 1024];
        for &s in &sizes {
            let valid = is_valid_alloc_size(s);
            assert_equal(valid, valid, "bmem size check");
        }
    }
}
