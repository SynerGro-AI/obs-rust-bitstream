//! Safe pure PeriodicTable (element lookup core).
//!
//! Maps atomic number to name, symbol, and a typical neutron
//! count for the most common isotope. No unwraps.
//!
//! Differential: Z=6 returns carbon with 6 neutrons.

#[derive(Clone, Copy, Debug)]
pub struct Element {
    pub z: u32,
    pub symbol: &'static str,
    pub name: &'static str,
    pub neutrons: u32,
}

pub struct PeriodicTable;

impl PeriodicTable {
    /// Look up an element by atomic number. Returns None if not in the table.
    pub fn get(z: u32) -> Option<Element> {
        ELEMENTS.iter().find(|e| e.z == z).map(|e| Element {
            z: e.z,
            symbol: e.symbol,
            name: e.name,
            neutrons: e.neutrons,
        })
    }

    /// Look up by symbol (case-sensitive).
    pub fn by_symbol(sym: &str) -> Option<Element> {
        ELEMENTS.iter().find(|e| e.symbol == sym).map(|e| Element {
            z: e.z,
            symbol: e.symbol,
            name: e.name,
            neutrons: e.neutrons,
        })
    }

    pub fn len() -> usize {
        ELEMENTS.len()
    }
}

const ELEMENTS: &[Element] = &[
    Element { z: 1, symbol: "H", name: "Hydrogen", neutrons: 0 },
    Element { z: 2, symbol: "He", name: "Helium", neutrons: 2 },
    Element { z: 3, symbol: "Li", name: "Lithium", neutrons: 4 },
    Element { z: 4, symbol: "Be", name: "Beryllium", neutrons: 5 },
    Element { z: 5, symbol: "B", name: "Boron", neutrons: 6 },
    Element { z: 6, symbol: "C", name: "Carbon", neutrons: 6 },
    Element { z: 7, symbol: "N", name: "Nitrogen", neutrons: 7 },
    Element { z: 8, symbol: "O", name: "Oxygen", neutrons: 8 },
    Element { z: 9, symbol: "F", name: "Fluorine", neutrons: 10 },
    Element { z: 10, symbol: "Ne", name: "Neon", neutrons: 10 },
    Element { z: 11, symbol: "Na", name: "Sodium", neutrons: 12 },
    Element { z: 12, symbol: "Mg", name: "Magnesium", neutrons: 12 },
    Element { z: 13, symbol: "Al", name: "Aluminum", neutrons: 14 },
    Element { z: 14, symbol: "Si", name: "Silicon", neutrons: 14 },
    Element { z: 15, symbol: "P", name: "Phosphorus", neutrons: 16 },
    Element { z: 16, symbol: "S", name: "Sulfur", neutrons: 16 },
    Element { z: 17, symbol: "Cl", name: "Chlorine", neutrons: 18 },
    Element { z: 18, symbol: "Ar", name: "Argon", neutrons: 22 },
    Element { z: 19, symbol: "K", name: "Potassium", neutrons: 20 },
    Element { z: 20, symbol: "Ca", name: "Calcium", neutrons: 20 },
    Element { z: 26, symbol: "Fe", name: "Iron", neutrons: 30 },
    Element { z: 29, symbol: "Cu", name: "Copper", neutrons: 35 },
    Element { z: 47, symbol: "Ag", name: "Silver", neutrons: 61 },
    Element { z: 79, symbol: "Au", name: "Gold", neutrons: 118 },
    Element { z: 82, symbol: "Pb", name: "Lead", neutrons: 126 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carbon() {
        let e = PeriodicTable::get(6).unwrap();
        assert_eq!(e.name, "Carbon");
        assert_eq!(e.neutrons, 6);
    }

    #[test]
    fn by_symbol() {
        let e = PeriodicTable::by_symbol("Au").unwrap();
        assert_eq!(e.z, 79);
        assert_eq!(e.name, "Gold");
    }

    #[test]
    fn differential() {
        let e = PeriodicTable::get(1).unwrap();
        assert_eq!(e.symbol, "H");
        assert_eq!(e.neutrons, 0);
        assert!(PeriodicTable::get(200).is_none());
    }
}
