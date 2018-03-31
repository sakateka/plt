#[macro_use]
extern crate clap;
extern crate itertools;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;

mod args;
mod cfg;
mod dfa;
mod generator;
mod pda;

use cfg::{Symbol, CFG};
use dfa::DFA;
use generator::{GeneratedItem, GeneratedSet, Generator};
use itertools::{join, Itertools};
use pda::DPDADesign;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::process;

pub fn get_output_stream(x: Option<&str>) -> Box<Write> {
    match x {
        Some(x) => {
            let path = Path::new(x);
            Box::new(File::create(&path).unwrap()) as Box<Write>
        }
        None => Box::new(io::stdout()) as Box<Write>,
    }
}

pub fn get_input_stream(x: Option<&str>) -> Box<Read> {
    match x {
        Some(x) => {
            let path = Path::new(x);
            Box::new(File::create(&path).unwrap()) as Box<Read>
        }
        None => Box::new(io::stdin()) as Box<Read>,
    }
}

fn main() {
    let app = args::build_app("plt");

    if let Some(matches) = app.subcommand_matches("gen") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::parse(grammar)
            .and_then(|x| {
                Ok(if matches.is_present("chomsky") {
                    x.chomsky()
                } else {
                    x.simplify()
                })
            })
            .unwrap();
        let mut min: u32 = 0;
        if matches.is_present("len-min") {
            min = value_t_or_exit!(matches, "len-min", u32);
        }
        let mut max: u32 = 8;
        if matches.is_present("len-max") {
            max = value_t_or_exit!(matches, "len-max", u32);
        }
        let left = !matches.is_present("right");
        let gen = Generator::new(cfg, min, max, left);
        let mut output_stream = BufWriter::new(get_output_stream(matches.value_of("OUT")));
        if matches.is_present("all") {
            for seq in gen {
                output_stream
                    .write_fmt(format_args!("{}\n", GeneratedItem(&seq)))
                    .unwrap();
            }
        } else {
            output_stream
                .write_fmt(format_args!(
                    "{}",
                    GeneratedSet(gen.collect::<HashSet<Vec<Symbol>>>())
                ))
                .unwrap();
        }
    } else if let Some(matches) = app.subcommand_matches("simplify") {
        let grammar = matches.value_of("CFG").unwrap();
        let mut cfg = CFG::parse(grammar).unwrap();

        let mut output_stream = BufWriter::new(get_output_stream(matches.value_of("OUT")));

        output_stream
            .write_fmt(format_args!("{}", cfg.simplify()))
            .unwrap();
        if matches.is_present("verbose") {
            eprint!("\nParsed CFG\n{}", cfg);
            cfg = cfg.remove_epsilon_rules();
            eprint!("Remove epsilon\n{}", cfg);
            if matches.is_present("debug") {
                eprintln!("{:?}\n", cfg);
            }
            cfg = cfg.remove_unit_rules();
            eprint!("Remove units\n{}", cfg);
            if matches.is_present("debug") {
                eprintln!("{:?}\n", cfg);
            }
            cfg = cfg.remove_useless_rules();
            eprint!("Remove useless\n{}", cfg);
            if matches.is_present("debug") {
                eprintln!("{:?}\n", cfg);
            }
            cfg = cfg.remove_unreachable_rules();
            eprint!("Remove unreachable\n{}", cfg);
            if matches.is_present("debug") {
                eprintln!("{:?}\n", cfg);
            }
            if matches.is_present("chomsky") {
                cfg = cfg.chomsky();
                eprint!("Convert to Chomsky Normal Form\n{}", cfg);
                if matches.is_present("debug") {
                    eprintln!("{:?}\n", cfg);
                }
            }
        }
    } else if let Some(matches) = app.subcommand_matches("dfa") {
        let dfa_table = matches.value_of("DFA").unwrap();
        let debug = matches.is_present("debug");
        let show_path = matches.is_present("path");
        let dfa = DFA::parse(dfa_table, debug).unwrap();
        let input = get_input_stream(matches.value_of("INPUT"));
        dfa.check(input, show_path).unwrap();
    } else if let Some(matches) = app.subcommand_matches("dpda") {
        let dpda_spec = matches.value_of("DPDA").unwrap();

        let input = get_input_stream(matches.value_of("INPUT"));

        let mut dpda_design = DPDADesign::load(dpda_spec).unwrap();
        dpda_design.remember_traversed_path = matches.is_present("path");

        let dpda_design = dpda_design;
        let buf = BufReader::new(input);
        for line in buf.lines() {
            let text = line.unwrap();
            let result = dpda_design.accepts(text.clone());
            if let Some(path) = result.path {
                println!(
                    "{} -> [{}]",
                    join(
                        path.iter()
                            .cloned()
                            .map(|x| {
                                if let Some(rule) = x {
                                    format!("{}", rule)
                                } else {
                                    String::new()
                                }
                            })
                            .collect::<Vec<String>>(),
                        " -> "
                    ),
                    result.cfg.state,
                );
            }
            if result.ok {
                println!("{} -> OK", text);
            } else {
                let msg = if text.len() == result.eaten_part.len() {
                    "EOL but not accepted".to_string()
                } else {
                    format!(
                        "Stuck after {} chars '{}'",
                        result.eaten_part.len(),
                        result.eaten_part
                    )
                };
                println!("{} -> ERR: {}, current {:?}", text, msg, result.cfg);
            }
        }
    } else if let Some(matches) = app.subcommand_matches("coursework") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::parse(grammar).expect("Parse CFG");
        let mut min: u32 = 0;
        if matches.is_present("len-min") {
            min = value_t_or_exit!(matches, "len-min", u32);
        }
        let mut max: u32 = 8;
        if matches.is_present("len-max") {
            max = value_t_or_exit!(matches, "len-max", u32);
        }
        if let Some(reason) = cfg.is_normal_form() {
            eprintln!("ERROR: {}\n{}", reason, cfg);
            process::exit(1);
        }
        let chomsky_gen = Generator::new(cfg.chomsky(), min, max, true);
        let gen = Generator::new(cfg, min, max, true);

        let mut output_stream = BufWriter::new(get_output_stream(matches.value_of("OUT")));

        let normal_set: HashSet<Vec<cfg::Symbol>> = gen.collect();
        let chomsky_set: HashSet<Vec<cfg::Symbol>> = chomsky_gen.collect();
        if normal_set != chomsky_set {
            output_stream
                .write(b"Sets of generated sequences do not match\n")
                .unwrap();
            let diff = GeneratedSet(chomsky_set.difference(&normal_set).cloned().collect());
            if !diff.0.is_empty() {
                output_stream
                    .write_fmt(format_args!("chomsky_set - normal_set =:\n{}\n", diff,))
                    .unwrap();
            }
            let diff = GeneratedSet(normal_set.difference(&chomsky_set).cloned().collect());
            if !diff.0.is_empty() {
                output_stream
                    .write_fmt(format_args!("normal_set - chomsky_set =:\n{}\n", diff,))
                    .unwrap();
            }
        } else {
            output_stream
                .write(b"OK! The generated sets are equal!\n")
                .unwrap();
        }
        if matches.is_present("verbose") {
            let mut result: HashMap<String, (String, String)> = HashMap::new();
            normal_set.iter().for_each(|x| {
                let key = GeneratedItem(x).to_string();
                result.insert(key.clone(), (key, "".to_owned()));
            });
            chomsky_set.iter().for_each(|x| {
                let key = GeneratedItem(x).to_string();
                let present = result.contains_key(&key);
                if present {
                    let val = result.get_mut(&key).unwrap();
                    val.1 = key;
                } else {
                    result.insert(key.clone(), ("".to_owned(), key));
                }
            });
            let num_width = format!("{}", result.len()).len();
            output_stream
                .write_fmt(format_args!(
                    "The generated sets\n{:>num_width$} {:width$} {:width$}\n",
                    "№",
                    "CFG",
                    "CNF",
                    num_width = num_width,
                    width = max as usize,
                ))
                .unwrap();
            for (idx, key) in result.keys().sorted().into_iter().enumerate() {
                let v = result.get(key).unwrap();
                output_stream
                    .write_fmt(format_args!(
                        "{:>num_width$} {:width$} {:width$}\n",
                        idx,
                        v.0,
                        v.1,
                        num_width = num_width,
                        width = max as usize,
                    ))
                    .unwrap();
            }
        }
    }
}
