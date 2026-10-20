//! Materials Lab: interactive periodic table, element detail card,
//! composition builder (formula or protons/neutrons/electrons), derived
//! render (PBR) and simulation properties, and the material library that
//! the 3D viewport draws with. Everything mutating goes through `apply`
//! (the AppCommand bus).

use std::collections::BTreeMap;

use crate::commands::AppCommand;
use crate::theme::{self, Palette, SP_SM, SP_XS};
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use egui_phosphor::regular as ic;
use obs_rust_bitstream::element_data::{self, Category, ElementData};
use obs_rust_bitstream::element_props;
use obs_rust_bitstream::material_env::P_ATM;
use obs_rust_bitstream::material_pbr::{parse_formula, preset, preset_profile, presets, profile, Composition, Pbr, Profile};
use obs_rust_bitstream::material_registry::MaterialRegistry;
use obs_rust_bitstream::materials_lab::Atom;
use obs_rust_bitstream::mesh3d::{shade_surface, Surface};
use obs_rust_bitstream::periodic_table::PeriodicTable;

/// Saved materials: the library registry plus the full profile per name.
#[derive(Clone, Debug, Default)]
pub struct MatLib {
    pub registry: MaterialRegistry,
    pub profiles: BTreeMap<String, Profile>,
}

impl MatLib {
    /// Save a profile under `name` (or its own name), updating the registry material.
    pub fn save(&mut self, prof: Profile, name: Option<&str>) -> Result<String, String> {
        let name = name.map(str::trim).filter(|s| !s.is_empty()).map(String::from).unwrap_or_else(|| prof.name.clone());
        if name.is_empty() {
            return Err("material name is empty".into());
        }
        if self.registry.get(&name).is_err() {
            self.registry.add(&name).map_err(|e| format!("registry: {:?}", e))?;
        }
        let m = self.registry.get_mut(&name).map_err(|e| format!("registry: {:?}", e))?;
        prof.pbr.apply(m);
        self.profiles.insert(name.clone(), prof);
        Ok(name)
    }

    /// A saved material name (case-insensitive), a preset key ("copper"), or a formula ("Fe2O3").
    pub fn resolve(&mut self, key: &str) -> Result<String, String> {
        let k = key.trim();
        if let Some(n) = self.profiles.keys().find(|n| n.eq_ignore_ascii_case(k)) {
            return Ok(n.clone());
        }
        if let Some(p) = preset(k) {
            return self.save(preset_profile(&p), None);
        }
        let c = parse_formula(k).map_err(|e| format!("'{}' is not a saved material, preset or formula ({})", k, e))?;
        self.save(profile(&c), None)
    }

    pub fn surface(&self, name: &str) -> Option<Surface> {
        self.profiles.get(name).map(|p| surface_of(&p.pbr))
    }
}

pub fn surface_of(p: &Pbr) -> Surface {
    Surface { base: p.base_color, metalness: p.metalness, roughness: p.roughness, ior: p.ior }
}

#[derive(Clone)]
pub struct MatSnap {
    lib: MatLib,
    comp: Composition,
    formula: String,
    built: Option<Profile>,
    atom_note: Option<Result<String, String>>,
}

pub struct MaterialsState {
    pub lib: MatLib,
    pub search: String,
    pub category: Option<Category>,
    pub hover: Option<u32>,
    pub selected: Option<u32>,
    pub comp: Composition,
    pub formula: String,
    pub builder_tab: usize,
    pub atom: [u32; 3],
    pub save_name: String,
    pub built: Option<Profile>,
    pub error: Option<String>,
    pub atom_note: Option<Result<String, String>>,
    preview: Option<(Pbr, egui::TextureHandle)>,
}

impl MaterialsState {
    pub fn new() -> Self {
        Self {
            lib: MatLib::default(),
            search: String::new(),
            category: None,
            hover: None,
            selected: Some(79),
            comp: Composition::default(),
            formula: String::new(),
            builder_tab: 0,
            atom: [6, 6, 6],
            save_name: String::new(),
            built: None,
            error: None,
            atom_note: None,
            preview: None,
        }
    }

    pub fn snap(&self) -> MatSnap {
        MatSnap { lib: self.lib.clone(), comp: self.comp.clone(), formula: self.formula.clone(), built: self.built.clone(), atom_note: self.atom_note.clone() }
    }

    pub fn restore(&mut self, s: MatSnap) {
        self.lib = s.lib;
        self.comp = s.comp;
        self.formula = s.formula;
        self.built = s.built;
        self.atom_note = s.atom_note;
        self.error = None;
    }

    pub fn context(&self) -> String {
        let sel = self.selected.and_then(element_data::get).map(|e| format!("{} ({})", e.name, e.symbol)).unwrap_or_else(|| "none".into());
        let built = self.built.as_ref().map(|p| format!("{} [{}] metalness {:.2} roughness {:.2} IOR {:.3} density {:.0} kg/m³, charge balanced {}", p.name, p.formula, p.pbr.metalness, p.pbr.roughness, p.pbr.ior, p.pbr.density, p.analysis.charge_balanced)).unwrap_or_else(|| "nothing built".into());
        let lib: Vec<&str> = self.lib.profiles.keys().map(|s| s.as_str()).collect();
        let pre: Vec<&str> = presets().iter().map(|p| p.key).collect();
        format!("Materials Lab: selected element {}; builder composition '{}'; built: {}; library: [{}]; presets: [{}]. Use materials_build to build a formula and material_assign to put a material on a 3D object.", sel, self.comp.formula(), built, lib.join(", "), pre.join(", "))
    }

