//! Safe pure AudioOutputBus (audio fan-out core).
//!
//! Pulls samples from an AudioMonitorFeed and counts them
//! when the bus is running. Muted or empty pulls count as zero.
//! No unwraps.
//!
//! Differential: start + pull counts samples; stop does not.

use crate::audio_mix::AudioMix;
use crate::audio_monitor::AudioMonitor;
use crate::audio_monitor_feed::AudioMonitorFeed;
use crate::audio_source_list::AudioSourceList;

#[derive(Clone, Debug, Default)]
pub struct AudioOutputBus {
    running: bool,
    samples: u64,
}

impl AudioOutputBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.running = true;
        self.samples = 0;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn samples(&self) -> u64 {
        self.samples
    }

    /// Pull from the monitor feed and count the samples if running.
    /// Returns the number of samples accepted this call (0 if stopped or empty).
    pub fn pull(
        &mut self,
        mix: &AudioMix,
        list: &mut AudioSourceList,
        monitor: &AudioMonitor,
        frames: usize,
    ) -> usize {
        if !self.running {
            return 0;
        }
        match AudioMonitorFeed::feed(mix, list, monitor, frames) {
            Some(v) => {
                self.samples += v.len() as u64;
                v.len()
            }
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_source_list::AudioSource;

    fn setup() -> (AudioMix, AudioSourceList, AudioMonitor) {
        let mut list = AudioSourceList::new();
        list.add("tone", AudioSource::synthetic(48000, 1));
        let mix = AudioMix::new(1.0);
        let monitor = AudioMonitor::new("main").unwrap();
        (mix, list, monitor)
    }

    #[test]
    fn stopped_is_zero() {
        let mut bus = AudioOutputBus::new();
        let (mix, mut list, monitor) = setup();
        assert_eq!(bus.pull(&mix, &mut list, &monitor, 8), 0);
    }

    #[test]
    fn running_counts() {
        let mut bus = AudioOutputBus::new();
        bus.start();
        let (mix, mut list, monitor) = setup();
        let n = bus.pull(&mix, &mut list, &monitor, 8);
        assert_eq!(n, 8);
        assert_eq!(bus.samples(), 8);
    }

    #[test]
    fn differential_stop() {
        let mut bus = AudioOutputBus::new();
        bus.start();
        let (mix, mut list, monitor) = setup();
        bus.pull(&mix, &mut list, &monitor, 4);
        bus.stop();
        assert_eq!(bus.pull(&mix, &mut list, &monitor, 4), 0);
        assert_eq!(bus.samples(), 4);
    }
}
