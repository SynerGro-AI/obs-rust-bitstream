//! Safe pure OutputDelay (delay buffer core).
//!
//! Holds frames for a number of ticks before releasing them.
//! No unwraps.
//!
//! Differential: a delay of 2 releases the first frame on the
//! third push.

#[derive(Clone, Debug)]
pub struct OutputDelay {
    delay: usize,
    buf: Vec<u64>,
}

impl OutputDelay {
    pub fn new(delay: usize) -> Self {
        Self {
            delay,
            buf: Vec::new(),
        }
    }

    pub fn delay(&self) -> usize {
        self.delay
    }

    /// Push a frame. Returns the frame that is now due, if any.
    pub fn push(&mut self, frame: u64) -> Option<u64> {
        self.buf.push(frame);
        if self.buf.len() > self.delay {
            Some(self.buf.remove(0))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn flush(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let d = OutputDelay::new(2);
        assert_eq!(d.delay(), 2);
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn differential() {
        let mut d = OutputDelay::new(2);
        assert!(d.push(1).is_none());
        assert!(d.push(2).is_none());
        assert_eq!(d.push(3), Some(1));
        assert_eq!(d.push(4), Some(2));
        assert_eq!(d.flush(), vec![3, 4]);
    }
}
