//! Safe pure pipe stub (OBS pipe.h core).
//!
//! The C version wraps platform pipe handles.
//! The safe path is a pure in-memory byte pipe (write then read).
//! No OS handles, no unwraps. Short reads return partial counts.
//!
//! Differential: written bytes equal read bytes.

#[cfg(test)]
use crate::diff::assert_equal;

#[derive(Clone, Debug, Default)]
pub struct Pipe {
    data: Vec<u8>,
    pos: usize,
}

impl Pipe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.data.extend_from_slice(buf);
        buf.len()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let available = self.data.len().saturating_sub(self.pos);
        let n = available.min(buf.len());
        if n > 0 {
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
        }
        n
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let mut p = Pipe::new();
        p.write(b"hello");
        let mut buf = [0u8; 5];
        assert_eq!(p.read(&mut buf), 5);
        assert_eq!(&buf, b"hello");
        assert!(p.is_empty());
    }

    #[test]
    fn short_read_safe() {
        let mut p = Pipe::new();
        p.write(b"ab");
        let mut buf = [0u8; 4];
        assert_eq!(p.read(&mut buf), 2);
    }

    #[test]
    fn empty() {
        let mut p = Pipe::new();
        let mut buf = [0u8; 1];
        assert_eq!(p.read(&mut buf), 0);
    }

    #[test]
    fn differential() {
        let mut p = Pipe::new();
        p.write(b"xyz");
        let mut buf = [0u8; 3];
        p.read(&mut buf);
        assert_equal(buf.to_vec(), b"xyz".to_vec(), "pipe round-trip");
    }
}
