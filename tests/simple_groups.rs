use ebnsf::parse_ebnf;

mod common;
use common::save_svg;

#[test]
fn simple_group_tests() {
    let rule = "\
<rule> ::= <foo> (<bar> \"|\" <bish> <bosh>)*
<rule> ::= <foo> | (<bish> | <bosh>)
<rule> ::= <foo> | (<bish> | <bosh>)?
<rule> ::= <foo> | (<bish> | <bosh>)+
<rule> ::= <foo> | (<bish> | <bosh>)*
";

    let svg = parse_ebnf(rule); //.expect("valid grammar").to_string();

    let svg = match svg {
        Ok(p) => p,
        Err(e) => {
            panic!("{}", e)
        }
    }
    .to_string();

    save_svg("simple_group_tests", &svg);

    insta::assert_snapshot!("simple_group_tests", svg);
}
