//! Safe pure GpuUpdater (driver check and update core).
//!
//! Checks if a driver update is available and can
//! run the OS updater only when asked. No unwraps.
//! Never runs automatically.
//!
//! Differential: check returns a status.

use crate::gpu_detect::{GpuDetect, GpuInfo, GpuKind};
use std::io::{self, Write};
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
        // Compare against a known minimum. A real check would query the vendor.
        let current = Self::parse_version(&info.version);
        let minimum = (535, 0);
        if current.0 < minimum.0 {
            UpdateStatus::UpdateAvailable
        } else {
            UpdateStatus::Current
        }
    }

    fn parse_version(s: &str) -> (u32, u32) {
        let mut major = 0u32;
        let mut minor = 0u32;
        let mut parts = s.split(|c: char| !c.is_ascii_digit());
        if let Some(m) = parts.find(|p| !p.is_empty()) {
            major = m.parse().unwrap_or(0);
        }
        if let Some(m) = parts.find(|p| !p.is_empty()) {
            minor = m.parse().unwrap_or(0);
        }
        (major, minor)
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

    /// Check and prompt for approval before upgrading.
    /// Returns Ok(true) if the user approved and the update ran.
    /// Returns Ok(false) if the user declined or no update is needed.
    pub fn prompt_and_upgrade() -> Result<bool, UpdateError> {
        let status = Self::check();
        match status {
            UpdateStatus::NoGpu => return Err(UpdateError::NoGpu),
            UpdateStatus::Current => {
                println!("GPU driver is current.");
                return Ok(false);
            }
            UpdateStatus::Unknown => {
                println!("GPU driver status unknown.");
                return Ok(false);
            }
            UpdateStatus::UpdateAvailable => {}
        }
        print!("Update available. Approve? [y/N] ");
        let _ = io::stdout().flush();
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|_| UpdateError::Failed)?;
        if answer.trim().eq_ignore_ascii_case("y") {
            Self::upgrade()?;
            Ok(true)
        } else {
            println!("Update declined.");
            Ok(false)
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
        assert_eq!(GpuUpdater::parse_version("535.129.03"), (535, 129));
        assert_eq!(GpuUpdater::parse_version("no version"), (0, 0));
    }
}
