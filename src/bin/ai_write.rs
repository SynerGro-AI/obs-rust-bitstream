//! GPU check demo.

use obs_rust_bitstream::gpu_detect::GpuDetect;
use obs_rust_bitstream::gpu_updater::GpuUpdater;

fn main() {
    let info = GpuDetect::detect();
    println!(
        "gpu: present={} kind={:?} driver={} version={}",
        info.is_present(),
        info.kind,
        info.driver,
        info.version
    );
    let status = GpuUpdater::check();
    println!("status: {:?}", status);
}