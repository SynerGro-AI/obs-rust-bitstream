//! Build a computer: transistors → gates → adder → register → CPU.
//!
//! Simulated at several levels, each feeding the next:
//! 1. **Transistor level** (exact within the device model). CMOS INV,
//!    NAND2 and NOR2 are built from EKV MOSFETs and solved by the MNA
//!    engine in `circuit`. `characterise` measures each gate at a die
//!    temperature T:
//!    * the voltage-transfer curve (V_OH, V_OL, V_IL, V_IH, noise margins);
//!    * the propagation delay into a 20 fF load, by transient simulation;
//!    * the static leakage from V_dd.
//! 2. **Gate level.** An event-driven simulator of 0/1/X nets with inertial
//!    delays taken from step 1, so a hot chip's slower gates are really
//!    slower. On top of it are built a 9-NAND full adder, a 4-bit ripple
//!    add/subtract ALU and a master-slave D flip-flop from NAND gates.
//! 3. **Register-transfer level.** A 4-bit accumulator CPU (16 × 8-bit
//!    ROM, 16 × 4-bit RAM, Z/C flags). Control, registers and memory are
//!    behavioural, and each register checks setup time. Every ADD/SUB goes
//!    through the gate-level ALU, which is sampled at the next clock edge.
//!    If the carry chain has not settled by then, the CPU latches whatever
//!    the nets show, and the program computes the wrong answer, as a real
//!    over-clocked or over-heated chip does.
//!
//! Approximations:
//! * The device parameters are generic 180 nm-class values, not a foundry
//!   PDK.
//! * No wire RC, no clock skew. The flip-flop's own timing is a fixed
//!   clock-to-Q of 2 NAND delays and a setup time of 2 NAND delays.
//! * Dynamic power is ½·C·V² per toggle of the modelled gates only.
//! * Failure modes modelled: timing (slower hot gates), logic levels /
//!   noise margins (leakage), and silicon going intrinsic (n_i above the
//!   well doping, ~820 K). Not modelled: junction leakage currents,
//!   electromigration, package and solder limits. A real die (rated about
//!   125 °C) gets into trouble well before 820 K.

use crate::circuit::{Circuit, MosParams, Wave};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const VDD: f64 = 1.8;
/// Load per gate output, F (fan-out of ~4 plus local wiring at this node).
pub const C_LOAD: f64 = 20e-15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GateKind {
    Inv,
    Nand2,
    Nor2,
}

impl GateKind {
    pub const ALL: [GateKind; 3] = [GateKind::Inv, GateKind::Nand2, GateKind::Nor2];
    pub fn name(self) -> &'static str {
        match self {
            GateKind::Inv => "INV",
            GateKind::Nand2 => "NAND2",
            GateKind::Nor2 => "NOR2",
        }
    }
    fn idx(self) -> usize {
        match self {
            GateKind::Inv => 0,
            GateKind::Nand2 => 1,
            GateKind::Nor2 => 2,
        }
    }
    fn inputs(self) -> usize {
        if self == GateKind::Inv {
            1
        } else {
            2
        }
    }
}

/// Transistor-level CMOS cell with inputs "a" ("b"), output "out", optional load.
pub fn cell(kind: GateKind, t: f64, va: Wave, vb: Wave, load: f64) -> Circuit {
    let mut c = Circuit::new();
    c.temp = t;
    c.vsrc("vdd", "0", Wave::Dc(VDD));
    c.vsrc("a", "0", va);
    match kind {
        GateKind::Inv => {
            c.mos("out", "a", "vdd", MosParams::pmos(6.0));
            c.mos("out", "a", "0", MosParams::nmos(2.0));
        }
        GateKind::Nand2 => {
            c.vsrc("b", "0", vb);
            c.mos("out", "a", "vdd", MosParams::pmos(6.0));
            c.mos("out", "b", "vdd", MosParams::pmos(6.0));
            c.mos("out", "a", "x", MosParams::nmos(4.0));
            c.mos("x", "b", "0", MosParams::nmos(4.0));
        }
        GateKind::Nor2 => {
            c.vsrc("b", "0", vb);
            c.mos("y", "a", "vdd", MosParams::pmos(12.0));
            c.mos("out", "b", "y", MosParams::pmos(12.0));
            c.mos("out", "a", "0", MosParams::nmos(2.0));
            c.mos("out", "b", "0", MosParams::nmos(2.0));
        }
    }
    if load > 0.0 {
        c.c("out", "0", load);
    }
    c
}

/// Measured properties of one gate at one temperature.
#[derive(Clone, Debug, PartialEq)]
pub struct CellChar {
    pub kind: GateKind,
    pub t: f64,
    pub voh: f64,
    pub vol: f64,
    pub vil: f64,
    pub vih: f64,
    /// switching threshold V_M (V_in = V_out)
    pub vm: f64,
    /// propagation delays, s (None = the output never crossed V_dd/2)
    pub t_phl: Option<f64>,
    pub t_plh: Option<f64>,
    /// average static supply current over input states, A
    pub leakage: f64,
    /// truth table from DC solves: (inputs, output voltage)
    pub table: Vec<(u8, f64)>,
    /// VTC samples (V_in, V_out) with the other input high (NAND) / low (NOR)
    pub vtc: Vec<(f64, f64)>,
}

