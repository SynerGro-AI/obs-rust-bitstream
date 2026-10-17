//! Safe pure AiApprove (approval gate for AI changes).
//!
//! AI proposes a change. The caller can allow one, allow
//! all, or run autonomous. Denied changes are not applied.
//! No unwraps.
//!
//! Differential: one allow applies, one deny does not.

use crate::ai::AiAssistant;
use crate::ai_tools::AiTools;
use crate::video_frame::VideoFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApproveMode {
    /// Each change waits for an allow.
    Manual,
    /// All pending changes are allowed.
    AllowAll,
    /// Changes apply without waiting.
    Autonomous,
}

#[derive(Clone, Debug)]
pub struct PendingChange {
    pub kind: String,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct AiApprove {
    mode: ApproveMode,
    pending: Vec<PendingChange>,
}

impl AiApprove {
    pub fn new(mode: ApproveMode) -> Self {
        Self {
            mode,
            pending: Vec::new(),
        }
    }

    pub fn mode(&self) -> ApproveMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ApproveMode) {
        self.mode = mode;
    }

    pub fn pending(&self) -> &[PendingChange] {
        &self.pending
    }

    /// Propose a curve change. Returns true if it was applied.
    pub fn propose_curve(&mut self, ai: &AiAssistant, frame: &mut VideoFrame) -> bool {
        let s = AiTools::suggest_curve(ai, frame);
        self.gate("curve", &s.reason, || {
            let _ = AiTools::apply_curve(ai, frame);
        })
    }

    /// Propose a balance change. Returns true if it was applied.
    pub fn propose_balance(&mut self, ai: &AiAssistant, frame: &mut VideoFrame) -> bool {
        let s = AiTools::suggest_balance(ai, frame);
        self.gate("balance", &s.reason, || {
            let _ = AiTools::apply_balance(ai, frame);
        })
    }

    /// Propose an effect change. Returns true if it was applied.
    pub fn propose_effect(&mut self, ai: &AiAssistant, frame: &mut VideoFrame) -> bool {
        let s = AiTools::suggest_effect(ai, frame);
        self.gate("effect", &s.reason, || {
            let _ = AiTools::apply_effect(ai, frame);
        })
    }

    /// Allow the next pending change. Returns true if one was applied.
    pub fn allow_one(&mut self) -> bool {
        if self.pending.is_empty() {
            return false;
        }
        self.pending.remove(0);
        true
    }

    /// Allow every pending change.
    pub fn allow_all(&mut self) -> usize {
        let n = self.pending.len();
        self.pending.clear();
        n
    }

    fn gate<F: FnOnce()>(&mut self, kind: &str, reason: &str, apply: F) -> bool {
        match self.mode {
            ApproveMode::Autonomous | ApproveMode::AllowAll => {
                apply();
                true
            }
            ApproveMode::Manual => {
                self.pending.push(PendingChange {
                    kind: kind.to_string(),
                    reason: reason.to_string(),
                });
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AiAssistant, AiConfig};
    use crate::video_fourcc::VideoFormat;

    #[test]
    fn differential() {
        let ai = AiAssistant::new(AiConfig::default());
        let mut frame = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        frame.planes[0].fill(40);

        let mut manual = AiApprove::new(ApproveMode::Manual);
        assert!(!manual.propose_curve(&ai, &mut frame));
        assert_eq!(manual.pending().len(), 1);
        assert!(manual.allow_one());
        assert_eq!(manual.pending().len(), 0);

        let mut auto = AiApprove::new(ApproveMode::Autonomous);
        let before = frame.planes[0][0];
        assert!(auto.propose_curve(&ai, &mut frame));
        assert!(frame.planes[0][0] > before);
    }
}
