#![allow(unused)]

use ::winnow::Parser as _;
// use ebnsf::{parse_ebnf, winnow};

use ebnsf::winnow::v4::Grammar;

fn main() {
    let mut input = "\"unclo\\sed\"";
    /*
     * let x = winnow::v4::terminal2.parse(input);
     * println!("{:?}", x.unwrap());
     */

    let g: Result<Grammar, _> = input.parse();

    match g {
        Ok(_) => {
            println!("Successful parse")
        }
        Err(e) => {
            println!("{e}")
        }
    }
    //
}
