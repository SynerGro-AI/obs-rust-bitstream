//! Safe pure Module (OBS module registration core).
//!
//! A module registers source, output, encoder, and filter type ids.
//! Empty ids are rejected. Duplicate ids within a category are rejected.
//! No unwraps.
//!
//! Differential: register + has returns true.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleError {
    EmptyId,
    Duplicate,
}

#[derive(Clone, Debug, Default)]
pub struct Module {
    id: String,
    sources: Vec<String>,
    outputs: Vec<String>,
    encoders: Vec<String>,
    filters: Vec<String>,
}

impl Module {
    pub fn new(id: &str) -> Result<Self, ModuleError> {
        if id.is_empty() {
            return Err(ModuleError::EmptyId);
        }
        Ok(Self {
            id: id.to_string(),
            ..Self::default()
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn register_source(&mut self, id: &str) -> Result<(), ModuleError> {
        push_id(&mut self.sources, id)
    }

    pub fn register_output(&mut self, id: &str) -> Result<(), ModuleError> {
        push_id(&mut self.outputs, id)
    }

    pub fn register_encoder(&mut self, id: &str) -> Result<(), ModuleError> {
        push_id(&mut self.encoders, id)
    }

    pub fn register_filter(&mut self, id: &str) -> Result<(), ModuleError> {
        push_id(&mut self.filters, id)
    }

    pub fn has_source(&self, id: &str) -> bool {
        self.sources.iter().any(|x| x == id)
    }

    pub fn has_filter(&self, id: &str) -> bool {
        self.filters.iter().any(|x| x == id)
    }

    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

fn push_id(list: &mut Vec<String>, id: &str) -> Result<(), ModuleError> {
    if id.is_empty() {
        return Err(ModuleError::EmptyId);
    }
    if list.iter().any(|x| x == id) {
        return Err(ModuleError::Duplicate);
    }
    list.push(id.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register() {
        let mut m = Module::new("obs-ffmpeg").unwrap();
        m.register_source("ffmpeg_source").unwrap();
        m.register_encoder("ffmpeg_aac").unwrap();
        assert!(m.has_source("ffmpeg_source"));
        assert_eq!(m.source_count(), 1);
    }

    #[test]
    fn empty_and_duplicate() {
        assert!(Module::new("").is_err());
        let mut m = Module::new("m").unwrap();
        m.register_source("s").unwrap();
        assert!(matches!(
            m.register_source("s"),
            Err(ModuleError::Duplicate)
        ));
        assert!(matches!(m.register_source(""), Err(ModuleError::EmptyId)));
    }

    #[test]
    fn differential() {
        let mut m = Module::new("core").unwrap();
        m.register_filter("gain_filter").unwrap();
        assert!(!m.has_source("gain_filter"));
    }
}
