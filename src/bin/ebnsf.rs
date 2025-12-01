#![allow(unused)]

use ::winnow::Parser as _;
use clap::Parser;
use ebnsf::{parse_ebnf, winnow};

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
    // test_term();

    // test_group();

    // test_rule();

    // test_sequence();

    // test_sequence_list();

    test_grammar();
}

fn test_grammar() {
    let mut input = "\
<rule> ::= <foo>
        | (<foo> | <bar>)
        | (
                <fizz>
              | (\"foo\" | \"bar\")
          )

<rule> ::= <foo>
        | (<foo> | <bar>)
        | (
                <fizz>
              | (\"foo\" | \"bar\")
          )

<rule> ::= <foo>
        | (<foo> | <bar> | (\"yoinky\"  \"sploinky\"))
        | (
                <aaa>
              | (\"bbb\" | \"ccc\")
          )
";

    let x = winnow::rules.parse_next(&mut input).unwrap();

    for r in &x.rules {
        println!("{r}")
    }

    //
}

// rule = { nonterminal ~ "::=" ~ sequence_list ~ NEWLINE*}
fn test_rule() {
    let mut input = "\
<rule> ::= <foo>
        | (<foo> | <bar>)
        | (
                <fizz>
              | (\"foo\" | \"bar\")
          )
";
    println!("input is: \n&{}&", input);
    let x = winnow::rule(&mut input).unwrap();
    println!("input is: &{}&", input);
    println!("x is:     {}", x);
}

// sequence_list = { sequence ~ ( "\n"* ~ "|" ~ "\n"* ~ sequence )* }
fn test_sequence_list() {
    let mut input = r#""hello" "world" | "foo" "bar" ("fizz", "buzz") "#;
    let mut input = "\
<rule> ::= <foo>
        | (<foo> | <bar>)
        | (
                <fizz>
              | (\"foo\" | \"bar\")
          )
";
    println!("input is: \n&{}&", input);
    let x = winnow::sequence_list(&mut input).unwrap();
    println!("input is: &{}&", input);
    for i in &x {
        println!("{i}")
    }
    // println!("x is:     {:#?}", x);
}

fn test_sequence() {
    /*
     * let mut input = r#""hello" "world" | "foo" "bar" ("fizz", "buzz") "#;
     *  println!("input is: \n&{}&", input);
     *  let x = winnow::sequence(&mut input).unwrap();
     *  println!("input is: &{}&", input);
     *  println!("x is:     {:#?}", x);
     */

    let mut input = r#"<world> ("hello")"#;
    println!("input is: \n&{}&", input);
    let x = winnow::sequence(&mut input).unwrap();
    println!("input is: &{}&", input);
    println!("x is:     {:#?}", x);
}

// group = { "(" ~ NEWLINE* ~ (sequence_list) ~ NEWLINE* ~ ")" ~ opt_modifier}
fn test_group() {
    let mut input = r#"("hello" |
    "world" "womp"
    )"#;
    println!("input is: \n&{}&", input);
    let x = winnow::group(&mut input).unwrap();
    println!("input is: &{}&", input);
    println!("x is:     {x}");
}

// term = { (literal | nonterminal) ~ opt_modifier}
fn test_term() {
    let mut input = r#""hello""#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);

    let mut input = r#""hello"+"#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);

    let mut input = r#""hello"?"#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);

    let mut input = r#""hello"*"#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);

    let mut input = r#"<hello>+"#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);

    let mut input = r#"<hello>?"#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);

    let mut input = r#"<hello>*"#;
    println!("input is: \n&{}&", input);
    let x = winnow::term(&mut input);
    println!("input is: &{}&", input);
    println!("x is:     {:?}", x);
}

fn main2() {
    let cli = Cli::parse();

    let ebnf = std::fs::read_to_string(&cli.input).unwrap();

    let diagram = match parse_ebnf(&ebnf) {
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
