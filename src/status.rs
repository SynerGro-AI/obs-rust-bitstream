//! Safe pure Status (output state core).
//!
//! Reports whether record, stream, and virtual camera are
//! running. No unwraps.
//!
//! Differential: all three running reports all three.

use crate::program_record::ProgramRecord;
use crate::program_stream::ProgramStream;
use crate::virtual_camera::VirtualCamera;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub recording: bool,
    pub streaming: bool,
    pub vcam: bool,
}

impl Status {
    pub fn from(record: &ProgramRecord, stream: &ProgramStream, vcam: &VirtualCamera) -> Self {
        Self {
            recording: record.is_recording(),
            streaming: stream.is_streaming(),
            vcam: vcam.is_running(),
        }
    }

    pub fn any(&self) -> bool {
        self.recording || self.streaming || self.vcam
    }

    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.recording {
            parts.push("REC");
        }
        if self.streaming {
            parts.push("LIVE");
        }
        if self.vcam {
            parts.push("VCAM");
        }
        if parts.is_empty() {
            "idle".to_string()
        } else {
            parts.join(" ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle() {
        let s = Status::from(
            &ProgramRecord::new(),
            &ProgramStream::new(),
            &VirtualCamera::new(),
        );
        assert_eq!(s.label(), "idle");
    }

    #[test]
    fn differential() {
        let mut r = ProgramRecord::new();
        let mut s = ProgramStream::new();
        let mut v = VirtualCamera::new();
        r.start();
        s.start("rtmp://live").unwrap();
        v.start();
        let st = Status::from(&r, &s, &v);
        assert!(st.any());
        assert_eq!(st.label(), "REC LIVE VCAM");
    }
}
