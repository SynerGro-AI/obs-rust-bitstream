//! Reusable differential test helpers.
//!
//! Work smarter: one small harness that can be pointed at any safe API
//! vs its C ABI shim (or a future C oracle).
//!
//! Usage pattern in a module's tests:
//! ```ignore
//! assert_eq!(safe_fn(input), abi_fn(input));
//! ```

/// Compare two results and panic with a clear message on mismatch.
pub fn assert_equal<T: PartialEq + std::fmt::Debug>(safe: T, abi: T, context: &str) {
    assert_eq!(safe, abi, "differential mismatch: {context}");
}

/// Run the same inputs through a safe function and an ABI function.
pub fn differential<I, O, S, A>(inputs: &[I], safe_fn: S, abi_fn: A, context: &str)
where
    S: Fn(&I) -> O,
    A: Fn(&I) -> O,
    O: PartialEq + std::fmt::Debug,
{
    for (i, input) in inputs.iter().enumerate() {
        let s = safe_fn(input);
        let a = abi_fn(input);
        assert_eq!(s, a, "differential mismatch on input #{i} ({context})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_differential() {
        let inputs = [1, 2, 3];
        differential(
            &inputs,
            |x| x * 2,
            |x| x + x,
            "double",
        );
    }
}
