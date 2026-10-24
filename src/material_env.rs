//! "Env metadata": the physical behaviour a simulation needs for an element
//! or material, with the heuristics documented next to each value.
//!
//! Measured data comes from `element_data` (PubChem) and `element_props`
//! (mendeleev / pymatgen). Where a property is a *pair* property in reality
//! (restitution, friction) or simply not tabulated (emissivity of an
//! arbitrary surface), a category estimate is used and flagged in `notes`.
//!
//! Phase versus pressure: the boiling point is moved with the
//! Clausius–Clapeyron relation `1/T_b(P) = 1/T_b0 − R·ln(P/P0)/ΔH_vap`
//! (ideal-gas vapour, constant ΔH_vap). The melting point is treated as
//! pressure-independent (the solid–liquid line is nearly vertical for
//! ordinary pressures). Sublimation (e.g. carbon at 1 atm) is flagged.

use crate::element_data::{self, Category};
use crate::element_props;

/// Molar gas constant (CODATA 2018, exact).
pub const R_GAS: f64 = 8.314_462_618;
/// Standard atmosphere (Pa).
pub const P_ATM: f64 = 101_325.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ui", derive(serde::Serialize, serde::Deserialize))]
pub enum Phase {
    Solid,
    Liquid,
    Gas,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Solid => "solid",
            Phase::Liquid => "liquid",
            Phase::Gas => "gas",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "ui", derive(serde::Serialize, serde::Deserialize))]
pub enum Flammability {
    NonFlammable,
    /// Burns when strongly heated / finely divided (e.g. Mg, Al, Fe powders).
    Combustible,
    Flammable,
    /// Ignites spontaneously in air (white phosphorus, Cs, Rb, …).
    Pyrophoric,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "ui", derive(serde::Serialize, serde::Deserialize))]
pub enum Reactivity {
    Inert,
    Low,
    Moderate,
    High,
    Violent,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ui", derive(serde::Serialize, serde::Deserialize))]
pub struct EnvMeta {
    /// g/mol of the formula unit that evaporates (H₂, N₂, O₂, halogens are
    /// diatomic, so twice the atomic mass).
    pub molar_mass: f32,
    /// kg/m³ in the standard state.
    pub density: Option<f32>,
    /// kg/m³ of the liquid, when it differs notably (water/ice) or is estimated.
    pub density_liquid: Option<f32>,
    pub melting_k: Option<f32>,
    pub boiling_k: Option<f32>,
    pub sublimes: bool,
    /// J/(kg·K)
    pub specific_heat: Option<f32>,
    /// W/(m·K)
    pub thermal_conductivity: Option<f32>,
    /// J/kg
    pub latent_fusion: Option<f32>,
    /// J/kg
    pub latent_vaporization: Option<f32>,
    /// Ω·m
    pub electrical_resistivity: Option<f64>,
    pub youngs_gpa: Option<f32>,
    /// Hemispherical total emissivity estimate (0..1).
    pub emissivity: f32,
    /// Coefficient of restitution estimate against a hard surface.
    pub restitution: f32,
    pub friction_static: f32,
    pub friction_kinetic: f32,
    pub flammability: Flammability,
    pub reactivity: Reactivity,
    pub oxidiser: bool,
    pub radioactive: bool,
    pub half_life_s: Option<f64>,
    /// Which values are estimates rather than measurements.
    #[cfg_attr(feature = "ui", serde(deserialize_with = "crate::serde_static::vec"))]
    pub notes: Vec<&'static str>,
}

impl EnvMeta {
    /// Boiling point at pressure `p` (Pa) via Clausius–Clapeyron. `None` when
    /// unknown; at `p <= 0` (vacuum) everything that isn't solid boils.
    pub fn boiling_at(&self, p: f64) -> Option<f32> {
        let tb = self.boiling_k? as f64;
        if p <= 0.0 {
            return Some(0.0);
        }
        let Some(lv) = self.latent_vaporization else {
            return Some(tb as f32);
        };
        let dh = lv as f64 * self.molar_mass as f64 / 1000.0; // J/mol
        if dh <= 0.0 {
            return Some(tb as f32);
        }
        let inv = 1.0 / tb - R_GAS * (p / P_ATM).ln() / dh;
        Some(if inv <= 0.0 {
            f32::INFINITY
        } else {
            (1.0 / inv) as f32
        })
    }

