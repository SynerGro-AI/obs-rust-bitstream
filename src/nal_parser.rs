//! Safe pure NalParser (AVC/HEVC NAL core).
//!
//! Splits a byte stream into NAL units. No unwraps.
//!
//! Differential: a three-byte start code splits two units.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NalError {
    Empty,
}

#[derive(Clone, Debug)]
pub struct NalUnit {
    pub nal_type: u8,
    pub payload: Vec<u8>,
}

pub struct NalParser;

impl NalParser {
    /// Split Annex-B bytes into NAL units. Start code is 00 00 01.
    pub fn parse(data: &[u8]) -> Result<Vec<NalUnit>, NalError> {
        if data.is_empty() {
            return Err(NalError::Empty);
        }
        let mut units = Vec::new();
        let mut i = 0;
        while i + 3 < data.len() {
            if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
                let start = i + 3;
                let mut end = start;
                while end + 3 < data.len() {
                    if data[end] == 0 && data[end + 1] == 0 && data[end + 2] == 1 {
                        break;
                    }
                    end += 1;
                }
                if end + 3 >= data.len() {
                    end = data.len();
                }
                if start < end {
                    let payload = data[start..end].to_vec();
                    let nal_type = payload.first().copied().unwrap_or(0) & 0x1F;
                    units.push(NalUnit { nal_type, payload });
                }
                i = end;
            } else {
                i += 1;
            }
        }
        Ok(units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(NalParser::parse(&[]).is_err());
    }

    #[test]
    fn differential() {
        let data = [0, 0, 1, 0x65, 1, 2, 0, 0, 1, 0x41, 3];
        let units = NalParser::parse(&data).unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].nal_type, 5);
        assert_eq!(units[1].nal_type, 1);
    }
}
