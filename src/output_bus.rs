//! Safe pure OutputBus (fan-out core).
//!
//! Sends one frame to the record, the stream, and the
//! virtual camera if each is running. No unwraps.
//!
//! Differential: all three running counts one frame each.

use crate::program_record::ProgramRecord;
use crate::program_stream::ProgramStream;
use crate::virtual_camera::VirtualCamera;

#[derive(Clone, Debug, Default)]
pub struct OutputBus;

impl OutputBus {
    /// Send one frame to every running output. Returns how many received it.
    pub fn send(
        record: &mut ProgramRecord,
        stream: &mut ProgramStream,
        vcam: &mut VirtualCamera,
    ) -> usize {
        let mut n = 0;
        if record.record_frame() {
            n += 1;
        }
        if stream.send_frame() {
            n += 1;
        }
        if vcam.send_frame() {
            n += 1;
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none() {
        let mut r = ProgramRecord::new();
        let mut s = ProgramStream::new();
        let mut v = VirtualCamera::new();
        assert_eq!(OutputBus::send(&mut r, &mut s, &mut v), 0);
    }

    #[test]
    fn differential() {
        let mut r = ProgramRecord::new();
        let mut s = ProgramStream::new();
        let mut v = VirtualCamera::new();
        r.start();
        s.start("rtmp://live").unwrap();
        v.start();
        assert_eq!(OutputBus::send(&mut r, &mut s, &mut v), 3);
        assert_eq!(r.frames(), 1);
        assert_eq!(s.frames(), 1);
        assert_eq!(v.frames(), 1);
    }
}
