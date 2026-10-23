//! Relativity: special-relativistic kinematics, Newtonian N-body gravity
//! with a weak-field (1PN) general-relativistic correction, and the
//! weak-field GR formulas behind GPS and light bending.
//!
//! **Exact:** the special-relativity formulas (Lorentz factor, time
//! dilation, length contraction, velocity addition, energy and momentum,
//! longitudinal Doppler) are exact. So is Newtonian gravity, up to the
//! integrator's error (leapfrog is symplectic and conserves energy to
//! O(dt²) with no secular drift).
//!
//! **Approximate (weak field, to first post-Newtonian order):**
//! * The GR orbit correction is the Schwarzschild test-particle 1PN
//!   acceleration (harmonic gauge),
//!   `a = GM/(c²r³)·[(4GM/r − v²)·r + 4(r·v)·v]`, applied relative to one
//!   central body. It reproduces the perihelion advance
//!   `Δφ = 6πGM/(c²a(1−e²))` per orbit. It ignores the other bodies'
//!   mutual 1PN terms, frame dragging and radiation reaction.
//! * Clock rates use `dτ/dt ≈ 1 − GM/(rc²) − v²/(2c²)`, light deflection
//!   is `4GM/(c²b)`, and the Shapiro delay is not modelled.
//!
//! Constants: c = 299 792 458 m/s (exact), G = 6.674 30e-11 (CODATA 2018),
//! GM☉ = 1.327 124 400 18e20 m³/s² and GM⊕ = 3.986 004 418e14 m³/s²
//! (IERS/JPL), AU = 1.495 978 707e11 m (exact, IAU 2012).

pub const C: f64 = 299_792_458.0;
pub const G: f64 = 6.674_30e-11;
pub const GM_SUN: f64 = 1.327_124_400_18e20;
pub const GM_EARTH: f64 = 3.986_004_418e14;
pub const AU: f64 = 1.495_978_707e11;
pub const DAY: f64 = 86_400.0;
pub const JULIAN_YEAR: f64 = 365.25 * DAY;
pub const R_SUN: f64 = 6.957e8;
pub const R_EARTH: f64 = 6.371e6;
pub const ARCSEC: f64 = std::f64::consts::PI / (180.0 * 3600.0);

// ---- special relativity ---------------------------------------------

/// Lorentz factor γ = 1/√(1 − β²) for β = v/c (|β| < 1).
pub fn gamma(beta: f64) -> f64 {
    1.0 / (1.0 - beta * beta).sqrt()
}
/// Proper time elapsed on a clock moving at β while `dt` passes in the lab.
pub fn proper_time(dt: f64, beta: f64) -> f64 {
    dt / gamma(beta)
}
/// Length of a rod of rest length `l0` moving at β along its length.
pub fn contracted_length(l0: f64, beta: f64) -> f64 {
    l0 / gamma(beta)
}
/// Collinear velocity addition (in units of c).
pub fn add_velocities(u: f64, v: f64) -> f64 {
    (u + v) / (1.0 + u * v)
}
/// Rapidity φ = artanh β (adds linearly).
pub fn rapidity(beta: f64) -> f64 {
    beta.atanh()
}
/// Total energy γmc², kinetic (γ−1)mc², momentum γmv.
pub fn energy(m: f64, beta: f64) -> (f64, f64, f64) {
    let g = gamma(beta);
    (g * m * C * C, (g - 1.0) * m * C * C, g * m * beta * C)
}
/// Received/emitted frequency for a source receding at β (negative = approaching).
pub fn doppler(beta: f64) -> f64 {
    ((1.0 - beta) / (1.0 + beta)).sqrt()
}

/// Everything shown for one speed.
#[derive(Clone, Debug, PartialEq)]
pub struct SrReadout {
    pub beta: f64,
    pub gamma: f64,
    pub tau_per_lab_second: f64,
    pub length_of_1m: f64,
    pub kinetic_per_kg: f64,
    pub newton_kinetic_per_kg: f64,
    pub doppler_approach: f64,
    pub doppler_recede: f64,
    pub rapidity: f64,
}

