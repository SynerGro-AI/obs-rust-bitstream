//! Safe pure buffered file serializer (OBS buffered-file-serializer.h core).
//!
//! The C version buffers writes before flushing to a FILE*.
//! The safe path accumulates into a Vec and flushes in chunks
//! into an underlying memory buffer. No raw FILE*, no unwraps.
//!
//! Differential: buffered writes produce the same final bytes
//! as direct writes.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Debug)]
pub struct BufferedSerializer {
    pending: Vec<u8>,
    flushed: Vec<u8>,
    max_bufsize: usize,
    chunk_size: usize,
}

impl BufferedSerializer {
    pub fn new(max_bufsize: usize, chunk_size: usize) -> Self {
        Self {
            pending: Vec::new(),
            flushed: Vec::new(),
            max_bufsize: max_bufsize.max(1),
            chunk_size: chunk_size.max(1),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(1024 * 1024, 64 * 1024)
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        self.pending.extend_from_slice(data);
        if self.pending.len() >= self.max_bufsize {
            self.flush();
        }
        data.len()
    }

    pub fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let mut offset = 0;
        while offset < self.pending.len() {
            let end = (offset + self.chunk_size).min(self.pending.len());
            self.flushed.extend_from_slice(&self.pending[offset..end]);
            offset = end;
        }
        self.pending.clear();
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        self.flush();
        self.flushed.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_write() {
        let mut b = BufferedSerializer::new(1024, 256);
        b.write(b"hello");
        let got = b.as_bytes();
        assert_eq!(&got, b"hello");
    }

    #[test]
    fn auto_flush() {
        let mut b = BufferedSerializer::new(4, 2);
        b.write(b"abcdef");
        let got = b.as_bytes();
        assert_eq!(&got, b"abcdef");
    }

    #[test]
    fn empty_safe() {
        let mut b = BufferedSerializer::new(10, 5);
        b.flush();
        assert!(b.as_bytes().is_empty());
    }

    #[test]
    fn differential() {
        let mut buffered = BufferedSerializer::new(8, 4);
        buffered.write(b"xyz");
        let got = buffered.as_bytes();
        let expected = b"xyz".to_vec();
        assert_equal(got, expected, "buffered bytes");
    }
}