    fn set_built(&mut self, p: Profile) -> String {
        let a = &p.analysis;
        let mut msg = format!("built {} ({}, {:.3} g/mol)", p.name, p.formula, a.molar_mass);
        if !a.charge_balanced {
            msg.push_str(": warning, no charge-balanced oxidation states");
        }
        if !a.all_stable {
            msg.push_str(": warning, an unstable nucleus");
        }
        self.save_name = p.name.clone();
        self.built = Some(p);
        self.error = None;
        msg
    }

    /// Execute a materials command. `None` = not a materials command.
    pub fn apply(&mut self, cmd: &AppCommand) -> Option<Result<String, String>> {
        use AppCommand as C;
        Some(match cmd {
            C::MaterialsSelect { element } => match element_data::find(element) {
                Some(e) => {
                    self.selected = Some(e.z);
                    Ok(format!("selected {} ({})", e.name, e.symbol))
                }
                None => Err(format!("unknown element '{}'", element)),
            },
            C::MaterialsSearch { query } => {
                self.search = query.clone();
                let n = element_data::all().iter().filter(|e| matches(e, query)).count();
                Ok(format!("{} element{} match '{}'", n, if n == 1 { "" } else { "s" }, query))
            }
            C::MaterialsAdd { element, count } => match element_data::find(element) {
                Some(e) => {
                    let n = count.unwrap_or(1);
                    if n == 0 || n > 999 {
                        return Some(Err("count must be 1-999".into()));
                    }
                    self.comp.add(e.z, n);
                    self.formula = self.comp.formula();
                    Ok(format!("composition {}", self.formula))
                }
                None => Err(format!("unknown element '{}'", element)),
            },
            C::MaterialsRemove { element } => match element_data::find(element) {
                Some(e) => {
                    let before = self.comp.0.len();
                    if let Some(i) = self.comp.0.iter().position(|x| x.0 == e.z) {
                        if self.comp.0[i].1 > 1 {
                            self.comp.0[i].1 -= 1;
                        } else {
                            self.comp.0.remove(i);
                        }
                    }
                    if before == 0 {
                        return Some(Err("composition is empty".into()));
                    }
                    self.formula = self.comp.formula();
                    Ok(format!("composition {}", if self.formula.is_empty() { "empty" } else { &self.formula }))
                }
                None => Err(format!("unknown element '{}'", element)),
            },
            C::MaterialsClear => {
                self.comp = Composition::default();
                self.formula.clear();
                self.built = None;
                self.error = None;
                Ok("composition cleared".into())
            }
            C::MaterialsBuild { formula } => {
                let src = formula.clone().unwrap_or_else(|| if self.formula.trim().is_empty() { self.comp.formula() } else { self.formula.clone() });
                match parse_formula(&src) {
                    Ok(c) => {
                        self.comp = c;
                        self.formula = self.comp.formula();
                        let p = profile(&self.comp);
                        Ok(self.set_built(p))
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        Err(format!("can't build '{}': {}", src, e))
                    }
                }
            }
            C::MaterialsPreset { preset: key } => match preset(key) {
                Some(pr) => {
                    self.comp = parse_formula(pr.formula).unwrap_or_default();
                    self.formula = pr.formula.to_string();
                    Ok(self.set_built(preset_profile(&pr)))
                }
                None => Err(format!("unknown preset '{}' ({})", key, presets().iter().map(|p| p.key).collect::<Vec<_>>().join(", "))),
            },
            C::MaterialsBuildAtom { protons, neutrons, electrons } => {
                let r = build_atom(*protons, *neutrons, *electrons);
                self.atom = [*protons, *neutrons, *electrons];
                match r {
                    Ok((p, note)) => {
                        self.comp = Composition(vec![(*protons, 1)]);
                        self.formula = self.comp.formula();
                        let msg = self.set_built(p);
                        self.atom_note = Some(Ok(note.clone()));
                        Ok(format!("{}; {}", msg, note))
                    }
                    Err(e) => {
                        self.atom_note = Some(Err(e.clone()));
                        Err(e)
                    }
                }
            }
            C::MaterialsSave { name } => match self.built.clone() {
                Some(p) => {
                    let nm = name.clone().or_else(|| Some(self.save_name.clone()).filter(|s| !s.trim().is_empty()));
                    self.lib.save(p, nm.as_deref()).map(|n| format!("saved '{}' to the material library ({} materials)", n, self.lib.profiles.len()))
                }
                None => Err("build a material first".into()),
            },
            _ => return None,
        })
    }

    // ---- UI -----------------------------------------------------------

