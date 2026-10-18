//! Safe pure AudioMonitorFeed (mix → monitor).
//!
//! Takes an AudioMix pull and applies an AudioMonitor's
//! volume and mute. Muted or missing pull returns None.
//! No unwraps.
//!
//! Differential: volume changes the peak.

use crate::audio_mix::AudioMix;
use crate::audio_monitor::AudioMonitor;
use crate::audio_source_list::AudioSourceList;

pub struct AudioMonitorFeed;

impl AudioMonitorFeed {
    /// Pull from the mix and apply the monitor.
    /// Returns None if the monitor is muted or the mix yields nothing.
    pub fn feed(
        mix: &AudioMix,
        list: &mut AudioSourceList,
        monitor: &AudioMonitor,
        frames: usize,
    ) -> Option<Vec<f32>> {
        if monitor.muted() {
            return None;
        }
        let samples = mix.pull(list, frames)?;
        let vol = monitor.volume();
        Some(samples.into_iter().map(|s| s * vol).collect())
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
    fn feed_applies_volume() {
        let (mix, mut list, mut monitor) = setup();
        monitor.set_volume(0.5);
        let out = AudioMonitorFeed::feed(&mix, &mut list, &monitor, 8).unwrap();
        assert_eq!(out.len(), 8);
    }

    #[test]
    fn muted_is_none() {
        let (mix, mut list, mut monitor) = setup();
        monitor.set_muted(true);
        assert!(AudioMonitorFeed::feed(&mix, &mut list, &monitor, 8).is_none());
    }

    #[test]
    fn differential_volume() {
        let (mix, mut list, mut monitor) = setup();
        monitor.set_volume(0.25);
        let low = AudioMonitorFeed::feed(&mix, &mut list, &monitor, 16)
            .unwrap()
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        monitor.set_volume(1.0);
        let high = AudioMonitorFeed::feed(&mix, &mut list, &monitor, 16)
            .unwrap()
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        assert!(high > low);
    }
}
