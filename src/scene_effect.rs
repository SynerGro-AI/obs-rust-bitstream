//! Safe pure SceneEffect (pipeline on a scene core).
//!
//! Holds a ToolPipeline and applies it to a frame.
//! Reads strength from LiveWeights. No unwraps.
//!
//! Differential: a bright step lifts a dark frame.

use crate::live_weights::LiveWeights;
use crate::tool_pipeline::{PipeError, ToolPipeline};
use crate::video_frame::VideoFrame;

#[derive(Clone, Debug, Default)]
pub struct SceneEffect {
    pipeline: ToolPipeline,
    enabled: bool,
}

impl SceneEffect {
    pub fn new() -> Self {
        Self {
            pipeline: ToolPipeline::new(),
            enabled: true,
        }
    }

    pub fn add(&mut self, name: &str) {
        self.pipeline.add(name);
    }

    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.pipeline.len()
    }

    /// Apply the pipeline if enabled.
    pub fn apply(&self, frame: &mut VideoFrame, weights: &LiveWeights) -> Result<(), PipeError> {
        if !self.enabled {
            return Ok(());
        }
        self.pipeline.run(frame, weights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let mut effect = SceneEffect::new();
        effect.add("bright");
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(40);
        let mut w = LiveWeights::new();
        w.set("pipe.bright", 20.0).unwrap();
        effect.apply(&mut f, &w).unwrap();
        assert_eq!(f.planes[0][0], 60);
        effect.set_enabled(false);
        f.planes[0].fill(40);
        effect.apply(&mut f, &w).unwrap();
        assert_eq!(f.planes[0][0], 40);
    }
}
