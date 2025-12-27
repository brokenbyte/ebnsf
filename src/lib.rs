pub mod winnow;

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use railroad::{self as rr, Diagram};

pub type DynNode = Box<dyn rr::Node>;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct EBNFParser;

pub fn parse_ebnf(src: &str) -> Result<Diagram<DynNode>, Box<pest::error::Error<Rule>>> {
    let mut result = EBNFParser::parse(Rule::syntax, src)?;

    let trees = result.next().expect("expected root expr").into_inner();

    let nodes = trees
        .map(|p| {
            Box::new(rr::Sequence::new(vec![
                Box::new(rr::SimpleStart) as DynNode,
                Box::new(make_node(p)),
                Box::new(rr::SimpleStart),
            ]))
        })
        .collect::<Vec<_>>();

    let mut diagram = rr::Diagram::new(Box::new(rr::VerticalGrid::new(nodes)) as DynNode);

    diagram.add_css(rr::DEFAULT_CSS);

    Ok(diagram)
}

fn make_node(pair: Pair<'_, Rule>) -> Box<dyn rr::Node> {
    use Rule as R;

    match pair.as_rule() {
        R::rule => {
            let mut pair = pair.into_inner();
            let rule = pair.next().expect("no rule found");
            let name = Box::new(rr::Comment::new(unescape(&rule))) as DynNode;

            let expr = pair.next().expect("rule must have definition").into_inner();
            let mut rule_def = expr.map(make_node).collect::<Vec<_>>();

            if rule_def.len() == 1 {
                let node = rule_def.remove(0);
                Box::new(rr::Sequence::new(vec![name, node]))
            } else {
                let x = vec![name, Box::new(rr::Choice::new(rule_def))];

                Box::new(rr::Sequence::new(x))
            }
        }
        R::sequence_list => {
            let mut nodes = pair.into_inner().map(make_node).collect::<Vec<_>>();

            if nodes.len() == 1 {
                nodes.remove(0)
            } else {
                Box::new(rr::Choice::new(nodes))
            }
        }
        R::sequence => {
            let nodes = pair.into_inner().map(make_node).collect();
            Box::new(rr::Sequence::new(nodes))
        }
        R::group => {
            let mut pairs = pair.into_inner();
            let seq_list = pairs.next().unwrap();
            let modifier = pairs.next().unwrap();

            let node = make_node(seq_list);
            parse_modifier(node, modifier)
        }
        R::term => parse_term(pair),
        _ => {
            unreachable!("unhandled rule '{:?}' ({})", pair.as_rule(), pair.as_str());
        }
    }
}

fn parse_term(pair: Pair<'_, Rule>) -> DynNode {
    use Rule as R;

    let mut pairs = pair.into_inner();

    let node = pairs.next().unwrap();
    let modifier = pairs.next().unwrap();

    let node: DynNode = match node.as_rule() {
        R::literal => Box::new(rr::Terminal::new(unescape(&node))),
        R::nonterminal => Box::new(rr::NonTerminal::new(unescape(&node))),
        _ => {
            unreachable!("unhandled rule '{:?}' ({})", node.as_rule(), node.as_str());
        }
    };

    parse_modifier(node, modifier)
}

fn parse_modifier(node: DynNode, opt: Pair<'_, Rule>) -> DynNode {
    let modifier = opt.into_inner().next();

    if let Some(m) = modifier {
        use Rule as R;

        match m.as_rule() {
            R::oper_cond => Box::new(rr::Optional::new(node)),
            R::oper_plus => Box::new(rr::Repeat::new(node, rr::Empty)),
            R::oper_star => Box::new(rr::Optional::new(rr::Repeat::new(node, rr::Empty))),
            _ => {
                unreachable!("unsupported modifier'{:?}' ({})", m.as_rule(), m.as_str());
            }
        }
    } else {
        node
    }
}

// Modified from https://github.com/lukaslueg/railroad_dsl/blob/06841c393b323c83925304011d965c43a10127e7/src/lib.rs#L19
fn unescape(pair: &Pair<'_, Rule>) -> String {
    let s = pair.as_str();
    let mut result = String::with_capacity(s.len());
    let mut iter = s[1..s.len() - 1].chars();
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
    result
}
