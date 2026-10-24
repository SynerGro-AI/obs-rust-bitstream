//! Run a simulation on its own thread and let the UI read snapshots (A8).
//!
//! The worker owns the state and runs submitted jobs strictly in order, so
//! the result depends only on the jobs, never on timing: the same jobs give
//! the same state as running them inline. After each job (and whenever a
//! long job calls [`Ctl::publish`]) a clone of the state is published for
//! the UI to pick up with [`SimWorker::latest`]. [`SimWorker::finish`]
//! drains the queue and hands the authoritative state back.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

type Job<S> = Box<dyn FnOnce(&mut S, &Ctl<S>) + Send>;

struct Shared<S> {
    snap: Mutex<Option<S>>,
    done: AtomicU64,
    cancel: AtomicBool,
    /// (work done, work total) reported by the running job.
    progress: Mutex<(u64, u64)>,
}

/// What a running job can see of its worker.
pub struct Ctl<S> {
    shared: Arc<Shared<S>>,
}

impl<S: Clone> Ctl<S> {
    /// Publish an intermediate snapshot and progress.
    pub fn publish(&self, s: &S, done: u64, total: u64) {
        *self.shared.progress.lock().unwrap() = (done, total);
        *self.shared.snap.lock().unwrap() = Some(s.clone());
    }

    /// True once [`SimWorker::cancel`] was called; long jobs should stop.
    pub fn cancelled(&self) -> bool {
        self.shared.cancel.load(Ordering::Relaxed)
    }
}

pub struct SimWorker<S> {
    tx: Option<Sender<Job<S>>>,
    shared: Arc<Shared<S>>,
    sent: u64,
    handle: Option<JoinHandle<S>>,
}

impl<S: Clone + Send + 'static> SimWorker<S> {
    pub fn spawn(name: &str, mut state: S) -> Self {
        let (tx, rx) = channel::<Job<S>>();
        let shared = Arc::new(Shared {
            snap: Mutex::new(None),
            done: AtomicU64::new(0),
            cancel: AtomicBool::new(false),
            progress: Mutex::new((0, 0)),
        });
        let ctl = Ctl {
            shared: shared.clone(),
        };
        let handle = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                for job in rx {
                    if !ctl.cancelled() {
                        job(&mut state, &ctl);
                        *ctl.shared.snap.lock().unwrap() = Some(state.clone());
                    }
                    ctl.shared.done.fetch_add(1, Ordering::Release);
                }
                state
            })
            .expect("spawn simulation thread");
        Self {
            tx: Some(tx),
            shared,
            sent: 0,
            handle: Some(handle),
        }
    }

    /// Queue a job; returns at once.
    pub fn submit(&mut self, job: impl FnOnce(&mut S, &Ctl<S>) + Send + 'static) {
        if let Some(tx) = &self.tx {
            if tx.send(Box::new(job)).is_ok() {
                self.sent += 1;
            }
        }
    }

    /// Jobs submitted but not finished yet.
    pub fn pending(&self) -> u64 {
        self.sent - self.shared.done.load(Ordering::Acquire)
    }

    /// The newest published snapshot, if one arrived since the last call.
    pub fn latest(&self) -> Option<S> {
        self.shared.snap.lock().unwrap().take()
    }

    /// Progress of the running job as (done, total); (0, 0) when idle.
    pub fn progress(&self) -> (u64, u64) {
        if self.pending() == 0 {
            (0, 0)
        } else {
            *self.shared.progress.lock().unwrap()
        }
    }

    /// Ask queued and running jobs to stop early (the state is then
    /// whatever they reached).
    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Relaxed);
    }

    /// Wait for every queued job and return the state.
    pub fn finish(mut self) -> S {
        self.tx = None;
        self.handle
            .take()
            .expect("worker joined once")
            .join()
            .expect("simulation thread panicked")
    }
}

impl<S> Drop for SimWorker<S> {
    fn drop(&mut self) {
        self.shared.cancel.store(true, Ordering::Relaxed);
        self.tx = None;
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{stress_pile, Environment, MAX_CATCH_UP};

    /// Frame times with jitter and a couple of stalls, like a real UI.
    fn frames() -> Vec<f32> {
        (0..400)
            .map(|i| match i {
                100 => 2.5,
                250 => 0.4,
                _ => 1.0 / 60.0 + ((i * 37) % 11) as f32 * 4e-4,
            })
            .collect()
    }

    #[test]
    fn arena_on_the_worker_matches_the_inline_arena_bit_for_bit() {
        let start = stress_pile(Environment::earth(), 120, 4);
        let mut inline = start.clone();
        for dt in frames() {
            inline.advance(dt);
        }
        let mut w = SimWorker::spawn("arena-test", start);
        for dt in frames() {
            w.submit(move |a: &mut crate::arena::Arena, _| {
                a.advance(dt);
            });
            // the UI reading snapshots meanwhile must not disturb the run
            let _ = w.latest();
        }
        let worker = w.finish();
        assert_eq!(format!("{:?}", worker), format!("{:?}", inline));
    }

    #[test]
    fn long_runs_publish_progress_and_can_be_cancelled() {
        let mut w = SimWorker::spawn("arena-test", stress_pile(Environment::moon(), 30, 2));
        w.submit(|a: &mut crate::arena::Arena, c| {
            let total = 240 * 600;
            for k in 0..total {
                if k % MAX_CATCH_UP as u64 == 0 {
                    c.publish(a, k, total);
                    if c.cancelled() {
                        return;
                    }
                }
                a.step();
            }
        });
        while w.progress().0 < 2 * MAX_CATCH_UP as u64 {
            std::thread::yield_now();
        }
        assert!(w.latest().is_some());
        w.cancel();
        let a = w.finish();
        assert!(a.time > 1.0 && a.time < 600.0, "stopped at {}", a.time);
    }
}
