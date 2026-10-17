//! Safe pure RawReader (read .obsr core).
//!
//! Reads a header and pixels into a FrameSequence.
//! Bad magic rejected. No unwraps.
//!
//! Differential: write + read round-trips.

use crate::fps_trailer::FpsTrailer;
use crate::frame_sequence::FrameSequence;
use crate::raw_writer::WriteError;
use crate::timed_sequence::TimedSequence;
use crate::video_fourcc::VideoFormat;
use crate::video_frame::VideoFrame;
use std::io::Read;

const MAGIC: &[u8; 4] = b"OBSR";

pub struct RawReader;

impl RawReader {
    /// Read a sequence from any Read.
    pub fn read<R: Read>(mut input: R) -> Result<FrameSequence, WriteError> {
        let mut magic = [0u8; 4];
        input.read_exact(&mut magic).map_err(|_| WriteError::Io)?;
        if &magic != MAGIC {
            return Err(WriteError::Io);
        }
        let mut buf = [0u8; 4];
        input.read_exact(&mut buf).map_err(|_| WriteError::Io)?;
        let width = u32::from_le_bytes(buf);
        input.read_exact(&mut buf).map_err(|_| WriteError::Io)?;
        let height = u32::from_le_bytes(buf);
        input.read_exact(&mut buf).map_err(|_| WriteError::Io)?;
        let count = u32::from_le_bytes(buf);
        let mut seq = FrameSequence::new();
        let size = (width as usize) * (height as usize);
        for _ in 0..count {
            let mut pixels = vec![0u8; size];
            input.read_exact(&mut pixels).map_err(|_| WriteError::Io)?;
            if let Some(mut frame) = VideoFrame::new(VideoFormat::Y800, width, height) {
                if let Some(plane) = frame.planes.get_mut(0) {
                    plane.copy_from_slice(&pixels);
                }
                seq.push(frame);
            }
        }
        Ok(seq)
    }

    /// Read frames and an fps trailer into a TimedSequence.
    pub fn read_timed<R: Read>(mut input: R) -> Result<TimedSequence, WriteError> {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf).map_err(|_| WriteError::Io)?;
        let fps = FpsTrailer::read(&buf).unwrap_or(30.0);
        let frames_end = if buf.len() >= 8 && &buf[buf.len() - 8..buf.len() - 4] == b"FPS!" {
            buf.len() - 8
        } else {
            buf.len()
        };
        let seq = Self::read(&buf[..frames_end])?;
        let mut timed = TimedSequence::new(fps).map_err(|_| WriteError::Io)?;
        for i in 0..seq.len() {
            if let Ok(f) = seq.get(i) {
                timed.push(f.clone());
            }
        }
        Ok(timed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw_writer::RawWriter;
    use std::io::Cursor;

    #[test]
    fn differential() {
        let mut f = VideoFrame::new(VideoFormat::Y800, 2, 2).unwrap();
        f.planes[0].fill(42);
        let mut seq = FrameSequence::new();
        seq.push(f);
        let mut buf = Vec::new();
        RawWriter::write(&seq, &mut buf).unwrap();
        let back = RawReader::read(Cursor::new(buf)).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back.get(0).unwrap().planes[0][0], 42);
        let mut buf2 = Vec::new();
        RawWriter::write(&seq, &mut buf2).unwrap();
        FpsTrailer::write(24.0, &mut buf2).unwrap();
        let timed = RawReader::read_timed(std::io::Cursor::new(buf2)).unwrap();
        assert_eq!(timed.fps(), 24.0);
        assert_eq!(timed.len(), 1);
    }
}
