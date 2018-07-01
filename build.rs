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

// TODO: Incorporate into build script
// GENERATING STATES
// const LB8_STATE: usize = NUM_OF_CLASSES + 1;
// const LB14_STATE: usize = NUM_OF_CLASSES + 2;
// const LB15_STATE: usize = NUM_OF_CLASSES + 3;
// const LB16_STATE: usize = NUM_OF_CLASSES + 4;
// const LB17_STATE: usize = NUM_OF_CLASSES + 5;
// const LB21A_HY_STATE: usize = NUM_OF_CLASSES + 6;
// const LB21A_BA_STATE: usize = NUM_OF_CLASSES + 7;
// const LB30A_EVEN_STATE: usize = NUM_OF_CLASSES + 8;
// const LB9_EXCEPTIONS: [usize; 8] = [
//     Class::BK as usize,
//     Class::CR as usize,
//     Class::LF as usize,
//     Class::NL as usize,
//     Class::SP as usize,
//     Class::ZW as usize,
//     Class::ZWJ as usize,
//     39,
// ];

// fn break_before(class: Class, b: bool, states: &mut Vec<[(usize, bool);
// NUM_OF_CLASSES]>) {     for state in states.iter_mut() {
//         state[class as usize].1 = b;
//     }
// }

// fn break_after(state: usize, b: bool, states: &mut Vec<[(usize, bool);
// NUM_OF_CLASSES]>) {     for c in states[state].iter_mut() {
//         c.1 = b;
//     }
// }

// fn not_allowed_between(c1: Class, c2: Class, states: &mut Vec<[(usize,
// bool); NUM_OF_CLASSES]>) {     states[c1 as usize][c2 as usize].1 = false;
// }

// const LB12A_EXCEPTIONS: [usize; 3] = [Class::SP as usize, Class::BA as
// usize, Class::HY as usize]; let mut states = Vec::new();
// let mut extra_states = Vec::new();

// for _ in 0..(NUM_OF_CLASSES + 1) {
//     states.push([
//         (0, true),
//         (1, true),
//         (2, true),
//         (3, true),
//         (4, true),
//         (5, true),
//         (6, true),
//         (7, true),
//         (8, true),
//         (9, true),
//         (10, true),
//         (11, true),
//         (12, true),
//         (13, true),
//         (14, true),
//         (15, true),
//         (16, true),
//         (17, true),
//         (18, true),
//         (19, true),
//         (20, true),
//         (21, true),
//         (22, true),
//         (23, true),
//         (24, true),
//         (25, true),
//         (26, true),
//         (27, true),
//         (28, true),
//         (29, true),
//         (30, true),
//         (31, true),
//         (32, true),
//         (33, true),
//         (34, true),
//         (35, true),
//         (36, true),
//         (37, true),
//         (38, true),
//     ]);
// }

// // LB30b
// not_allowed_between(Class::EB, Class::EM, &mut states);

// // LB30a
// not_allowed_between(Class::RI, Class::RI, &mut states);
// states[Class::RI as usize][Class::RI as usize].0 = LB30A_EVEN_STATE;

// // LB30
// not_allowed_between(Class::AL, Class::OP, &mut states);
// not_allowed_between(Class::HL, Class::OP, &mut states);
// not_allowed_between(Class::NU, Class::OP, &mut states);

// not_allowed_between(Class::CP, Class::AL, &mut states);
// not_allowed_between(Class::CP, Class::HL, &mut states);
// not_allowed_between(Class::CP, Class::NU, &mut states);

// // LB29
// not_allowed_between(Class::IS, Class::AL, &mut states);
// not_allowed_between(Class::IS, Class::HL, &mut states);

// // LB28
// not_allowed_between(Class::AL, Class::AL, &mut states);
// not_allowed_between(Class::AL, Class::HL, &mut states);
// not_allowed_between(Class::HL, Class::AL, &mut states);
// not_allowed_between(Class::HL, Class::HL, &mut states);

// // LB27
// not_allowed_between(Class::JL, Class::IN, &mut states);
// not_allowed_between(Class::JV, Class::IN, &mut states);
// not_allowed_between(Class::JT, Class::IN, &mut states);
// not_allowed_between(Class::H2, Class::IN, &mut states);
// not_allowed_between(Class::H3, Class::IN, &mut states);

// not_allowed_between(Class::JL, Class::PO, &mut states);
// not_allowed_between(Class::JV, Class::PO, &mut states);
// not_allowed_between(Class::JT, Class::PO, &mut states);
// not_allowed_between(Class::H2, Class::PO, &mut states);
// not_allowed_between(Class::H3, Class::PO, &mut states);

