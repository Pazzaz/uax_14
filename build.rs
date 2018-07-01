extern crate regex;
use regex::Regex;
use std::collections::HashMap;
use std::u32;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const LINEBREAK: &'static str = include_str!("unicode-data/LineBreak-11.0.0.txt");
const UNICODEDATA: &'static str = include_str!("unicode-data/UnicodeData.txt");

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("convert_to_break_class");
    let mut f = File::create(&dest_path).unwrap();

    // Extract all codepoints that belong to the general category of Mn or Mc
    let re1 = Regex::new(r"(?P<codepoint>[0-9A-F]+);[^;]+;(?P<category>(Mn)|(Mc))").unwrap();
    let mut mn = Vec::new();
    let mut mc = Vec::new();
    for caps in re1.captures_iter(UNICODEDATA) {
        let number =
            u32::from_str_radix(&caps["codepoint"], 16).expect("Could not parse codepoint");
        match &caps["category"] {
            "Mn" => mn.push((number, None)),
            "Mc" => mc.push((number, None)),
            _ => unreachable!(),
        };
    }
    let compact_mn = squish(mn);
    let compact_mc = squish(mc);

    let re2 = Regex::new(
        r"(?P<left_n>[0-9A-F]+)(\.\.(?P<right_n>[0-9A-F]+))?;(?P<class>[A-Z0-9]+)",
    ).unwrap();
    let mut hash: HashMap<&str, Vec<(u32, Option<u32>)>> = HashMap::new();
    for caps in re2.captures_iter(LINEBREAK) {
        // (u32, Some(u32)): 123A..123F
        // (u32, None)     : 123A
        let numbers: (u32, Option<u32>) = match &caps.name("right_n") {
            Some(right_n_match) => {
                let left_n_str = &caps["left_n"];
                let left_n =
                    u32::from_str_radix(left_n_str, 16).expect("Could not parse left of range");

                let right_n_str = right_n_match.as_str();
                let right_n =
                    u32::from_str_radix(right_n_str, 16).expect("Could not parse right of range");
                (left_n, Some(right_n))
            }
            None => {
                let n =
                    u32::from_str_radix(&caps["left_n"], 16).expect("Could not parse codepoint");
                (n, None)
            }
        };
        let class = caps.name("class").unwrap().as_str();
        hash.entry(class).or_insert(Vec::new()).push(numbers);
    }

    write!(f, "match n as u32 {{").unwrap();
    for (key, value) in hash.into_iter().map(|(key, list)| (key, squish(list))) {
        match key {
            "SA" => write!(
                f,
                "0x{} => match n as u32 {{0x{}|0x{} => Class::CM,_ => Class::AL}}",
                value.join(" | 0x"),
                compact_mn.join(" | 0x"),
                compact_mc.join(" | 0x")
            ).unwrap(),
            "XX" | "SG" | "AI" => write!(f, "0x{} => Class::AL,", value.join(" | 0x")).unwrap(),
            "CJ" => write!(f, "0x{} => Class::NS,", value.join(" | 0x")).unwrap(),
            _ => write!(f, "0x{} => Class::{},", value.join(" | 0x"), key).unwrap(),
        }
    }
    write!(
        f,
        "0x1F000...0x1FFFD => Class::ID, 0x20A0...0x20CF => Class::PR, _ => Class::AL}}"
    ).unwrap();

    let dest_path = Path::new(&out_dir).join("states");
    let mut f = File::create(&dest_path).unwrap();
    write_states(&mut f);
}

// Convert a list of codepoints / ranges of codepoints into a list with the
// minimal number of entries to represent the same codepoints
fn squish(values: Vec<(u32, Option<u32>)>) -> Vec<String> {
    let mut lower = values[0].0;
    let mut higher = values[0].1;
    let mut out = Vec::new();
    for window in values.windows(2) {
        let (left_0, right_0) = window[0];
        let (left_1, right_1) = window[1];
        match (right_0, right_1) {
            (Some(right_0_value), Some(right_1_value)) => {
                if right_0_value == left_1 - 1 {
                    higher = Some(right_1_value);
                } else {
                    out.push(format_codepoints(lower, higher));
                    higher = Some(right_1_value);
                    lower = left_1;
                }
            }
            (Some(right_0_value), None) => {
                if right_0_value == left_1 - 1 {
                    higher = Some(left_1);
                } else {
                    out.push(format_codepoints(lower, higher));
                    higher = None;
                    lower = left_1;
                }
            }
            (None, Some(right_1_value)) => {
                if left_0 == left_1 - 1 {
                    higher = Some(right_1_value);
                } else {
                    out.push(format_codepoints(lower, higher));
                    higher = Some(right_1_value);
                    lower = left_1;
                }
            }
            (None, None) => {
                if left_0 == left_1 - 1 {
                    higher = Some(left_1);
                } else {
                    out.push(format_codepoints(lower, higher));
                    higher = None;
                    lower = left_1;
                }
            }
        }
    }
    out.push(format_codepoints(lower, higher));
    out
}

