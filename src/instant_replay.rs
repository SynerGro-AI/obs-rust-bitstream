//! Safe pure InstantReplay (last-N frames core).
//!
//! Keeps the last N frames. push adds one, save returns
//! the buffer. No unwraps.
//!
//! Differential: push past capacity drops the oldest.

#[derive(Clone, Debug)]
pub struct InstantReplay {
    capacity: usize,
    frames: Vec<u64>,
}

impl InstantReplay {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            frames: Vec::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Push a frame number. Drops the oldest when full.
    pub fn push(&mut self, frame: u64) {
        if self.frames.len() == self.capacity {
            self.frames.remove(0);
        }
        self.frames.push(frame);
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn frames(&self) -> &[u64] {
        &self.frames
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let r = InstantReplay::new(3);
        assert_eq!(r.capacity(), 3);
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn differential() {
        let mut r = InstantReplay::new(3);
        r.push(1);
        r.push(2);
        r.push(3);
        r.push(4);
        assert_eq!(r.frames(), &[2, 3, 4]);
    }
}
