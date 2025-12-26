use winnow::ModalResult;
use winnow::ascii::{alpha1, multispace0, multispace1, space0, space1};
use winnow::combinator::{alt, delimited, preceded, separated};
use winnow::prelude::*;
use winnow::token::take_while;

// Custom data structures for EBNF AST
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Terminal(String),
    Nonterminal(String),
    Group(Alternative),
}

pub type Sequence = Vec<Element>;
pub type Alternative = Vec<Sequence>;

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub alternatives: Alternative,
}

pub type Grammar = Vec<Rule>;

// Terminal parser: parses "alphanum..."
pub fn terminal<'a>(input: &'a mut &str) -> ModalResult<Element> {
    delimited('"', rule_name, '"')
        .map(Element::Terminal)
        .parse_next(input)
}

// Nonterminal parser: parses <alphanum...>
pub fn nonterminal<'a>(input: &'a mut &str) -> ModalResult<Element> {
    delimited('<', rule_name, '>')
        .map(Element::Nonterminal)
        .parse_next(input)
}

// Atom parser: chooses between terminal, nonterminal, or group
pub fn atom<'a>(input: &'a mut &str) -> ModalResult<Element> {
    alt((terminal, nonterminal, group)).parse_next(input)
}

// Sequence parser: space-separated list of atoms
pub fn sequence<'a>(input: &'a mut &str) -> ModalResult<Sequence> {
    separated(1.., atom, space1).parse_next(input)
}

// Top-level parser: trims whitespace and parses sequences separated by |
pub fn parse_sequence<'a>(input: &'a mut &str) -> ModalResult<Alternative> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    separated(1.., sequence, delimited(multispace0, "|", multispace0)).parse_next(input)
}

// Group parser: parses (sequence) recursively, supporting alternatives
pub fn group<'a>(input: &'a mut &str) -> ModalResult<Element> {
    delimited('(', parse_sequence.map(Element::Group), ')').parse_next(input)
}

// Parses a single rule: <rule_name> ::= <alternatives>
pub fn parse_rule<'a>(input: &'a mut &str) -> ModalResult<Rule> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    let (name_elem, alts) = (
        nonterminal,
        preceded((space0, "::=", space0), parse_sequence),
    )
        .parse_next(input)?;
    let name = match name_elem {
        Element::Nonterminal(n) => n,
        _ => unreachable!(),
    };
    Ok(Rule {
        name,
        alternatives: alts,
    })
}

// Parses multiple rules separated by whitespace/newlines
pub fn parse_grammar<'a>(input: &'a mut &str) -> ModalResult<Grammar> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    separated(1.., parse_rule, multispace1).parse_next(input)
}

