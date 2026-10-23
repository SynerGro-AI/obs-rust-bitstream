//! Composition → render (PBR) and simulation properties.
//!
//! * `parse_formula` reads `H2O`, `Fe2O3`, `Ca(OH)2`, `CuZn` …
//! * `analyze` runs every constituent through `MaterialsLab`'s nuclear
//!   stability check and searches the elements' common oxidation states for
//!   a charge-balanced assignment.
//! * `profile` derives a [`Pbr`] and an [`EnvMeta`] for the composition.
//!
//! Heuristics (documented; measured values always win via presets):
//! 1. **Known materials** (`PRESETS`) use measured values: base colour =
//!    linear-sRGB specular F0 for metals (Naty Hoffman / "Physically Based"
//!    database, physicallybased.info), albedo for dielectrics; IOR at 589 nm
//!    (refractiveindex.info); density, melting/boiling point, specific heat,
//!    thermal conductivity, latent heats from the CRC Handbook.
//! 2. **Metal or alloy** (every element a metal): metalness 1, base colour =
//!    atom-fraction mix of the elements' measured F0 (grey 0.6 if unknown),
//!    roughness 0.35, IOR 1.5 (unused for metals).
//! 3. **Anything containing a non-metal** is a dielectric: metalness 0,
//!    IOR 1.5 (typical for oxides/salts/glasses), roughness 0.5, base colour
//!    white-ish (0.82) unless a chromophore transition-metal ion is present
//!    (Cu → blue-green, Fe → rust, Cr → green, Co → blue, Ni → green,
//!    Mn → pink), then that tint. Pure non-metal elements use a tabulated
//!    colour (sulfur yellow, carbon black, …) or the CPK colour.
//! 4. **Density** of mixtures: ρ = Σmᵢ / Σ(mᵢ/ρᵢ) (ideal mixing by volume).
//!    For compounds this is only an estimate (bonding changes volume) and is
//!    flagged in `notes`.
//! 5. **Heat capacity**: Kopp–Neumann law (molar heat = Σ element molar
//!    heats). **Melting point** of alloys: mass-weighted mean (ignores
//!    eutectics; flagged). Compounds without a preset: unknown.

use std::fmt;

use crate::element_data::{self, Category};
use crate::element_props;
use crate::material::Material;
use crate::material_env::{element_env, EnvMeta, Flammability, Reactivity};
use crate::materials_lab::Atom;
use crate::periodic_table::PeriodicTable;
use crate::vec3::Vec3;

/// A composition: (Z, count) pairs in formula order; counts may be
/// fractional-free integers only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Composition(pub Vec<(u32, u32)>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormulaError {
    Empty,
    UnknownElement(String),
    Syntax(String),
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormulaError::Empty => write!(f, "empty formula"),
            FormulaError::UnknownElement(s) => write!(f, "unknown element '{}'", s),
            FormulaError::Syntax(s) => write!(f, "formula syntax: {}", s),
        }
    }
}

impl Composition {
    pub fn add(&mut self, z: u32, n: u32) {
        if n == 0 {
            return;
        }
        if let Some(e) = self.0.iter_mut().find(|e| e.0 == z) {
            e.1 += n;
        } else {
            self.0.push((z, n));
        }
    }