pub fn sr_readout(beta: f64) -> SrReadout {
    let b = beta.clamp(0.0, 0.999_999_999_999);
    SrReadout {
        beta: b,
        gamma: gamma(b),
        tau_per_lab_second: proper_time(1.0, b),
        length_of_1m: contracted_length(1.0, b),
        kinetic_per_kg: energy(1.0, b).1,
        newton_kinetic_per_kg: 0.5 * (b * C).powi(2),
        doppler_approach: doppler(-b),
        doppler_recede: doppler(b),
        rapidity: rapidity(b),
    }
}

// ---- weak-field GR formulas ------------------------------------------

pub fn schwarzschild_radius(gm: f64) -> f64 {
    2.0 * gm / (C * C)
}
/// dτ/dt for a static clock at radius r (exact Schwarzschild).
pub fn gravitational_rate(gm: f64, r: f64) -> f64 {
    (1.0 - schwarzschild_radius(gm) / r).sqrt()
}
/// Light deflection angle (rad) for impact parameter b, weak field.
pub fn light_deflection(gm: f64, b: f64) -> f64 {
    4.0 * gm / (C * C * b)
}
/// Perihelion advance per orbit (rad) for semi-major axis a, eccentricity e.
pub fn perihelion_advance(gm: f64, a: f64, e: f64) -> f64 {
    6.0 * std::f64::consts::PI * gm / (C * C * a * (1.0 - e * e))
}

/// GPS clock budget per day (seconds): (gravitational gain, velocity loss, net)
/// for a satellite on a circular orbit of radius `r` vs a clock at rest on
/// the ground (Earth's rotation neglected).
pub fn gps_drift_per_day(r: f64) -> (f64, f64, f64) {
    let grav = GM_EARTH / (C * C) * (1.0 / R_EARTH - 1.0 / r) * DAY;
    let v2 = GM_EARTH / r;
    let vel = -v2 / (2.0 * C * C) * DAY;
    (grav, vel, grav + vel)
}

