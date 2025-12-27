use ebnsf::winnow::v4::parse_ebnf;

mod common;
use common::save_svg;

#[test]
fn simple_choice() {
    let rule = "<rule> ::= <foo> | <bar>";

    let svg = parse_ebnf(rule).expect("valid grammar").to_string();

    save_svg("simple_choice", &svg);

    insta::assert_snapshot!(svg);
}

#[test]
fn simple_optional() {
    let rule = "<rule> ::= <foo>?";

    let svg = parse_ebnf(rule).expect("valid grammar").to_string();

    save_svg("simple_optional", &svg);

    insta::assert_snapshot!(svg);
}

#[test]
fn simple_star() {
    let rule = "<rule> ::= <foo>*";

    let svg = parse_ebnf(rule).expect("valid grammar").to_string();

    save_svg("simple_star", &svg);

    insta::assert_snapshot!(svg);
}

#[test]
fn simple_plus() {
    let rule = "<rule> ::= <foo>+";

    let svg = parse_ebnf(rule).expect("valid grammar").to_string();

    save_svg("simple_plus", &svg);

    insta::assert_snapshot!(svg);
}
