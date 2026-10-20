//! Safe dynamic array (OBS darray.h).
//!
//! The C version is an untyped growable buffer with manual capacity
//! doubling and memmove on insert/erase. The safe path is a thin
//! wrapper over Vec so growth, zero-size, out-of-bounds, and empty
//! cases cannot overrun or leak.
//!
//! Differential: push / insert / erase sequences produce the same
//! logical content as a reference Vec.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Debug, Default)]
pub struct DArray<T> {
    inner: Vec<T>,
}

impl<T> DArray<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn push(&mut self, item: T) {
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn insert(&mut self, idx: usize, item: T) {
        if idx > self.inner.len() {
            // Safe: clamp or ignore; C would assert. We clamp to end.
            self.inner.push(item);
        } else {
            self.inner.insert(idx, item);
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx < self.inner.len() {
            Some(self.inner.remove(idx))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.inner.get(idx)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }
}

impl<T: Clone> DArray<T> {
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.inner.resize(new_len, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut da = DArray::new();
        da.push(1u8);
        da.push(2);
        assert_eq!(da.len(), 2);
        assert_eq!(da.pop(), Some(2));
        assert_eq!(da.pop(), Some(1));
        assert!(da.is_empty());
    }

    #[test]
    fn insert_remove() {
        let mut da = DArray::new();
        da.push(1);
        da.push(3);
        da.insert(1, 2);
        assert_eq!(da.as_slice(), &[1, 2, 3]);
        assert_eq!(da.remove(1), Some(2));
        assert_eq!(da.as_slice(), &[1, 3]);
    }

    #[test]
    fn out_of_bounds_safe() {
        let mut da = DArray::<u8>::new();
        assert_eq!(da.remove(0), None);
        da.insert(5, 42); // clamps to end
        assert_eq!(da.as_slice(), &[42]);
    }

    #[test]
    fn differential() {
        let mut safe = DArray::new();
        safe.push(10);
        safe.push(20);
        safe.insert(1, 15);
        let got = safe.as_slice().to_vec();
        let expected = vec![10, 15, 20];
        assert_equal(got, expected, "darray content");
    }
}
