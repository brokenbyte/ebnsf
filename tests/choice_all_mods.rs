use ebnsf::parse_ebnf;

mod common;
use common::save_svg;

#[test]
fn choice_all_mods() {
    let ebnf = "\
<star or opt>  ::= <foo>* | <bar>?
<star or star> ::= <foo>* | <bar>*
<star or plus> ::= <foo>* | <bar>+

<plus or opt>  ::= <foo>+ | <bar>?
<plus or star> ::= <foo>+ | <bar>*
<plus or plus> ::= <foo>+ | <bar>+

<opt or opt>  ::= <foo>? | <bar>?
<opt or star> ::= <foo>? | <bar>*
<opt or plus> ::= <foo>? | <bar>+
";

    let svg = parse_ebnf(ebnf); //.expect("valid grammar").to_string();

    let svg = match svg {
        Ok(p) => p,
        Err(e) => {
            panic!("{}", e)
        }
    }
    .to_string();

    save_svg("choice_all_mods", &svg);

    // insta::assert_snapshot!(svg);
}