fn format_codepoints(lower: u32, higher: Option<u32>) -> String {
    match higher {
        Some(x) => format!("{:X}...0x{:X}", lower, x),
        None => format!("{:X}", lower),
    }
}

const NUM_OF_CLASSES: usize = 39;

fn write_states(f: &mut File) {
    const BK: usize = 0;
    const CR: usize = 1;
    const LF: usize = 2;
    const CM: usize = 3;
    const NL: usize = 4;
    const WJ: usize = 5;
    const ZW: usize = 6;
    const GL: usize = 7;
    const SP: usize = 8;
    const ZWJ: usize = 9;
    const B2: usize = 10;
    const BA: usize = 11;
    const BB: usize = 12;
    const HY: usize = 13;
    const CB: usize = 14;
    const CL: usize = 15;
    const CP: usize = 16;
    const EX: usize = 17;
    const IN: usize = 18;
    const NS: usize = 19;
    const OP: usize = 20;
    const QU: usize = 21;
    const IS: usize = 22;
    const NU: usize = 23;
    const PO: usize = 24;
    const PR: usize = 25;
    const SY: usize = 26;
    const AL: usize = 27;
    const EB: usize = 28;
    const EM: usize = 29;
    const H2: usize = 30;
    const H3: usize = 31;
    const HL: usize = 32;
    const ID: usize = 33;
    const JL: usize = 34;
    const JV: usize = 35;
    const JT: usize = 36;
    const RI: usize = 37;
    const XX: usize = 38;
    const LB8_STATE: usize = NUM_OF_CLASSES + 1;
    const LB14_STATE: usize = NUM_OF_CLASSES + 2;
    const LB15_STATE: usize = NUM_OF_CLASSES + 3;
    const LB16_STATE: usize = NUM_OF_CLASSES + 4;
    const LB17_STATE: usize = NUM_OF_CLASSES + 5;
    const LB21A_HY_STATE: usize = NUM_OF_CLASSES + 6;
    const LB21A_BA_STATE: usize = NUM_OF_CLASSES + 7;
    const LB30A_EVEN_STATE: usize = NUM_OF_CLASSES + 8;
    const LB9_EXCEPTIONS: [usize; 8] = [BK, CR, LF, NL, SP, ZW, ZWJ, 39];

    fn break_before(class: usize, b: bool, states: &mut Vec<[(usize, bool); NUM_OF_CLASSES]>) {
        for state in states.iter_mut() {
            state[class].1 = b;
        }
    }

    fn break_after(state: usize, b: bool, states: &mut Vec<[(usize, bool); NUM_OF_CLASSES]>) {
        for c in states[state].iter_mut() {
            c.1 = b;
        }
    }

    fn not_allowed_between(
        c1: usize,
        c2: usize,
        states: &mut Vec<[(usize, bool); NUM_OF_CLASSES]>,
    ) {
        states[c1][c2].1 = false;
    }

    const LB12A_EXCEPTIONS: [usize; 3] = [SP, BA, HY];
    let mut states = Vec::new();
    let mut extra_states = Vec::new();

    for _ in 0..(NUM_OF_CLASSES + 1) {
        states.push([
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true),
            (6, true),
            (7, true),
            (8, true),
            (9, true),
            (10, true),
            (11, true),
            (12, true),
            (13, true),
            (14, true),
            (15, true),
            (16, true),
            (17, true),
            (18, true),
            (19, true),
            (20, true),
            (21, true),
            (22, true),
            (23, true),
            (24, true),
            (25, true),
            (26, true),
            (27, true),
            (28, true),
            (29, true),
            (30, true),
            (31, true),
            (32, true),
            (33, true),
            (34, true),
            (35, true),
            (36, true),
            (37, true),
            (38, true),
        ]);
    }

    // LB30b
    not_allowed_between(EB, EM, &mut states);

    // LB30a
    not_allowed_between(RI, RI, &mut states);
    states[RI][RI].0 = LB30A_EVEN_STATE;

    // LB30
    not_allowed_between(AL, OP, &mut states);
    not_allowed_between(HL, OP, &mut states);
    not_allowed_between(NU, OP, &mut states);

    not_allowed_between(CP, AL, &mut states);
    not_allowed_between(CP, HL, &mut states);
    not_allowed_between(CP, NU, &mut states);

    // LB29
    not_allowed_between(IS, AL, &mut states);
    not_allowed_between(IS, HL, &mut states);

    // LB28
    not_allowed_between(AL, AL, &mut states);
    not_allowed_between(AL, HL, &mut states);
    not_allowed_between(HL, AL, &mut states);
    not_allowed_between(HL, HL, &mut states);

    // LB27
    not_allowed_between(JL, IN, &mut states);
    not_allowed_between(JV, IN, &mut states);
    not_allowed_between(JT, IN, &mut states);
    not_allowed_between(H2, IN, &mut states);
    not_allowed_between(H3, IN, &mut states);

    not_allowed_between(JL, PO, &mut states);
    not_allowed_between(JV, PO, &mut states);
    not_allowed_between(JT, PO, &mut states);
    not_allowed_between(H2, PO, &mut states);
    not_allowed_between(H3, PO, &mut states);

    not_allowed_between(PR, JL, &mut states);
    not_allowed_between(PR, JV, &mut states);
    not_allowed_between(PR, JT, &mut states);
    not_allowed_between(PR, H2, &mut states);
    not_allowed_between(PR, H3, &mut states);

    // LB26
    not_allowed_between(JL, JL, &mut states);
    not_allowed_between(JL, JV, &mut states);
    not_allowed_between(JL, H2, &mut states);
    not_allowed_between(JL, H3, &mut states);

    not_allowed_between(JV, JV, &mut states);
    not_allowed_between(JV, JT, &mut states);
    not_allowed_between(H2, JV, &mut states);
    not_allowed_between(H2, JT, &mut states);

    not_allowed_between(JT, JT, &mut states);
    not_allowed_between(H3, JT, &mut states);

    // LB25
    not_allowed_between(CL, PO, &mut states);
    not_allowed_between(CP, PO, &mut states);
    not_allowed_between(CL, PR, &mut states);
    not_allowed_between(CP, PR, &mut states);
    not_allowed_between(NU, PO, &mut states);
    not_allowed_between(NU, PR, &mut states);
    not_allowed_between(PO, OP, &mut states);
    not_allowed_between(PO, NU, &mut states);
    not_allowed_between(PR, OP, &mut states);
    not_allowed_between(PR, NU, &mut states);
    not_allowed_between(HY, NU, &mut states);
    not_allowed_between(IS, NU, &mut states);
    not_allowed_between(NU, NU, &mut states);
    not_allowed_between(SY, NU, &mut states);

    // LB24
    not_allowed_between(PR, AL, &mut states);
    not_allowed_between(PR, HL, &mut states);
    not_allowed_between(PO, AL, &mut states);
    not_allowed_between(PO, HL, &mut states);
    not_allowed_between(AL, PR, &mut states);
    not_allowed_between(AL, PO, &mut states);
    not_allowed_between(HL, PR, &mut states);
    not_allowed_between(HL, PO, &mut states);

    // LB23a
    not_allowed_between(PR, ID, &mut states);
    not_allowed_between(PR, EB, &mut states);
    not_allowed_between(PR, EM, &mut states);
    not_allowed_between(ID, PO, &mut states);
    not_allowed_between(EB, PO, &mut states);
    not_allowed_between(EM, PO, &mut states);

    // LB23
    not_allowed_between(AL, NU, &mut states);
    not_allowed_between(HL, NU, &mut states);
    not_allowed_between(NU, AL, &mut states);
    not_allowed_between(NU, HL, &mut states);

    // LB22
    not_allowed_between(AL, IN, &mut states);
    not_allowed_between(HL, IN, &mut states);
    not_allowed_between(EX, IN, &mut states);
    not_allowed_between(ID, IN, &mut states);
    not_allowed_between(EB, IN, &mut states);
    not_allowed_between(EM, IN, &mut states);
    not_allowed_between(IN, IN, &mut states);
    not_allowed_between(NU, IN, &mut states);

    // LB21b
    not_allowed_between(SY, HL, &mut states);

    // LB21a
    states[HL][HY].0 = LB21A_HY_STATE;
    states[HL][BA].0 = LB21A_BA_STATE;

    // LB21
    break_before(BA, false, &mut states);
    break_before(HY, false, &mut states);
    break_before(NS, false, &mut states);
    break_after(BB, false, &mut states);

    // LB20
    break_before(CB, true, &mut states);
    break_after(CB, true, &mut states);

    // LB19
    break_before(QU, false, &mut states);
    break_after(QU, false, &mut states);

    // LB18
    break_after(SP, true, &mut states);

    // LB17
    not_allowed_between(B2, B2, &mut states);
    states[B2][B2].1 = false;
    states[B2][SP].0 = LB17_STATE;

    // LB16
    not_allowed_between(CL, NS, &mut states);
    states[CL][SP].0 = LB16_STATE;

    not_allowed_between(CP, NS, &mut states);
    states[CP][SP].0 = LB16_STATE;

    // LB15
    states[QU][OP].1 = false;
    states[QU][SP].0 = LB15_STATE;

    // LB14
    break_after(OP, false, &mut states);
    states[OP][SP].0 = LB14_STATE;

    // LB13
    break_before(CL, false, &mut states);
    break_before(CP, false, &mut states);
    break_before(EX, false, &mut states);
    break_before(IS, false, &mut states);
    break_before(SY, false, &mut states);

    // LB12a
    for state in states.iter_mut().enumerate().filter_map(|(index, state)| {
        if LB12A_EXCEPTIONS.contains(&index) {
            None
        } else {
            Some(state)
        }
    }) {
        state[GL].1 = false;
    }

    // LB12
    break_after(GL, false, &mut states);

    // LB11
    break_after(WJ, false, &mut states);
    break_before(WJ, false, &mut states);

    // LB10
    states[AL][CM].1 = false;
    states[AL][ZWJ].1 = false;

    states[CM] = states[AL];
    states[ZWJ] = states[AL];

    // LB9
    for (i, state) in states.iter_mut().enumerate().filter_map(|(index, state)| {
        if LB9_EXCEPTIONS.contains(&index) {
            None
        } else {
            Some((index, state))
        }
    }) {
        state[CM] = (i, false);
        state[ZWJ] = (i, false);
    }

    // LB8a
    break_after(ZWJ, false, &mut states);

    // LB8
    break_after(ZW, true, &mut states);
    states[ZW][SP].0 = LB8_STATE;

    // LB7
    break_before(SP, false, &mut states);
    break_before(ZW, false, &mut states);

    // LB6
    break_before(BK, false, &mut states);
    break_before(CR, false, &mut states);
    break_before(LF, false, &mut states);
    break_before(NL, false, &mut states);

    // LB5
    break_after(CR, true, &mut states);
    break_after(LF, true, &mut states);
    break_after(NL, true, &mut states);
    not_allowed_between(CR, LF, &mut states);

    // LB4
    break_after(BK, true, &mut states);

    // LB2
    break_after(NUM_OF_CLASSES, false, &mut states);

    // Special extra states

    // LB8
    let mut new_state = states[SP].clone();
    for part in new_state.iter_mut().enumerate().filter_map(|(i, s)| {
        if [BK, CR, LF, NL, SP, ZW].contains(&i) {
            None
        } else {
            Some(s)
        }
    }) {
        part.1 = true;
    }
    extra_states.push(new_state);

    // LB14
    let mut new_state = states[SP].clone();
    for part in new_state.iter_mut() {
        part.1 = false;
    }
    extra_states.push(new_state);

    // LB15
    let mut new_state = states[SP].clone();
    new_state[OP].1 = false;
    extra_states.push(new_state);

    // LB16
    let mut new_state = states[SP].clone();
    new_state[NS].1 = false;
    extra_states.push(new_state);

    // LB17
    let mut new_state = states[SP].clone();
    new_state[B2].1 = false;
    extra_states.push(new_state);

    // LB21a
    let mut hy_state = states[HY].clone();
    for part in hy_state.iter_mut() {
        part.1 = false;
    }
    let mut ba_state = states[BA].clone();
    for part in ba_state.iter_mut() {
        part.1 = false;
    }
    extra_states.push(hy_state);
    extra_states.push(ba_state);

    // LB30a
    let mut even_state = states[RI].clone();
    even_state[RI] = (RI, true);
    extra_states.push(even_state);

    states.extend(extra_states.into_iter());
    write!(f, "const NUM_OF_CLASSES: usize = {};\nconst STATES: [[(usize, bool); NUM_OF_CLASSES]; {}] = [", NUM_OF_CLASSES, states.len()).unwrap();
    for state in states {
        write!(f, "[").unwrap();
        for value in state.iter() {
            write!(f, "{:?},", value).unwrap();
        }
        write!(f, "],").unwrap();
    }
    write!(f, "];").unwrap();
}