    pub fn tools(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        theme::card(ui, p, "Search", |ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Symbol, name or Z").desired_width(f32::INFINITY));
            if r.changed() {
                bus.push(AppCommand::MaterialsSearch { query: self.search.clone() });
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(e) = element_data::all().iter().find(|e| matches(e, &self.search)) {
                    bus.push(AppCommand::MaterialsSelect { element: e.symbol.into() });
                }
            }
            let n = element_data::all().iter().filter(|e| matches(e, &self.search) && self.category.map(|c| c == e.category).unwrap_or(true)).count();
            ui.label(theme::muted(p, &format!("{} of 118 elements", n)));
        });
        theme::card(ui, p, "Categories", |ui| {
            for c in Category::ALL {
                let on = self.category == Some(c);
                let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), egui::Sense::click());
                if on || resp.hovered() {
                    ui.painter().rect_filled(r, Rounding::same(4.0), p.surface_hi);
                }
                let sw = Rect::from_min_size(r.left_center() + Vec2::new(4.0, -5.0), Vec2::splat(10.0));
                ui.painter().rect_filled(sw, Rounding::same(2.0), cat_colour(c));
                ui.painter().text(sw.right_center() + Vec2::new(8.0, 0.0), egui::Align2::LEFT_CENTER, c.label(), egui::FontId::proportional(12.0), if on { p.accent } else { p.text });
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.category = if on { None } else { Some(c) };
                }
            }
        });
        theme::card(ui, p, "Presets", |ui| {
            for pr in presets() {
                let r = ui.add_sized([ui.available_width(), 24.0], theme::toggle_button(p, &format!("{}  {}", pr.name, pr.formula), self.built.as_ref().and_then(|b| b.preset) == Some(pr.key)));
                if r.clicked() {
                    bus.push(AppCommand::MaterialsPreset { preset: pr.key.into() });
                }
            }
        });
    }

    pub fn center(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(SP_SM);
            ui.horizontal(|ui| {
                ui.add_space(SP_SM);
                ui.label(egui::RichText::new(format!("{}  Periodic Table", ic::ATOM)).size(16.0).strong().color(p.text));
                ui.label(theme::muted(p, "Click an element for details. Double-click adds it to the composition."));
            });
            ui.add_space(SP_XS);
            self.table(ui, p, bus);
            ui.add_space(SP_SM);
            let shown = self.hover.or(self.selected);
            if let Some(e) = shown.and_then(element_data::get) {
                self.detail(ui, p, e, bus);
            }
        });
    }

    fn table(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>) {
        let avail = ui.available_width() - 2.0 * SP_SM;
        let gap = 3.0;
        let cw = ((avail - 17.0 * gap) / 18.0).clamp(26.0, 64.0);
        let ch = cw * 1.12;
        let rows_h = 7.0 * (ch + gap) + 0.45 * ch + 2.0 * (ch + gap);
        let (area, _) = ui.allocate_exact_size(Vec2::new(avail + 2.0 * SP_SM, rows_h + 22.0), egui::Sense::hover());
        let origin = area.min + Vec2::new(SP_SM + (avail - (18.0 * cw + 17.0 * gap)).max(0.0) / 2.0, 0.0);
        let cell = |col: u8, row: u8| -> Rect {
            let y = if row >= 9 { 7.0 * (ch + gap) + 0.45 * ch + (row as f32 - 9.0) * (ch + gap) } else { (row as f32 - 1.0) * (ch + gap) };
            Rect::from_min_size(origin + Vec2::new((col as f32 - 1.0) * (cw + gap), y), Vec2::new(cw, ch))
        };
        let painter = ui.painter_at(area);
        let mut hover = None;
        // f-block placeholders in group 3
        for (row, label) in [(6u8, "57–71"), (7u8, "89–103")] {
            let r = cell(3, row);
            painter.rect(r, Rounding::same(4.0), p.surface, Stroke::new(1.0_f32, p.border));
            painter.text(r.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional((cw * 0.2).clamp(8.0, 11.0)), p.muted);
        }
        // group numbers
        for g in 1..=18u8 {
            let r = cell(g, 1);
            let y = if matches!(g, 1 | 18) { r.top() } else if matches!(g, 2 | 13..=17) { cell(g, 2).top() } else { cell(g, 4).top() };
            painter.text(Pos2::new(r.center().x, y - 1.0), egui::Align2::CENTER_BOTTOM, g.to_string(), egui::FontId::proportional(9.0), p.muted);
        }
        for e in element_data::all() {
            let r = cell(e.col, e.row);
            let id = ui.id().with(("pt", e.z));
            let resp = ui.interact(r, id, egui::Sense::click());
            let lit = matches(e, &self.search) && self.category.map(|c| c == e.category).unwrap_or(true);
            let cc = cat_colour(e.category);
            let sel = self.selected == Some(e.z);
            let mut fill = theme::mix(p.surface, cc, if lit { 0.42 } else { 0.08 });
            if resp.hovered() {
                fill = theme::mix(fill, cc, 0.35);
                hover = Some(e.z);
            }
            let in_comp = self.comp.0.iter().any(|x| x.0 == e.z);
            let stroke = if sel { Stroke::new(2.0_f32, p.accent) } else if in_comp { Stroke::new(1.5_f32, theme::mix(p.accent, cc, 0.3)) } else { Stroke::new(1.0_f32, theme::mix(p.border, cc, if lit { 0.5 } else { 0.15 })) };
            painter.rect(r, Rounding::same(4.0), fill, stroke);
            let tc = if lit { p.text } else { p.muted.gamma_multiply(0.6) };
            painter.text(r.left_top() + Vec2::new(3.0, 2.0), egui::Align2::LEFT_TOP, e.z.to_string(), egui::FontId::proportional((cw * 0.19).clamp(7.0, 10.0)), if lit { p.muted } else { tc });
            painter.text(r.center() - Vec2::new(0.0, ch * 0.04), egui::Align2::CENTER_CENTER, e.symbol, egui::FontId::proportional((cw * 0.36).clamp(10.0, 20.0)), if sel { p.accent } else { tc });
            if cw >= 44.0 {
                painter.text(r.center_bottom() - Vec2::new(0.0, 3.0), egui::Align2::CENTER_BOTTOM, short(e.name, cw), egui::FontId::proportional(8.5), if lit { p.muted } else { tc });
            }
            if in_comp {
                let n = self.comp.0.iter().find(|x| x.0 == e.z).map(|x| x.1).unwrap_or(0);
                let b = Rect::from_min_size(r.right_top() + Vec2::new(-14.0, 2.0), Vec2::new(12.0, 11.0));
                painter.rect_filled(b, Rounding::same(3.0), p.accent);
                painter.text(b.center(), egui::Align2::CENTER_CENTER, n.to_string(), egui::FontId::proportional(8.5), p.accent_text);
            }
            if resp.double_clicked() {
                bus.push(AppCommand::MaterialsAdd { element: e.symbol.into(), count: Some(1) });
            } else if resp.clicked() {
                bus.push(AppCommand::MaterialsSelect { element: e.symbol.into() });
            }
            resp.on_hover_cursor(egui::CursorIcon::PointingHand);
        }
        self.hover = hover;
        // legend line
        let ly = area.bottom() - 12.0;
        painter.text(Pos2::new(origin.x, ly), egui::Align2::LEFT_CENTER, "Data: PubChem (NIH), mendeleev, pymatgen. Lanthanides and actinides shown below the main table.", egui::FontId::proportional(10.0), p.muted);
    }

    fn detail(&self, ui: &mut egui::Ui, p: &Palette, e: &ElementData, bus: &mut Vec<AppCommand>) {
        let pr = element_props::props(e.z);
        let env = obs_rust_bitstream::material_env::element_env(e.z);
        let cc = cat_colour(e.category);
        egui::Frame::none().fill(p.panel).stroke(Stroke::new(1.0_f32, p.line)).rounding(Rounding::same(8.0)).inner_margin(egui::Margin::same(12.0)).outer_margin(egui::Margin::symmetric(SP_SM, 0.0)).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // big tile
                let (r, _) = ui.allocate_exact_size(Vec2::new(112.0, 128.0), egui::Sense::hover());
                ui.painter().rect(r, Rounding::same(8.0), theme::mix(p.surface, cc, 0.45), Stroke::new(2.0_f32, p.accent));
                ui.painter().text(r.left_top() + Vec2::new(8.0, 6.0), egui::Align2::LEFT_TOP, e.z.to_string(), egui::FontId::proportional(14.0), p.text);
                ui.painter().text(r.center() - Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, e.symbol, egui::FontId::proportional(40.0), p.accent);
                ui.painter().text(r.center_bottom() - Vec2::new(0.0, 22.0), egui::Align2::CENTER_BOTTOM, e.name, egui::FontId::proportional(13.0), p.text);
                ui.painter().text(r.center_bottom() - Vec2::new(0.0, 7.0), egui::Align2::CENTER_BOTTOM, format!("{:.4}", e.mass), egui::FontId::proportional(11.0), p.muted);
                ui.add_space(SP_SM);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(e.name).size(18.0).strong().color(p.text));
                        theme::pill(ui, p, cc, e.category.label());
                        if pr.map(|x| x.radioactive).unwrap_or(false) {
                            theme::pill(ui, p, p.warn, "radioactive");
                        }
                    });
                    if let Some(x) = pr.filter(|x| !x.appearance.is_empty()) {
                        ui.label(theme::muted(p, x.appearance));
                    }
                    ui.horizontal(|ui| {
                        if ui.add(theme::primary_button(p, &format!("{} Add to composition", ic::PLUS))).clicked() {
                            bus.push(AppCommand::MaterialsAdd { element: e.symbol.into(), count: Some(1) });
                        }
                        if ui.add(theme::toggle_button(p, &format!("{} Build as material", ic::FLASK), false)).clicked() {
                            bus.push(AppCommand::MaterialsBuild { formula: Some(e.symbol.into()) });
                        }
                    });
                });
            });
            ui.add_space(SP_SM);
            let opt = |v: Option<f32>, unit: &str, prec: usize| v.map(|x| format!("{:.*} {}", prec, x, unit)).unwrap_or_else(|| "unknown".into());
            let sci = |v: Option<f64>, unit: &str| v.map(|x| format!("{:.3e} {}", x, unit)).unwrap_or_else(|| "unknown".into());
            ui.columns(3, |cols| {
                let k = |ui: &mut egui::Ui, a: &str, b: String| {
                    if b.chars().count() > 30 {
                        ui.label(egui::RichText::new(a).color(p.muted).size(11.5));
                        ui.add(egui::Label::new(egui::RichText::new(b).color(p.text).size(11.0)).wrap());
                    } else {
                        theme::kv(ui, p, a, b)
                    }
                };
                let c = &mut cols[0];
                c.label(theme::section(p, "ATOMIC"));
                k(c, "Atomic number", e.z.to_string());
                k(c, "Atomic mass", format!("{} u", e.mass));
                let pt = PeriodicTable::get(e.z);
                if let Some(pt) = pt {
                    k(c, "Common isotope", format!("{}{} ({}p {}n)", e.z + pt.neutrons, e.symbol, e.z, pt.neutrons));
                }
                k(c, "Period / group", format!("{} / {}", e.period, e.group.map(|g| g.to_string()).unwrap_or_else(|| "f-block".into())));
                k(c, "Block", e.block.to_string());
                k(c, "Configuration", e.config.to_string());
                if let Some(x) = pr {
                    k(c, "Shells", x.shells.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" · "));
                }
                k(c, "Oxidation states", e.oxidation.to_string());
                k(c, "Electronegativity", e.electronegativity.map(|x| format!("{:.2} (Pauling)", x)).unwrap_or_else(|| "unknown".into()));
                k(c, "1st ionization", opt(e.ionization_ev, "eV", 3));
                k(c, "Electron affinity", opt(e.electron_affinity_ev, "eV", 3));
                k(c, "Radius (vdW)", opt(e.radius_pm, "pm", 0));
                k(c, "Covalent radius", opt(pr.and_then(|x| x.covalent_radius_pm), "pm", 0));
                let c = &mut cols[1];
                c.label(theme::section(p, "PHYSICAL"));
                k(c, "State (STP)", e.state.to_string());
                if let Some(env) = &env {
                    k(c, "Phase at 20 °C", env.phase_at(293.15, P_ATM).label().to_string());
                }
                k(c, "Melting point", e.melting_k.map(|t| format!("{:.2} K ({:.1} °C)", t, t - 273.15)).unwrap_or_else(|| "unknown".into()));
                k(c, "Boiling point", e.boiling_k.map(|t| format!("{:.2} K ({:.1} °C)", t, t - 273.15)).unwrap_or_else(|| "unknown".into()));
                k(c, "Density", e.density.map(|d| if e.state.contains("Gas") { format!("{:.4} g/L", d * 1000.0) } else { format!("{} g/cm³", d) }).unwrap_or_else(|| "unknown".into()));
                k(c, "Specific heat", opt(pr.and_then(|x| x.specific_heat), "J/(g·K)", 3));
                k(c, "Thermal cond.", opt(pr.and_then(|x| x.thermal_conductivity), "W/(m·K)", 1));
                k(c, "Resistivity", sci(pr.and_then(|x| x.electrical_resistivity), "Ω·m"));
                k(c, "Heat of fusion", opt(pr.and_then(|x| x.fusion_heat_kj_mol), "kJ/mol", 2));
                k(c, "Heat of vaporization", opt(pr.and_then(|x| x.vaporization_heat_kj_mol), "kJ/mol", 1));
                k(c, "Thermal expansion", sci(pr.and_then(|x| x.thermal_expansion.map(|v| v as f64)), "1/K"));
                let c = &mut cols[2];
                c.label(theme::section(p, "MECHANICAL · NUCLEAR"));
                k(c, "Crystal structure", pr.map(|x| x.crystal.to_string()).unwrap_or_default());
                k(c, "Young's modulus", opt(pr.and_then(|x| x.youngs_gpa), "GPa", 0));
                k(c, "Bulk modulus", opt(pr.and_then(|x| x.bulk_gpa), "GPa", 0));
                k(c, "Mohs hardness", opt(pr.and_then(|x| x.mohs), "", 1));
                k(c, "Speed of sound", opt(pr.and_then(|x| x.sound_speed), "m/s", 0));
                let iso = pr.map(|x| x.stable_isotopes).filter(|s| !s.is_empty()).unwrap_or("none");
                k(c, "Stable isotopes", iso.to_string());
                if let Some(x) = pr.filter(|x| x.longest_half_life_s.is_some()) {
                    k(c, "Longest-lived", format!("{}{}: {}", x.longest_lived_isotope.unwrap_or(0), e.symbol, half_life(x.longest_half_life_s.unwrap_or(0.0))));
                }
                k(c, "Discovered", match pr.map(|x| x.discoverers).filter(|d| !d.is_empty()) {
                    Some(d) => format!("{} ({})", e.discovered, d),
                    None => e.discovered.to_string(),
                });
                if let Some(env) = &env {
                    k(c, "Reactivity", format!("{:?}{}", env.reactivity, if env.oxidiser { ", oxidiser" } else { "" }));
                    k(c, "Flammability", format!("{:?}", env.flammability));
                }
            });
        });
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui, p: &Palette, bus: &mut Vec<AppCommand>, selected_obj: Option<&str>) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            theme::card(ui, p, "Composition Builder", |ui| {
                // width must not exceed the panel or the resizable panel grows every frame
                if let Some(t) = theme::segmented(ui, p, "mat_builder_tab", &["Formula", "Atom (p/n/e)"], self.builder_tab, ((ui.available_width() - 12.0) / 2.0).floor().max(60.0)) {
                    self.builder_tab = t;
                }
                ui.add_space(SP_XS);
                if self.builder_tab == 0 {
                    ui.horizontal(|ui| {
                        let r = ui.add(egui::TextEdit::singleline(&mut self.formula).hint_text("e.g. H2O, Fe2O3, Ca(OH)2").desired_width(ui.available_width() - 64.0));
                        let go = ui.add(theme::primary_button(p, "Build")).clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                        if go {
                            bus.push(AppCommand::MaterialsBuild { formula: Some(self.formula.clone()) });
                        }
                    });
                    if let Some(e) = &self.error {
                        ui.label(egui::RichText::new(format!("{} {}", ic::WARNING, e)).color(p.danger));
                    }
                    ui.add_space(SP_XS);
                    if self.comp.is_empty() {
                        ui.label(theme::muted(p, "Double-click elements in the table, or type a formula."));
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            for &(z, n) in self.comp.0.clone().iter() {
                                if let Some(e) = element_data::get(z) {
                                    ui.group(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        ui.label(egui::RichText::new(format!("{}×{}", e.symbol, n)).color(p.text).strong());
                                        if ui.small_button("−").on_hover_text("Remove one").clicked() {
                                            bus.push(AppCommand::MaterialsRemove { element: e.symbol.into() });
                                        }
                                        if ui.small_button("+").on_hover_text("Add one").clicked() {
                                            bus.push(AppCommand::MaterialsAdd { element: e.symbol.into(), count: Some(1) });
                                        }
                                    });
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.add(theme::primary_button(p, &format!("{} Build {}", ic::FLASK, self.comp.formula()))).clicked() {
                                bus.push(AppCommand::MaterialsBuild { formula: Some(self.comp.formula()) });
                            }
                            if ui.add(theme::toggle_button(p, "Clear", false)).clicked() {
                                bus.push(AppCommand::MaterialsClear);
                            }
                        });
                    }
                } else {
                    let labels = ["Protons", "Neutrons", "Electrons"];
                    for (i, l) in labels.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_sized([70.0, 20.0], egui::Label::new(theme::muted(p, l)));
                            ui.add(egui::DragValue::new(&mut self.atom[i]).range(0..=300).speed(0.2));
                        });
                    }
                    let [z, n, el] = self.atom;
                    let name = element_data::get(z).map(|e| format!("{}-{}", e.name, z + n)).unwrap_or_else(|| "no element".into());
                    ui.label(theme::muted(p, &format!("{}  ·  charge {:+}", name, z as i64 - el as i64)));
                    if ui.add(theme::primary_button(p, &format!("{} Synthesize", ic::ATOM))).clicked() {
                        bus.push(AppCommand::MaterialsBuildAtom { protons: z, neutrons: n, electrons: el });
                    }
                    match &self.atom_note {
                        Some(Ok(t)) => {
                            ui.label(egui::RichText::new(format!("{} {}", ic::CHECK_CIRCLE, t)).color(p.success));
                        }
                        Some(Err(e)) => {
                            ui.label(egui::RichText::new(format!("{} {}", ic::WARNING, e)).color(p.danger));
                        }
                        None => {}
                    }
                }
            });
            if let Some(b) = self.built.clone() {
                theme::card(ui, p, "Analysis", |ui| {
                    let a = &b.analysis;
                    theme::kv(ui, p, "Material", b.name.clone());
                    theme::kv(ui, p, "Formula", b.formula.clone());
                    theme::kv(ui, p, "Molar mass", format!("{:.3} g/mol", a.molar_mass));
                    ui.horizontal_wrapped(|ui| {
                        theme::pill(ui, p, if a.charge_balanced { p.success } else { p.danger }, if a.charge_balanced { "charge balanced" } else { "charge unbalanced" });
                        theme::pill(ui, p, if a.all_stable { p.success } else { p.warn }, if a.all_stable { "nuclei stable" } else { "unstable nucleus" });
                        if a.metallic_bond {
                            theme::pill(ui, p, p.accent, "metallic");
                        }
                    });
                    if let Some(ox) = &a.oxidation {
                        theme::kv(ui, p, "Oxidation states", ox.iter().map(|(s, o)| format!("{} {:+}", s, o)).collect::<Vec<_>>().join(", "));
                    }
                    for (s, z, n, ok) in &a.nuclei {
                        theme::kv(ui, p, &format!("{}-{}", s, z + n), format!("{}p {}n  {}", z, n, if *ok { "stable" } else { "outside N/Z band" }));
                    }
                });
                theme::card(ui, p, "Render (PBR)", |ui| {
                    self.ensure_preview(ui.ctx(), &b.pbr);
                    ui.horizontal_top(|ui| {
                        if let Some((_, t)) = &self.preview {
                            ui.image((t.id(), Vec2::splat(96.0)));
                        }
                        ui.vertical(|ui| {
                            let c = b.pbr.base_color;
                            let hex = format!("#{:02X}{:02X}{:02X}", lin8(c[0]), lin8(c[1]), lin8(c[2]));
                            theme::kv(ui, p, "Base colour", hex);
                            theme::kv(ui, p, "Metalness", format!("{:.2}", b.pbr.metalness));
                            theme::kv(ui, p, "Roughness", format!("{:.2}", b.pbr.roughness));
                            theme::kv(ui, p, "IOR", format!("{:.3}", b.pbr.ior));
                            theme::kv(ui, p, "Density", format!("{:.0} kg/m³", b.pbr.density));
                            if b.pbr.transmission > 0.0 {
                                theme::kv(ui, p, "Transmission", format!("{:.2}", b.pbr.transmission));
                            }
                        });
                    });
                    for n in &b.notes {
                        ui.label(theme::muted(p, &format!("• {}", n)));
                    }
                });
                theme::card(ui, p, "Simulation (env)", |ui| {
                    let e = &b.env;
                    let t = |v: Option<f32>| v.map(|x| format!("{:.1} K", x)).unwrap_or_else(|| "unknown".into());
                    let ph = e.phase_at(293.15, P_ATM);
                    theme::kv(ui, p, "Phase at 20 °C", ph.label().to_string());
                    theme::kv(ui, p, if e.sublimes { "Sublimes" } else { "Melts" }, if e.sublimes { t(e.boiling_k) } else { t(e.melting_k) });
                    if !e.sublimes {
                        theme::kv(ui, p, "Boils", t(e.boiling_k));
                    }
                    theme::kv(ui, p, "Specific heat", e.specific_heat.map(|c| format!("{:.0} J/(kg·K)", c)).unwrap_or_else(|| "unknown".into()));
                    theme::kv(ui, p, "Conductivity", e.thermal_conductivity.map(|c| format!("{:.2} W/(m·K)", c)).unwrap_or_else(|| "unknown".into()));
                    theme::kv(ui, p, "Restitution / friction", format!("{:.2} / {:.2}", e.restitution, e.friction_kinetic));
                    theme::kv(ui, p, "Flammability", format!("{:?}", e.flammability));
                    theme::kv(ui, p, "Reactivity", format!("{:?}", e.reactivity));
                    if e.radioactive {
                        theme::kv(ui, p, "Radioactive", e.half_life_s.map(half_life).unwrap_or_else(|| "yes".into()));
                    }
                });
                theme::card(ui, p, "Save & assign", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.save_name).hint_text("Material name").desired_width(ui.available_width() - 64.0));
                        if ui.add(theme::primary_button(p, "Save")).clicked() {
                            bus.push(AppCommand::MaterialsSave { name: Some(self.save_name.clone()) });
                        }
                    });
                    let label = match selected_obj {
                        Some(o) => format!("{} Assign to '{}' in 3D", ic::CUBE, o),
                        None => format!("{} Assign (select a 3D mesh first)", ic::CUBE),
                    };
                    let r = ui.add_enabled(selected_obj.is_some(), theme::toggle_button(p, &label, false));
                    if r.clicked() {
                        let name = if self.save_name.trim().is_empty() { b.name.clone() } else { self.save_name.trim().to_string() };
                        if !self.lib.profiles.contains_key(&name) {
                            bus.push(AppCommand::MaterialsSave { name: Some(name.clone()) });
                        }
                        bus.push(AppCommand::MaterialAssign { object: None, material: name });
                    }
                });
            }
            theme::card(ui, p, &format!("Library ({})", self.lib.profiles.len()), |ui| {
                if self.lib.profiles.is_empty() {
                    ui.label(theme::muted(p, "Saved materials appear here and in the 3D inspector."));
                }
                for (name, pr) in &self.lib.profiles {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), egui::Sense::hover());
                        let c = pr.pbr.base_color;
                        ui.painter().rect(r, Rounding::same(3.0), Color32::from_rgb(lin8(c[0]), lin8(c[1]), lin8(c[2])), Stroke::new(1.0_f32, p.line));
                        ui.label(egui::RichText::new(name).color(p.text));
                        ui.label(theme::muted(p, &pr.formula));
                    });
                }
            });
        });
    }

    fn ensure_preview(&mut self, ctx: &egui::Context, pbr: &Pbr) {
        if self.preview.as_ref().map(|(p, _)| p == pbr).unwrap_or(false) {
            return;
        }
        let img = sphere_preview(pbr, 96);
        match &mut self.preview {
            Some((p, t)) => {
                t.set(img, egui::TextureOptions::LINEAR);
                *p = *pbr;
            }
            None => self.preview = Some((*pbr, ctx.load_texture("mat_preview", img, egui::TextureOptions::LINEAR))),
        }
    }
}