    /// Phase at temperature `t` (K) and pressure `p` (Pa).
    pub fn phase_at(&self, t: f32, p: f64) -> Phase {
        let melt = self.melting_k;
        let boil = self.boiling_at(p);
        if self.sublimes {
            return match boil {
                Some(b) if t >= b => Phase::Gas,
                _ => Phase::Solid,
            };
        }
        if let Some(m) = melt {
            if t < m {
                // in hard vacuum a solid stays solid (sublimation is slow; ignored)
                return Phase::Solid;
            }
        }
        match boil {
            Some(b) if t >= b => Phase::Gas,
            _ if melt.is_some() => Phase::Liquid,
            _ => Phase::Solid,
        }
    }

    /// Density in the given phase (kg/m³). Gas density from the ideal gas law.
    pub fn density_in(&self, phase: Phase, t: f32, p: f64) -> Option<f32> {
        match phase {
            Phase::Solid => self.density,
            Phase::Liquid => self.density_liquid.or(self.density),
            Phase::Gas => {
                Some((p * self.molar_mass as f64 / 1000.0 / (R_GAS * t.max(1.0) as f64)) as f32)
            }
        }
    }
}

/// Elements that are effectively inert noble metals.
const NOBLE_METALS: [&str; 8] = ["Au", "Pt", "Ir", "Os", "Rh", "Ru", "Pd", "Ag"];

/// Env metadata for an element in its standard state.
pub fn element_env(z: u32) -> Option<EnvMeta> {
    let e = element_data::get(z)?;
    let p = element_props::props(z)?;
    let mut notes = Vec::new();
    let gas = e.state.contains("Gas");
    // PubChem densities are g/cm³ (gases at STP) → kg/m³
    let density = e.density.map(|d| d * 1000.0);
    // Liquid density estimate: ~5 % below the solid for metals (typical
    // volume change on melting is +2…+6 %); same as solid otherwise.
    let density_liquid = if e.category.is_metal() && !gas {
        notes.push("liquid density estimated as 0.95 × solid");
        density.map(|d| d * 0.95)
    } else {
        None
    };
    // diatomic elements: tabulated ΔH are per mole of molecules
    let diatomic = matches!(e.symbol, "H" | "N" | "O" | "F" | "Cl" | "Br" | "I");
    let molar = if diatomic { 2.0 * e.mass } else { e.mass };
    let per_kg = |kj_mol: Option<f32>| kj_mol.map(|h| h * 1000.0 / (molar / 1000.0));
    let sublimes = e.symbol == "C" || e.symbol == "As";
    let (emissivity, restitution, fs, fk) = match e.category {
        // polished metal surfaces: low emissivity; metal-on-metal friction
        c if c.is_metal() => (0.1, 0.55, 0.6, 0.45),
        Category::Metalloid => (0.6, 0.5, 0.5, 0.4),
        _ => (0.9, 0.4, 0.5, 0.4),
    };
    notes.push("emissivity, restitution and friction are category estimates");
    let flammability = match e.symbol {
        "P" | "Cs" | "Rb" | "Fr" | "Pu" => Flammability::Pyrophoric,
        "H" | "C" | "S" | "Li" | "Na" | "K" | "Mg" | "Ca" | "Sr" | "Ba" | "B" | "Si" | "Se" => {
            Flammability::Flammable
        }
        "Al" | "Fe" | "Ti" | "Zr" | "Hf" | "Zn" | "Mn" | "U" | "Th" | "Ce" | "La" | "Nd" => {
            Flammability::Combustible
        }
        _ if e.category == Category::Lanthanide || e.category == Category::Actinide => {
            Flammability::Combustible
        }
        _ => Flammability::NonFlammable,
    };
    let reactivity = match e.category {
        Category::NobleGas => Reactivity::Inert,
        Category::AlkaliMetal => Reactivity::Violent,
        Category::Halogen if e.symbol == "F" => Reactivity::Violent,
        Category::Halogen => Reactivity::High,
        Category::AlkalineEarthMetal if e.symbol == "Be" || e.symbol == "Mg" => {
            Reactivity::Moderate
        }
        Category::AlkalineEarthMetal => Reactivity::High,
        _ if NOBLE_METALS.contains(&e.symbol) => Reactivity::Inert,
        Category::Lanthanide | Category::Actinide => Reactivity::High,
        Category::TransitionMetal | Category::PostTransitionMetal | Category::Metalloid => {
            Reactivity::Low
        }
        _ if e.symbol == "O" => Reactivity::High,
        _ if e.symbol == "N" => Reactivity::Low,
        _ => Reactivity::Moderate,
    };
    Some(EnvMeta {
        molar_mass: molar,
        density,
        density_liquid,
        melting_k: if sublimes { None } else { e.melting_k },
        boiling_k: if sublimes {
            e.boiling_k.or(e.melting_k)
        } else {
            e.boiling_k
        },
        sublimes,
        specific_heat: p.specific_heat.map(|c| c * 1000.0),
        thermal_conductivity: p.thermal_conductivity,
        latent_fusion: per_kg(p.fusion_heat_kj_mol),
        latent_vaporization: per_kg(p.vaporization_heat_kj_mol),
        electrical_resistivity: p.electrical_resistivity,
        youngs_gpa: p.youngs_gpa,
        emissivity,
        restitution,
        friction_static: fs,
        friction_kinetic: fk,
        flammability,
        reactivity,
        oxidiser: matches!(e.symbol, "O" | "F" | "Cl"),
        radioactive: p.radioactive,
        half_life_s: p.longest_half_life_s,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element_data::by_symbol;

    fn env(s: &str) -> EnvMeta {
        element_env(by_symbol(s).unwrap().z).unwrap()
    }

    #[test]
    fn phases_at_room_temperature_match_reality() {
        let room = 298.15;
        assert_eq!(env("Fe").phase_at(room, P_ATM), Phase::Solid);
        assert_eq!(env("Hg").phase_at(room, P_ATM), Phase::Liquid);
        assert_eq!(env("Br").phase_at(room, P_ATM), Phase::Liquid);
        assert_eq!(env("O").phase_at(room, P_ATM), Phase::Gas);
        assert_eq!(
            env("Ga").phase_at(310.0, P_ATM),
            Phase::Liquid,
            "gallium melts in the hand (302.9 K)"
        );
        assert_eq!(env("Al").phase_at(933.0, P_ATM), Phase::Solid);
        assert_eq!(env("Al").phase_at(934.0, P_ATM), Phase::Liquid);
        assert_eq!(
            env("C").phase_at(3000.0, P_ATM),
            Phase::Solid,
            "carbon sublimes, never liquid at 1 atm"
        );
    }

    #[test]
    fn clausius_clapeyron_moves_the_boiling_point() {
        let hg = env("Hg");
        let tb0 = hg.boiling_at(P_ATM).unwrap();
        assert!((tb0 - 629.88).abs() < 0.5);
        // lower pressure, lower boiling point; higher, higher
        assert!(hg.boiling_at(P_ATM * 0.1).unwrap() < tb0 - 50.0);
        assert!(hg.boiling_at(P_ATM * 10.0).unwrap() > tb0 + 50.0);
        assert_eq!(hg.boiling_at(0.0), Some(0.0), "vacuum");
    }

    #[test]
    fn units_and_classes() {
        let cu = env("Cu");
        assert!((cu.density.unwrap() - 8960.0).abs() < 40.0);
        assert!((cu.specific_heat.unwrap() - 385.0).abs() < 5.0);
        // Cu heat of fusion 13.26 kJ/mol → ~209 kJ/kg (CRC: 208.7)
        assert!(
            (cu.latent_fusion.unwrap() - 2.09e5).abs() < 0.06e5,
            "{:?}",
            cu.latent_fusion
        );
        assert!(!cu.radioactive && cu.reactivity == Reactivity::Low);
        assert_eq!(env("Au").reactivity, Reactivity::Inert);
        assert_eq!(env("Na").reactivity, Reactivity::Violent);
        assert_eq!(env("Ar").reactivity, Reactivity::Inert);
        assert!(env("U").radioactive && env("U").half_life_s.is_some());
        assert!(env("O").oxidiser);
        // N₂ gas density from the ideal gas law at STP: 1.2506 kg/m³ measured
        let n = env("N");
        let rho = n.density_in(Phase::Gas, 273.15, P_ATM).unwrap();
        assert!((rho - 1.2506).abs() < 0.005, "N2: {}", rho);
        // Hg ΔH_vap 59.1 kJ/mol → 295 kJ/kg (CRC 294.7)
        let hg = env("Hg").latent_vaporization.unwrap();
        assert!((hg - 2.95e5).abs() < 0.1e5, "{}", hg);
    }
}
