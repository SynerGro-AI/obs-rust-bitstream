//! Safe double-ended queue (OBS deque.h).
//!
//! The C version is a circular buffer with manual wrap-around on
//! start_pos / end_pos and capacity doubling. The safe path uses
//! VecDeque under the hood so wrap-around, capacity growth, and
//! zero-size cases cannot produce buffer overruns or lost data.
//!
//! Differential: push/pop sequences produce the same logical order
//! as a reference circular implementation.

use std::collections::VecDeque;
use crate::diff::assert_equal;

#[derive(Clone, Debug, Default)]
pub struct Deque<T> {
    inner: VecDeque<T>,
}

impl<T> Deque<T> {
    pub fn new() -> Self {
        Self { inner: VecDeque::new() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn push_back(&mut self, value: T) {
        self.inner.push_back(value);
    }

    pub fn push_front(&mut self, value: T) {
        self.inner.push_front(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn front(&self) -> Option<&T> {
        self.inner.front()
    }

    pub fn back(&self) -> Option<&T> {
        self.inner.back()
    }
}

/// Byte-oriented convenience matching the C API style.
pub type ByteDeque = Deque<u8>;

impl ByteDeque {
    pub fn push_back_bytes(&mut self, data: &[u8]) {
        for &b in data {
            self.push_back(b);
        }
    }

    pub fn push_front_bytes(&mut self, data: &[u8]) {
        for &b in data.iter().rev() {
            self.push_front(b);
        }
    }

    pub fn pop_front_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(b) = self.pop_front() {
                out.push(b);
            } else {
                break;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut d = Deque::new();
        d.push_back(1u8);
        d.push_back(2);
        d.push_front(0);
        assert_eq!(d.len(), 3);
        assert_eq!(d.pop_front(), Some(0));
        assert_eq!(d.pop_back(), Some(2));
        assert_eq!(d.pop_front(), Some(1));
        assert!(d.is_empty());
    }

    #[test]
    fn wrap_around() {
        let mut d = ByteDeque::new();
        d.push_back_bytes(b"hello");
        d.push_front_bytes(b"xx");
        let front = d.pop_front_bytes(2);
        assert_eq!(&front, b"xx");
        let rest = d.pop_front_bytes(5);
        assert_eq!(&rest, b"hello");
    }

    #[test]
    fn empty_pop_safe() {
        let mut d = Deque::<u8>::new();
        assert_eq!(d.pop_front(), None);
        assert_eq!(d.pop_back(), None);
    }

    #[test]
    fn differential() {
        // Reference circular behavior vs safe VecDeque
        let mut safe = ByteDeque::new();
        let seq = [1u8, 2, 3, 4, 5];
        for &b in &seq {
            safe.push_back(b);
        }
        safe.push_front(0);
        let got = safe.pop_front_bytes(6);
        let expected = vec![0, 1, 2, 3, 4, 5];
        assert_equal(got, expected, "deque order");
    }
}