/// A lit sphere rendered with the same shader as the 3D viewport.
pub fn sphere_preview(pbr: &Pbr, n: usize) -> egui::ColorImage {
    let s = surface_of(pbr);
    let mut px = vec![Color32::TRANSPARENT; n * n];
    let light = [-1.2, 1.4, 2.0];
    let eye = [0.0, 0.0, 3.0];
    for y in 0..n {
        for x in 0..n {
            let u = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let v = 1.0 - (y as f32 + 0.5) / n as f32 * 2.0;
            let r2 = u * u + v * v;
            if r2 > 1.0 {
                continue;
            }
            let z = (1.0 - r2).sqrt();
            let nrm = [u, v, z];
            let to_l = [light[0] - u, light[1] - v, light[2] - z];
            let to_e = [eye[0] - u, eye[1] - v, eye[2] - z];
            let c = shade_surface(nrm, to_l, to_e, &s);
            let a = ((1.0 - r2.sqrt()) * n as f32 * 0.5).clamp(0.0, 1.0);
            px[y * n + x] = Color32::from_rgba_unmultiplied(c[0], c[1], c[2], (a * 255.0) as u8);
        }
    }
    egui::ColorImage { size: [n, n], pixels: px }
}

fn lin8(c: f32) -> u8 {
    obs_rust_bitstream::mesh3d::linear_to_srgb(c)
}

