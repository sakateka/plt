extern crate clap;

mod args;
mod cfg;

fn main() {
    let app = args::build_app("plt");

    if let Some(matches) = app.subcommand_matches("gen") {
        let grammar = matches.value_of("CFG").unwrap();
        let cfg = cfg::CFG::parse(grammar).unwrap();
        println!("Yep!");
    }
}