    pub fn atoms(&self) -> u32 {
        self.0.iter().map(|e| e.1).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// g/mol
    pub fn molar_mass(&self) -> f32 {
        self.0
            .iter()
            .map(|&(z, n)| element_data::get(z).map(|e| e.mass).unwrap_or(0.0) * n as f32)
            .sum()
    }

    /// Hill-ish formula in insertion order, e.g. `H2O`.
    pub fn formula(&self) -> String {
        let mut s = String::new();
        for &(z, n) in &self.0 {
            if let Some(e) = element_data::get(z) {
                s.push_str(e.symbol);
                if n > 1 {
                    s.push_str(&n.to_string());
                }
            }
        }
        s
    }

    fn mass_fractions(&self) -> Vec<(u32, f32)> {
        let m = self.molar_mass().max(1e-6);
        self.0
            .iter()
            .map(|&(z, n)| {
                (
                    z,
                    element_data::get(z).map(|e| e.mass).unwrap_or(0.0) * n as f32 / m,
                )
            })
            .collect()
    }
}

/// Parse a chemical formula, with nested parentheses and multipliers.
pub fn parse_formula(src: &str) -> Result<Composition, FormulaError> {
    let s: Vec<char> = src.chars().filter(|c| !c.is_whitespace()).collect();
    if s.is_empty() {
        return Err(FormulaError::Empty);
    }
    let mut i = 0;
    let groups = parse_group(&s, &mut i, 0)?;
    if i != s.len() {
        return Err(FormulaError::Syntax(format!("unexpected '{}'", s[i])));
    }
    let mut c = Composition::default();
    for (z, n) in groups {
        c.add(z, n);
    }
    if c.is_empty() {
        return Err(FormulaError::Empty);
    }
    Ok(c)
}

fn parse_num(s: &[char], i: &mut usize) -> Option<u32> {
    let st = *i;
    while *i < s.len() && s[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i == st {
        None
    } else {
        s[st..*i].iter().collect::<String>().parse().ok()
    }
}

fn parse_group(s: &[char], i: &mut usize, depth: u32) -> Result<Vec<(u32, u32)>, FormulaError> {
    let mut out = Vec::new();
    while *i < s.len() {
        let c = s[*i];
        if c == '(' || c == '[' {
            *i += 1;
            let inner = parse_group(s, i, depth + 1)?;
            if *i >= s.len() || !(s[*i] == ')' || s[*i] == ']') {
                return Err(FormulaError::Syntax("unclosed '('".into()));
            }
            *i += 1;
            let k = parse_num(s, i).unwrap_or(1);
            out.extend(inner.into_iter().map(|(z, n)| (z, n * k)));
        } else if c == ')' || c == ']' {
            if depth == 0 {
                return Err(FormulaError::Syntax("unmatched ')'".into()));
            }
            return Ok(out);
        } else if c.is_ascii_uppercase() {
            let mut sym = c.to_string();
            *i += 1;
            while *i < s.len() && s[*i].is_ascii_lowercase() {
                sym.push(s[*i]);
                *i += 1;
            }
            let e =
                element_data::by_symbol(&sym).ok_or(FormulaError::UnknownElement(sym.clone()))?;
            let k = parse_num(s, i).unwrap_or(1);
            if k == 0 {
                return Err(FormulaError::Syntax(format!("zero count for {}", sym)));
            }
            out.push((e.z, k));
        } else {
            return Err(FormulaError::Syntax(format!("unexpected '{}'", c)));
        }
    }
    if depth > 0 {
        return Err(FormulaError::Syntax("unclosed '('".into()));
    }
    Ok(out)
}

/// Result of running a composition through the lab.
#[derive(Clone, Debug, PartialEq)]
pub struct Analysis {
    pub formula: String,
    pub molar_mass: f32,
    /// (symbol, protons, neutrons, stable) for each constituent element's
    /// most common isotope.
    pub nuclei: Vec<(&'static str, u32, u32, bool)>,
    pub all_stable: bool,
    /// Oxidation-state assignment that sums to zero, if one exists.
    pub oxidation: Option<Vec<(&'static str, i32)>>,
    /// Pure element or all-metal alloy (charge balance trivially satisfied).
    pub metallic_bond: bool,
    pub charge_balanced: bool,
}

/// Parse the PubChem oxidation-state list ("+2, +1", "-2") into integers;
/// always includes 0 for metals (elemental / alloy state).
pub fn oxidation_states(z: u32) -> Vec<i32> {
    let Some(e) = element_data::get(z) else {
        return vec![];
    };
    let mut v: Vec<i32> = e
        .oxidation
        .split(',')
        .filter_map(|t| t.trim().trim_start_matches('+').parse().ok())
        .collect();
    if v.is_empty() {
        v.push(0);
    }
    v
}

pub fn analyze(c: &Composition) -> Analysis {
    let mut nuclei = Vec::new();
    for &(z, _) in &c.0 {
        if let (Some(e), Some(pt)) = (element_data::get(z), PeriodicTable::get(z)) {
            let atom = Atom::new(z, pt.neutrons, z);
            nuclei.push((e.symbol, z, pt.neutrons, atom.is_stable()));
        }
    }
    let all_stable = !nuclei.is_empty() && nuclei.iter().all(|n| n.3);
    let metallic_bond = c.0.len() == 1
        || c.0.iter().all(|&(z, _)| {
            element_data::get(z)
                .map(|e| e.category.is_metal())
                .unwrap_or(false)
        });
    // depth-first search over oxidation states (≤ 8 elements × ≤ 8 states)
    let opts: Vec<Vec<i32>> = c.0.iter().map(|&(z, _)| oxidation_states(z)).collect();
    let mut pick = vec![0i32; opts.len()];
    fn dfs(i: usize, sum: i32, c: &Composition, opts: &[Vec<i32>], pick: &mut Vec<i32>) -> bool {
        if i == opts.len() {
            return sum == 0;
        }
        for &o in &opts[i] {
            pick[i] = o;
            if dfs(i + 1, sum + o * c.0[i].1 as i32, c, opts, pick) {
                return true;
            }
        }
        false
    }
    let found = c.0.len() > 1 && c.0.len() <= 8 && dfs(0, 0, c, &opts, &mut pick);
    let oxidation = found.then(|| {
        c.0.iter()
            .zip(&pick)
            .filter_map(|(&(z, _), &o)| element_data::get(z).map(|e| (e.symbol, o)))
            .collect()
    });
    let charge_balanced = metallic_bond || found;
    Analysis {
        formula: c.formula(),
        molar_mass: c.molar_mass(),
        nuclei,
        all_stable,
        oxidation,
        metallic_bond,
        charge_balanced,
    }
}

/// Render parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pbr {
    /// Linear RGB, 0..1.
    pub base_color: [f32; 3],
    pub metalness: f32,
    pub roughness: f32,
    pub ior: f32,
    /// kg/m³
    pub density: f32,
    /// 0 opaque … 1 clear (glass/water/diamond).
    pub transmission: f32,
}

impl Pbr {
    pub fn to_material(&self, name: &str) -> Option<Material> {
        let mut m = Material::new(name).ok()?;
        self.apply(&mut m);
        Some(m)
    }

    pub fn apply(&self, m: &mut Material) {
        m.set_base_color(Vec3::new(
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
        ));
        m.set_metallic(self.metalness);
        m.set_roughness(self.roughness);
        m.set_ior(self.ior);
        m.set_density(self.density);
    }

    /// Dielectric normal-incidence reflectance from IOR.
    pub fn f0(&self) -> f32 {
        let r = (self.ior - 1.0) / (self.ior + 1.0);
        r * r
    }
}

/// Measured specular colour (linear F0) of pure metals.
/// Source: physicallybased.info / Hoffman, "Physics and Math of Shading".
pub fn metal_f0(symbol: &str) -> Option<[f32; 3]> {
    Some(match symbol {
        "Au" => [1.000, 0.766, 0.336],
        "Cu" => [0.955, 0.638, 0.538],
        "Ag" => [0.972, 0.960, 0.915],
        "Al" => [0.913, 0.922, 0.924],
        "Fe" => [0.562, 0.565, 0.578],
        "Ni" => [0.660, 0.609, 0.526],
        "Pt" => [0.673, 0.637, 0.585],
        "Ti" => [0.542, 0.497, 0.449],
        "Cr" => [0.549, 0.556, 0.554],
        "Co" => [0.662, 0.655, 0.634],
        "Zn" => [0.664, 0.824, 0.850],
        "Pb" => [0.632, 0.626, 0.641],
        "Sn" => [0.700, 0.700, 0.700],
        "Mg" => [0.800, 0.800, 0.810],
        "Hg" => [0.781, 0.779, 0.779],
        "W" => [0.504, 0.498, 0.478],
        "Pd" => [0.733, 0.697, 0.652],
        "Na" | "K" | "Li" => [0.850, 0.850, 0.860],
        _ => return None,
    })
}

fn nonmetal_colour(symbol: &str) -> Option<[f32; 3]> {
    Some(match symbol {
        "S" => [0.90, 0.80, 0.10],
        "C" => [0.04, 0.04, 0.045],
        "P" => [0.85, 0.20, 0.15],
        "I" => [0.20, 0.10, 0.25],
        "Br" => [0.45, 0.08, 0.03],
        "Cl" => [0.70, 0.85, 0.40],
        "F" => [0.90, 0.95, 0.70],
        "Se" => [0.25, 0.25, 0.27],
        "B" => [0.15, 0.12, 0.10],
        "Si" => [0.34, 0.36, 0.40],
        "Ge" => [0.45, 0.45, 0.48],
        "As" => [0.40, 0.40, 0.42],
        "Te" => [0.50, 0.50, 0.52],
        "Sb" => [0.55, 0.55, 0.58],
        _ => return None,
    })
}

fn chromophore(symbol: &str) -> Option<[f32; 3]> {
    Some(match symbol {
        "Cu" => [0.10, 0.45, 0.65],
        "Fe" => [0.45, 0.16, 0.07],
        "Cr" => [0.15, 0.45, 0.15],
        "Co" => [0.10, 0.20, 0.70],
        "Ni" => [0.30, 0.60, 0.30],
        "Mn" => [0.80, 0.45, 0.55],
        "V" => [0.60, 0.50, 0.10],
        _ => return None,
    })
}

/// A named, measured material.
#[derive(Clone, Debug)]
pub struct Preset {
    pub key: &'static str,
    pub name: &'static str,
    pub formula: &'static str,
    pub pbr: Pbr,
    pub env: EnvMeta,
}

#[allow(clippy::too_many_arguments)]
fn env(
    molar_mass: f32,
    density: f32,
    melt: Option<f32>,
    boil: Option<f32>,
    cp: f32,
    k: f32,
    lf: Option<f32>,
    lv: Option<f32>,
    resist: Option<f64>,
    emiss: f32,
    rest: f32,
    fric: (f32, f32),
    flam: Flammability,
    react: Reactivity,
) -> EnvMeta {
    EnvMeta {
        molar_mass,
        density: Some(density),
        density_liquid: None,
        melting_k: melt,
        boiling_k: boil,
        sublimes: false,
        specific_heat: Some(cp),
        thermal_conductivity: Some(k),
        latent_fusion: lf,
        latent_vaporization: lv,
        electrical_resistivity: resist,
        youngs_gpa: None,
        emissivity: emiss,
        restitution: rest,
        friction_static: fric.0,
        friction_kinetic: fric.1,
        flammability: flam,
        reactivity: react,
        oxidiser: false,
        radioactive: false,
        half_life_s: None,
        notes: vec!["CRC Handbook values; restitution/friction are typical engineering estimates"],
    }
}

/// Presets with measured values (CRC Handbook of Chemistry and Physics, 97th
/// ed.; refractiveindex.info at 589 nm; physicallybased.info for F0).
pub fn presets() -> Vec<Preset> {
    use Flammability::*;
    use Reactivity::*;
    let p = |c: [f32; 3], m: f32, r: f32, ior: f32, d: f32, t: f32| Pbr {
        base_color: c,
        metalness: m,
        roughness: r,
        ior,
        density: d,
        transmission: t,
    };
    let mut v = vec![
        Preset {
            key: "gold",
            name: "Gold",
            formula: "Au",
            pbr: p([1.0, 0.766, 0.336], 1.0, 0.25, 0.47, 19_300.0, 0.0),
            env: env(
                196.97,
                19_300.0,
                Some(1337.33),
                Some(3129.0),
                129.0,
                318.0,
                Some(6.37e4),
                Some(1.645e6),
                Some(2.214e-8),
                0.03,
                0.5,
                (0.49, 0.40),
                NonFlammable,
                Inert,
            ),
        },
        Preset {
            key: "copper",
            name: "Copper",
            formula: "Cu",
            pbr: p([0.955, 0.638, 0.538], 1.0, 0.3, 0.64, 8_960.0, 0.0),
            env: env(
                63.546,
                8_960.0,
                Some(1357.77),
                Some(2835.0),
                385.0,
                401.0,
                Some(2.09e5),
                Some(4.73e6),
                Some(1.68e-8),
                0.04,
                0.55,
                (0.53, 0.36),
                NonFlammable,
                Low,
            ),
        },
        Preset {
            key: "iron",
            name: "Iron",
            formula: "Fe",
            pbr: p([0.562, 0.565, 0.578], 1.0, 0.45, 2.9, 7_874.0, 0.0),
            env: env(
                55.845,
                7_874.0,
                Some(1811.0),
                Some(3134.0),
                449.0,
                80.4,
                Some(2.47e5),
                Some(6.09e6),
                Some(9.71e-8),
                0.2,
                0.6,
                (0.74, 0.57),
                Combustible,
                Low,
            ),
        },
        Preset {
            key: "aluminum",
            name: "Aluminium",
            formula: "Al",
            pbr: p([0.913, 0.922, 0.924], 1.0, 0.35, 1.44, 2_700.0, 0.0),
            env: env(
                26.982,
                2_700.0,
                Some(933.47),
                Some(2743.0),
                897.0,
                237.0,
                Some(3.97e5),
                Some(1.09e7),
                Some(2.65e-8),
                0.09,
                0.55,
                (0.61, 0.47),
                Combustible,
                Low,
            ),
        },
        Preset {
            key: "silicon",
            name: "Silicon",
            formula: "Si",
            pbr: p([0.34, 0.36, 0.40], 0.3, 0.3, 3.96, 2_329.0, 0.0),
            env: env(
                28.086,
                2_329.0,
                Some(1687.0),
                Some(3538.0),
                705.0,
                149.0,
                Some(1.79e6),
                Some(1.28e7),
                Some(2.3e3),
                0.6,
                0.5,
                (0.5, 0.4),
                NonFlammable,
                Low,
            ),
        },
        Preset {
            key: "glass",
            name: "Glass (fused silica)",
            formula: "SiO2",
            pbr: p([0.95, 0.95, 0.95], 0.0, 0.05, 1.458, 2_200.0, 0.9),
            env: env(
                60.08,
                2_200.0,
                Some(1986.0),
                Some(2503.0),
                740.0,
                1.38,
                Some(1.42e5),
                None,
                Some(1e16),
                0.9,
                0.7,
                (0.94, 0.40),
                NonFlammable,
                Inert,
            ),
        },
        Preset {
            key: "water",
            name: "Water",
            formula: "H2O",
            pbr: p([0.80, 0.90, 0.95], 0.0, 0.05, 1.333, 997.0, 0.95),
            env: env(
                18.015,
                997.0,
                Some(273.15),
                Some(373.124),
                4181.0,
                0.606,
                Some(3.3355e5),
                Some(2.2565e6),
                Some(1.8e5),
                0.96,
                0.0,
                (0.1, 0.05),
                NonFlammable,
                Low,
            ),
        },
        Preset {
            key: "graphite",
            name: "Graphite",
            formula: "C",
            pbr: p([0.04, 0.04, 0.045], 0.0, 0.6, 2.15, 2_260.0, 0.0),
            env: env(
                12.011,
                2_260.0,
                None,
                Some(3915.0),
                709.0,
                140.0,
                None,
                None,
                Some(1.0e-5),
                0.85,
                0.45,
                (0.1, 0.1),
                Combustible,
                Low,
            ),
        },
        Preset {
            key: "diamond",
            name: "Diamond",
            formula: "C",
            pbr: p([1.0, 1.0, 1.0], 0.0, 0.02, 2.417, 3_510.0, 0.95),
            env: env(
                12.011,
                3_510.0,
                None,
                Some(3915.0),
                509.0,
                2200.0,
                None,
                None,
                Some(1e13),
                0.1,
                0.6,
                (0.1, 0.05),
                Combustible,
                Inert,
            ),
        },
    ];
    for pr in v.iter_mut() {
        match pr.key {
            // ice is less dense than water: 917 kg/m³; liquid 997
            "water" => {
                pr.env.density = Some(917.0);
                pr.env.density_liquid = Some(997.0);
                pr.env.specific_heat = Some(4181.0);
                pr.env
                    .notes
                    .push("solid density is ice (917 kg/m³); cp/conductivity are the liquid's");
            }
            "graphite" | "diamond" => {
                pr.env.sublimes = true;
                pr.env.notes.push("carbon sublimes at ~3915 K at 1 atm");
            }
            "silicon" => pr
                .env
                .notes
                .push("resistivity is intrinsic (undoped) silicon; band gap 1.12 eV"),
            _ => {}
        }
        if pr.env.density_liquid.is_none() && pr.pbr.metalness > 0.5 {
            pr.env.density_liquid = pr.env.density.map(|d| d * 0.95);
        }
    }
    v
}

pub fn preset(key: &str) -> Option<Preset> {
    let k = key.trim().to_ascii_lowercase();
    let k = match k.as_str() {
        "aluminium" | "al" => "aluminum".to_string(),
        "au" => "gold".into(),
        "cu" => "copper".into(),
        "fe" => "iron".into(),
        "si" => "silicon".into(),
        "sio2" | "silica" | "quartz" => "glass".into(),
        "h2o" | "ice" => "water".into(),
        "carbon" | "c" => "graphite".into(),
        _ => k,
    };
    presets().into_iter().find(|p| p.key == k)
}

/// Full derived profile for a composition.
#[derive(Clone, Debug)]
pub struct Profile {
    pub name: String,
    pub formula: String,
    pub analysis: Analysis,
    pub pbr: Pbr,
    pub env: EnvMeta,
    pub notes: Vec<String>,
    pub preset: Option<&'static str>,
}

/// Profile of a preset (measured values), whatever its formula collides with.
pub fn preset_profile(p: &Preset) -> Profile {
    let c = parse_formula(p.formula).unwrap_or_default();
    Profile {
        name: p.name.to_string(),
        formula: p.formula.to_string(),
        analysis: analyze(&c),
        pbr: p.pbr,
        env: p.env.clone(),
        notes: vec![format!("measured values (preset '{}')", p.key)],
        preset: Some(p.key),
    }
}

fn elem_density(z: u32) -> Option<f32> {
    element_data::get(z)
        .and_then(|e| e.density)
        .map(|d| d * 1000.0)
}

pub fn profile(c: &Composition) -> Profile {
    let analysis = analyze(c);
    let formula = c.formula();
    if let Some(p) = presets().into_iter().find(|p| p.formula == formula) {
        return Profile {
            name: p.name.to_string(),
            formula,
            analysis,
            pbr: p.pbr,
            env: p.env,
            notes: vec![format!("measured values (preset '{}')", p.key)],
            preset: Some(p.key),
        };
    }
    let mut notes = Vec::new();
    let els: Vec<&element_data::ElementData> =
        c.0.iter()
            .filter_map(|&(z, _)| element_data::get(z))
            .collect();
    let atoms = c.atoms().max(1) as f32;
    let all_metal = els.iter().all(|e| e.category.is_metal());
    let mf = c.mass_fractions();
    // density: ideal mixing by volume
    let inv: Option<f32> = mf
        .iter()
        .map(|&(z, w)| elem_density(z).map(|d| w / d.max(1e-6)))
        .sum();
    let density = inv.map(|i| 1.0 / i.max(1e-9)).unwrap_or(1000.0);
    if c.0.len() > 1 {
        notes.push("density: ideal volume mixing of the elements (estimate)".to_string());
    }
    let mut pbr = if all_metal {
        let mut col = [0.0f32; 3];
        for (&(_, n), e) in c.0.iter().zip(&els) {
            let f = metal_f0(e.symbol).unwrap_or([0.6, 0.6, 0.6]);
            for k in 0..3 {
                col[k] += f[k] * n as f32 / atoms;
            }
        }
        notes
            .push("metal: metalness 1, base colour = atom-fraction mix of measured F0".to_string());
        Pbr {
            base_color: col,
            metalness: 1.0,
            roughness: 0.35,
            ior: 1.5,
            density,
            transmission: 0.0,
        }
    } else if c.0.len() == 1 {
        let e = els[0];
        let col = nonmetal_colour(e.symbol).unwrap_or_else(|| {
            let h = e.cpk.unwrap_or(0xcccccc);
            [
                ((h >> 16) & 255) as f32 / 255.0,
                ((h >> 8) & 255) as f32 / 255.0,
                (h & 255) as f32 / 255.0,
            ]
        });
        let metalloid = e.category == Category::Metalloid;
        notes.push(format!(
            "pure {}: tabulated/CPK colour",
            if metalloid { "metalloid" } else { "non-metal" }
        ));
        Pbr {
            base_color: col,
            metalness: if metalloid { 0.3 } else { 0.0 },
            roughness: 0.5,
            ior: 1.5,
            density,
            transmission: 0.0,
        }
    } else {
        let tint = els.iter().find_map(|e| chromophore(e.symbol));
        if tint.is_some() {
            notes.push("dielectric: tinted by transition-metal chromophore".to_string());
        } else {
            notes.push("dielectric: metalness 0, IOR 1.5, off-white".to_string());
        }
        Pbr {
            base_color: tint.unwrap_or([0.82, 0.82, 0.80]),
            metalness: 0.0,
            roughness: 0.5,
            ior: 1.5,
            density,
            transmission: 0.0,
        }
    };
    pbr.roughness = pbr.roughness.clamp(0.02, 1.0);
    // env
    let envs: Vec<EnvMeta> = c.0.iter().filter_map(|&(z, _)| element_env(z)).collect();
    let mut env = if c.0.len() == 1 && !envs.is_empty() {
        envs[0].clone()
    } else {
        let molar = c.molar_mass();
        // Kopp–Neumann: molar heat = Σ nᵢ·Cᵢ
        let cmol: Option<f32> =
            c.0.iter()
                .map(|&(z, n)| {
                    element_props::props(z)
                        .and_then(|p| p.molar_heat)
                        .map(|m| m * n as f32)
                })
                .sum();
        let cp = cmol.map(|m| m / (molar / 1000.0));
        let wmean = |f: &dyn Fn(&EnvMeta) -> Option<f32>| -> Option<f32> {
            mf.iter()
                .zip(&envs)
                .map(|(&(_, w), e)| f(e).map(|v| v * w))
                .sum()
        };
        let melt = if all_metal {
            wmean(&|e| e.melting_k)
        } else {
            None
        };
        let boil = if all_metal {
            wmean(&|e| e.boiling_k)
        } else {
            None
        };
        let k = if all_metal {
            wmean(&|e| e.thermal_conductivity)
        } else {
            None
        };
        let lf = if all_metal {
            wmean(&|e| e.latent_fusion)
        } else {
            None
        };
        notes.push("heat capacity: Kopp–Neumann law".to_string());
        if all_metal {
            notes.push("alloy melting/boiling/conductivity: mass-weighted means (estimate, ignores eutectics)".to_string());
        } else {
            notes.push("compound phase-change points unknown (no measured data)".to_string());
        }
        let worst_flam = envs
            .iter()
            .map(|e| e.flammability)
            .max()
            .unwrap_or(Flammability::NonFlammable);
        // oxidised compounds (contain O or F with a positive partner) don't burn further
        let oxidised = !all_metal && els.iter().any(|e| e.symbol == "O" || e.symbol == "F");
        let hydrocarbon = els.iter().all(|e| e.symbol == "C" || e.symbol == "H") && els.len() == 2;
        EnvMeta {
            molar_mass: molar,
            density: Some(density),
            density_liquid: if all_metal {
                Some(density * 0.95)
            } else {
                None
            },
            melting_k: melt,
            boiling_k: boil,
            sublimes: false,
            specific_heat: cp,
            thermal_conductivity: k,
            latent_fusion: lf,
            latent_vaporization: None,
            electrical_resistivity: None,
            youngs_gpa: None,
            emissivity: if all_metal { 0.15 } else { 0.9 },
            restitution: if all_metal { 0.55 } else { 0.45 },
            friction_static: 0.55,
            friction_kinetic: 0.42,
            flammability: if hydrocarbon {
                Flammability::Flammable
            } else if oxidised {
                Flammability::NonFlammable
            } else {
                worst_flam.min(Flammability::Combustible)
            },
            reactivity: if analysis.charge_balanced {
                Reactivity::Low
            } else {
                Reactivity::High
            },
            oxidiser: false,
            radioactive: envs.iter().any(|e| e.radioactive),
            half_life_s: envs.iter().filter_map(|e| e.half_life_s).reduce(f64::min),
            notes: vec!["derived from constituent elements (estimates)"],
        }
    };
    if env.density.is_none() {
        env.density = Some(density);
    }
    let name = if c.0.len() == 1 {
        els.first().map(|e| e.name.to_string()).unwrap_or_default()
    } else {
        formula.clone()
    };
    Profile {
        name,
        formula,
        analysis,
        pbr,
        env,
        notes,
        preset: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str) -> Composition {
        parse_formula(s).unwrap()
    }

    #[test]
    fn parses_formulas() {
        assert_eq!(f("H2O").0, vec![(1, 2), (8, 1)]);
        assert_eq!(f("Ca(OH)2").0, vec![(20, 1), (8, 2), (1, 2)]);
        assert_eq!(f("Fe2O3").formula(), "Fe2O3");
        assert_eq!(f("CH3COOH").0, vec![(6, 2), (1, 4), (8, 2)]);
        assert_eq!(f("K4[Fe(CN)6]").0, vec![(19, 4), (26, 1), (6, 6), (7, 6)]);
        assert!(matches!(
            parse_formula("Xx2"),
            Err(FormulaError::UnknownElement(_))
        ));
        assert!(matches!(
            parse_formula("H2(O"),
            Err(FormulaError::Syntax(_))
        ));
        assert!(matches!(parse_formula(")"), Err(FormulaError::Syntax(_))));
        assert_eq!(parse_formula(""), Err(FormulaError::Empty));
        assert!((f("H2O").molar_mass() - 18.015).abs() < 0.01);
    }

    #[test]
    fn charge_balance_and_stability() {
        let w = analyze(&f("H2O"));
        assert!(w.charge_balanced && w.all_stable && !w.metallic_bond);
        assert_eq!(w.oxidation, Some(vec![("H", 1), ("O", -2)]));
        assert!(analyze(&f("Fe2O3")).charge_balanced);
        assert!(analyze(&f("NaCl")).charge_balanced);
        assert!(
            !analyze(&f("NaCl2")).charge_balanced,
            "no Na/Cl states give NaCl2"
        );
        assert!(analyze(&f("CuZn")).metallic_bond, "brass is an alloy");
        // technetium has no stable isotope in the lab's N/Z table? it uses
        // the most common isotope; the check is the nuclear band, report it
        assert!(analyze(&f("Au")).all_stable);
    }

    #[test]
    fn presets_and_heuristics() {
        let cu = profile(&f("Cu"));
        assert_eq!(cu.preset, Some("copper"));
        assert_eq!(cu.pbr.metalness, 1.0);
        assert!(
            cu.pbr.base_color[0] > cu.pbr.base_color[2],
            "copper is reddish"
        );
        let w = profile(&f("H2O"));
        assert!((w.pbr.ior - 1.333).abs() < 1e-3 && w.pbr.metalness == 0.0);
        assert!((w.pbr.f0() - 0.02).abs() < 0.001, "water F0 ≈ 2 %");
        let g = profile(&f("SiO2"));
        assert!((g.pbr.ior - 1.458).abs() < 1e-3);
        // brass Cu3Zn2: density between Zn and Cu, metal
        let b = profile(&f("Cu3Zn2"));
        assert_eq!(b.pbr.metalness, 1.0);
        assert!(
            b.pbr.density > 7_100.0 && b.pbr.density < 8_960.0,
            "{}",
            b.pbr.density
        );
        assert!(b.env.melting_k.unwrap() > 692.0 && b.env.melting_k.unwrap() < 1358.0);
        // rust is dielectric and rust-coloured
        let r = profile(&f("Fe2O3"));
        assert_eq!(r.pbr.metalness, 0.0);
        assert!(r.pbr.base_color[0] > r.pbr.base_color[2]);
        assert_eq!(r.env.flammability, Flammability::NonFlammable);
        // methane burns; Kopp's law heat capacity is positive
        let m = profile(&f("CH4"));
        assert_eq!(m.env.flammability, Flammability::Flammable);
        assert!(m.env.specific_heat.unwrap() > 0.0);
        assert!(preset("diamond").unwrap().pbr.ior > 2.4);
        assert_eq!(preset("aluminium").unwrap().key, "aluminum");
        let mat = cu.pbr.to_material("Copper").unwrap();
        assert!((mat.density() - 8960.0).abs() < 1.0);
    }
}