/// (profile, note) for an atom built from protons/neutrons/electrons.
pub fn build_atom(protons: u32, neutrons: u32, electrons: u32) -> Result<(Profile, String), String> {
    let e = element_data::get(protons).ok_or_else(|| if protons == 0 { "an atom needs at least one proton".to_string() } else { format!("no element has {} protons (1-118)", protons) })?;
    let atom = Atom::new(protons, neutrons, electrons);
    if !atom.is_neutral() {
        let q = protons as i64 - electrons as i64;
        return Err(format!("{}-{} with {} electrons is an ion (charge {:+}); a bulk material needs neutral atoms", e.name, protons + neutrons, electrons, q));
    }
    if !atom.is_stable() {
        return Err(format!("{}-{} is unstable: N/Z = {:.2} is outside the lab's stability band (0.8-1.6)", e.name, protons + neutrons, neutrons as f32 / protons as f32));
    }
    let mut p = profile(&Composition(vec![(protons, 1)]));
    if let Some(pr) = p.preset.and_then(preset) {
        p = preset_profile(&pr);
    }
    let note = format!("{}-{}: neutral, stable, binding ≈ {:.2} MeV/nucleon (lab model)", e.name, atom.mass_number(), atom.binding_per_nucleon());
    Ok((p, note))
}

