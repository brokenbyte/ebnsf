mod common;
use common::save_svg;
use ebnsf::winnow::v4::parse_ebnf;

#[test]
fn grammars() {
    let x = std::fs::read_dir("grammars/")
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<_>>();

    for ele in x {
        let file = ele.path();
        let ebnf = std::fs::read_to_string(&file).expect("read grammar");

        let grammar = file.file_stem().expect("valid filename").to_string_lossy();

        let svg = parse_ebnf(&ebnf).expect("valid grammar").to_string();

        save_svg(&grammar, &svg);

        insta::assert_snapshot!(grammar.as_ref(), svg);
    }
}
