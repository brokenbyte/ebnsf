#![allow(unused)]

use ::winnow::Parser as _;
use clap::Parser;
use ebnsf::{
    parse_ebnf,
    winnow::{self, EbnfGrammar},
};
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
    let mut input = r#"
<foo> <bar> ("hello" <baz>)
<womp>
"#.trim_start();

    let p = winnow::v4::parse_sequence(&mut input).unwrap();
    // let p = winnow::list2(&mut input).unwrap();
    dbg!(p);
    dbg!(input);

    /*
     *     let ebnf = r#"<valid semver> ::= <version core>
     *                   <version core> "-" <pre-release>
     *                  | <version core> "+" <build>
     *                  | <version core> "-" <pre-release> "+" <build>
     * "#;
     *     let diagram = match parse_ebnf(ebnf) {
     *         Ok(p) => p,
     *         Err(e) => {
     *             println!("{e}");
     *             std::process::exit(1);
     *         }
     *     };
     */
    // test_term();

    // test_group();

    // test_rule();

    // test_sequence();

    // test_sequence_list();

    // test_grammar();
}

fn test_sequence() {
    /*
     * let mut input = r#""hello" "world" | "foo" "bar" ("fizz", "buzz") "#;
     *  println!("input is: \n&{}&", input);
     *  let x = winnow::sequence(&mut input).unwrap();
     *  println!("input is: &{}&", input);
     *  println!("x is:     {:#?}", x);
     */

    /*
     * let mut input = r#"<world> ("hello")
     * "#;
     * println!("input is: \n&{}&", input);
     * let x = winnow::sequence(&mut input).unwrap();
     * println!("input is: &{}&", input);
     * println!("x is:     {:#?}", x);
     */
}

pub type DynNode = Box<dyn rr::Node>;

/*
 * fn render_grammar(ebnf: &EbnfGrammar) -> rr::Diagram<DynNode> {
 *     let nodes = ebnf
 *         .rules
 *         .iter()
 *         .map(|r| {
 *             Box::new(rr::Sequence::new(vec![
 *                 Box::new(rr::SimpleStart) as DynNode,
 *                 render_rule(r),
 *                 Box::new(rr::SimpleStart),
 *             ]))
 *         })
 *         .collect::<Vec<_>>();
 *
 *     let mut diagram = rr::Diagram::new(Box::new(rr::VerticalGrid::new(nodes)) as DynNode);
 *     diagram.add_css(rr::DEFAULT_CSS);
 *
 *     diagram
 * }
 */

/*
 * fn render_rule(rule: &winnow::Rule) -> DynNode {
 *     let name = Box::new(rr::Comment::new(unescape(&rule.name))) as DynNode;
 *     let mut choices = rule
 *         .choices
 *         .iter() // Each arm of the rule, e.g. <foo> | <bar>
 *         .map(|choice| {
 *             let items = choice;
 *             items
 *                 .iter() // Each node in the current rule
 *                 .map(|item| render_sequence_item(item))
 *                 .collect::<Vec<_>>()
 *         })
 *         .collect::<Vec<_>>();
 *
 *     if choices.len() == 1 {
 *         // Only one choice/production for the rule
 *         let definition = choices.remove(0);
 *
 *         let mut rule = Vec::with_capacity(1 + definition.len());
 *         rule.insert(0, name);
 *         rule.extend(definition);
 *
 *         Box::new(rr::Sequence::new(rule))
 *     } else {
 *         // Multiple choices/productions for the rule
 *         let productions = choices
 *             .into_iter()
 *             .map(rr::Sequence::new)
 *             .collect::<Vec<_>>();
 *
 *         let definitino = Box::new(rr::Choice::new(productions));
 *
 *         Box::new(rr::Sequence::new(vec![name, definitino]))
 *     }
 *
 * }
 */

/*
 * fn render_sequence_item(seq_item: &winnow::SequenceItem) -> DynNode {
 *     match seq_item {
 *         winnow::SequenceItem::Term(term) => render_term(term),
 *         winnow::SequenceItem::Group(group) => render_group(group),
 *     }
 * }
 */

/*
 * fn render_group(group: &winnow::Group) -> DynNode {
 *     let items = group
 *         .items
 *         .iter()
 *         .map(render_sequence_item)
 *         .collect::<Vec<_>>();
 *
 *     let s = Box::new(rr::Sequence::new(items));
 *
 *     render_modifier(s, &group.modifier)
 * }
 *
 */