pub fn matches(e: &ElementData, q: &str) -> bool {
    let q = q.trim();
    if q.is_empty() {
        return true;
    }
    if let Ok(z) = q.parse::<u32>() {
        return e.z == z;
    }
    let l = q.to_lowercase();
    e.symbol.to_lowercase() == l || e.name.to_lowercase().contains(&l) || (l == "aluminium" && e.z == 13)
}

fn short(name: &str, cw: f32) -> String {
    let max = ((cw - 6.0) / 4.6) as usize;
    if name.chars().count() <= max {
        name.to_string()
    } else {
        let s: String = name.chars().take(max.saturating_sub(1)).collect();
        format!("{}.", s)
    }
}

pub fn half_life(s: f64) -> String {
    let y = 365.25 * 86400.0;
    if s >= y * 1e6 {
        format!("{:.3e} years", s / y)
    } else if s >= y {
        format!("{:.4} years", s / y)
    } else if s >= 86400.0 {
        format!("{:.3} days", s / 86400.0)
    } else if s >= 1.0 {
        format!("{:.3} s", s)
    } else {
        format!("{:.3e} s", s)
    }
}

pub fn cat_colour(c: Category) -> Color32 {
    match c {
        Category::AlkaliMetal => Color32::from_rgb(0xC0, 0x5E, 0x4F),
        Category::AlkalineEarthMetal => Color32::from_rgb(0xD0, 0x8A, 0x45),
        Category::TransitionMetal => Color32::from_rgb(0xC2, 0xA2, 0x58),
        Category::PostTransitionMetal => Color32::from_rgb(0x72, 0xA0, 0x8A),
        Category::Metalloid => Color32::from_rgb(0x5C, 0x9F, 0xAE),
        Category::Nonmetal => Color32::from_rgb(0x62, 0x8A, 0xC8),
        Category::Halogen => Color32::from_rgb(0x91, 0x74, 0xC6),
        Category::NobleGas => Color32::from_rgb(0xB8, 0x6B, 0xA4),
        Category::Lanthanide => Color32::from_rgb(0xA4, 0x84, 0x66),
        Category::Actinide => Color32::from_rgb(0x9A, 0x60, 0x60),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_save_resolve() {
        let mut m = MaterialsState::new();
        assert!(m.apply(&AppCommand::MaterialsSave { name: None }).unwrap().is_err(), "nothing built yet");
        let r = m.apply(&AppCommand::MaterialsBuild { formula: Some("H2O".into()) }).unwrap().unwrap();
        assert!(r.contains("Water"), "{}", r);
        assert_eq!(m.built.as_ref().unwrap().preset, Some("water"));
        assert!(m.apply(&AppCommand::MaterialsBuild { formula: Some("H2Xq".into()) }).unwrap().is_err());
        assert!(m.error.is_some());
        m.apply(&AppCommand::MaterialsClear).unwrap().unwrap();
        m.apply(&AppCommand::MaterialsAdd { element: "Na".into(), count: Some(1) }).unwrap().unwrap();
        m.apply(&AppCommand::MaterialsAdd { element: "chlorine".into(), count: Some(2) }).unwrap().unwrap();
        assert_eq!(m.comp.formula(), "NaCl2");
        let r = m.apply(&AppCommand::MaterialsBuild { formula: None }).unwrap().unwrap();
        assert!(r.contains("no charge-balanced"), "{}", r);
        m.apply(&AppCommand::MaterialsRemove { element: "Cl".into() }).unwrap().unwrap();
        assert_eq!(m.comp.formula(), "NaCl");
        m.apply(&AppCommand::MaterialsBuild { formula: None }).unwrap().unwrap();
        assert!(m.built.as_ref().unwrap().analysis.charge_balanced);
        let s = m.apply(&AppCommand::MaterialsSave { name: Some("Salt".into()) }).unwrap().unwrap();
        assert!(s.contains("Salt"));
        assert!(m.lib.registry.get("Salt").is_ok());
        assert_eq!(m.lib.resolve("salt").unwrap(), "Salt");
        assert_eq!(m.lib.resolve("copper").unwrap(), "Copper");
        assert!((m.lib.registry.get("Copper").unwrap().metallic() - 1.0).abs() < 1e-6);
        assert_eq!(m.lib.resolve("Fe2O3").unwrap(), "Fe2O3");
        assert!(m.lib.resolve("unobtainium").is_err());
        assert!(m.lib.surface("Copper").is_some());
    }

    #[test]
    fn atoms_from_particles() {
        assert!(build_atom(6, 6, 6).is_ok());
        let e = build_atom(6, 6, 5).unwrap_err();
        assert!(e.contains("ion") && e.contains("+1"), "{}", e);
        assert!(build_atom(6, 20, 6).unwrap_err().contains("unstable"));
        assert!(build_atom(0, 1, 0).is_err());
        assert!(build_atom(119, 180, 119).is_err());
        let (p, _) = build_atom(79, 118, 79).unwrap();
        assert_eq!(p.preset, Some("gold"));
        let mut m = MaterialsState::new();
        assert!(m.apply(&AppCommand::MaterialsBuildAtom { protons: 8, neutrons: 8, electrons: 10 }).unwrap().is_err());
        assert!(matches!(m.atom_note, Some(Err(_))));
    }

    #[test]
    fn search_and_preview() {
        let al = element_data::find("Al").unwrap();
        assert!(matches(al, "alum") && matches(al, "13") && matches(al, "al") && matches(al, "aluminium"));
        assert!(!matches(al, "Fe"));
        let mut m = MaterialsState::new();
        assert_eq!(m.apply(&AppCommand::MaterialsSearch { query: "ium".into() }).unwrap().unwrap().split(' ').next().unwrap().parse::<usize>().unwrap() > 40, true);
        assert!(m.apply(&AppCommand::MaterialsSelect { element: "Xx".into() }).unwrap().is_err());
        let img = sphere_preview(&preset("copper").unwrap().pbr, 32);
        let c = img.pixels[16 * 32 + 16];
        assert!(c.a() > 200 && c.r() > c.b(), "{:?}", c);
        assert_eq!(img.pixels[0].a(), 0);
    }
}
