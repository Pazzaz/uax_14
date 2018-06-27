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
