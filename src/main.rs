#[macro_use]
extern crate clap;

mod args;

fn main() {
    let app = args::build_app("bmper");

    if let Some(matches) = app.subcommand_matches("generate") {
        println!("Yep!");
    }
}
