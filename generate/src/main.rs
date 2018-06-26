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
            Some(_) => mn.push((number, None)),
            None => mc.push((number, None)),
        };
    }
    let final_mn = squish(mn);
    let final_mc = squish(mc);

    let re = Regex::new(r"(([0-9a-zA-Z]+)(\.\.([0-9a-zA-Z]+))?);([A-Z0-9]+)").unwrap(); // ZWJ
    let mut hash: HashMap<&str, Vec<(u32, Option<u32>)>> = HashMap::new();
    for caps in re.captures_iter(LINEBREAK) {
        let numbers: (u32, Option<u32>) = match caps.get(4) {
            Some(right_n) => {
                let left_n = u32::from_str_radix(caps.get(2).unwrap().as_str(), 16).expect("Could not parse u32");
                let right_n = u32::from_str_radix(right_n.as_str(), 16).expect("Could not parse u32");
                (left_n, Some(right_n))
            }
            None => {
                let left_n = u32::from_str_radix(caps.get(1).unwrap().as_str(), 16).expect("invalid u32");
                (left_n, None)
            },
        };
        let class = caps.get(5).unwrap().as_str();
        hash.entry(class).or_insert(Vec::new()).push(numbers);
    }
    println!("// Automatically generated from the code in `../generate`
use Class;
/// Converts a `char` to its corresponding [Line Breaking Class].
///
/// For more information see [`Class`].
///
/// [Line Breaking Class]: https://www.unicode.org/reports/tr14/#Table1
pub fn convert_to_break_class(n: char) -> Class {{match n as u32 {{");
    for (key, value) in hash.into_iter().map(|(key, list)|(key, squish(list))) {
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
        "0x1F000...0x1FFFD // Plane 1 range
        => Class::ID,
        0x20A0...0x20CF // Currency Symbols
        => Class::PR,
        _ => Class::AL // Actually XX
    }}
}}"
    );
}

fn squish(values: Vec<(u32, Option<u32>)>) -> Vec<String> {
    let mut collected: Vec<(u32, Option<u32>)> = Vec::new();
    let mut lower = values[0].0;
    let mut higher = values[0].1;
    for window in values.windows(2) {
        let (left_0, right_0) = window[0];
        let (left_1, right_1) = window[1];
        match right_0 {
            Some(right_0_value) => {
                match right_1 {
                    Some(right_1_value) => {
                        // (u32, Some(a)) (u32, Some(b))
                        if right_0_value == left_1 - 1 {
                            higher = Some(right_1_value);
                        } else {
                            collected.push((lower, higher));
                            higher = Some(right_1_value);
                            lower = left_1;
                        }
                    }
                    None => {
                        // (u32, Some(a)) (u32, None)

                        if right_0_value == left_1 - 1 {
                            higher = Some(left_1);
                        } else {
                            collected.push((lower, higher));
                            higher = None;
                            lower = left_1;
                        }
                    }
                }
            }
            None => {
                match right_1 {
                    Some(right_1_value) => {
                        // (u32, None) (u32, Some(b))

                        if left_0 == left_1 - 1 {
                            higher = Some(right_1_value);
                        } else {
                            collected.push((lower, higher));
                            higher = Some(right_1_value);
                            lower = left_1;
                        }
                    }
                    None => {
                        // (u32, Some(a)) (u32, None)

                        if left_0 == left_1 - 1 {
                            higher = Some(left_1);
                        } else {
                            collected.push((lower, higher));
                            higher = None;
                            lower = left_1;
                        }
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    collected.push((lower, higher));
    for part in collected {
        match part.1 {
            Some(x) => out.push(format!("{:X}...0x{:X}", part.0, x)),
            None => out.push(format!("{:X}", part.0)),
        }
    }
    out
}
