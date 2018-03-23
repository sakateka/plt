#[macro_use]
extern crate clap;
extern crate itertools;

mod args;
mod cfg;
mod generator;
mod dfa;
mod pda;

use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::{self, Read, Write, BufWriter};
use generator::{GeneratedItem, GeneratedSet, Generator};
use cfg::{Symbol, CFG};
use dfa::DFA;

fn main() {
    let app = args::build_app("plt");

    if let Some(matches) = app.subcommand_matches("gen") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = CFG::parse(grammar).unwrap();
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
        let mut output_stream = match matches.value_of("OUT") {
            Some(x) => {
                let path = Path::new(x);
                BufWriter::new(Box::new(File::create(&path).unwrap()) as Box<Write>)
            }
            None => BufWriter::new(Box::new(io::stdout()) as Box<Write>),
        };
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

        let mut output_stream = match matches.value_of("OUT") {
            Some(x) => {
                if matches.is_present("verbose") {
                    eprintln!("Write simplified grammar to {:?}", x);
                }
                let path = Path::new(x);
                Box::new(File::create(&path).unwrap()) as Box<Write>
            }
            None => Box::new(io::stdout()) as Box<Write>,
        };

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
        }
    } else if let Some(matches) = app.subcommand_matches("dfa") {
        let dfa_table = matches.value_of("DFA").unwrap();
        let debug = matches.is_present("debug");
        let show_path = matches.is_present("path");
        let mut dfa = DFA::parse(dfa_table, debug).unwrap();
        let mut input = match matches.value_of("INPUT") {
            Some(x) => {
                let path = Path::new(x);
                Box::new(File::open(&path).unwrap()) as Box<Read>
            }
            None => Box::new(io::stdin()) as Box<Read>,
        };
        dfa.check(input, show_path).unwrap();
    }
}
