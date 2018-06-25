extern crate regex;
use regex::Regex;
use std::collections::HashMap;
use std::u32;

// These are not included in the repository and has to be downloaded seperately
// https://www.unicode.org/Public/UCD/latest/ucd/LineBreak.txt
// http://ftp.unicode.org/Public/UNIDATA/UnicodeData.txt
const LINEBREAK: &'static str = include_str!("LineBreak-11.0.0.txt");
const UNICODEDATA: &'static str = include_str!("UnicodeData.txt");

fn main() {
    let re1 = Regex::new(r"([0-9A-F]+);[^;]+;((Mn)|(Mc));").unwrap();
    let mut mn = Vec::new();
    let mut mc = Vec::new();
    for caps in re1.captures_iter(UNICODEDATA) {
        let number: u32 =
            u32::from_str_radix(caps.get(1).unwrap().as_str(), 16).expect("Couldn't parse number");
        match caps.get(3) {
            Some(_) => mn.push(number),
            None => mc.push(number),
        };
    }
    let final_mn = squish(mn);
    let final_mc = squish(mc);

    let re = Regex::new(r"(([0-9a-zA-Z]+)(\.\.([0-9a-zA-Z]+))?);([A-Z0-9]+)").unwrap(); // ZWJ
    let mut hash: HashMap<&str, Vec<String>> = HashMap::new();
    for caps in re.captures_iter(LINEBREAK) {
        let mut numbers = String::new();
        match caps.get(4) {
            Some(right_n) => {
                let left_n = caps.get(2).unwrap().as_str();
                numbers.push_str(left_n);
                numbers.push_str("..=0x");
                numbers.push_str(right_n.as_str());
            }
            None => numbers.push_str(caps.get(1).unwrap().as_str()),
        }
        let class = caps.get(5).unwrap().as_str();
        hash.entry(class).or_insert(Vec::new()).push(numbers);
    }
    println!("match n as u32 {{");
    for (key, value) in hash {
        match key {
            "SA" => println!(
                "0x{} => match n as u32 {{0x{}|0x{} => Class::CM,_ => Class::AL}}",
                value.join(" | 0x"),
                final_mn.join(" | 0x"),
                final_mc.join(" | 0x")
            ),
            "XX" | "SG" | "AI" => println!("0x{} => Class::AL,", value.join(" | 0x")),
            "CJ" => println!("0x{} => Class::NS,", value.join(" | 0x")),
            _ => println!("0x{} => Class::{},", value.join(" | 0x"), key),
        }
    }
    println!(
        "not_covered => match not_covered {{
            0x3400..=0x4DBF  // CJK Unified Ideographs Extension A
            | 0x4E00..=0x9FFF // CJK Unified Ideographs
            | 0xF900..=0xFAFF // CJK Compatibility Ideographs
            | 0x20000..=0x2FFFD // Plane 2
            | 0x30000..=0x3FFFD // Plane 3
            | 0x1F000..=0x1FFFD // Plane 1 range
            => Class::ID,
            0x20A0..=0x20CF // Currency Symbols
            => Class::PR,
            _ => Class::AL // Actually XX
        }},
    }}"
    );
}

fn squish(values: Vec<u32>) -> Vec<String> {
    let mut collected: Vec<(u32, Option<u32>)> = Vec::new();
    let mut lower = values[0];
    let mut higher = None;
    for window in values.windows(2) {
        if window[0] == window[1] - 1 {
            higher = Some(window[1]);
        } else {
            collected.push((lower, higher));
            higher = None;
            lower = window[1]
        }
    }
    let mut out = Vec::new();
    collected.push((lower, higher));
    for part in collected {
        match part.1 {
            Some(x) => out.push(format!("{:X}..=0x{:X}", part.0, x)),
            None => out.push(format!("{:X}", part.0)),
        }
    }
    out
}
