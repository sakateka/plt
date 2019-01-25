#[macro_use]
extern crate clap;
extern crate itertools;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_yaml;

mod args;
mod cfg;
mod cyk;
mod dfa;
mod earley;
mod generator;
mod pda;
mod pdt;
mod sdt;

use cfg::{Symbol, CFG};
use cyk::CYKParser;
use dfa::DFA;
use earley::EarleyParser;
use generator::{GeneratedItem, GeneratedSet, Generator};
use itertools::{join, Itertools};
use pda::DPDADesign;
use pdt::DPDTDesign;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Write};
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
    let mut app = args::build_app("plt");
    let mut help = Cursor::new(Vec::new());
    let _ = app.write_long_help(&mut help);
    let arg_matches = app.get_matches();

    //
    //// Gen
    //
    if let Some(matches) = arg_matches.subcommand_matches("gen") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::load(grammar)
            .and_then(|x| {
                Ok(if matches.is_present("chomsky") {
                    x.chomsky()
                } else {
                    x.simplify()
                })
            }).unwrap();
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
        let mut visited = HashSet::new();
        for seq in gen {
            if !matches.is_present("all") {
                if visited.contains(&seq) {
                    continue;
                } else {
                    visited.insert(seq.clone());
                }
            }
            output_stream
                .write_fmt(format_args!("{}\n", GeneratedItem(&seq)))
                .unwrap();
        }

    //
    //// Simplify
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("simplify") {
        let grammar = matches.value_of("CFG").unwrap();
        let mut cfg = CFG::load(grammar).unwrap();

        let mut output_stream = get_output_stream(matches.value_of("OUT"));

        let verbose = |title: &str, cfg: &CFG| {
            if matches.is_present("verbose") {
                eprintln!("{}\n{}", title, cfg);
            }
            if matches.is_present("debug") {
                eprintln!("{}\n{:?}\n", title, cfg);
            }
        };
        verbose("Load CFG", &cfg);
        let remove_epsilon_and_unit = |mut cfg: CFG| -> CFG {
            cfg = cfg.remove_epsilon_rules();
            verbose("Remove epsilon", &cfg);

            cfg = cfg.remove_unit_rules();
            verbose("Remove units", &cfg);

            cfg
        };

        let remove_useless_and_unreachable = |mut cfg: CFG| -> CFG {
            cfg = cfg.remove_useless_rules();
            verbose("Remove useless", &cfg);

            cfg = cfg.remove_unreachable_rules();
            verbose("Remove unreachable", &cfg);

            cfg
        };
        if matches.is_present("reverse") {
            cfg = remove_useless_and_unreachable(cfg);
            cfg = remove_epsilon_and_unit(cfg);
        } else {
            // default order
            cfg = remove_epsilon_and_unit(cfg);
            cfg = remove_useless_and_unreachable(cfg);
        }

        if matches.is_present("chomsky") {
            cfg = cfg.chomsky();
            verbose("Chomsky Normal Form", &cfg);
        }
        output_stream.write_all(cfg.to_string().as_bytes()).unwrap();

    //
    //// CYK
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("cyk") {
        let show_path = matches.is_present("parse");

        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::load(grammar).unwrap();
        let cyk = CYKParser::new(&cfg);

        let input = BufReader::new(get_input_stream(matches.value_of("INPUT")));

        for line in input.lines() {
            let text = line.unwrap();
            print!("'{}'", text);
            if show_path {
                if let Some(path) = cyk.parse(&text) {
                    println!("- ACCEPT");
                    for item in path {
                        println!("{:4} -> {}", item.left.to_string(), join(&item.right, ""));
                    }
                } else {
                    println!("- REFUSE");
                }
            } else {
                if cyk.accepts(&text) {
                    println!("- ACCEPT");
                } else {
                    println!("- REFUSE");
                }
            }
        }

    //
    //// EarleyParser
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("earley") {
        let grammar = matches.value_of("CFG").unwrap();
        let mut cfg = CFG::load(grammar).unwrap();
        if matches.is_present("simplify") {
            cfg = cfg.simplify()
        }
        if matches.is_present("chomsky") {
            cfg = cfg.chomsky()
        }
        let input = get_input_stream(matches.value_of("INPUT"));
        let earley = EarleyParser::new(&cfg);
        let buf = BufReader::new(input);
        for line in buf.lines() {
            let text = line.unwrap();
            let states = earley.parse(&text);
            earley.print(&states);
            //earley.derivation_path(&states);
        }

    //
    //// DFA
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("dfa") {
        let dfa_table = matches.value_of("DFA").unwrap();
        let debug = matches.is_present("debug");
        let show_path = matches.is_present("path");
        let dfa = DFA::load(dfa_table, debug).unwrap();
        let input = get_input_stream(matches.value_of("INPUT"));
        dfa.check(input, show_path).unwrap();

    //
    //// DPDA
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("dpda") {
        let dpda_spec = matches.value_of("DPDA").unwrap();

        let input = get_input_stream(matches.value_of("INPUT"));

        let mut dpda_design = DPDADesign::load(dpda_spec).unwrap();

        let dpda_design = dpda_design;
        let buf = BufReader::new(input);
        for line in buf.lines() {
            let text = line.unwrap();
            let result = dpda_design.accepts(&text);
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

    //
    //// DPDT
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("dpdt") {
        let dpdt_spec = matches.value_of("DPDT").unwrap();

        let input = get_input_stream(matches.value_of("INPUT"));

        let mut dpdt_design = DPDTDesign::load(dpdt_spec).unwrap();

        let dpdt_design = dpdt_design;
        let buf = BufReader::new(input);
        for line in buf.lines() {
            let text = line.unwrap();
            let result = dpdt_design.accepts(&text);
            if result.ok {
                println!(
                    "OK: {} -> {}",
                    text,
                    result
                        .cfg
                        .translated
                        .iter()
                        .fold(String::new(), |mut acc, x| {
                            acc.push_str(x);
                            acc
                        })
                );
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
                println!("ERR: {}: {}, current {:?}", text, msg, result.cfg);
            }
        }

    //
    //// Course Work
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("coursework") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::load(grammar).expect("Load CFG");
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
        let mut output_stream = BufWriter::new(get_output_stream(matches.value_of("OUT")));

        let chomsky_cfg = cfg.chomsky();
        output_stream
            .write_fmt(format_args!(
                "Chomsky Normal Form\nG({{{}}}, {{{}}}, P, {}) where P:\n{}\n",
                join(chomsky_cfg.get_terminals().iter().collect::<Vec<_>>(), ","),
                join(chomsky_cfg.get_variables().iter().collect::<Vec<_>>(), ","),
                chomsky_cfg.start,
                chomsky_cfg,
            )).unwrap();

        let chomsky_gen = Generator::new(chomsky_cfg, min, max, true);
        let gen = Generator::new(cfg, min, max, true);

        let normal_set: HashSet<Vec<Symbol>> = gen.collect();
        let chomsky_set: HashSet<Vec<Symbol>> = chomsky_gen.collect();
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
                )).unwrap();
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
                    )).unwrap();
            }
        }
    } else {
        print!("{}", String::from_utf8(help.into_inner()).unwrap());
    }
}
