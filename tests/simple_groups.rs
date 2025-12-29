use ebnsf::winnow::v4::parse_ebnf;

mod common;
use common::save_svg;

#[test]
fn simple_groups() {
    let rule = "\
<rule> ::= <foo> (<bar> \"|\" <bish> <bosh>)*
<rule> ::= <foo> | (<bish> | <bosh>)
<rule> ::= <foo> | (<bish> | <bosh>)?
<rule> ::= <foo> | (<bish> | <bosh>)+
<rule> ::= <foo> | (<bish> | <bosh>)*
<rule> ::= (<foo> | (<bish> | <bosh>)*)?
";

    let svg = parse_ebnf(rule).expect("valid grammar").to_string();

    save_svg("simple_groups", &svg);

    insta::assert_snapshot!("simple_groups", svg);
}

#[test]
fn multiline_groups() {
    let rule = "\
<rule> ::= <foo>
        | (<foo> | <bar>)
        | (
                <fizz>
              | (\"foo\" | \"bar\")
          )
";

    let svg = parse_ebnf(rule).expect("valid grammar").to_string();

    save_svg("multiline_groups", &svg);

    insta::assert_snapshot!("multiline_groups", svg);
}
