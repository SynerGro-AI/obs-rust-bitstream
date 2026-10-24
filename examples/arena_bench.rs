//! Step cost of the physics arena, all-pairs vs sweep-and-prune contacts.
//! `cargo run --release --example arena_bench`
use obs_rust_bitstream::arena::{stress_pile, Collide, Environment};
use std::time::Instant;

fn main() {
    for n in [50, 200, 500, 1000, 2000] {
        let mut row = format!("{n:>5} bodies:");
        let mut states = Vec::new();
        for mode in [Collide::Brute, Collide::Sweep] {
            let mut a = stress_pile(Environment::earth(), n, 11);
            let steps = 240;
            let t = Instant::now();
            for _ in 0..steps {
                a.step_with(mode);
            }
            let ms = t.elapsed().as_secs_f64() * 1e3 / steps as f64;
            row += &format!(
                "  {mode:?} {ms:8.3} ms/step ({:6.1} ms per 60 fps frame)",
                ms * 4.0
            );
            states.push(format!("{:?}", a));
        }
        println!("{row}  identical: {}", states[0] == states[1]);
    }
}
