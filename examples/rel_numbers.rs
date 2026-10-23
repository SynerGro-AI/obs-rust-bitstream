use obs_rust_bitstream::relativity::*;
fn main() {
    let (m, p) = mercury_precession(5, 20_000);
    let per_century = 100.0 * JULIAN_YEAR / (87.969 * DAY) / ARCSEC;
    println!(
        "Mercury: simulated {:.3e} rad/orbit = {:.2}\"/century; predicted {:.2}\"",
        m,
        m * per_century,
        p * per_century
    );
    let (g, v, n) = gps_drift_per_day(26_559.7e3);
    println!(
        "GPS: +{:.2} µs (grav) {:.2} µs (vel) net {:.2} µs/day",
        g * 1e6,
        v * 1e6,
        n * 1e6
    );
}
