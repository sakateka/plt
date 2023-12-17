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

use crate::cfg::{Symbol, CFG};
use crate::cyk::CYKParser;
use crate::dfa::DFA;
use crate::earley::EarleyParser;
use crate::generator::{GeneratedItem, GeneratedSet, Generator};
use itertools::{join, Itertools};
use crate::pda::DPDADesign;
use crate::pdt::DPDTDesign;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;
use std::process;

pub fn get_output_stream(x: Option<&String>) -> Box<dyn Write> {
    match x {
        Some(x) => {
            let path = Path::new(x);
            Box::new(File::create(path).unwrap()) as Box<dyn Write>
        }
        None => Box::new(io::stdout()) as Box<dyn Write>,
    }
}

pub fn get_input_stream(x: Option<&String>) -> Box<dyn Read> {
    match x {
        Some(x) => {
            let path = Path::new(x);
            Box::new(File::create(path).unwrap()) as Box<dyn Read>
        }
        None => Box::new(io::stdin()) as Box<dyn Read>,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = args::build_app();
    let mut help = Cursor::new(Vec::new());
    let _ = app.write_long_help(&mut help);
    let arg_matches = app.get_matches();

    //
    //// Gen
    //
    if let Some(matches) = arg_matches.subcommand_matches("gen") {
        let grammar = matches.get_one::<String>("CFG").unwrap();
        let cfg = CFG::load(grammar)
            .map(|x| {
                if matches.get_flag("chomsky") {
                    x.chomsky()
                } else {
                    x.simplify()
                }
            })
            .unwrap();
        let min: u32 = *matches.get_one::<u32>("len_min").unwrap_or(&0);
        let max: u32 = *matches.get_one::<u32>("len_max").unwrap_or(&8);
        let left = !matches.get_flag("right");
        let gen = Generator::new(cfg, min, max, left);
        let mut output_stream = BufWriter::new(get_output_stream(matches.get_one::<String>("OUT")));
        let mut visited = HashSet::new();
        for seq in gen {
            if !matches.get_flag("all") {
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
        let grammar = matches.get_one::<String>("CFG").unwrap();
        let mut cfg = CFG::load(grammar).unwrap();

        let mut output_stream = get_output_stream(matches.get_one::<String>("OUT"));

        let verbose = |title: &str, cfg: &CFG| {
            if matches.get_flag("verbose") {
                eprintln!("{}\n{}", title, cfg);
            }
            if matches.get_flag("debug") {
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
        if matches.get_flag("reverse") {
            cfg = remove_useless_and_unreachable(cfg);
            cfg = remove_epsilon_and_unit(cfg);
        } else {
            // default order
            cfg = remove_epsilon_and_unit(cfg);
            cfg = remove_useless_and_unreachable(cfg);
        }

        if matches.get_flag("chomsky") {
            cfg = cfg.chomsky();
            verbose("Chomsky Normal Form", &cfg);
        }
        output_stream.write_all(cfg.to_string().as_bytes()).unwrap();

    //
    //// CYK
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("cyk") {
        let show_path = matches.get_flag("parse");

        let grammar = matches.get_one::<String>("CFG").unwrap();
        let cfg = CFG::load(grammar).unwrap();
        let cyk = CYKParser::new(&cfg);

        let input = BufReader::new(get_input_stream(matches.get_one::<String>("INPUT")));

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
            } else if cyk.accepts(&text) {
                println!("- ACCEPT");
            } else {
                println!("- REFUSE");
            }
        }

    //
    //// EarleyParser
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("earley") {
        let grammar = matches.get_one::<String>("CFG").unwrap();
        let mut cfg = CFG::load(grammar).unwrap();
        if matches.get_flag("simplify") {
            cfg = cfg.simplify()
        }
        if matches.get_flag("chomsky") {
            cfg = cfg.chomsky()
        }
        let input = get_input_stream(matches.get_one::<String>("INPUT"));
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
        let dfa_table = matches.get_one::<String>("DFA").unwrap();
        let debug = matches.get_flag("debug");
        let show_path = matches.get_flag("path");
        let dfa = DFA::load(dfa_table, debug).unwrap();
        let input = get_input_stream(matches.get_one::<String>("INPUT"));
        dfa.check(input, show_path).unwrap();

    //
    //// DPDA
    //
    } else if let Some(matches) = arg_matches.subcommand_matches("dpda") {
        let dpda_spec = matches.get_one::<String>("DPDA").unwrap().as_str();

        let input = get_input_stream(matches.get_one::<String>("INPUT"));

        let dpda_design = DPDADesign::load(dpda_spec).unwrap();
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
        let dpdt_spec = matches.get_one::<String>("DPDT").unwrap().as_str();

        let input = get_input_stream(matches.get_one::<String>("INPUT"));

        let dpdt_design = DPDTDesign::load(dpdt_spec).unwrap();
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
        let grammar = matches.get_one::<String>("CFG").unwrap();
        let cfg = CFG::load(grammar).expect("Load CFG");

        let min: u32 = *matches.get_one::<u32>("len-min").unwrap_or(&0);
        let max: u32 = *matches.get_one::<u32>("len-max").unwrap_or(&8);
        if let Some(reason) = cfg.is_normal_form() {
            eprintln!("ERROR: {}\n{}", reason, cfg);
            process::exit(1);
        }
        let mut output_stream = BufWriter::new(get_output_stream(matches.get_one::<String>("OUT")));

        let chomsky_cfg = cfg.chomsky();
        output_stream
            .write_fmt(format_args!(
                "Chomsky Normal Form\nG({{{}}}, {{{}}}, P, {}) where P:\n{}\n",
                join(chomsky_cfg.get_terminals().iter().collect::<Vec<_>>(), ","),
                join(chomsky_cfg.get_variables().iter().collect::<Vec<_>>(), ","),
                chomsky_cfg.start,
                chomsky_cfg,
            ))
            .unwrap();

        let chomsky_gen = Generator::new(chomsky_cfg, min, max, true);
        let gen = Generator::new(cfg, min, max, true);

        let normal_set: HashSet<Vec<Symbol>> = gen.collect();
        let chomsky_set: HashSet<Vec<Symbol>> = chomsky_gen.collect();
        if normal_set != chomsky_set {
            output_stream
                .write_all(b"Sets of generated sequences do not match\n")
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
                .write_all(b"OK! The generated sets are equal!\n")
                .unwrap();
        }
        if matches.get_flag("verbose") {
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
            for (idx, key) in result.keys().sorted().enumerate() {
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
    } else {
        print!("{}", String::from_utf8(help.into_inner()).unwrap());
    }
    Ok(())
}