/*
 * fn render_modifier(node: DynNode, modifier: &Option<winnow::Modifier>) -> DynNode {
 *     if let Some(m) = modifier {
 *         match m {
 *             winnow::Modifier::QMark => Box::new(rr::Optional::new(node)),
 *             winnow::Modifier::Plus => Box::new(rr::Repeat::new(node, rr::Empty)),
 *             winnow::Modifier::Star => Box::new(rr::Optional::new(rr::Repeat::new(node, rr::Empty))),
 *         }
 *     } else {
 *         node
 *     }
 * }
 */

/*
 * fn render_term(term: &winnow::Term) -> DynNode {
 *     let atom: DynNode = match &term.atom {
 *         winnow::Token::Terminal(s) => Box::new(rr::Terminal::new(unescape(s))),
 *         winnow::Token::NonTerminal(s) => Box::new(rr::NonTerminal::new(unescape(s))),
 *     };
 *
 *     render_modifier(atom, &term.modifier)
 * }
 */
fn unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut iter = s.chars();
    while let Some(ch) = iter.next() {
        result.push(match ch {
            '\\' => {
                let mut peekable = iter.clone().peekable();
                let escaped = peekable.peek().expect("no escaped char?");
                if ['"', '\'', '\\'].contains(escaped) {
                    iter.next().unwrap()
                } else {
                    ch
                }
            }
            _ => ch,
        });
    }
    println!("unescape input: {s}");
    println!("unescape output: {result}");
    result
}

fn test_grammar() {
    // let mut input = "<syntax> ::= <rule>\n";
    /*
     *
     * let mut input = "\
     * <syntax>         ::= <rule>+
     *
     *
     * <rule>           ::= <opt_whitespace> \"<\" <rule_name> \">\" <opt_whitespace> \"::=\" <opt_whitespace> <group> <line_end>
     * ";
     */
    /*
     * let x = winnow::rules.parse(input).unwrap();
     * dbg!(x);
     */

    /*
     * for r in &x.rules {
     *     println!("{r}")
     * }
     */

    //
}

// rule = { nonterminal ~ "::=" ~ sequence_list ~ NEWLINE*}
/*
 * fn test_rule() {
 *     let mut input = "\
 * <rule> ::= <foo>
 *         | (<foo> | <bar>)
 *         | (
 *                 <fizz>
 *               | (\"foo\" | \"bar\")
 *           )
 * ";
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::rule(&mut input).unwrap();
 *
 *     println!("input is: &{}&", input);
 *     println!("x is:     {}", x);
 * }
 */

// sequence_list = { sequence ~ ( "\n"* ~ "|" ~ "\n"* ~ sequence )* }
/*
 * fn test_sequence_list() {
 *     let mut input = r#""hello" "world" | "foo" "bar" ("fizz", "buzz") "#;
 *     let mut input = "\
 * <rule> ::= <foo>
 *         | (<foo> | <bar>)
 *         | (
 *                 <fizz>
 *               | (\"foo\" | \"bar\")
 *           )
 * ";
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::sequence_list(&mut input).unwrap();
 *     println!("input is: &{}&", input);
 *     for i in &x {
 *         println!("{i}")
 *     }
 *     // println!("x is:     {:#?}", x);
 * }
 */

// group = { "(" ~ NEWLINE* ~ (sequence_list) ~ NEWLINE* ~ ")" ~ opt_modifier}
/*
 * fn test_group() {
 *     let mut input = r#"("hello" |
 *     "world" "womp"
 *     )"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::group(&mut input).unwrap();
 *     println!("input is: &{}&", input);
 *     println!("x is:     {x}");
 * }
 */

// term = { (literal | nonterminal) ~ opt_modifier}
/*
 * fn test_term() {
 *     let mut input = r#""hello""#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 *
 *     let mut input = r#""hello"+"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 *
 *     let mut input = r#""hello"?"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 *
 *     let mut input = r#""hello"*"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 *
 *     let mut input = r#"<hello>+"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 *
 *     let mut input = r#"<hello>?"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 *
 *     let mut input = r#"<hello>*"#;
 *     println!("input is: \n&{}&", input);
 *     let x = winnow::term(&mut input);
 *     println!("input is: &{}&", input);
 *     println!("x is:     {:?}", x);
 * }
 */

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
