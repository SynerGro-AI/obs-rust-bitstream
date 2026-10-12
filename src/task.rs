//! Safe pure task queue (OBS task.h core).
//!
//! The C version is a threaded queue of function pointers.
//! The safe path is a pure Vec of closures that can be
//! drained and executed on demand. No threads, no raw
//! function pointers, no unwraps.
//!
//! Differential: tasks execute in the order they were queued.

use crate::diff::assert_equal;

type Task = Box<dyn FnOnce() + Send>;

#[derive(Default)]
pub struct TaskQueue {
    tasks: Vec<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue<F>(&mut self, f: F) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        self.tasks.push(Box::new(f));
        true
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Drain and run all queued tasks in order.
    pub fn drain(&mut self) {
        for task in self.tasks.drain(..) {
            task();
        }
    }

    /// Run the next task if any.
    pub fn run_one(&mut self) -> bool {
        if let Some(task) = self.tasks.first() {
            // Can't easily pop and call without ownership issues;
            // use swap_remove or just drain one.
            let task = self.tasks.remove(0);
            task();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn queue_and_drain() {
        let mut q = TaskQueue::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        let l1 = log.clone();
        q.queue(move || l1.lock().unwrap().push(1));
        let l2 = log.clone();
        q.queue(move || l2.lock().unwrap().push(2));
        assert_eq!(q.len(), 2);
        q.drain();
        assert!(q.is_empty());
        assert_eq!(&*log.lock().unwrap(), &[1, 2]);
    }

    #[test]
    fn empty_safe() {
        let mut q = TaskQueue::new();
        assert!(!q.run_one());
        q.drain(); // no-op
    }

    #[test]
    fn differential() {
        let mut q = TaskQueue::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        for i in 0..3 {
            let l = log.clone();
            q.queue(move || l.lock().unwrap().push(i));
        }
        q.drain();
        let got = log.lock().unwrap().clone();
        let expected = vec![0, 1, 2];
        assert_equal(got, expected, "task order");
    }
}
