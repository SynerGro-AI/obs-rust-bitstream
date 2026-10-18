//! Safe pure StreamStart (service + OBS defaults → stream).
//!
//! Starts a ProgramStream with a service name and applies
//! the OBS encoder defaults to the weights. Empty service
//! is rejected. No unwraps.
//!
//! Differential: a higher bitrate setting produces a higher value.

use crate::live_weights::LiveWeights;
use crate::obs_encoder_defaults::ObsEncoderDefaults;
use crate::output_info::OutputInfo;
use crate::output_service::OutputService;
use crate::program_stream::{ProgramStream, StreamError};

pub struct StreamStart;

impl StreamStart {
    /// Start the stream. Sets the service and writes OBS defaults
    /// into the weights so the encoder uses the shippable metrics.
    /// Returns the applied defaults.
    pub fn start(
        stream: &mut ProgramStream,
        service: &mut OutputService,
        output: &OutputInfo,
        weights: &mut LiveWeights,
        name: &str,
        id: &str,
    ) -> Result<ObsEncoderDefaults, StreamError> {
        if name.is_empty() {
            return Err(StreamError::EmptyTarget);
        }
        service
            .set(output, name, id)
            .map_err(|_| StreamError::EmptyTarget)?;
        let defaults = ObsEncoderDefaults::from_weights(weights);
        let _ = weights.set("h264.bitrate", defaults.bitrate_kbps as f32);
        let _ = weights.set("h264.crf", defaults.crf as f32);
        stream.start(name)?;
        Ok(defaults)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output_info::{OutputInfo, OUTPUT_AV};

    #[test]
    fn starts() {
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        let d = StreamStart::start(&mut stream, &mut svc, &out, &mut w, "twitch", "rtmp_common")
            .unwrap();
        assert!(stream.is_streaming());
        assert_eq!(svc.service().unwrap().name(), "twitch");
        assert_eq!(d.bitrate_kbps, 6000);
        assert_eq!(w.get("h264.bitrate").unwrap(), 6000.0);
    }

    #[test]
    fn empty_rejected() {
        let mut stream = ProgramStream::new();
        let mut svc = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut w = LiveWeights::new();
        assert!(StreamStart::start(&mut stream, &mut svc, &out, &mut w, "", "rtmp").is_err());
        assert!(!stream.is_streaming());
    }

    #[test]
    fn differential() {
        let mut a = ProgramStream::new();
        let mut b = ProgramStream::new();
        let mut sa = OutputService::new();
        let mut sb = OutputService::new();
        let out = OutputInfo::new("rec", "Recording", OUTPUT_AV).unwrap();
        let mut wa = LiveWeights::new();
        let mut wb = LiveWeights::new();
        wb.set("h264.bitrate", 8000.0).unwrap();
        let da = StreamStart::start(&mut a, &mut sa, &out, &mut wa, "twitch", "rtmp").unwrap();
        let db = StreamStart::start(&mut b, &mut sb, &out, &mut wb, "youtube", "rtmp").unwrap();
        assert!(db.bitrate_kbps > da.bitrate_kbps);
    }
}
