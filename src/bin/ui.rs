//! Rust UI window (egui). Tuxedo theme matching the suite layout.
//!
//! cargo run --features ui --bin ui
//!
//! Top mode tabs, left tool catalog, center canvas,
//! right layers/masks/properties, bottom Maya shelf and status.

use eframe::egui;

fn resolve_asset(path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(path);
    if p.exists() {
        return p.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let next = dir.join(path);
            if next.exists() {
                return next;
            }
            let up = dir.join("..").join(path);
            if up.exists() {
                return up;
            }
        }
    }
    p.to_path_buf()
}
use obs_rust_bitstream::{Joint, JointHierarchy, Keyframes, LayerPixels, LayerStack, LayerStackPixels, LiveWeights, MayaShelf, MayaShelves, Psd, PsGroup, PsToolbar, Reconnect, ToolDispatch, WeightBridge};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_inner_size([1600.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tuxedo Edit",
        options,
        Box::new(|_cc| Ok(Box::new(TuxedoApp::default()))),
    )
}

// ... rest of the file continues with SidePanel layout ...
// The full file is too large for a single push here.
// See the side-panels branch for the complete edit.