// not_allowed_between(Class::PR, Class::JL, &mut states);
// not_allowed_between(Class::PR, Class::JV, &mut states);
// not_allowed_between(Class::PR, Class::JT, &mut states);
// not_allowed_between(Class::PR, Class::H2, &mut states);
// not_allowed_between(Class::PR, Class::H3, &mut states);

// // LB26
// not_allowed_between(Class::JL, Class::JL, &mut states);
// not_allowed_between(Class::JL, Class::JV, &mut states);
// not_allowed_between(Class::JL, Class::H2, &mut states);
// not_allowed_between(Class::JL, Class::H3, &mut states);

// not_allowed_between(Class::JV, Class::JV, &mut states);
// not_allowed_between(Class::JV, Class::JT, &mut states);
// not_allowed_between(Class::H2, Class::JV, &mut states);
// not_allowed_between(Class::H2, Class::JT, &mut states);

// not_allowed_between(Class::JT, Class::JT, &mut states);
// not_allowed_between(Class::H3, Class::JT, &mut states);

// // LB25
// not_allowed_between(Class::CL, Class::PO, &mut states);
// not_allowed_between(Class::CP, Class::PO, &mut states);
// not_allowed_between(Class::CL, Class::PR, &mut states);
// not_allowed_between(Class::CP, Class::PR, &mut states);
// not_allowed_between(Class::NU, Class::PO, &mut states);
// not_allowed_between(Class::NU, Class::PR, &mut states);
// not_allowed_between(Class::PO, Class::OP, &mut states);
// not_allowed_between(Class::PO, Class::NU, &mut states);
// not_allowed_between(Class::PR, Class::OP, &mut states);
// not_allowed_between(Class::PR, Class::NU, &mut states);
// not_allowed_between(Class::HY, Class::NU, &mut states);
// not_allowed_between(Class::IS, Class::NU, &mut states);
// not_allowed_between(Class::NU, Class::NU, &mut states);
// not_allowed_between(Class::SY, Class::NU, &mut states);

// // LB24
// not_allowed_between(Class::PR, Class::AL, &mut states);
// not_allowed_between(Class::PR, Class::HL, &mut states);
// not_allowed_between(Class::PO, Class::AL, &mut states);
// not_allowed_between(Class::PO, Class::HL, &mut states);
// not_allowed_between(Class::AL, Class::PR, &mut states);
// not_allowed_between(Class::AL, Class::PO, &mut states);
// not_allowed_between(Class::HL, Class::PR, &mut states);
// not_allowed_between(Class::HL, Class::PO, &mut states);

// // LB23a
// not_allowed_between(Class::PR, Class::ID, &mut states);
// not_allowed_between(Class::PR, Class::EB, &mut states);
// not_allowed_between(Class::PR, Class::EM, &mut states);
// not_allowed_between(Class::ID, Class::PO, &mut states);
// not_allowed_between(Class::EB, Class::PO, &mut states);
// not_allowed_between(Class::EM, Class::PO, &mut states);

// // LB23
// not_allowed_between(Class::AL, Class::NU, &mut states);
// not_allowed_between(Class::HL, Class::NU, &mut states);
// not_allowed_between(Class::NU, Class::AL, &mut states);
// not_allowed_between(Class::NU, Class::HL, &mut states);

// // LB22
// not_allowed_between(Class::AL, Class::IN, &mut states);
// not_allowed_between(Class::HL, Class::IN, &mut states);
// not_allowed_between(Class::EX, Class::IN, &mut states);
// not_allowed_between(Class::ID, Class::IN, &mut states);
// not_allowed_between(Class::EB, Class::IN, &mut states);
// not_allowed_between(Class::EM, Class::IN, &mut states);
// not_allowed_between(Class::IN, Class::IN, &mut states);
// not_allowed_between(Class::NU, Class::IN, &mut states);

// // LB21b
// not_allowed_between(Class::SY, Class::HL, &mut states);

// // LB21a
// states[Class::HL as usize][Class::HY as usize].0 = LB21A_HY_STATE;
// states[Class::HL as usize][Class::BA as usize].0 = LB21A_BA_STATE;

// // LB21
// break_before(Class::BA, false, &mut states);
// break_before(Class::HY, false, &mut states);
// break_before(Class::NS, false, &mut states);
// break_after(Class::BB as usize, false, &mut states);

// // LB20
// break_before(Class::CB, true, &mut states);
// break_after(Class::CB as usize, true, &mut states);

// // LB19
// break_before(Class::QU, false, &mut states);
// break_after(Class::QU as usize, false, &mut states);

// // LB18
// break_after(Class::SP as usize, true, &mut states);

// // LB17
// not_allowed_between(Class::B2, Class::B2, &mut states);
// states[Class::B2 as usize][Class::B2 as usize].1 = false;
// states[Class::B2 as usize][Class::SP as usize].0 = LB17_STATE;