// Helper: rule_name for terminals/nonterminals (alphanumeric starting with letter)
pub fn rule_name<'a>(input: &'a mut &str) -> ModalResult<String> {
    let rule = (
        alpha1,
        take_while(0.., ('a'..='z', 'A'..='Z', '0'..='9', ' ', '_', '-')),
    )
        .take()
        .parse_next(input)?
        .to_string();

    Ok(rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sequence() {
        let mut input = "<foo> \"bar\" <baz>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec![
                Element::Nonterminal("foo".to_string()),
                Element::Terminal("bar".to_string()),
                Element::Nonterminal("baz".to_string())
            ]]
        );

        let mut input2 = "<foo> <woo> <boo> \"baz\"";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec![
                Element::Nonterminal("foo".to_string()),
                Element::Nonterminal("woo".to_string()),
                Element::Nonterminal("boo".to_string()),
                Element::Terminal("baz".to_string())
            ]]
        );

        let mut input3 = "\"honk\" \"bonk\" \"tonk\" <wonk>";
        assert_eq!(
            parse_sequence(&mut input3).unwrap(),
            vec![vec![
                Element::Terminal("honk".to_string()),
                Element::Terminal("bonk".to_string()),
                Element::Terminal("tonk".to_string()),
                Element::Nonterminal("wonk".to_string())
            ]]
        );

        let mut input4 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz>";
        assert_eq!(
            parse_sequence(&mut input4).unwrap(),
            vec![vec![
                Element::Nonterminal("foo".to_string()),
                Element::Nonterminal("bar".to_string()),
                Element::Group(vec![vec![
                    Element::Terminal("hello".to_string()),
                    Element::Nonterminal("baz".to_string())
                ]]),
                Element::Terminal("world".to_string()),
                Element::Nonterminal("buz".to_string())
            ]]
        );
    }

    #[test]
    fn test_parse_alternatives() {
        let mut input = "<foo> <bar> | \"baz\" <qux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![
                vec![
                    Element::Nonterminal("foo".to_string()),
                    Element::Nonterminal("bar".to_string())
                ],
                vec![
                    Element::Terminal("baz".to_string()),
                    Element::Nonterminal("qux".to_string())
                ]
            ]
        );

        let mut input2 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz> | \"honk\" <bonk> (<tonk> \"shonk\" <donk>) \"lonk\"";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![
                vec![
                    Element::Nonterminal("foo".to_string()),
                    Element::Nonterminal("bar".to_string()),
                    Element::Group(vec![vec![
                        Element::Terminal("hello".to_string()),
                        Element::Nonterminal("baz".to_string())
                    ]]),
                    Element::Terminal("world".to_string()),
                    Element::Nonterminal("buz".to_string())
                ],
                vec![
                    Element::Terminal("honk".to_string()),
                    Element::Nonterminal("bonk".to_string()),
                    Element::Group(vec![vec![
                        Element::Nonterminal("tonk".to_string()),
                        Element::Terminal("shonk".to_string()),
                        Element::Nonterminal("donk".to_string())
                    ]]),
                    Element::Terminal("lonk".to_string())
                ]
            ]
        );
    }

    #[test]
    fn test_nested_groups() {
        let mut input = "<foo> (\"bar\" (<baz> \"qux\")) <quux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec![
                Element::Nonterminal("foo".to_string()),
                Element::Group(vec![vec![
                    Element::Terminal("bar".to_string()),
                    Element::Group(vec![vec![
                        Element::Nonterminal("baz".to_string()),
                        Element::Terminal("qux".to_string())
                    ]])
                ]]),
                Element::Nonterminal("quux".to_string())
            ]]
        );

        let mut input2 = "(<a> (\"b\" <c>)) | (<d> \"e\")";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![
                vec![Element::Group(vec![vec![
                    Element::Nonterminal("a".to_string()),
                    Element::Group(vec![vec![
                        Element::Terminal("b".to_string()),
                        Element::Nonterminal("c".to_string())
                    ]])
                ]])],
                vec![Element::Group(vec![vec![
                    Element::Nonterminal("d".to_string()),
                    Element::Terminal("e".to_string())
                ]])]
            ]
        );
    }

    #[test]
    fn test_alternatives_in_groups() {
        let mut input = "<foo> (\"bar\" | \"baz\") <qux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec![
                Element::Nonterminal("foo".to_string()),
                Element::Group(vec![
                    vec![Element::Terminal("bar".to_string())],
                    vec![Element::Terminal("baz".to_string())]
                ]),
                Element::Nonterminal("qux".to_string())
            ]]
        );

        let mut input2 = "(<a> | <b>) (\"c\" | \"d\")";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec![
                Element::Group(vec![
                    vec![Element::Nonterminal("a".to_string())],
                    vec![Element::Nonterminal("b".to_string())]
                ]),
                Element::Group(vec![
                    vec![Element::Terminal("c".to_string())],
                    vec![Element::Terminal("d".to_string())]
                ])
            ]]
        );
    }

    #[test]
    fn test_multiline_alternatives() {
        let mut input = "<foo> <bar>\n| <fizz> <buzz>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![
                vec![
                    Element::Nonterminal("foo".to_string()),
                    Element::Nonterminal("bar".to_string())
                ],
                vec![
                    Element::Nonterminal("fizz".to_string()),
                    Element::Nonterminal("buzz".to_string())
                ]
            ]
        );
    }

    #[test]
    fn test_parse_rule() {
        let mut input = "<foo> ::= <bar> | \"baz\"";
        let result = parse_rule(&mut input).unwrap();
        assert_eq!(result.name, "foo");
        assert_eq!(
            result.alternatives,
            vec![
                vec![Element::Nonterminal("bar".to_string())],
                vec![Element::Terminal("baz".to_string())]
            ]
        );
    }

    #[test]
    fn test_parse_grammar() {
        let mut input = "<foo> ::= <bar>\n<baz> ::= \"qux\" | <quux>";
        let result = parse_grammar(&mut input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "foo");
        assert_eq!(
            result[0].alternatives,
            vec![vec![Element::Nonterminal("bar".to_string())]]
        );
        assert_eq!(result[1].name, "baz");
        assert_eq!(
            result[1].alternatives,
            vec![
                vec![Element::Terminal("qux".to_string())],
                vec![Element::Nonterminal("quux".to_string())]
            ]
        );
    }
}
