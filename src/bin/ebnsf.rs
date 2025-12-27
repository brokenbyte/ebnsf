#![allow(unused)]

use ::winnow::Parser as _;
use clap::Parser;
use ebnsf::{parse_ebnf, winnow};
use railroad::{self as rr, Diagram, Empty};

use std::path::PathBuf;

#[derive(clap::Parser)]
struct Cli {
    /// File to read EBNF spec from
    input: String,

    /// Where to save the rendered SVG
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let ebnf = std::fs::read_to_string(&cli.input).unwrap();

    let diagram = match winnow::v4::parse_ebnf(&ebnf) {
        Ok(p) => p,
        Err(e) => {
            println!("{e}");
            std::process::exit(1);
        }
    };

    let output = if let Some(path) = cli.output {
        PathBuf::from(path)
    } else {
        let mut path = PathBuf::from(cli.input);
        path.set_extension("svg");
        path
    };

    std::fs::write(&output, diagram.to_string().into_bytes()).unwrap();
}