// // LB16
// not_allowed_between(Class::CL, Class::NS, &mut states);
// states[Class::CL as usize][Class::SP as usize].0 = LB16_STATE;

// not_allowed_between(Class::CP, Class::NS, &mut states);
// states[Class::CP as usize][Class::SP as usize].0 = LB16_STATE;

// // LB15
// states[Class::QU as usize][Class::OP as usize].1 = false;
// states[Class::QU as usize][Class::SP as usize].0 = LB15_STATE;

// // LB14
// break_after(Class::OP as usize, false, &mut states);
// states[Class::OP as usize][Class::SP as usize].0 = LB14_STATE;

// // LB13
// break_before(Class::CL, false, &mut states);
// break_before(Class::CP, false, &mut states);
// break_before(Class::EX, false, &mut states);
// break_before(Class::IS, false, &mut states);
// break_before(Class::SY, false, &mut states);

// // LB12a
// for state in states.iter_mut().enumerate().filter_map(|(index, state)| {
//     if LB12A_EXCEPTIONS.contains(&index) {
//         None
//     } else {
//         Some(state)
//     }
// }) {
//     state[Class::GL as usize].1 = false;
// }

// // LB12
// break_after(Class::GL as usize, false, &mut states);

// // LB11
// break_after(Class::WJ as usize, false, &mut states);
// break_before(Class::WJ, false, &mut states);

// // LB10
// states[Class::AL as usize][Class::CM as usize].1 = false;
// states[Class::AL as usize][Class::ZWJ as usize].1 = false;

// states[Class::CM as usize] = states[Class::AL as usize];
// states[Class::ZWJ as usize] = states[Class::AL as usize];

// // LB9
// for (i, state) in states.iter_mut().enumerate().filter_map(|(index, state)| {
//     if LB9_EXCEPTIONS.contains(&index) {
//         None
//     } else {
//         Some((index, state))
//     }
// }) {
//     state[Class::CM as usize] = (i, false);
//     state[Class::ZWJ as usize] = (i, false);
// }

// // LB8a
// break_after(Class::ZWJ as usize, false, &mut states);

// // LB8
// break_after(Class::ZW as usize, true, &mut states);
// states[Class::ZW as usize][Class::SP as usize].0 = LB8_STATE;

// // LB7
// break_before(Class::SP, false, &mut states);
// break_before(Class::ZW, false, &mut states);

// // LB6
// break_before(Class::BK, false, &mut states);
// break_before(Class::CR, false, &mut states);
// break_before(Class::LF, false, &mut states);
// break_before(Class::NL, false, &mut states);

// // LB5
// break_after(Class::CR as usize, true, &mut states);
// break_after(Class::LF as usize, true, &mut states);
// break_after(Class::NL as usize, true, &mut states);
// not_allowed_between(Class::CR, Class::LF, &mut states);

// // LB4
// break_after(Class::BK as usize, true, &mut states);

// // LB2
// break_after(NUM_OF_CLASSES, false, &mut states);

// // Special extra states

// // LB8
// let mut new_state = states[Class::SP as usize].clone();
// for part in new_state.iter_mut().enumerate().filter_map(|(i, s)| {
//     if [
//         Class::BK as usize,
//         Class::CR as usize,
//         Class::LF as usize,
//         Class::NL as usize,
//         Class::SP as usize,
//         Class::ZW as usize,
//     ].contains(&i)
//     {
//         None
//     } else {
//         Some(s)
//     }
// }) {
//     part.1 = true;
// }
// extra_states.push(new_state);

// // LB14
// let mut new_state = states[Class::SP as usize].clone();
// for part in new_state.iter_mut() {
//     part.1 = false;
// }
// extra_states.push(new_state);

// // LB15
// let mut new_state = states[Class::SP as usize].clone();
// new_state[Class::OP as usize].1 = false;
// extra_states.push(new_state);

// // LB16
// let mut new_state = states[Class::SP as usize].clone();
// new_state[Class::NS as usize].1 = false;
// extra_states.push(new_state);

// // LB17
// let mut new_state = states[Class::SP as usize].clone();
// new_state[Class::B2 as usize].1 = false;
// extra_states.push(new_state);

// // LB21a
// let mut hy_state = states[Class::HY as usize].clone();
// for part in hy_state.iter_mut() {
//     part.1 = false;
// }
// let mut ba_state = states[Class::BA as usize].clone();
// for part in ba_state.iter_mut() {
//     part.1 = false;
// }
// extra_states.push(hy_state);
// extra_states.push(ba_state);

// // LB30a
// let mut even_state = states[Class::RI as usize].clone();
// even_state[Class::RI as usize] = (Class::RI as usize, true);
// extra_states.push(even_state);

// states.extend(extra_states.into_iter());
