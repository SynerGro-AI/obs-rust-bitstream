//! Safe pure PacketCallback (OBS packet_callback).
//!
//! Parsed from libobs/obs-internal.h.
//! Holds a list of named callbacks that receive packets.
//! Empty names are rejected. Invoke returns the count of
//! callbacks that accepted the packet. No unwraps.
//!
//! Differential: more callbacks accept more packets.

use crate::encoder_packet::EncoderPacket;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallbackError {
    EmptyName,
}

#[derive(Clone, Debug)]
pub struct PacketCallback {
    name: String,
    accept: bool,
}

impl PacketCallback {
    /// Create a callback. Empty name is rejected.
    pub fn new(name: &str) -> Result<Self, CallbackError> {
        if name.is_empty() {
            return Err(CallbackError::EmptyName);
        }
        Ok(Self {
            name: name.to_string(),
            accept: true,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_accept(&mut self, accept: bool) {
        self.accept = accept;
    }

    /// Return true if this callback accepts the packet.
    pub fn call(&self, _pkt: &EncoderPacket) -> bool {
        self.accept
    }
}

#[derive(Clone, Debug, Default)]
pub struct PacketCallbackList {
    callbacks: Vec<PacketCallback>,
}

impl PacketCallbackList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a callback. Empty name is rejected.
    pub fn add(&mut self, name: &str) -> Result<(), CallbackError> {
        let cb = PacketCallback::new(name)?;
        self.callbacks.push(cb);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.callbacks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }

    /// Invoke all callbacks. Returns the number that accepted.
    pub fn invoke(&self, pkt: &EncoderPacket) -> usize {
        self.callbacks.iter().filter(|c| c.call(pkt)).count()
    }

    /// Get a callback by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut PacketCallback> {
        self.callbacks.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder_context::EncoderType;

    #[test]
    fn add_and_invoke() {
        let mut list = PacketCallbackList::new();
        list.add("mux").unwrap();
        list.add("log").unwrap();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        assert_eq!(list.invoke(&pkt), 2);
    }

    #[test]
    fn empty_name() {
        let mut list = PacketCallbackList::new();
        assert!(list.add("").is_err());
    }

    #[test]
    fn reject_one() {
        let mut list = PacketCallbackList::new();
        list.add("a").unwrap();
        list.add("b").unwrap();
        list.get_mut(1).unwrap().set_accept(false);
        let pkt = EncoderPacket::new(EncoderType::Audio, 0, 0, vec![2]);
        assert_eq!(list.invoke(&pkt), 1);
    }

    #[test]
    fn differential() {
        let mut a = PacketCallbackList::new();
        let mut b = PacketCallbackList::new();
        a.add("one").unwrap();
        b.add("one").unwrap();
        b.add("two").unwrap();
        let pkt = EncoderPacket::new(EncoderType::Video, 0, 0, vec![1]);
        assert!(b.invoke(&pkt) > a.invoke(&pkt));
    }
}
