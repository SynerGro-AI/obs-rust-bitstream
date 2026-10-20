//! Physics Arena: rigid bodies under real environmental conditions.
//!
//! What is simulated (SI units throughout):
//! * **Motion**: fixed-timestep (1/240 s) semi-implicit Euler. Gravity,
//!   quadratic aerodynamic drag `F = −½ ρ C_d A |v−w| (v−w)` with wind `w`,
//!   and Archimedes buoyancy `ρ_air V g`. Translational only: bodies do not
//!   rotate (no angular momentum or torque).
//! * **Contacts**: ground plane y = 0 and axis-aligned box–box contacts,
//!   resolved with impulses: restitution `e = √(e₁e₂)` along the normal and
//!   Coulomb friction `|Δv_t| ≤ μ·|Δv_n|`, `μ = √(μ₁μ₂)`, kinetic coefficient.
//! * **Mass** = density × mesh volume, from the body's material.
//! * **Heat**: a heater term, Newtonian convection `h·A·(T_air − T)` with
//!   `h = 10 W/(m²K) × ρ_air/1.225` (natural convection in still air, scaled
//!   with air density; 0 in vacuum), and Stefan–Boltzmann radiation
//!   `εσA(T_env⁴ − T⁴)`. Temperature is uniform inside a body (lumped).
//! * **Phase changes**: at the melting / boiling point (boiling point moved
//!   with pressure by Clausius–Clapeyron, see `material_env`), heat goes into
//!   latent heat `m·L` before the temperature moves on. Liquids slump into a
//!   puddle (same volume); vaporised bodies leave the simulation.
//!
//! Constants: g₀ = 9.80665 m/s² (standard gravity, exact), σ = 5.670374419e-8
//! W/(m²K⁴) (CODATA 2018). Moon 1.62 m/s², Mars 3.721 m/s², Mars surface
//! pressure ≈ 610 Pa and air density ≈ 0.020 kg/m³ (NASA fact sheets).

use crate::material_env::{EnvMeta, Phase};

pub const G0: f32 = 9.80665;
pub const SIGMA: f64 = 5.670_374_419e-8;
pub const DT: f32 = 1.0 / 240.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    pub name: String,
    /// m/s² (downward).
    pub gravity: f32,
    /// kg/m³
    pub air_density: f32,
    /// Pa
    pub pressure: f64,
    /// K
    pub temperature: f32,
    /// m/s
    pub wind: [f32; 3],
}

impl Environment {
    pub fn earth() -> Self {
        // ISA sea level: 15 °C, 101 325 Pa, 1.225 kg/m³
        Self { name: "Earth".into(), gravity: G0, air_density: 1.225, pressure: 101_325.0, temperature: 288.15, wind: [0.0; 3] }
    }
    pub fn moon() -> Self {
        Self { name: "Moon".into(), gravity: 1.62, air_density: 0.0, pressure: 0.0, temperature: 250.0, wind: [0.0; 3] }
    }
    pub fn mars() -> Self {
        Self { name: "Mars".into(), gravity: 3.721, air_density: 0.020, pressure: 610.0, temperature: 210.0, wind: [0.0; 3] }
    }
    /// Earth gravity with the air pumped out (the hammer-and-feather demo).
    pub fn vacuum() -> Self {
        Self { name: "Vacuum chamber".into(), gravity: G0, air_density: 0.0, pressure: 0.0, temperature: 293.15, wind: [0.0; 3] }
    }
    /// Deep space: no gravity, no air, 2.725 K background.
    pub fn space() -> Self {
        Self { name: "Deep space".into(), gravity: 0.0, air_density: 0.0, pressure: 0.0, temperature: 2.725, wind: [0.0; 3] }
    }

