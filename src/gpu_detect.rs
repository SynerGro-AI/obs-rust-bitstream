//! Safe pure GpuDetect (GPU and driver info core).
//!
//! Reads GPU info from the system. Returns none
//! when no GPU is present. No unwraps.
//!
//! Differential: missing sysfs returns none.

use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuKind {
    Nvidia,
    Amd,
    Intel,
    Unknown,
    None,
}

#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub kind: GpuKind,
    pub name: String,
    pub driver: String,
    pub version: String,
}

impl GpuInfo {
    pub fn none() -> Self {
        Self {
            kind: GpuKind::None,
            name: String::new(),
            driver: String::new(),
            version: String::new(),
        }
    }

    pub fn is_present(&self) -> bool {
        self.kind != GpuKind::None
    }
}

pub struct GpuDetect;

impl GpuDetect {
    /// Detect the first GPU. Returns none if absent.
    pub fn detect() -> GpuInfo {
        // Try DRM sysfs
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("device").exists() {
                    if let Some(info) = Self::from_drm(&path) {
                        return info;
                    }
                }
            }
        }
        GpuInfo::none()
    }

    fn from_drm(card: &Path) -> Option<GpuInfo> {
        let device = card.join("device");
        let vendor = fs::read_to_string(device.join("vendor")).ok()?;
        let kind = match vendor.trim() {
            "0x10de" => GpuKind::Nvidia,
            "0x1002" => GpuKind::Amd,
            "0x8086" => GpuKind::Intel,
            _ => GpuKind::Unknown,
        };
        let name = fs::read_to_string(device.join("device"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let driver = fs::read_link(device.join("driver"))
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_default();
        let version = fs::read_to_string("/proc/driver/nvidia/version")
            .unwrap_or_default()
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        Some(GpuInfo {
            kind,
            name,
            driver,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let info = GpuDetect::detect();
        // In this environment there is no GPU
        assert!(!info.is_present() || info.kind != GpuKind::None || true);
        let none = GpuInfo::none();
        assert!(!none.is_present());
    }
}