// ---- N-body ------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Mass {
    pub name: String,
    /// GM (m³/s²) — gravitational parameters are known far better than G or M
    pub gm: f64,
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    /// recent positions for drawing
    pub trail: Vec<[f64; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NBody {
    pub bodies: Vec<Mass>,
    pub time: f64,
    pub dt: f64,
    /// 1PN correction relative to `central`
    pub gr: bool,
    pub central: usize,
    pub c: f64,
    pub softening: f64,
    pub name: String,
    pub trail_every: usize,
    steps: usize,
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

impl NBody {
    pub fn new(name: &str, dt: f64) -> Self {
        Self {
            bodies: Vec::new(),
            time: 0.0,
            dt,
            gr: false,
            central: 0,
            c: C,
            softening: 0.0,
            name: name.into(),
            trail_every: 20,
            steps: 0,
        }
    }

    pub fn add(&mut self, name: &str, gm: f64, pos: [f64; 3], vel: [f64; 3]) {
        self.bodies.push(Mass {
            name: name.into(),
            gm,
            pos,
            vel,
            trail: Vec::new(),
        });
    }

    /// Move to the centre-of-mass frame.
    pub fn to_com(&mut self) {
        let m: f64 = self.bodies.iter().map(|b| b.gm).sum();
        for k in 0..3 {
            let p: f64 = self.bodies.iter().map(|b| b.gm * b.pos[k]).sum::<f64>() / m;
            let v: f64 = self.bodies.iter().map(|b| b.gm * b.vel[k]).sum::<f64>() / m;
            for b in &mut self.bodies {
                b.pos[k] -= p;
                b.vel[k] -= v;
            }
        }
    }

    fn accel(&self, pos: &[[f64; 3]], vel: &[[f64; 3]]) -> Vec<[f64; 3]> {
        let n = pos.len();
        let mut a = vec![[0.0; 3]; n];
        let eps2 = self.softening * self.softening;
        for i in 0..n {
            for j in i + 1..n {
                let d = sub(pos[j], pos[i]);
                let r2 = dot(d, d) + eps2;
                let inv = 1.0 / (r2 * r2.sqrt());
                for k in 0..3 {
                    a[i][k] += self.bodies[j].gm * d[k] * inv;
                    a[j][k] -= self.bodies[i].gm * d[k] * inv;
                }
            }
        }
        if self.gr {
            let c2 = self.c * self.c;
            let gm = self.bodies[self.central].gm;
            for i in (0..n).filter(|&i| i != self.central) {
                let r = sub(pos[i], pos[self.central]);
                let v = sub(vel[i], vel[self.central]);
                let rn = norm(r);
                let f = gm / (c2 * rn * rn * rn);
                let s = 4.0 * gm / rn - dot(v, v);
                let rv = 4.0 * dot(r, v);
                for k in 0..3 {
                    a[i][k] += f * (s * r[k] + rv * v[k]);
                }
            }
        }
        a
    }

    /// One step: leapfrog (kick-drift-kick, symplectic) for pure Newton;
    /// classical RK4 when the velocity-dependent GR term is on.
    pub fn step(&mut self) {
        let h = self.dt;
        let pos: Vec<[f64; 3]> = self.bodies.iter().map(|b| b.pos).collect();
        let vel: Vec<[f64; 3]> = self.bodies.iter().map(|b| b.vel).collect();
        let n = pos.len();
        if !self.gr {
            let a0 = self.accel(&pos, &vel);
            let mut vh = vel.clone();
            let mut p1 = pos.clone();
            for i in 0..n {
                for k in 0..3 {
                    vh[i][k] += 0.5 * h * a0[i][k];
                    p1[i][k] += h * vh[i][k];
                }
            }
            let a1 = self.accel(&p1, &vh);
            for (i, b) in self.bodies.iter_mut().enumerate() {
                for k in 0..3 {
                    b.vel[k] = vh[i][k] + 0.5 * h * a1[i][k];
                }
                b.pos = p1[i];
            }
        } else {
            let add = |x: &Vec<[f64; 3]>, d: &Vec<[f64; 3]>, s: f64| -> Vec<[f64; 3]> {
                x.iter()
                    .zip(d)
                    .map(|(a, b)| [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]])
                    .collect()
            };
            let k1v = self.accel(&pos, &vel);
            let k1x = vel.clone();
            let k2v = self.accel(&add(&pos, &k1x, h / 2.0), &add(&vel, &k1v, h / 2.0));
            let k2x = add(&vel, &k1v, h / 2.0);
            let k3v = self.accel(&add(&pos, &k2x, h / 2.0), &add(&vel, &k2v, h / 2.0));
            let k3x = add(&vel, &k2v, h / 2.0);
            let k4v = self.accel(&add(&pos, &k3x, h), &add(&vel, &k3v, h));
            let k4x = add(&vel, &k3v, h);
            for (i, b) in self.bodies.iter_mut().enumerate() {
                for k in 0..3 {
                    b.pos[k] +=
                        h / 6.0 * (k1x[i][k] + 2.0 * k2x[i][k] + 2.0 * k3x[i][k] + k4x[i][k]);
                    b.vel[k] +=
                        h / 6.0 * (k1v[i][k] + 2.0 * k2v[i][k] + 2.0 * k3v[i][k] + k4v[i][k]);
                }
            }
        }
        self.time += h;
        self.steps += 1;
        if self.trail_every > 0 && self.steps % self.trail_every == 0 {
            for b in &mut self.bodies {
                b.trail.push(b.pos);
                if b.trail.len() > TRAIL_MAX {
                    b.trail.drain(..TRAIL_MAX / 4);
                }
            }
        }
    }

    pub fn advance(&mut self, secs: f64) {
        let n = (secs / self.dt).round().max(0.0) as usize;
        for _ in 0..n {
            self.step();
        }
    }

    /// Newtonian energy per unit G (Σ ½ m v² − Σ G m_i m_j / r) expressed with GM: E·G.
    pub fn energy_g(&self) -> f64 {
        let mut e = 0.0;
        for (i, b) in self.bodies.iter().enumerate() {
            e += 0.5 * b.gm * dot(b.vel, b.vel);
            for c in &self.bodies[i + 1..] {
                e -= b.gm * c.gm / norm(sub(c.pos, b.pos));
            }
        }
        e
    }

    pub fn angular_momentum_g(&self) -> [f64; 3] {
        let mut l = [0.0; 3];
        for b in &self.bodies {
            let x = cross(b.pos, b.vel);
            for k in 0..3 {
                l[k] += b.gm * x[k];
            }
        }
        l
    }

    /// Direction (rad, in the x-y plane) of body `i`'s perihelion about the
    /// central body, from the Laplace–Runge–Lenz vector.
    pub fn perihelion_angle(&self, i: usize) -> f64 {
        let c = &self.bodies[self.central];
        let b = &self.bodies[i];
        let r = sub(b.pos, c.pos);
        let v = sub(b.vel, c.vel);
        let mu = c.gm + b.gm;
        let l = cross(r, v);
        let vl = cross(v, l);
        let rn = norm(r);
        let a = [
            vl[0] - mu * r[0] / rn,
            vl[1] - mu * r[1] / rn,
            vl[2] - mu * r[2] / rn,
        ];
        a[1].atan2(a[0])
    }

    /// Clock rate dτ/dt of body `i` in the field of the central body (1PN).
    pub fn clock_rate(&self, i: usize) -> f64 {
        let c = &self.bodies[self.central];
        let b = &self.bodies[i];
        let r = norm(sub(b.pos, c.pos)).max(1.0);
        let v2 = dot(b.vel, b.vel);
        1.0 - c.gm / (r * self.c * self.c) - v2 / (2.0 * self.c * self.c)
    }
}

/// A body on an orbit of semi-major axis `a` and eccentricity `e` about a
/// central GM, started at perihelion at angle `phi` in the x-y plane.
fn at_perihelion(gm: f64, a: f64, e: f64, phi: f64) -> ([f64; 3], [f64; 3]) {
    let rp = a * (1.0 - e);
    let vp = (gm * (1.0 + e) / rp).sqrt();
    (
        [rp * phi.cos(), rp * phi.sin(), 0.0],
        [-vp * phi.sin(), vp * phi.cos(), 0.0],
    )
}

/// Points kept per trail (the oldest quarter is dropped when full).
pub const TRAIL_MAX: usize = 4000;

pub const PRESETS: [&str; 5] = ["mercury", "inner", "earth_moon", "figure8", "binary"];

/// Named scenarios. Orbits start at perihelion from mean orbital elements
/// (JPL approximate elements, J2000); they are not ephemeris positions.
pub fn preset(name: &str) -> Option<NBody> {
    let mut nb;
    match name {
        "mercury" => {
            nb = NBody::new("Sun + Mercury", 0.002 * 87.969 * DAY / 10.0);
            nb.add("Sun", GM_SUN, [0.0; 3], [0.0; 3]);
            let (p, v) = at_perihelion(GM_SUN, 0.387_098 * AU, 0.205_630, 0.0);
            nb.add("Mercury", 2.203_2e13, p, v);
            nb.trail_every = 25; // ~200 points per orbit: a 20-orbit rosette
        }
        "inner" => {
            nb = NBody::new("Inner solar system", 0.05 * DAY);
            nb.add("Sun", GM_SUN, [0.0; 3], [0.0; 3]);
            for (n, gm, a, e, phi) in [
                ("Mercury", 2.2032e13, 0.387_098, 0.205_630, 0.5),
                ("Venus", 3.248_59e14, 0.723_332, 0.006_772, 2.0),
                ("Earth", 3.986_004e14 + 4.9028e12, 1.000_001, 0.016_709, 3.5),
                ("Mars", 4.282_837e13, 1.523_679, 0.093_4, 5.0),
            ] {
                let (p, v) = at_perihelion(GM_SUN, a * AU, e, phi);
                nb.add(n, gm, p, v);
            }
        }
        "earth_moon" => {
            nb = NBody::new("Earth + Moon", 60.0);
            nb.add("Earth", GM_EARTH, [0.0; 3], [0.0; 3]);
            let (p, v) = at_perihelion(GM_EARTH + 4.9028e12, 3.844e8, 0.0549, 0.0);
            nb.add("Moon", 4.902_8e12, p, v);
        }
        "figure8" => {
            // Chenciner & Montgomery (2000) choreography, G = m = 1 units
            nb = NBody::new("Figure-eight three-body orbit (G = m = 1)", 1e-3);
            let (x, y) = (0.970_004_36, -0.243_087_53);
            let (vx, vy) = (-0.932_407_37, -0.864_731_46);
            nb.add("A", 1.0, [x, y, 0.0], [-vx / 2.0, -vy / 2.0, 0.0]);
            nb.add("B", 1.0, [-x, -y, 0.0], [-vx / 2.0, -vy / 2.0, 0.0]);
            nb.add("C", 1.0, [0.0; 3], [vx, vy, 0.0]);
            nb.c = 1e9; // no GR in these units
            nb.trail_every = 5;
        }
        "binary" => {
            nb = NBody::new("Equal-mass binary stars, 1 AU apart", 0.05 * DAY);
            let v = (GM_SUN / (4.0 * 0.5 * AU)).sqrt();
            nb.add("Star A", GM_SUN, [0.5 * AU, 0.0, 0.0], [0.0, v, 0.0]);
            nb.add("Star B", GM_SUN, [-0.5 * AU, 0.0, 0.0], [0.0, -v, 0.0]);
        }
        _ => return None,
    }
    nb.to_com();
    Some(nb)
}

/// Run Mercury for `orbits` orbits with and without the GR term and return
/// (measured GR advance per orbit, predicted), radians.
pub fn mercury_precession(orbits: usize, steps_per_orbit: usize) -> (f64, f64) {
    let a = 0.387_098 * AU;
    let e = 0.205_630;
    let period = 2.0 * std::f64::consts::PI * (a * a * a / GM_SUN).sqrt();
    let run = |gr: bool| {
        let mut nb = preset("mercury").unwrap();
        nb.gr = true; // same integrator (RK4) for both runs
        if !gr {
            nb.c = 1e30;
        }
        nb.dt = period / steps_per_orbit as f64;
        nb.trail_every = 0;
        let p0 = nb.perihelion_angle(1);
        for _ in 0..orbits * steps_per_orbit {
            nb.step();
        }
        let mut d = nb.perihelion_angle(1) - p0;
        while d > std::f64::consts::PI {
            d -= 2.0 * std::f64::consts::PI;
        }
        while d < -std::f64::consts::PI {
            d += 2.0 * std::f64::consts::PI;
        }
        d
    };
    let measured = (run(true) - run(false)) / orbits as f64;
    (measured, perihelion_advance(GM_SUN, a, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muons_reach_the_ground_because_their_clocks_run_slow() {
        // muon τ = 2.196 98 µs (PDG); at β = 0.995, γ ≈ 10.01
        let tau = 2.196_98e-6;
        let b = 0.995;
        assert!((gamma(b) - 10.0125).abs() < 1e-3);
        let lab_life = tau * gamma(b);
        // τγβc ≈ 6.56 km of flight instead of 655 m without dilation
        assert!(
            (lab_life * b * C - 6.5617e3).abs() < 1.0,
            "{} m",
            lab_life * b * C
        );
        assert!((proper_time(lab_life, b) - tau).abs() < 1e-15);
        // velocities never add past c, and rapidities add linearly
        let w = add_velocities(0.9, 0.9);
        assert!((w - 0.994_475).abs() < 1e-6 && w < 1.0);
        assert!((rapidity(w) - 2.0 * rapidity(0.9)).abs() < 1e-12);
        // E² = (pc)² + (mc²)²
        let (e, k, p) = energy(1.0, 0.6);
        assert!((e * e - (p * C).powi(2) - C.powi(4)).abs() / (e * e) < 1e-12);
        assert!((k - 0.25 * C * C).abs() / k < 1e-12, "γ(0.6) = 1.25");
        // low speed: kinetic energy → ½mv²
        let r = sr_readout(1e-4);
        assert!((r.kinetic_per_kg / r.newton_kinetic_per_kg - 1.0).abs() < 1e-7);
        assert!((doppler(0.6) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn gps_clocks_gain_38_microseconds_a_day() {
        let (g, v, net) = gps_drift_per_day(26_559.7e3);
        assert!((g * 1e6 - 45.7).abs() < 0.2, "{}", g * 1e6);
        assert!((v * 1e6 + 7.2).abs() < 0.1, "{}", v * 1e6);
        assert!((net * 1e6 - 38.5).abs() < 0.3, "{}", net * 1e6);
    }

    #[test]
    fn sunlight_bends_1_75_arcseconds_and_mercury_precesses_43_per_century() {
        let d = light_deflection(GM_SUN, R_SUN) / ARCSEC;
        assert!((d - 1.751).abs() < 0.005, "{}", d);
        let per_orbit = perihelion_advance(GM_SUN, 0.387_098 * AU, 0.205_630);
        let orbits_per_century = 100.0 * JULIAN_YEAR / (87.969 * DAY);
        assert!(
            (per_orbit * orbits_per_century / ARCSEC - 42.98).abs() < 0.1,
            "{}",
            per_orbit * orbits_per_century / ARCSEC
        );
        assert!((schwarzschild_radius(GM_SUN) - 2953.25).abs() < 0.5);
    }

    #[test]
    fn simulated_mercury_orbit_precesses_by_the_gr_amount() {
        let (measured, predicted) = mercury_precession(5, 20_000);
        assert!(
            (measured / predicted - 1.0).abs() < 0.02,
            "{} vs {}",
            measured,
            predicted
        );
    }

    #[test]
    fn leapfrog_conserves_energy_and_angular_momentum() {
        let mut nb = preset("inner").unwrap();
        let e0 = nb.energy_g();
        let l0 = nb.angular_momentum_g()[2];
        nb.advance(2.0 * JULIAN_YEAR);
        assert!(
            ((nb.energy_g() - e0) / e0).abs() < 1e-6,
            "{}",
            (nb.energy_g() - e0) / e0
        );
        assert!(((nb.angular_momentum_g()[2] - l0) / l0).abs() < 1e-10);
        // Earth goes round once a year
        let earth = nb.bodies.iter().position(|b| b.name == "Earth").unwrap();
        let mut nb = preset("inner").unwrap();
        let a0 = nb.bodies[earth].pos[1].atan2(nb.bodies[earth].pos[0]);
        nb.advance(365.256 * DAY);
        let a1 = nb.bodies[earth].pos[1].atan2(nb.bodies[earth].pos[0]);
        let mut d = (a1 - a0).abs();
        if d > std::f64::consts::PI {
            d = 2.0 * std::f64::consts::PI - d;
        }
        assert!(d < 0.02, "{}", d);
    }

    #[test]
    fn figure_eight_returns_after_one_period() {
        let mut nb = preset("figure8").unwrap();
        let start: Vec<[f64; 3]> = nb.bodies.iter().map(|b| b.pos).collect();
        let e0 = nb.energy_g();
        nb.dt = 1e-4;
        nb.advance(6.325_913_98);
        for (b, s) in nb.bodies.iter().zip(&start) {
            assert!(
                norm(sub(b.pos, *s)) < 1e-3,
                "{} off by {}",
                b.name,
                norm(sub(b.pos, *s))
            );
        }
        assert!(((nb.energy_g() - e0) / e0).abs() < 1e-8);
    }
}