    pub fn preset(name: &str) -> Option<Self> {
        Some(match name.trim().to_lowercase().as_str() {
            "earth" => Self::earth(),
            "moon" | "luna" => Self::moon(),
            "mars" => Self::mars(),
            "vacuum" | "vacuum chamber" => Self::vacuum(),
            "space" | "deep space" | "zero-g" | "zero_g" => Self::space(),
            "custom" => Self { name: "Custom".into(), ..Self::earth() },
            _ => return None,
        })
    }

    pub const PRESETS: [&'static str; 5] = ["earth", "moon", "mars", "vacuum", "space"];

    /// Natural-convection coefficient, W/(m²·K).
    pub fn convection_h(&self) -> f32 {
        10.0 * self.air_density / 1.225
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub name: String,
    pub material: String,
    pub env: EnvMeta,
    /// Box half extents (m) as built (solid).
    pub half0: [f32; 3],
    /// Current half extents (a liquid slumps).
    pub half: [f32; 3],
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// m³
    pub volume: f32,
    /// kg
    pub mass: f32,
    pub temp: f32,
    pub phase: Phase,
    /// J banked toward the next phase change (+ melting/boiling, − freezing/condensing).
    pub latent: f32,
    /// W
    pub heater: f32,
    pub fixed: bool,
    pub drag_cd: f32,
    pub vaporised: bool,
}

impl Body {
    /// A body from a material and a box; `volume` from the mesh (or the box when `None`).
    pub fn new(name: &str, material: &str, env: EnvMeta, half: [f32; 3], pos: [f32; 3], volume: Option<f32>, temp: f32, pressure: f64) -> Self {
        let v = volume.unwrap_or(8.0 * half[0] * half[1] * half[2]).max(1e-9);
        let phase = env.phase_at(temp, pressure);
        let rho = env.density_in(phase, temp, pressure).unwrap_or(1000.0);
        Self {
            name: name.to_string(),
            material: material.to_string(),
            env,
            half0: half,
            half,
            pos,
            vel: [0.0; 3],
            volume: v,
            mass: rho * v,
            temp,
            phase,
            latent: 0.0,
            heater: 0.0,
            fixed: false,
            drag_cd: 1.05, // cube face-on
            vaporised: false,
        }
    }

    pub fn area_frontal(&self, dir: [f32; 3]) -> f32 {
        let h = self.half;
        let d = norm(dir);
        if d < 1e-9 {
            return 4.0 * h[0] * h[2];
        }
        // projected area of a box along dir
        4.0 * (h[1] * h[2] * dir[0].abs() + h[0] * h[2] * dir[1].abs() + h[0] * h[1] * dir[2].abs()) / d
    }

    pub fn surface_area(&self) -> f32 {
        let h = self.half;
        8.0 * (h[0] * h[1] + h[1] * h[2] + h[0] * h[2])
    }

    pub fn kinetic(&self) -> f64 {
        0.5 * self.mass as f64 * dot(self.vel, self.vel) as f64
    }

    pub fn specific_heat(&self) -> f32 {
        self.env.specific_heat.unwrap_or(1000.0)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Events {
    pub log: Vec<(f64, String)>,
}

#[derive(Clone, Debug)]
pub struct Arena {
    pub env: Environment,
    pub bodies: Vec<Body>,
    pub time: f64,
    pub ground: bool,
    /// Heat exchange with the environment (convection + radiation).
    pub exchange: bool,
    pub events: Events,
    acc: f32,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

impl Arena {
    pub fn new(env: Environment) -> Self {
        Self { env, bodies: Vec::new(), time: 0.0, ground: true, exchange: true, events: Events::default(), acc: 0.0 }
    }

    pub fn add(&mut self, b: Body) -> usize {
        self.bodies.push(b);
        self.bodies.len() - 1
    }

    pub fn body(&self, name: &str) -> Option<&Body> {
        self.bodies.iter().find(|b| b.name == name)
    }

    pub fn body_mut(&mut self, name: &str) -> Option<&mut Body> {
        self.bodies.iter_mut().find(|b| b.name == name)
    }

    /// Advance by real time `secs` using fixed steps; returns the steps taken.
    pub fn advance(&mut self, secs: f32) -> usize {
        self.acc += secs.max(0.0);
        let mut n = 0;
        while self.acc >= DT && n < 2400 {
            self.step();
            self.acc -= DT;
            n += 1;
        }
        n
    }

    pub fn step(&mut self) {
        let dt = DT;
        let g = self.env.gravity;
        let rho_air = self.env.air_density;
        let wind = self.env.wind;
        for b in self.bodies.iter_mut().filter(|b| !b.fixed && !b.vaporised) {
            let m = b.mass.max(1e-9);
            let mut f = [0.0, -m * g, 0.0];
            // buoyancy
            f[1] += rho_air * b.volume * g;
            // drag on the velocity relative to the air
            let rel = [b.vel[0] - wind[0], b.vel[1] - wind[1], b.vel[2] - wind[2]];
            let sp = norm(rel);
            if sp > 1e-6 && rho_air > 0.0 {
                let a = b.area_frontal(rel);
                let k = 0.5 * rho_air * b.drag_cd * a * sp;
                for i in 0..3 {
                    f[i] -= k * rel[i];
                }
            }
            for i in 0..3 {
                b.vel[i] += f[i] / m * dt;
                b.pos[i] += b.vel[i] * dt;
            }
        }
        self.contacts();
        self.thermal(dt);
        self.time += dt as f64;
    }

    fn contacts(&mut self) {
        let g = self.env.gravity;
        // ground
        if self.ground {
            for b in self.bodies.iter_mut().filter(|b| !b.fixed && !b.vaporised) {
                let bottom = b.pos[1] - b.half[1];
                if bottom < 0.0 {
                    b.pos[1] -= bottom;
                    if b.vel[1] < 0.0 {
                        let vn = -b.vel[1];
                        // liquids don't bounce
                        let e = if b.phase == Phase::Solid { b.env.restitution } else { 0.0 };
                        // rest threshold: below the speed gained in one step, stop bouncing
                        let e = if vn < 2.0 * g * DT + 1e-3 { 0.0 } else { e };
                        let jn = (1.0 + e) * vn;
                        b.vel[1] = e * vn;
                        friction(&mut b.vel, jn, b.env.friction_kinetic, b.env.friction_static, g);
                    } else if b.vel[1].abs() < 1e-6 {
                        // resting contact: friction from the normal force over one step
                        friction(&mut b.vel, g * DT, b.env.friction_kinetic, b.env.friction_static, g);
                    }
                }
            }
        }
        // box–box
        let n = self.bodies.len();
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = {
                    let (l, r) = self.bodies.split_at_mut(j);
                    (&mut l[i], &mut r[0])
                };
                if a.vaporised || b.vaporised || (a.fixed && b.fixed) {
                    continue;
                }
                let d = [b.pos[0] - a.pos[0], b.pos[1] - a.pos[1], b.pos[2] - a.pos[2]];
                let ov = [a.half[0] + b.half[0] - d[0].abs(), a.half[1] + b.half[1] - d[1].abs(), a.half[2] + b.half[2] - d[2].abs()];
                if ov[0] <= 0.0 || ov[1] <= 0.0 || ov[2] <= 0.0 {
                    continue;
                }
                // axis of least penetration
                let ax = if ov[0] < ov[1] && ov[0] < ov[2] { 0 } else if ov[1] < ov[2] { 1 } else { 2 };
                let mut nrm = [0.0; 3];
                nrm[ax] = if d[ax] >= 0.0 { 1.0 } else { -1.0 };
                let (ia, ib) = (if a.fixed { 0.0 } else { 1.0 / a.mass.max(1e-9) }, if b.fixed { 0.0 } else { 1.0 / b.mass.max(1e-9) });
                let it = ia + ib;
                if it <= 0.0 {
                    continue;
                }
                // positional correction split by inverse mass
                let pen = ov[ax];
                for k in 0..3 {
                    a.pos[k] -= nrm[k] * pen * ia / it;
                    b.pos[k] += nrm[k] * pen * ib / it;
                }
                let rv = [b.vel[0] - a.vel[0], b.vel[1] - a.vel[1], b.vel[2] - a.vel[2]];
                let vn = dot(rv, nrm);
                if vn >= 0.0 {
                    continue;
                }
                let solid = a.phase == Phase::Solid && b.phase == Phase::Solid;
                let e = if solid { (a.env.restitution * b.env.restitution).sqrt() } else { 0.0 };
                let e = if -vn < 2.0 * self.env.gravity * DT + 1e-3 { 0.0 } else { e };
                let jn = -(1.0 + e) * vn / it;
                for k in 0..3 {
                    a.vel[k] -= jn * nrm[k] * ia;
                    b.vel[k] += jn * nrm[k] * ib;
                }
                // friction on the tangential relative velocity
                let rv = [b.vel[0] - a.vel[0], b.vel[1] - a.vel[1], b.vel[2] - a.vel[2]];
                let vn2 = dot(rv, nrm);
                let vt = [rv[0] - vn2 * nrm[0], rv[1] - vn2 * nrm[1], rv[2] - vn2 * nrm[2]];
                let st = norm(vt);
                if st > 1e-9 {
                    let mu = (a.env.friction_kinetic * b.env.friction_kinetic).sqrt();
                    let jt = (st / it).min(mu * jn);
                    for k in 0..3 {
                        a.vel[k] += jt * vt[k] / st * ia;
                        b.vel[k] -= jt * vt[k] / st * ib;
                    }
                }
            }
        }
    }

    fn thermal(&mut self, dt: f32) {
        let env_t = self.env.temperature as f64;
        let h = self.env.convection_h();
        let p = self.env.pressure;
        let exchange = self.exchange;
        let time = self.time;
        for b in self.bodies.iter_mut().filter(|b| !b.vaporised) {
            let a = b.surface_area() as f64;
            let t = b.temp as f64;
            let mut q = b.heater as f64;
            if exchange {
                q += h as f64 * a * (env_t - t);
                q += b.env.emissivity as f64 * SIGMA * a * (env_t.powi(4) - t.powi(4));
            }
            let e = q * dt as f64; // J this step
            apply_heat(b, e as f32, p, time, &mut self.events);
        }
    }

    pub fn total_kinetic(&self) -> f64 {
        self.bodies.iter().filter(|b| !b.vaporised).map(|b| b.kinetic()).sum()
    }

    pub fn total_potential(&self) -> f64 {
        self.bodies.iter().filter(|b| !b.vaporised).map(|b| b.mass as f64 * self.env.gravity as f64 * (b.pos[1] - b.half[1]) as f64).sum()
    }
}

fn friction(v: &mut [f32; 3], jn: f32, mu_k: f32, _mu_s: f32, _g: f32) {
    let vt = (v[0] * v[0] + v[2] * v[2]).sqrt();
    if vt < 1e-9 {
        return;
    }
    let dv = (mu_k * jn).min(vt);
    v[0] -= dv * v[0] / vt;
    v[2] -= dv * v[2] / vt;
}

/// Put `e` joules into a body, honouring latent heat at phase boundaries.
pub fn apply_heat(b: &mut Body, mut e: f32, pressure: f64, time: f64, ev: &mut Events) {
    let cp = b.specific_heat();
    let m = b.mass.max(1e-9);
    let melt = if b.env.sublimes { None } else { b.env.melting_k };
    let boil = b.env.boiling_at(pressure);
    let mut guard = 0;
    while e.abs() > 1e-9 && guard < 8 {
        guard += 1;
        // the next boundary in the direction of heat flow
        let (bound, latent) = match (e > 0.0, b.phase) {
            (true, Phase::Solid) => match (melt, b.env.sublimes) {
                (Some(t), false) => (Some(t), b.env.latent_fusion),
                _ => (boil, b.env.latent_vaporization.or(b.env.latent_fusion)),
            },
            (true, Phase::Liquid) => (boil, b.env.latent_vaporization),
            (false, Phase::Liquid) => (melt, b.env.latent_fusion),
            (false, Phase::Gas) => (boil, b.env.latent_vaporization),
            _ => (None, None),
        };
        let Some(tb) = bound else {
            b.temp = (b.temp + e / (m * cp)).max(0.0);
            return;
        };
        let to_bound = (tb - b.temp) * m * cp; // J to reach the boundary (signed)
        if (e > 0.0 && e < to_bound) || (e < 0.0 && e > to_bound) {
            b.temp += e / (m * cp);
            return;
        }
        // reach the boundary, then bank latent heat
        e -= to_bound;
        b.temp = tb;
        let need = latent.unwrap_or(0.0) * m;
        b.latent += e;
        if b.latent.abs() < need {
            return;
        }
        // phase change complete
        let spill = b.latent - need * b.latent.signum();
        b.latent = 0.0;
        let from = b.phase;
        b.phase = match (from, spill >= 0.0) {
            (Phase::Solid, true) if b.env.sublimes || melt.is_none() => Phase::Gas,
            (Phase::Solid, true) => Phase::Liquid,
            (Phase::Liquid, true) => Phase::Gas,
            (Phase::Liquid, false) => Phase::Solid,
            (Phase::Gas, false) => Phase::Liquid,
            (p, _) => p,
        };
        on_phase(b, from, time, ev);
        e = spill;
        // nudge off the boundary so the next loop picks the next boundary
        if b.vaporised {
            return;
        }
    }
}

fn on_phase(b: &mut Body, from: Phase, time: f64, ev: &mut Events) {
    let to = b.phase;
    ev.log.push((time, format!("{} ({}) {} → {} at {:.1} K", b.name, b.material, from.label(), to.label(), b.temp)));
    match to {
        Phase::Liquid => {
            // slump into a puddle of the same volume: quarter height, double width/depth
            let h = b.half0;
            b.half = [h[0] * 2.0, h[1] * 0.25, h[2] * 2.0];
            b.pos[1] -= h[1] * 0.75;
        }
        Phase::Solid => {
            // a refrozen puddle stays puddle-shaped (it froze where it lay)
        }
        Phase::Gas => {
            b.vaporised = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material_pbr::{preset, preset_profile};

    fn env_of(key: &str) -> EnvMeta {
        preset_profile(&preset(key).unwrap()).env
    }

    fn cube(name: &str, mat: &str, y: f32, env: &Environment) -> Body {
        Body::new(name, mat, env_of(mat), [0.5, 0.5, 0.5], [0.0, y, 0.0], None, env.temperature, env.pressure)
    }

    #[test]
    fn mass_is_density_times_volume() {
        let e = Environment::earth();
        let g = cube("g", "gold", 1.0, &e);
        assert!((g.mass - 19_300.0).abs() < 1.0);
        let a = Body::new("a", "aluminum", env_of("aluminum"), [0.05, 0.05, 0.05], [0.0; 3], None, 293.0, e.pressure);
        assert!((a.mass - 2.7).abs() < 0.01, "10 cm Al cube = 2.7 kg: {}", a.mass);
        // water at 20 °C is liquid (997), ice below 0 °C solid (917)
        let w = Body::new("w", "water", env_of("water"), [0.5; 3], [0.0; 3], None, 293.0, e.pressure);
        let ice = Body::new("i", "water", env_of("water"), [0.5; 3], [0.0; 3], None, 263.0, e.pressure);
        assert_eq!(w.phase, Phase::Liquid);
        assert_eq!(ice.phase, Phase::Solid);
        assert!((w.mass - 997.0).abs() < 1.0 && (ice.mass - 917.0).abs() < 1.0);
    }

    #[test]
    fn free_fall_matches_kinematics_in_vacuum_and_on_the_moon() {
        for env in [Environment::vacuum(), Environment::moon()] {
            let mut a = Arena::new(env.clone());
            a.ground = false;
            a.add(cube("hammer", "iron", 100.0, &env));
            a.add(Body::new("feather", "graphite", env_of("graphite"), [0.1, 0.001, 0.02], [3.0, 100.0, 0.0], None, env.temperature, env.pressure));
            for _ in 0..480 {
                a.step();
            }
            let t = a.time as f32;
            let expect = 100.0 - 0.5 * env.gravity * t * t;
            for b in &a.bodies {
                assert!((b.pos[1] - expect).abs() < 0.02 * (100.0 - expect).max(1.0), "{} {}: {} vs {}", env.name, b.name, b.pos[1], expect);
            }
            // hammer and feather land together without air
            assert!((a.bodies[0].pos[1] - a.bodies[1].pos[1]).abs() < 1e-3);
        }
    }

    #[test]
    fn air_drag_reaches_terminal_velocity() {
        let env = Environment::earth();
        let mut a = Arena::new(env.clone());
        a.ground = false;
        // 10 cm aluminium cube
        let mut b = Body::new("al", "aluminum", env_of("aluminum"), [0.05; 3], [0.0, 5000.0, 0.0], None, env.temperature, env.pressure);
        b.drag_cd = 1.05;
        a.add(b);
        a.exchange = false;
        for _ in 0..240 * 60 {
            a.step();
        }
        let b = &a.bodies[0];
        let area = 0.01;
        // v_t = sqrt(2 (m g − ρ V g) / (ρ C_d A))
        let vt = (2.0 * (b.mass - 1.225 * b.volume) * G0 / (1.225 * 1.05 * area)).sqrt();
        assert!((-b.vel[1] - vt).abs() / vt < 0.01, "{} vs {}", -b.vel[1], vt);
        // feather vs hammer in air: the light, broad one is slower
        let mut a = Arena::new(env.clone());
        a.ground = false;
        a.add(cube("hammer", "iron", 100.0, &env));
        a.add(Body::new("feather", "graphite", env_of("graphite"), [0.1, 0.001, 0.02], [3.0, 100.0, 0.0], None, env.temperature, env.pressure));
        for _ in 0..480 {
            a.step();
        }
        assert!(a.bodies[1].pos[1] > a.bodies[0].pos[1] + 5.0);
    }

    #[test]
    fn wind_pushes_and_buoyancy_lifts() {
        let mut env = Environment::earth();
        env.wind = [10.0, 0.0, 0.0];
        let mut a = Arena::new(env.clone());
        a.ground = false;
        a.exchange = false;
        a.add(Body::new("sheet", "graphite", env_of("graphite"), [0.001, 0.1, 0.1], [0.0, 100.0, 0.0], None, env.temperature, env.pressure));
        for _ in 0..240 {
            a.step();
        }
        assert!(a.bodies[0].vel[0] > 1.0, "wind accelerates it downwind: {:?}", a.bodies[0].vel);
        // a body less dense than air rises (helium-like)
        let mut light = env_of("water");
        light.density = Some(0.18);
        light.density_liquid = Some(0.18);
        light.melting_k = None;
        light.boiling_k = None;
        let mut a = Arena::new(Environment::earth());
        a.ground = false;
        a.exchange = false;
        a.add(Body::new("balloon", "helium-ish", light, [0.5; 3], [0.0, 1.0, 0.0], None, 288.0, 101_325.0));
        for _ in 0..240 {
            a.step();
        }
        assert!(a.bodies[0].pos[1] > 1.5, "{}", a.bodies[0].pos[1]);
    }

    #[test]
    fn bounce_height_is_e_squared() {
        let env = Environment::vacuum();
        let mut a = Arena::new(env.clone());
        let mut b = cube("ball", "glass", 2.5, &env);
        b.env.restitution = 0.7;
        a.add(b);
        a.exchange = false;
        let mut peak_after = 0.0f32;
        let mut bounced = false;
        for _ in 0..240 * 3 {
            a.step();
            let b = &a.bodies[0];
            if !bounced && b.vel[1] > 0.0 {
                bounced = true;
            }
            if bounced {
                peak_after = peak_after.max(b.pos[1] - 0.5);
            }
            if bounced && b.vel[1] < 0.0 && peak_after > 0.0 {
                break;
            }
        }
        let expect = 2.0 * 0.49;
        assert!((peak_after - expect).abs() < 0.03, "{} vs {}", peak_after, expect);
    }

    #[test]
    fn friction_stops_a_sliding_block_in_v2_over_2_mu_g() {
        let env = Environment::vacuum();
        let mut a = Arena::new(env.clone());
        a.exchange = false;
        let mut b = cube("block", "iron", 0.5, &env);
        b.vel = [4.0, 0.0, 0.0];
        a.add(b);
        for _ in 0..240 * 5 {
            a.step();
        }
        let mu = env_of("iron").friction_kinetic;
        let expect = 16.0 / (2.0 * mu * G0);
        let got = a.bodies[0].pos[0];
        assert!((got - expect).abs() / expect < 0.03, "{} vs {}", got, expect);
        assert!(a.bodies[0].vel[0].abs() < 1e-4);
    }

    #[test]
    fn head_on_collision_conserves_momentum() {
        let env = Environment::space();
        let mut a = Arena::new(env.clone());
        a.ground = false;
        a.exchange = false;
        let mut l = cube("l", "copper", 0.0, &env);
        let mut r = cube("r", "aluminum", 0.0, &env);
        l.pos = [-2.0, 0.0, 0.0];
        r.pos = [2.0, 0.0, 0.0];
        l.vel = [3.0, 0.0, 0.0];
        r.vel = [-1.0, 0.0, 0.0];
        a.add(l);
        a.add(r);
        let p0: f32 = a.bodies.iter().map(|b| b.mass * b.vel[0]).sum();
        for _ in 0..480 {
            a.step();
        }
        let p1: f32 = a.bodies.iter().map(|b| b.mass * b.vel[0]).sum();
        assert!((p1 - p0).abs() / p0.abs() < 1e-4, "{} {}", p0, p1);
        assert!(a.bodies[0].vel[0] < 3.0 && a.bodies[1].vel[0] > -1.0, "they bounced");
        // relative speed scaled by e = √(e₁e₂)
        let e = (env_of("copper").restitution * env_of("aluminum").restitution).sqrt();
        let rel = a.bodies[1].vel[0] - a.bodies[0].vel[0];
        assert!((rel - 4.0 * e).abs() < 1e-3, "{} vs {}", rel, 4.0 * e);
    }

    #[test]
    fn ice_melts_and_boils_with_real_latent_heat() {
        // 1 kg-ish ice cube, insulated, 1 kW heater: time to 0 °C, melt, 100 °C, boil
        let env = Environment::earth();
        let mut a = Arena::new(env.clone());
        a.exchange = false;
        let mut ice = Body::new("ice", "water", env_of("water"), [0.05, 0.05, 0.05], [0.0, 0.05, 0.0], None, 263.15, env.pressure);
        ice.heater = 1000.0;
        let m = ice.mass;
        a.add(ice);
        let cp = 4181.0; // preset uses liquid cp for both phases (documented)
        let t_warm = m * cp * 10.0 / 1000.0;
        let t_melt = m * 3.3355e5 / 1000.0;
        let t_heat = m * cp * 100.0 / 1000.0;
        let t_boil = m * 2.2565e6 / 1000.0;
        let mut melted_at = None;
        let mut boiled_at = None;
        while a.time < (t_warm + t_melt + t_heat + t_boil) as f64 * 1.05 {
            a.step();
            let b = &a.bodies[0];
            if melted_at.is_none() && b.phase == Phase::Liquid {
                melted_at = Some(a.time as f32);
            }
            if boiled_at.is_none() && b.vaporised {
                boiled_at = Some(a.time as f32);
            }
        }
        let melted_at = melted_at.expect("melted");
        let boiled_at = boiled_at.expect("boiled");
        assert!((melted_at - (t_warm + t_melt)).abs() < 0.5, "melt {} vs {}", melted_at, t_warm + t_melt);
        let total = t_warm + t_melt + t_heat + t_boil;
        assert!((boiled_at - total).abs() / total < 0.002, "boil {} vs {}", boiled_at, total);
        assert!(a.events.log.iter().any(|(_, s)| s.contains("solid → liquid")));
        // a liquid puddle is flatter and wider
        let mut b2 = Body::new("w", "water", env_of("water"), [0.05; 3], [0.0, 0.05, 0.0], None, 272.0, env.pressure);
        let mut ev = Events::default();
        apply_heat(&mut b2, 1e6, env.pressure, 0.0, &mut ev);
        assert_eq!(b2.phase, Phase::Liquid);
        assert!(b2.half[1] < 0.02 && b2.half[0] > 0.09);
    }

    #[test]
    fn cools_by_convection_and_radiation_toward_ambient() {
        let env = Environment::earth();
        let mut a = Arena::new(env.clone());
        a.add(Body::new("hot", "copper", env_of("copper"), [0.05; 3], [0.0, 0.05, 0.0], None, 600.0, env.pressure));
        let t0 = a.bodies[0].temp;
        for _ in 0..240 * 60 {
            a.step();
        }
        let t1 = a.bodies[0].temp;
        assert!(t1 < t0 && t1 > env.temperature, "{} → {}", t0, t1);
        // exact first step: dT = (h A ΔT + εσA(T∞⁴−T⁴)) dt / (m c)
        let mut a = Arena::new(env.clone());
        a.add(Body::new("hot", "copper", env_of("copper"), [0.05; 3], [0.0, 0.05, 0.0], None, 600.0, env.pressure));
        a.step();
        let b = &a.bodies[0];
        let area = 0.06f64;
        let q = 10.0 * area * (288.15 - 600.0) + 0.04 * SIGMA * area * (288.15f64.powi(4) - 600f64.powi(4));
        let dt_expect = q * DT as f64 / (b.mass as f64 * 385.0);
        assert!(((b.temp as f64 - 600.0) - dt_expect).abs() < 1e-4, "{} vs {}", b.temp - 600.0, dt_expect);
        // in vacuum there is no convection, only radiation: cools slower
        let mut v = Arena::new(Environment::vacuum());
        v.add(Body::new("hot", "copper", env_of("copper"), [0.05; 3], [0.0, 0.05, 0.0], None, 600.0, 0.0));
        for _ in 0..240 * 60 {
            v.step();
        }
        assert!(v.bodies[0].temp > t1);
    }

    #[test]
    fn stacking_settles_and_energy_does_not_appear_from_nowhere() {
        let env = Environment::earth();
        let mut a = Arena::new(env.clone());
        a.exchange = false;
        a.add(cube("bottom", "iron", 0.5, &env));
        a.add(cube("top", "copper", 2.0, &env));
        let e0 = a.total_kinetic() + a.total_potential();
        let mut max_e = e0;
        for _ in 0..240 * 4 {
            a.step();
            max_e = max_e.max(a.total_kinetic() + a.total_potential());
        }
        assert!(max_e <= e0 * 1.001 + 1.0, "mechanical energy never increases: {} > {}", max_e, e0);
        let top = a.body("top").unwrap();
        assert!((top.pos[1] - 1.5).abs() < 0.02, "rests on the bottom cube: {}", top.pos[1]);
        assert!(top.vel[1].abs() < 0.05);
    }
}
