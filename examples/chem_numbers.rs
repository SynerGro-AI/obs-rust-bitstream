use obs_rust_bitstream::chem::*;
fn main() {
    let e = parse_equation("CH4 + O2 -> CO2 + H2O").unwrap();
    println!("{}  Tad(air) = {:.0} K", e.display(), adiabatic_temperature(&e, T0, true).unwrap());
    let h2 = parse_equation("H2 + O2 -> H2O").unwrap();
    println!("{}  Tad(air) = {:.0} K", h2.display(), adiabatic_temperature(&h2, T0, true).unwrap());
    let th = parse_equation("Fe2O3 + Al -> Al2O3 + Fe").unwrap();
    println!("{}  Tad = {:.0} K (no melting modelled)", th.display(), adiabatic_temperature(&th, T0, false).unwrap());
    let mut r = Reactor::new(&e, 1500.0, 101325.0, 1.0, true).unwrap();
    r.advance(0.05);
    println!("CH4/air from 1500 K, 50 ms: conversion {:.4}, T {:.0} K", r.conversion(), r.t);
    for t in [1000.0, 1200.0, 1400.0] {
        let mut r = Reactor::new(&e, t, 101325.0, 1.0, true).unwrap();
        r.advance(1.0);
        println!("  start {:.0} K, 1 s: conversion {:.4}, T {:.0}", t, r.conversion(), r.t);
    }
}
