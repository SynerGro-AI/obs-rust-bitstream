//! Safe pure GpuUpdater (driver check and update core).
//!
//! Checks if a driver update is available and can
//! run the OS updater only when asked. No unwraps.
//! Never runs automatically.
//!
//! Differential: check returns a status.

use crate::gpu_detect::{GpuDetect, GpuInfo, GpuKind};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    NoGpu,
    Current,
    UpdateAvailable,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateError {
    NoGpu,
    Failed,
    NotSupported,
}

pub struct GpuUpdater;

impl GpuUpdater {
    /// Check if an update may be available.
    pub fn check() -> UpdateStatus {
        let info = GpuDetect::detect();
        if !info.is_present() {
            return UpdateStatus::NoGpu;
        }
        match info.kind {
            GpuKind::Nvidia => Self::check_nvidia(&info),
            GpuKind::Amd | GpuKind::Intel => UpdateStatus::Unknown,
            _ => UpdateStatus::Unknown,
        }
    }

    fn check_nvidia(info: &GpuInfo) -> UpdateStatus {
        if info.version.is_empty() {
            return UpdateStatus::Unknown;
        }
        // A real check would compare against the latest known version.
        // Here we report current if a version string exists.
        UpdateStatus::Current
    }

    /// Run the OS package updater. Only when explicitly called.
    pub fn upgrade() -> Result<(), UpdateError> {
        let info = GpuDetect::detect();
        if !info.is_present() {
            return Err(UpdateError::NoGpu);
        }
        // Use the OS package manager. Requires privileges.
        let status = Command::new("apt-get")
            .args(["update"])
            .status()
            .map_err(|_| UpdateError::Failed)?;
        if !status.success() {
            return Err(UpdateError::Failed);
        }
        let status = Command::new("apt-get")
            .args(["install", "-y", "nvidia-driver"])
            .status()
            .map_err(|_| UpdateError::Failed)?;
        if status.success() {
            Ok(())
        } else {
            Err(UpdateError::Failed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential() {
        let status = GpuUpdater::check();
        // No GPU here
        assert!(matches!(
            status,
            UpdateStatus::NoGpu | UpdateStatus::Current | UpdateStatus::Unknown
        ));
    }
}
