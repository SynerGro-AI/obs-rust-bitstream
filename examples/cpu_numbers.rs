use obs_rust_bitstream::computer::*;
fn main() {
    let f = max_clock("fibonacci", 300.0).unwrap();
    println!("fmax(fibonacci, 300 K) = {:.2} GHz", f / 1e9);
    let r = run_program("fibonacci", 300.0, 0.9 * f).unwrap();
    println!(
        "300 K @ 0.9 fmax: out {:?} settle {:.0} ps power dyn {:.3e} W leak {:.3e} W",
        r.out, r.worst_settle_ps, r.power_w.0, r.power_w.1
    );
    let runs = heat_to_failure("fibonacci", 0.9 * f, 300.0, 10.0, 900.0).unwrap();
    let l = runs.last().unwrap();
    println!(
        "heated @0.9 fmax: fails at {:.0} K: {:?} out {:?}",
        l.t, l.failure, l.out
    );
    let slow = heat_to_failure("multiply", 1e7, 300.0, 25.0, 1700.0).unwrap();
    let l = slow.last().unwrap();
    println!("10 MHz: fails at {:.0} K: {:?}", l.t, l.failure);
    for t in [300.0, 400.0, 500.0, 600.0] {
        let lib = Library::at(t);
        let n = lib.get(GateKind::Nand2);
        println!(
            "{:.0} K NAND2 tpd {:?} ps leak {:.2e} A NMH {:.2} NML {:.2}",
            t,
            n.t_pd().map(|d| (d * 1e12 * 10.0).round() / 10.0),
            n.leakage,
            n.nm_high(),
            n.nm_low()
        );
    }
}
