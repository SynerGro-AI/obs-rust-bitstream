//! Sets BUILD_TIME so the status can report when
//! the binary was built.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let time = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        });
    println!("cargo:rustc-env=BUILD_TIME={}", time);
}
