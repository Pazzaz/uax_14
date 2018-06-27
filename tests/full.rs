extern crate regex;
extern crate uax_14;
use regex::Regex;
use std::char;
use uax_14::{convert_to_break_class, Break, Class, LineBreaks};

// LB25 Disagrees with these tests
const SKIP_TESTS: [usize; 30] = [
    1113, 1115, 1117, 1119, 1281, 1283, 1285, 1287, 2953, 2955, 4469, 4471, 4637, 4639, 5137, 5139,
    7109, 7118, 7123, 7208, 7209, 7210, 7211, 7212, 7213, 7215, 7216, 7217, 7218, 7219,
];

const DATA: &'static str = include_str!("data.txt");

fn main() {
    let re1 = Regex::new(r"×(( [0-9A-F]+ [÷×])+)").unwrap();
    let re2 = Regex::new(r"([0-9A-F]+) ([÷×])").unwrap();
    let mut correct = 0;
    let mut total = 0;
    let mut printing = true;
    for (i, caps) in re1.captures_iter(DATA).enumerate() {
        if SKIP_TESTS.contains(&(i + 1)) {
            if printing {
                print!(".");
            }
            continue;
        }
        total += 1;

        let parts = caps.get(1).unwrap().as_str();
        let mut converted: Vec<(u32, Break)> = Vec::new();
        for caps in re2.captures_iter(parts) {
            let number_str = caps.get(1).unwrap().as_str();
            let number = u32::from_str_radix(number_str, 16).expect("Failed to parse");
            let br = match caps.get(2).unwrap().as_str() {
                "÷" => Break::Opportunity,
                "×" => Break::Prohibited,
                _ => panic!(),
            };
            converted.push((number, br));
        }
        let just_codepoints: Vec<u32> = converted.iter().map(|(a, _)| *a).collect();
        let input_string: String = just_codepoints
            .clone()
            .iter()
            .map(|i| char::from_u32(*i).unwrap())
            .collect();
        let my_answer: Vec<(u32, Break)> = LineBreaks::new(&input_string)
            .map(|(a, b)| {
                if b == Break::Mandatory {
                    (a as u32, Break::Opportunity)
                } else {
                    (a as u32, b)
                }
            })
            .collect();
        if my_answer == converted {
            correct += 1;
            if printing {
                print!("i");
            }
        } else {
            if printing {
                print!("\x1B[31;40mf\x1B[0m");
                println!(
                    "\nindex: {}\nMy answer:\n{:?}\nRight answer:\n{:?}\nMy Classes:\n{:?}",
                    i + 1,
                    my_answer,
                    converted,
                    just_codepoints
                        .iter()
                        .map(|a| convert_to_break_class(char::from_u32(*a).unwrap()))
                        .collect::<Vec<Class>>()
                );
            }
            printing = false;
        }
    }
    println!("\n{}/{} ({} ignored)", correct, total, SKIP_TESTS.len());
}