impl CellChar {
    pub fn nm_high(&self) -> f64 {
        self.voh - self.vih
    }
    pub fn nm_low(&self) -> f64 {
        self.vil - self.vol
    }
    pub fn t_pd(&self) -> Option<f64> {
        Some(0.5 * (self.t_phl? + self.t_plh?))
    }
    /// The cell still behaves as logic: correct truth table with clean
    /// levels and positive noise margins.
    pub fn works(&self) -> bool {
        let ideal = |inp: u8| -> bool {
            let (a, b) = (inp & 1 == 1, inp & 2 == 2);
            match self.kind {
                GateKind::Inv => !a,
                GateKind::Nand2 => !(a && b),
                GateKind::Nor2 => !(a || b),
            }
        };
        self.table.iter().all(|(inp, v)| {
            if ideal(*inp) {
                *v > 0.8 * VDD
            } else {
                *v < 0.2 * VDD
            }
        }) && self.nm_high() > 0.1
            && self.nm_low() > 0.1
            && self.t_pd().is_some()
    }
}

fn dc_out(kind: GateKind, t: f64, a: f64, b: f64) -> Option<(f64, f64)> {
    let mut c = cell(kind, t, Wave::Dc(a), Wave::Dc(b), 0.0);
    c.dc().ok()?;
    // supply current = −(current through the V_dd source, a → b inside it)
    Some((c.voltage("out"), -c.current(0)))
}

/// Characterise a gate at die temperature `t` (K).
pub fn characterise(kind: GateKind, t: f64) -> CellChar {
    let other = if kind == GateKind::Nor2 { 0.0 } else { VDD };
    // VTC
    let n = 72;
    let vtc: Vec<(f64, f64)> = (0..=n)
        .map(|i| {
            let vin = VDD * i as f64 / n as f64;
            (
                vin,
                dc_out(kind, t, vin, other).map(|x| x.0).unwrap_or(f64::NAN),
            )
        })
        .collect();
    let voh = vtc[0].1;
    let vol = vtc[n].1;
    // unity-gain points
    let mut vil = 0.0;
    let mut vih = VDD;
    let mut found_il = false;
    for w in vtc.windows(2) {
        let slope = (w[1].1 - w[0].1) / (w[1].0 - w[0].0);
        if !found_il && slope < -1.0 {
            vil = w[0].0;
            found_il = true;
        }
        if found_il && slope < -1.0 {
            vih = w[1].0;
        }
    }
    let vm = vtc
        .windows(2)
        .find(|w| w[0].1 >= w[0].0 && w[1].1 < w[1].0)
        .map(|w| 0.5 * (w[0].0 + w[1].0))
        .unwrap_or(f64::NAN);
    // truth table + leakage
    let mut table = Vec::new();
    let mut leak = 0.0;
    let states: Vec<u8> = if kind.inputs() == 1 {
        vec![0, 1]
    } else {
        vec![0, 1, 2, 3]
    };
    for s in &states {
        let (a, b) = (
            if s & 1 == 1 { VDD } else { 0.0 },
            if s & 2 == 2 { VDD } else { 0.0 },
        );
        let (v, i) = dc_out(kind, t, a, b).unwrap_or((f64::NAN, 0.0));
        table.push((*s, v));
        leak += i.abs();
    }
    let leakage = leak / states.len() as f64;
    // transient delays: input a steps 0→1 at 20 ps then back at 420 ps
    let pulse = Wave::Pulse {
        v0: 0.0,
        v1: VDD,
        delay: 20e-12,
        rise: 5e-12,
        width: 400e-12,
        period: 0.0,
    };
    let mut c = cell(kind, t, pulse, Wave::Dc(other), C_LOAD);
    let _ = c.dc();
    let h = 0.25e-12;
    let half = VDD / 2.0;
    let (mut t_in_rise, mut t_in_fall, mut t_out_fall, mut t_out_rise) = (None, None, None, None);
    let mut prev_in = c.voltage("a");
    let mut prev_out = c.voltage("out");
    for _ in 0..(840e-12 / h) as usize {
        if c.step(h).is_err() {
            break;
        }
        let (vi, vo) = (c.voltage("a"), c.voltage("out"));
        let tt = c.time;
        // linear interpolation of the V_dd/2 crossing inside the step
        let cross = |p: f64, q: f64| tt - h + h * (half - p) / (q - p);
        if prev_in < half && vi >= half {
            t_in_rise = Some(cross(prev_in, vi));
        }
        if prev_in >= half && vi < half {
            t_in_fall = Some(cross(prev_in, vi));
        }
        if t_in_rise.is_some() && t_in_fall.is_none() && prev_out >= half && vo < half {
            t_out_fall = Some(cross(prev_out, vo));
        }
        if t_in_fall.is_some() && prev_out < half && vo >= half {
            t_out_rise = Some(cross(prev_out, vo));
        }
        prev_in = vi;
        prev_out = vo;
    }
    let t_phl = t_out_fall.zip(t_in_rise).map(|(o, i)| o - i);
    let t_plh = t_out_rise.zip(t_in_fall).map(|(o, i)| o - i);
    CellChar {
        kind,
        t,
        voh,
        vol,
        vil,
        vih,
        vm,
        t_phl,
        t_plh,
        leakage,
        table,
        vtc,
    }
}

