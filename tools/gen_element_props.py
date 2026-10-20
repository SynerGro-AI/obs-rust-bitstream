#!/usr/bin/env python3
"""Regenerate src/element_props.rs.

Inputs (see the module header for sources/licences):
  /tmp/pmg_pt.json   pymatgen-core periodic_table.json (gunzipped)
  /tmp/bowser.json   Bowserinator PeriodicTableJSON.json
  mendeleev (pip)    elements + isotopes tables
Run with a Python that has mendeleev installed.
"""
import json, math, re, sys
from mendeleev.fetch import fetch_table
pm = json.load(open('/tmp/pmg_pt.json'))
bw = {e['number']: e for e in json.load(open('/tmp/bowser.json'))['elements']}
t = fetch_table('elements').set_index('atomic_number')
iso = fetch_table('isotopes')
YEAR = 365.25 * 86400
unit = {'asec':1e-18,'zsec':1e-21,'ysec':1e-24,'psec':1e-12,'nsec':1e-9,'usec':1e-6,'msec':1e-3,'sec':1,'minute':60,'hour':3600,'day':86400,'year':YEAR,'kyear':1e3*YEAR,'Myear':1e6*YEAR,'Gyear':1e9*YEAR,'Tyear':1e12*YEAR,'Pyear':1e15*YEAR,'Eyear':1e18*YEAR,'Zyear':1e21*YEAR,'Yyear':1e24*YEAR}
def num(v):
    if v is None: return None
    try:
        f = float(v)
        return None if math.isnan(f) else f
    except Exception:
        return None
def of(v, suffix='_f32'):
    v = num(v); return "None" if v is None else "Some(%r%s)" % (v, suffix)
def s(v): return '"%s"' % str(v).replace('\\', '\\\\').replace('"', '\\"')
cores = {'He':'1s2','Ne':'[He] 2s2 2p6','Ar':'[Ne] 3s2 3p6','Kr':'[Ar] 3d10 4s2 4p6','Xe':'[Kr] 4d10 5s2 5p6','Rn':'[Xe] 4f14 5d10 6s2 6p6'}
def expand(c):
    m = re.match(r'\s*\[(\w+)\](.*)', c)
    if m: return expand(cores[m.group(1)]) + ' ' + m.group(2)
    return c
def shells(z):
    cfg = t.loc[z, 'electronic_configuration']
    sh = {}
    for n, l, k in re.findall(r'(\d)([spdfg])(\d*)', expand(cfg)):
        sh[int(n)] = sh.get(int(n), 0) + (int(k) if k else 1)
    return [sh.get(i, 0) for i in range(1, max(sh) + 1)], cfg
lat = {'BCC':'Body-centred cubic','FCC':'Face-centred cubic','HEX':'Hexagonal','DIA':'Diamond cubic','CUB':'Cubic','SC':'Simple cubic','TET':'Tetragonal','ORC':'Orthorhombic','MCL':'Monoclinic','RHL':'Rhombohedral'}
out = []
for z in range(1, 119):
    r = t.loc[z]; sym = r['symbol']; p = pm.get(sym, {})
    sh, cfg = shells(z)
    assert sum(sh) == z, (sym, sh, cfg)
    ie = [x for x in (p.get('Ionization energies') or []) if num(x) is not None][:10]
    ii = iso[iso.atomic_number == z]
    stable = ii[ii.is_radioactive == 0]
    st = ["%d%s %.4g%%" % (row.mass_number, sym, row.abundance) if num(row.abundance) is not None else "%d%s" % (row.mass_number, sym) for row in stable.itertuples()]
    hl = None; hlm = None
    for row in ii.itertuples():
        if row.is_radioactive == 1 and num(row.half_life) is not None and str(row.half_life_unit) in unit:
            v = row.half_life * unit[str(row.half_life_unit)]
            if hl is None or v > hl: hl = v; hlm = row.mass_number
    radio = bool(r['is_radioactive'])
    keep = hl is not None and (radio or not len(stable))
    yr = num(r['discovery_year'])
    app = bw.get(z, {}).get('appearance') or ''
    disc = r['discoverers'] if isinstance(r['discoverers'], str) else ''
    out.append("    ElementProps { z: %d, shells: &%s, config_full: %s, ionization_ev: &[%s], covalent_radius_pm: %s, specific_heat: %s, molar_heat: %s, thermal_conductivity: %s, electrical_resistivity: %s, crystal: %s, youngs_gpa: %s, bulk_gpa: %s, shear_gpa: %s, poisson: %s, mohs: %s, thermal_expansion: %s, sound_speed: %s, fusion_heat_kj_mol: %s, vaporization_heat_kj_mol: %s, appearance: %s, discovery_year: %s, discoverers: %s, radioactive: %s, stable_isotopes: %s, longest_half_life_s: %s, longest_lived_isotope: %s }," % (
        z, json.dumps(sh), s(cfg), ", ".join("%r_f32" % float(x) for x in ie), of(r['covalent_radius_pyykko']), of(r['specific_heat_capacity']), of(r['molar_heat_capacity']), of(r['thermal_conductivity']), of(p.get('Electrical resistivity'), '_f64'), s(lat.get(str(r['lattice_structure']), 'unknown')), of(p.get('Youngs modulus')), of(p.get('Bulk modulus')), of(p.get('Rigidity modulus')), of(p.get('Poissons ratio')), of(p.get('Mineral hardness')), of(p.get('Coefficient of linear thermal expansion')), of(p.get('Velocity of sound')), of(r['fusion_heat']), of(r['evaporation_heat']), s(app), ("Some(%d)" % yr) if yr is not None else "None", s(disc), 'true' if radio else 'false', s(", ".join(st)), ("Some(%r_f64)" % hl) if keep else "None", ("Some(%d)" % hlm) if keep else "None"))
header = open(sys.argv[1]).read()
open('src/element_props.rs', 'w').write(header.replace('@@ROWS@@', "\n".join(out)))
print("wrote", len(out))