/// All three gates at one temperature.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub t: f64,
    pub cells: [CellChar; 3],
}

impl Library {
    pub fn at(t: f64) -> Self {
        Self {
            t,
            cells: GateKind::ALL.map(|k| characterise(k, t)),
        }
    }
    pub fn get(&self, k: GateKind) -> &CellChar {
        &self.cells[k.idx()]
    }
    pub fn works(&self) -> bool {
        self.cells.iter().all(|c| c.works())
    }
    /// Delays in femtoseconds for the event simulator (a gate that never
    /// switches gets an effectively infinite delay).
    pub fn delays_fs(&self) -> [u64; 3] {
        self.cells.clone().map(|c| {
            c.t_pd()
                .map(|d| (d * 1e15).round().max(1.0) as u64)
                .unwrap_or(u64::MAX / 4)
        })
    }
}

/// Logic value: 0, 1 or unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L {
    Zero,
    One,
    X,
}

impl L {
    pub fn from(b: bool) -> Self {
        if b {
            L::One
        } else {
            L::Zero
        }
    }
    pub fn bit(self) -> Option<bool> {
        match self {
            L::Zero => Some(false),
            L::One => Some(true),
            L::X => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gate {
    pub kind: GateKind,
    pub ins: [usize; 2],
    pub out: usize,
}

/// Gate-level netlist with an event-driven, inertial-delay simulator.
#[derive(Clone, Debug)]
pub struct Netlist {
    pub gates: Vec<Gate>,
    pub names: Vec<String>,
    fanout: Vec<Vec<usize>>,
    pub values: Vec<L>,
    /// latest scheduled (seq, value) per net, for inertial cancellation
    pending: Vec<Option<(u64, L)>>,
    queue: BinaryHeap<Reverse<(u64, u64, usize)>>,
    queued_value: std::collections::HashMap<u64, L>,
    seq: u64,
    pub now_fs: u64,
    pub delays: [u64; 3],
    pub toggles: u64,
    /// time of the last value change on any net
    pub last_change_fs: u64,
}

impl Netlist {
    pub fn new() -> Self {
        Self {
            gates: Vec::new(),
            names: Vec::new(),
            fanout: Vec::new(),
            values: Vec::new(),
            pending: Vec::new(),
            queue: BinaryHeap::new(),
            queued_value: Default::default(),
            seq: 0,
            now_fs: 0,
            delays: [30_000, 40_000, 50_000],
            toggles: 0,
            last_change_fs: 0,
        }
    }

    pub fn net(&mut self, name: &str) -> usize {
        self.names.push(name.into());
        self.fanout.push(Vec::new());
        self.values.push(L::X);
        self.pending.push(None);
        self.names.len() - 1
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    fn gate(&mut self, kind: GateKind, a: usize, b: usize) -> usize {
        let out = self.net(&format!(
            "{}{}",
            kind.name().to_lowercase(),
            self.gates.len()
        ));
        let gi = self.gates.len();
        self.gates.push(Gate {
            kind,
            ins: [a, b],
            out,
        });
        self.fanout[a].push(gi);
        if kind.inputs() == 2 && b != a {
            self.fanout[b].push(gi);
        }
        out
    }
    pub fn nand(&mut self, a: usize, b: usize) -> usize {
        self.gate(GateKind::Nand2, a, b)
    }
    pub fn nor(&mut self, a: usize, b: usize) -> usize {
        self.gate(GateKind::Nor2, a, b)
    }
    pub fn inv(&mut self, a: usize) -> usize {
        self.gate(GateKind::Inv, a, a)
    }
    /// XOR from four NANDs.
    pub fn xor(&mut self, a: usize, b: usize) -> usize {
        let n1 = self.nand(a, b);
        let n2 = self.nand(a, n1);
        let n3 = self.nand(b, n1);
        self.nand(n2, n3)
    }
    /// The 9-NAND full adder: (sum, carry out).
    pub fn full_adder(&mut self, a: usize, b: usize, cin: usize) -> (usize, usize) {
        let n1 = self.nand(a, b);
        let n2 = self.nand(a, n1);
        let n3 = self.nand(b, n1);
        let s1 = self.nand(n2, n3);
        let n4 = self.nand(s1, cin);
        let n5 = self.nand(s1, n4);
        let n6 = self.nand(cin, n4);
        let sum = self.nand(n5, n6);
        let cout = self.nand(n4, n1);
        (sum, cout)
    }
    /// Positive-edge D flip-flop: two NAND D-latches (master on !clk, slave on clk).
    pub fn dff(&mut self, d: usize, clk: usize) -> usize {
        let nclk = self.inv(clk);
        let latch = |nl: &mut Netlist, d: usize, en: usize| -> usize {
            let nd = nl.inv(d);
            let s = nl.nand(d, en);
            let r = nl.nand(nd, en);
            // cross-coupled pair: q = nand(s, qb), qb = nand(r, q)
            let q = nl.net("q");
            let qb = nl.net("qb");
            let gq = nl.gates.len();
            nl.gates.push(Gate {
                kind: GateKind::Nand2,
                ins: [s, qb],
                out: q,
            });
            nl.fanout[s].push(gq);
            nl.fanout[qb].push(gq);
            let gqb = nl.gates.len();
            nl.gates.push(Gate {
                kind: GateKind::Nand2,
                ins: [r, q],
                out: qb,
            });
            nl.fanout[r].push(gqb);
            nl.fanout[q].push(gqb);
            q
        };
        let m = latch(self, d, nclk);
        latch(self, m, clk)
    }

    fn eval(&self, g: &Gate) -> L {
        let a = self.values[g.ins[0]];
        let b = self.values[g.ins[1]];
        match g.kind {
            GateKind::Inv => match a {
                L::Zero => L::One,
                L::One => L::Zero,
                L::X => L::X,
            },
            GateKind::Nand2 => {
                if a == L::Zero || b == L::Zero {
                    L::One
                } else if a == L::One && b == L::One {
                    L::Zero
                } else {
                    L::X
                }
            }
            GateKind::Nor2 => {
                if a == L::One || b == L::One {
                    L::Zero
                } else if a == L::Zero && b == L::Zero {
                    L::One
                } else {
                    L::X
                }
            }
        }
    }

    fn schedule(&mut self, net: usize, v: L, at: u64) {
        // inertial: a newer schedule replaces an older one on the same net
        if let Some((_, pv)) = self.pending[net] {
            if pv == v {
                return;
            }
        } else if self.values[net] == v {
            return;
        }
        self.seq += 1;
        self.pending[net] = Some((self.seq, v));
        self.queued_value.insert(self.seq, v);
        self.queue.push(Reverse((at, self.seq, net)));
    }

    /// Drive a primary input now.
    pub fn set(&mut self, net: usize, v: L) {
        let now = self.now_fs;
        self.schedule(net, v, now);
    }

    /// Run events up to (and including) time `until_fs`.
    pub fn run_until(&mut self, until_fs: u64) {
        while let Some(Reverse((t, seq, net))) = self.queue.peek().copied() {
            if t > until_fs {
                break;
            }
            self.queue.pop();
            let v = self.queued_value.remove(&seq).unwrap_or(L::X);
            if self.pending[net].map(|(s, _)| s) != Some(seq) {
                continue; // cancelled by a later schedule
            }
            self.pending[net] = None;
            self.now_fs = t;
            if self.values[net] == v {
                continue;
            }
            self.values[net] = v;
            self.toggles += 1;
            self.last_change_fs = t;
            for gi in self.fanout[net].clone() {
                let g = self.gates[gi].clone();
                let nv = self.eval(&g);
                let d = self.delays[g.kind.idx()];
                self.schedule(g.out, nv, t.saturating_add(d));
            }
        }
        self.now_fs = self.now_fs.max(until_fs);
    }

    /// Run until quiet (or `limit_fs` from now); returns the settle time from now.
    pub fn settle(&mut self, limit_fs: u64) -> u64 {
        let start = self.now_fs;
        let until = start.saturating_add(limit_fs);
        self.run_until(until);
        self.last_change_fs.saturating_sub(start)
    }

    pub fn word(&self, nets: &[usize]) -> Option<u8> {
        let mut w = 0u8;
        for (i, n) in nets.iter().enumerate() {
            if self.values[*n].bit()? {
                w |= 1 << i;
            }
        }
        Some(w)
    }

    pub fn set_word(&mut self, nets: &[usize], w: u8) {
        for (i, n) in nets.iter().enumerate() {
            self.set(*n, L::from(w >> i & 1 == 1));
        }
    }
}

impl Default for Netlist {
    fn default() -> Self {
        Self::new()
    }
}

/// The 4-bit add/subtract ALU at gate level.
#[derive(Clone, Debug)]
pub struct Alu {
    pub nl: Netlist,
    pub a: [usize; 4],
    pub b: [usize; 4],
    pub sub: usize,
    pub sum: [usize; 4],
    pub cout: usize,
}

impl Alu {
    pub fn new() -> Self {
        let mut nl = Netlist::new();
        let a = [0, 1, 2, 3].map(|i| nl.net(&format!("a{}", i)));
        let b = [0, 1, 2, 3].map(|i| nl.net(&format!("b{}", i)));
        let sub = nl.net("sub");
        let mut carry = sub;
        let mut sum = [0; 4];
        for i in 0..4 {
            let bx = nl.xor(b[i], sub);
            let (s, c) = nl.full_adder(a[i], bx, carry);
            sum[i] = s;
            carry = c;
        }
        Self {
            nl,
            a,
            b,
            sub,
            sum,
            cout: carry,
        }
    }
    pub fn gate_count(&self) -> usize {
        self.nl.gates.len()
    }
    /// Apply operands now and settle fully; returns (sum, carry, settle fs).
    pub fn compute(&mut self, a: u8, b: u8, sub: bool) -> (Option<u8>, Option<bool>, u64) {
        self.nl.set_word(&self.a, a);
        self.nl.set_word(&self.b, b);
        self.nl.set(self.sub, L::from(sub));
        let t = self.nl.settle(1_000_000_000);
        (self.nl.word(&self.sum), self.nl.values[self.cout].bit(), t)
    }
}

impl Default for Alu {
    fn default() -> Self {
        Self::new()
    }
}

// ---- the CPU ----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Nop = 0,
    Ldi = 1,
    Lda = 2,
    Sta = 3,
    Add = 4,
    Addi = 5,
    Sub = 6,
    Subi = 7,
    Jmp = 8,
    Jz = 9,
    Jc = 10,
    Out = 11,
    Hlt = 15,
}

impl Op {
    pub fn from(n: u8) -> Self {
        match n {
            1 => Op::Ldi,
            2 => Op::Lda,
            3 => Op::Sta,
            4 => Op::Add,
            5 => Op::Addi,
            6 => Op::Sub,
            7 => Op::Subi,
            8 => Op::Jmp,
            9 => Op::Jz,
            10 => Op::Jc,
            11 => Op::Out,
            15 => Op::Hlt,
            _ => Op::Nop,
        }
    }
    pub fn mnemonic(self) -> &'static str {
        match self {
            Op::Nop => "NOP",
            Op::Ldi => "LDI",
            Op::Lda => "LDA",
            Op::Sta => "STA",
            Op::Add => "ADD",
            Op::Addi => "ADDI",
            Op::Sub => "SUB",
            Op::Subi => "SUBI",
            Op::Jmp => "JMP",
            Op::Jz => "JZ",
            Op::Jc => "JC",
            Op::Out => "OUT",
            Op::Hlt => "HLT",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        [
            Op::Nop,
            Op::Ldi,
            Op::Lda,
            Op::Sta,
            Op::Add,
            Op::Addi,
            Op::Sub,
            Op::Subi,
            Op::Jmp,
            Op::Jz,
            Op::Jc,
            Op::Out,
            Op::Hlt,
        ]
        .into_iter()
        .find(|o| o.mnemonic().eq_ignore_ascii_case(s))
    }
}

/// Assemble "label: OP arg ; comment" lines into ≤16 bytes.
pub fn assemble(src: &str) -> Result<Vec<u8>, String> {
    let mut labels = std::collections::HashMap::new();
    let mut lines = Vec::new();
    for raw in src.lines() {
        let l = raw.split(';').next().unwrap_or("").trim();
        if l.is_empty() {
            continue;
        }
        let mut rest = l;
        if let Some((lab, r)) = l.split_once(':') {
            labels.insert(lab.trim().to_lowercase(), lines.len() as u8);
            rest = r.trim();
            if rest.is_empty() {
                continue;
            }
        }
        lines.push(rest.to_string());
    }
    if lines.len() > 16 {
        return Err(format!("{} instructions; the ROM holds 16", lines.len()));
    }
    lines
        .iter()
        .map(|l| {
            let mut it = l.split_whitespace();
            let m = it.next().unwrap_or("");
            let op = Op::parse(m).ok_or_else(|| format!("unknown instruction '{}'", m))?;
            let arg = match it.next() {
                None => 0,
                Some(a) => {
                    let a = a.trim_start_matches('#');
                    match a.parse::<u8>() {
                        Ok(v) if v < 16 => v,
                        Ok(v) => return Err(format!("operand {} does not fit 4 bits", v)),
                        Err(_) => *labels
                            .get(&a.to_lowercase())
                            .ok_or_else(|| format!("unknown label '{}'", a))?,
                    }
                }
            };
            Ok((op as u8) << 4 | arg)
        })
        .collect()
}

pub const PROGRAMS: &[(&str, &str, &[u8])] = &[
    ("multiply", "LDI 0\nSTA 15      ; product = 0\nLDI 5\nSTA 14      ; counter = 5\nloop: LDA 15\nADDI 3      ; product += 3\nSTA 15\nLDA 14\nSUBI 1      ; counter -= 1\nSTA 14\nJZ done\nJMP loop\ndone: LDA 15\nOUT         ; 3 × 5 = 15\nHLT", &[15]),
    ("fibonacci", "LDI 1\nSTA 0       ; a = 1\nSTA 1       ; b = 1\nOUT\nloop: LDA 1\nOUT         ; print b\nADD 0       ; a + b\nJC done     ; stop on 4-bit overflow\nSTA 2\nLDA 1\nSTA 0       ; a = b\nLDA 2\nSTA 1       ; b = a + b\nJMP loop\ndone: HLT", &[1, 1, 2, 3, 5, 8, 13]),
    ("countdown", "LDI 9\nloop: OUT\nSUBI 1\nJZ done\nJMP loop\ndone: OUT\nHLT", &[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]),
];

pub fn program(name: &str) -> Option<(&'static str, &'static [u8])> {
    PROGRAMS.iter().find(|p| p.0 == name).map(|p| (p.1, p.2))
}

#[derive(Clone, Debug)]
pub struct Cpu {
    pub rom: Vec<u8>,
    pub ram: [u8; 16],
    pub pc: u8,
    pub acc: u8,
    pub z: bool,
    pub c: bool,
    pub halted: bool,
    pub out: Vec<u8>,
    pub cycles: u64,
    pub alu: Alu,
    /// clock period, fs
    pub period_fs: u64,
    /// clock-to-Q and setup, fs
    pub clk_q_fs: u64,
    pub setup_fs: u64,
    /// ALU results latched before the carry chain settled
    pub timing_violations: u64,
    /// worst ALU settle time seen, fs
    pub worst_settle_fs: u64,
    pub lib: Option<Library>,
}

impl Cpu {
    pub fn new(rom: Vec<u8>, lib: &Library, clock_hz: f64) -> Self {
        let mut alu = Alu::new();
        alu.nl.delays = lib.delays_fs();
        let nand = lib.delays_fs()[1];
        // power-on: settle the ALU from all-zero inputs
        alu.compute(0, 0, false);
        Self {
            rom,
            ram: [0; 16],
            pc: 0,
            acc: 0,
            z: true,
            c: false,
            halted: false,
            out: Vec::new(),
            cycles: 0,
            alu,
            period_fs: (1e15 / clock_hz) as u64,
            clk_q_fs: 2 * nand,
            setup_fs: 2 * nand,
            timing_violations: 0,
            worst_settle_fs: 0,
            lib: Some(lib.clone()),
        }
    }

    /// Run the ALU for one clock cycle starting at a clock edge and return
    /// what the result register latches at the next edge.
    fn alu_cycle(&mut self, b: u8, sub: bool) -> (u8, bool) {
        let edge = self.alu.nl.now_fs;
        // the accumulator and operand change clk→Q after the edge
        self.alu.nl.run_until(edge + self.clk_q_fs);
        self.alu.nl.set_word(&self.alu.a.clone(), self.acc);
        self.alu.nl.set_word(&self.alu.b.clone(), b);
        let s = self.alu.sub;
        self.alu.nl.set(s, L::from(sub));
        let start = self.alu.nl.now_fs;
        let sample = edge + self.period_fs.saturating_sub(self.setup_fs);
        self.alu.nl.run_until(sample);
        let early_sum = self.alu.nl.word(&self.alu.sum);
        let early_c = self.alu.nl.values[self.alu.cout].bit();
        // how long it really needed (a copy runs on to quiescence)
        let mut probe = self.alu.nl.clone();
        probe.settle(1_000_000_000);
        let settled_at = probe.last_change_fs.max(start) - start;
        self.worst_settle_fs = self.worst_settle_fs.max(settled_at);
        let violated = probe.last_change_fs > sample;
        if violated {
            self.timing_violations += 1;
        }
        self.alu.nl.run_until(edge + self.period_fs);
        (early_sum.unwrap_or(0), early_c.unwrap_or(false))
    }

    /// One instruction per clock cycle.
    pub fn step(&mut self) {
        if self.halted {
            return;
        }
        let ins = self.rom.get(self.pc as usize).copied().unwrap_or(0xF0);
        let (op, arg) = (Op::from(ins >> 4), ins & 0xF);
        let mut next = (self.pc + 1) & 0xF;
        match op {
            Op::Nop => {}
            Op::Ldi => self.acc = arg,
            Op::Lda => self.acc = self.ram[arg as usize],
            Op::Sta => self.ram[arg as usize] = self.acc,
            Op::Add | Op::Addi | Op::Sub | Op::Subi => {
                let b = if matches!(op, Op::Add | Op::Sub) {
                    self.ram[arg as usize]
                } else {
                    arg
                };
                let (s, c) = self.alu_cycle(b, matches!(op, Op::Sub | Op::Subi));
                self.acc = s & 0xF;
                self.c = c;
            }
            Op::Jmp => next = arg,
            Op::Jz => {
                if self.z {
                    next = arg
                }
            }
            Op::Jc => {
                if self.c {
                    next = arg
                }
            }
            Op::Out => self.out.push(self.acc),
            Op::Hlt => self.halted = true,
        }
        if !matches!(op, Op::Add | Op::Addi | Op::Sub | Op::Subi) {
            // non-ALU cycles still advance time on the ALU's clock
            let now = self.alu.nl.now_fs;
            self.alu.nl.run_until(now + self.period_fs);
        }
        if matches!(
            op,
            Op::Ldi | Op::Lda | Op::Add | Op::Addi | Op::Sub | Op::Subi
        ) {
            self.z = self.acc == 0;
        }
        self.pc = next;
        self.cycles += 1;
    }

    pub fn run(&mut self, max_cycles: u64) {
        while !self.halted && self.cycles < max_cycles {
            self.step();
        }
    }

    /// Dynamic + leakage power, W, over the run so far.
    pub fn power(&self) -> (f64, f64) {
        let secs = (self.cycles.max(1) * self.period_fs) as f64 * 1e-15;
        let dynamic = self.alu.nl.toggles as f64 * 0.5 * C_LOAD * VDD * VDD / secs;
        let leak = self
            .lib
            .as_ref()
            .map(|l| l.get(GateKind::Nand2).leakage)
            .unwrap_or(0.0)
            * self.alu.gate_count() as f64
            * VDD;
        (dynamic, leak)
    }

    /// Disassembly of the ROM.
    pub fn listing(&self) -> Vec<String> {
        self.rom
            .iter()
            .map(|b| {
                let op = Op::from(b >> 4);
                match op {
                    Op::Nop | Op::Out | Op::Hlt => op.mnemonic().to_string(),
                    _ => format!("{} {}", op.mnemonic(), b & 0xF),
                }
            })
            .collect()
    }
}

/// Result of running a program at one temperature and clock.
#[derive(Clone, Debug, PartialEq)]
pub struct RunReport {
    pub t: f64,
    pub clock_hz: f64,
    pub out: Vec<u8>,
    pub expected: Vec<u8>,
    pub correct: bool,
    pub cycles: u64,
    pub timing_violations: u64,
    pub worst_settle_ps: f64,
    pub gates_work: bool,
    pub nand_delay_ps: Option<f64>,
    pub power_w: (f64, f64),
    pub failure: Option<String>,
}

pub fn run_program(name: &str, t: f64, clock_hz: f64) -> Result<RunReport, String> {
    let (src, expected) = program(name).ok_or_else(|| {
        format!(
            "unknown program '{}' (multiply, fibonacci, countdown)",
            name
        )
    })?;
    let lib = Library::at(t);
    run_with(&lib, src, expected, clock_hz)
}

/// Well / substrate doping, cm⁻³. When thermally generated carriers exceed
/// it, the p–n junctions that isolate transistors stop isolating.
pub const WELL_DOPING: f64 = 1e17;

pub fn run_with(
    lib: &Library,
    src: &str,
    expected: &[u8],
    clock_hz: f64,
) -> Result<RunReport, String> {
    let rom = assemble(src)?;
    let ni = crate::circuit::si_intrinsic(lib.t);
    let intrinsic = ni > WELL_DOPING;
    let gates_work = lib.works() && !intrinsic;
    let nand = lib.get(GateKind::Nand2).t_pd();
    let mut cpu = Cpu::new(rom, lib, clock_hz);
    let mut failure = None;
    if gates_work {
        cpu.run(400);
    } else if intrinsic {
        failure = Some(format!("silicon has gone intrinsic at {:.0} K: n_i = {:.1e} cm⁻³ exceeds the {:.0e} cm⁻³ well doping, so junction isolation is lost", lib.t, ni, WELL_DOPING));
    } else {
        let c = lib.cells.iter().find(|c| !c.works()).unwrap();
        failure = Some(format!("{} no longer works as logic at {:.0} K (V_OH {:.2} V, V_OL {:.2} V, NM_H {:.2} V, NM_L {:.2} V)", c.kind.name(), lib.t, c.voh, c.vol, c.nm_high(), c.nm_low()));
    }
    let correct = gates_work && cpu.out == expected && cpu.halted;
    if failure.is_none() && !correct {
        failure = Some(if cpu.timing_violations > 0 {
            format!("timing: the ALU carry chain needs {:.0} ps but the clock leaves {:.0} ps; {} results latched early", cpu.worst_settle_fs as f64 / 1e3, (cpu.period_fs as i64 - cpu.setup_fs as i64 - cpu.clk_q_fs as i64) as f64 / 1e3, cpu.timing_violations)
        } else if !cpu.halted {
            "did not halt".into()
        } else {
            "wrong output".into()
        });
    }
    Ok(RunReport {
        t: lib.t,
        clock_hz,
        out: cpu.out.clone(),
        expected: expected.to_vec(),
        correct,
        cycles: cpu.cycles,
        timing_violations: cpu.timing_violations,
        worst_settle_ps: cpu.worst_settle_fs as f64 / 1e3,
        gates_work,
        nand_delay_ps: nand.map(|d| d * 1e12),
        power_w: cpu.power(),
        failure,
    })
}

/// Heat the die from `t0` in `dt` steps until the program fails; returns
/// every run (the last one is the first failure, if any within `t_max`).
pub fn heat_to_failure(
    name: &str,
    clock_hz: f64,
    t0: f64,
    dt: f64,
    t_max: f64,
) -> Result<Vec<RunReport>, String> {
    let mut out = Vec::new();
    let mut t = t0;
    while t <= t_max {
        let r = run_program(name, t, clock_hz)?;
        let fail = !r.correct;
        out.push(r);
        if fail {
            break;
        }
        t += dt;
    }
    Ok(out)
}

/// Highest clock (Hz) at which the program still runs correctly at `t`
/// (bisection on the period).
pub fn max_clock(name: &str, t: f64) -> Result<f64, String> {
    let (src, expected) = program(name).ok_or("unknown program")?;
    let lib = Library::at(t);
    let (mut lo, mut hi) = (1e7, 1e11);
    for _ in 0..30 {
        let mid = (lo * hi as f64).sqrt();
        if run_with(&lib, src, expected, mid)?.correct {
            lo = mid
        } else {
            hi = mid
        }
    }
    Ok(lo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transistor_level_gates_have_correct_truth_tables_and_margins() {
        let lib = Library::at(300.0);
        for c in &lib.cells {
            assert!(c.works(), "{:?}", c);
            let d = c.t_pd().unwrap();
            assert!(d > 5e-12 && d < 200e-12, "{} delay {}", c.kind.name(), d);
            assert!(c.leakage < 1e-9, "{} leaks {}", c.kind.name(), c.leakage);
        }
        let inv = lib.get(GateKind::Inv);
        assert!((inv.vm - 0.9).abs() < 0.06, "V_M {}", inv.vm);
        assert!(inv.nm_high() > 0.5 && inv.nm_low() > 0.5);
    }

    #[test]
    fn hot_gates_are_slower_and_leak_more() {
        let cold = Library::at(300.0);
        let hot = Library::at(400.0);
        let (dc, dh) = (
            cold.get(GateKind::Nand2).t_pd().unwrap(),
            hot.get(GateKind::Nand2).t_pd().unwrap(),
        );
        assert!(dh > 1.1 * dc, "{} vs {}", dh, dc);
        eprintln!(
            "NAND2 t_pd 300 K {:.1} ps, 400 K {:.1} ps; leakage {:.3e} → {:.3e} A",
            dc * 1e12,
            dh * 1e12,
            cold.get(GateKind::Nand2).leakage,
            hot.get(GateKind::Nand2).leakage
        );
        assert!(hot.get(GateKind::Nand2).leakage > 50.0 * cold.get(GateKind::Nand2).leakage);
    }

    #[test]
    fn gate_level_adder_adds_every_pair_and_subtracts() {
        let mut alu = Alu::new();
        alu.nl.delays = Library::at(300.0).delays_fs();
        assert_eq!(alu.gate_count(), 16 + 36);
        for a in 0..16u8 {
            for b in 0..16u8 {
                let (s, c, _) = alu.compute(a, b, false);
                assert_eq!(s, Some((a + b) & 15), "{}+{}", a, b);
                assert_eq!(c, Some(a + b > 15));
                let (d, nb, _) = alu.compute(a, b, true);
                assert_eq!(d, Some(a.wrapping_sub(b) & 15), "{}-{}", a, b);
                assert_eq!(nb, Some(a >= b), "carry = no borrow");
            }
        }
        // the full carry ripple (15 + 1) takes longer than no carry (0 + 0)
        alu.compute(0, 0, false);
        let (_, _, long) = alu.compute(15, 1, false);
        alu.compute(0, 0, false);
        let (_, _, short) = alu.compute(1, 0, false);
        assert!(long > short, "{} vs {}", long, short);
    }

    #[test]
    fn nand_flip_flops_make_a_register() {
        let mut nl = Netlist::new();
        nl.delays = Library::at(300.0).delays_fs();
        let clk = nl.net("clk");
        let d: Vec<usize> = (0..4).map(|i| nl.net(&format!("d{}", i))).collect();
        let q: Vec<usize> = d.iter().map(|&di| nl.dff(di, clk)).collect();
        nl.set(clk, L::Zero);
        for (i, v) in [0b1010u8, 0b0110, 0b1111, 0b0001].into_iter().enumerate() {
            nl.set_word(&d, v);
            nl.settle(10_000_000);
            nl.set(clk, L::One);
            nl.settle(10_000_000);
            assert_eq!(nl.word(&q), Some(v), "load {}", i);
            // data changes while the clock is high/low do not leak through
            nl.set_word(&d, !v & 15);
            nl.settle(10_000_000);
            nl.set(clk, L::Zero);
            nl.settle(10_000_000);
            assert_eq!(nl.word(&q), Some(v), "held {}", i);
        }
    }

    #[test]
    fn assembler_and_programs() {
        for (name, src, expected) in PROGRAMS {
            let rom = assemble(src).unwrap();
            assert!(rom.len() <= 16, "{}", name);
            let r = run_program(name, 300.0, 500e6).unwrap();
            assert!(
                r.correct,
                "{}: {:?} vs {:?} ({:?})",
                name, r.out, expected, r.failure
            );
            assert_eq!(r.timing_violations, 0);
        }
        assert!(assemble("FOO 1").is_err());
        assert!(assemble("LDI 16").is_err());
        assert!(assemble("JMP nowhere").is_err());
    }

    #[test]
    fn overclocking_breaks_the_adder_first() {
        let fmax = max_clock("fibonacci", 300.0).unwrap();
        assert!(fmax > 5e8 && fmax < 2e10, "{}", fmax);
        let r = run_program("fibonacci", 300.0, fmax * 1.5).unwrap();
        assert!(!r.correct && r.timing_violations > 0, "{:?}", r);
        assert!(r.failure.unwrap().starts_with("timing"));
    }

    #[test]
    fn heating_the_die_makes_it_fail() {
        // clock it at 90 % of its room-temperature limit, then heat it
        let fmax = max_clock("fibonacci", 300.0).unwrap();
        let runs = heat_to_failure("fibonacci", fmax * 0.9, 300.0, 20.0, 900.0).unwrap();
        let last = runs.last().unwrap();
        assert!(runs[0].correct);
        assert!(!last.correct, "it should fail somewhere below 900 K");
        assert!(
            last.t > 300.0 && last.t < 500.0,
            "fails at {} K: {:?}",
            last.t,
            last.failure
        );
        // a slow clock survives much hotter, until the gates stop being logic
        let slow = heat_to_failure("multiply", 1e7, 300.0, 100.0, 1500.0).unwrap();
        let l = slow.last().unwrap();
        assert!(l.t > last.t + 100.0, "{} vs {}", l.t, last.t);
        assert!(
            l.t > 700.0 && l.t < 950.0 && l.failure.as_deref().unwrap().contains("intrinsic"),
            "{} {:?}",
            l.t,
            l.failure
        );
    }
}
