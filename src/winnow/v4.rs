use railroad::{self as rr, Diagram};
use std::error::Error;
use winnow::ModalResult;
use winnow::ascii::{alpha1, multispace0, multispace1, space0, space1};
use winnow::combinator::{alt, delimited, preceded, separated};
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_while;

pub type DynNode = Box<dyn rr::Node>;

// Custom data structures for EBNF AST
#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    Optional,   // ?
    ZeroOrMore, // *
    OneOrMore,  // +
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Terminal(String),
    Nonterminal(String),
    Group(Alternative),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub atom: Atom,
    pub modifier: Option<Modifier>,
}

pub type Sequence = Vec<Element>;
pub type Alternative = Vec<Sequence>;

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub alternatives: Alternative,
}

pub type Grammar = Vec<Rule>;

// Modifier parser: parses ?, *, + (optional, at most one)
pub fn modifier(input: &mut &str) -> ModalResult<Option<Modifier>> {
    use winnow::combinator::opt;
    use winnow::token::one_of;
    opt(one_of(['?', '*', '+']).map(|c| match c {
        '?' => Modifier::Optional,
        '*' => Modifier::ZeroOrMore,
        '+' => Modifier::OneOrMore,
        _ => unreachable!(),
    }))
    .parse_next(input)
}

// Element parser: combines atom with optional modifier
pub fn element(input: &mut &str) -> ModalResult<Element> {
    let (atom, modifier) = (atom, modifier).parse_next(input)?;
    Ok(Element { atom, modifier })
}

// Terminal parser: parses quoted (single/double) strings
pub fn terminal(input: &mut &str) -> ModalResult<Element> {
    use winnow::token::take_until;
    alt((
        delimited('"', take_until(1.., '"'), '"'),
        delimited('\'', take_until(1.., '\''), '\''),
    ))
    .map(|content: &str| Element {
        atom: Atom::Terminal(content.to_string()),
        modifier: None,
    })
    .parse_next(input)
}

// Nonterminal parser: parses <alphanum...>
pub fn nonterminal(input: &mut &str) -> ModalResult<Element> {
    use winnow::token::take_until;
    delimited('<', take_until(1.., '>'), '>')
        .map(|content: &str| Element {
            atom: Atom::Nonterminal(content.to_string()),
            modifier: None,
        })
        .parse_next(input)
}

// Atom parser: chooses between terminal, nonterminal, or group
pub fn atom(input: &mut &str) -> ModalResult<Atom> {
    alt((
        terminal.map(|e| e.atom),
        nonterminal.map(|e| e.atom),
        group.map(Atom::Group),
    ))
    .parse_next(input)
}

// Sequence parser: space-separated list of elements
pub fn sequence(input: &mut &str) -> ModalResult<Sequence> {
    (multispace0).void().parse_next(input)?;
    separated(1.., element, space1).parse_next(input)
}

// Top-level parser: trims whitespace and parses sequences separated by |
pub fn parse_sequence(input: &mut &str) -> ModalResult<Alternative> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    separated(1.., sequence, delimited(multispace0, "|", multispace0)).parse_next(input)
}

// Group parser: parses (sequence) recursively, supporting alternatives
pub fn group(input: &mut &str) -> ModalResult<Alternative> {
    delimited('(', parse_sequence, ')').parse_next(input)
}

// Parses a single rule: <rule_name> ::= <alternatives>
pub fn parse_rule(input: &mut &str) -> ModalResult<Rule> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    let (name_elem, alts) = (
        nonterminal,
        preceded((space0, "::=", space0), parse_sequence),
    )
        .parse_next(input)?;
    let name = match name_elem.atom {
        Atom::Nonterminal(n) => n,
        _ => unreachable!(),
    };
    Ok(Rule {
        name,
        alternatives: alts,
    })
}

// Parses multiple rules separated by whitespace/newlines
pub fn parse_grammar(input: &mut &str) -> ModalResult<Grammar> {
    (multispace0).void().parse_next(input)?; // Skip leading whitespace
    separated(1.., parse_rule, multispace1).parse_next(input)
}

// Parses EBNF and builds a railroad diagram
pub fn parse_ebnf(src: &str) -> Result<Diagram<DynNode>, Box<dyn Error>> {
    use winnow::combinator::eof;
    let mut input = src;
    let grammar = parse_grammar(&mut input).map_err(|e| format!("Parsing error: {:?}", e))?;
    multispace0::<&str, ContextError<()>>
        .parse_next(&mut input)
        .map_err(|e| format!("Trailing whitespace error: {:?}", e))?;
    eof::<&str, ContextError<()>>
        .parse_next(&mut input)
        .map_err(|e| format!("Extra input: {:?}", e))?;
    let diagram = build_diagram(grammar);
    Ok(diagram)
}

fn build_diagram(grammar: Grammar) -> Diagram<DynNode> {
    let nodes: Vec<DynNode> = grammar
        .into_iter()
        .map(|rule| {
            Box::new(rr::Sequence::new(vec![
                Box::new(rr::SimpleStart) as DynNode,
                build_rule(&rule),
                Box::new(rr::SimpleStart),
            ])) as DynNode
        })
        .collect();

    let mut diagram = Diagram::new(Box::new(rr::VerticalGrid::new(nodes)) as DynNode);
    diagram.add_css(rr::DEFAULT_CSS);
    diagram
}

fn build_rule(rule: &Rule) -> DynNode {
    let name = Box::new(rr::Comment::new(rule.name.clone())) as DynNode;
    let alt_node = build_alternative(&rule.alternatives);

    Box::new(rr::Sequence::new(vec![name, alt_node]))
}

fn build_alternative(alt: &Alternative) -> DynNode {
    if alt.len() == 1 {
        build_sequence(&alt[0])
    } else {
        Box::new(rr::Choice::new(alt.iter().map(build_sequence).collect()))
    }
}

fn build_sequence(seq: &Sequence) -> DynNode {
    Box::new(rr::Sequence::new(seq.iter().map(build_element).collect()))
}

fn build_element(elem: &Element) -> DynNode {
    let mut node = build_atom(&elem.atom);
    if let Some(modifier) = &elem.modifier {
        node = apply_modifier(node, modifier);
    }
    node
}

fn build_atom(atom: &Atom) -> DynNode {
    match atom {
        Atom::Terminal(s) => Box::new(rr::Terminal::new(s.clone())),
        Atom::Nonterminal(s) => Box::new(rr::NonTerminal::new(s.clone())),
        Atom::Group(alt) => build_alternative(alt),
    }
}

fn apply_modifier(node: DynNode, modifier: &Modifier) -> DynNode {
    match modifier {
        Modifier::Optional => Box::new(rr::Optional::new(node)),
        Modifier::ZeroOrMore => Box::new(rr::Optional::new(rr::Repeat::new(node, rr::Empty))),
        Modifier::OneOrMore => Box::new(rr::Repeat::new(node, rr::Empty)),
    }
}

// Helper: rule_name for terminals/nonterminals (alphanumeric starting with letter)
pub fn rule_name(input: &mut &str) -> ModalResult<String> {
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
                Element {
                    atom: Atom::Nonterminal("foo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("bar".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("baz".to_string()),
                    modifier: None
                }
            ]]
        );

        let mut input2 = "<foo> <woo> <boo> \"baz\"";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Nonterminal("foo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("woo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("boo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("baz".to_string()),
                    modifier: None
                }
            ]]
        );

        let mut input3 = "\"honk\" \"bonk\" \"tonk\" <wonk>";
        assert_eq!(
            parse_sequence(&mut input3).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Terminal("honk".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("bonk".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("tonk".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("wonk".to_string()),
                    modifier: None
                }
            ]]
        );

        let mut input4 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz>";
        assert_eq!(
            parse_sequence(&mut input4).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Nonterminal("foo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("bar".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Group(vec![vec![
                        Element {
                            atom: Atom::Terminal("hello".to_string()),
                            modifier: None
                        },
                        Element {
                            atom: Atom::Nonterminal("baz".to_string()),
                            modifier: None
                        }
                    ]]),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("world".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("buz".to_string()),
                    modifier: None
                }
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
                    Element {
                        atom: Atom::Nonterminal("foo".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("bar".to_string()),
                        modifier: None
                    }
                ],
                vec![
                    Element {
                        atom: Atom::Terminal("baz".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("qux".to_string()),
                        modifier: None
                    }
                ]
            ]
        );

        let mut input2 = "<foo> <bar> (\"hello\" <baz>) \"world\" <buz> | \"honk\" <bonk> (<tonk> \"shonk\" <donk>) \"lonk\"";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![
                vec![
                    Element {
                        atom: Atom::Nonterminal("foo".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("bar".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Group(vec![vec![
                            Element {
                                atom: Atom::Terminal("hello".to_string()),
                                modifier: None
                            },
                            Element {
                                atom: Atom::Nonterminal("baz".to_string()),
                                modifier: None
                            }
                        ]]),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Terminal("world".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("buz".to_string()),
                        modifier: None
                    }
                ],
                vec![
                    Element {
                        atom: Atom::Terminal("honk".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("bonk".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Group(vec![vec![
                            Element {
                                atom: Atom::Nonterminal("tonk".to_string()),
                                modifier: None
                            },
                            Element {
                                atom: Atom::Terminal("shonk".to_string()),
                                modifier: None
                            },
                            Element {
                                atom: Atom::Nonterminal("donk".to_string()),
                                modifier: None
                            }
                        ]]),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Terminal("lonk".to_string()),
                        modifier: None
                    }
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
                Element {
                    atom: Atom::Nonterminal("foo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Group(vec![vec![
                        Element {
                            atom: Atom::Terminal("bar".to_string()),
                            modifier: None
                        },
                        Element {
                            atom: Atom::Group(vec![vec![
                                Element {
                                    atom: Atom::Nonterminal("baz".to_string()),
                                    modifier: None
                                },
                                Element {
                                    atom: Atom::Terminal("qux".to_string()),
                                    modifier: None
                                }
                            ]]),
                            modifier: None
                        }
                    ]]),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("quux".to_string()),
                    modifier: None
                }
            ]]
        );

        let mut input2 = "(<a> (\"b\" <c>)) | (<d> \"e\")";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![
                vec![Element {
                    atom: Atom::Group(vec![vec![
                        Element {
                            atom: Atom::Nonterminal("a".to_string()),
                            modifier: None
                        },
                        Element {
                            atom: Atom::Group(vec![vec![
                                Element {
                                    atom: Atom::Terminal("b".to_string()),
                                    modifier: None
                                },
                                Element {
                                    atom: Atom::Nonterminal("c".to_string()),
                                    modifier: None
                                }
                            ]]),
                            modifier: None
                        }
                    ]]),
                    modifier: None
                }],
                vec![Element {
                    atom: Atom::Group(vec![vec![
                        Element {
                            atom: Atom::Nonterminal("d".to_string()),
                            modifier: None
                        },
                        Element {
                            atom: Atom::Terminal("e".to_string()),
                            modifier: None
                        }
                    ]]),
                    modifier: None
                }]
            ]
        );
    }

    #[test]
    fn test_alternatives_in_groups() {
        let mut input = "<foo> (\"bar\" | \"baz\") <qux>";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Nonterminal("foo".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Group(vec![
                        vec![Element {
                            atom: Atom::Terminal("bar".to_string()),
                            modifier: None
                        }],
                        vec![Element {
                            atom: Atom::Terminal("baz".to_string()),
                            modifier: None
                        }]
                    ]),
                    modifier: None
                },
                Element {
                    atom: Atom::Nonterminal("qux".to_string()),
                    modifier: None
                }
            ]]
        );

        let mut input2 = "(<a> | <b>) (\"c\" | \"d\")";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Group(vec![
                        vec![Element {
                            atom: Atom::Nonterminal("a".to_string()),
                            modifier: None
                        }],
                        vec![Element {
                            atom: Atom::Nonterminal("b".to_string()),
                            modifier: None
                        }]
                    ]),
                    modifier: None
                },
                Element {
                    atom: Atom::Group(vec![
                        vec![Element {
                            atom: Atom::Terminal("c".to_string()),
                            modifier: None
                        }],
                        vec![Element {
                            atom: Atom::Terminal("d".to_string()),
                            modifier: None
                        }]
                    ]),
                    modifier: None
                }
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
                    Element {
                        atom: Atom::Nonterminal("foo".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("bar".to_string()),
                        modifier: None
                    }
                ],
                vec![
                    Element {
                        atom: Atom::Nonterminal("fizz".to_string()),
                        modifier: None
                    },
                    Element {
                        atom: Atom::Nonterminal("buzz".to_string()),
                        modifier: None
                    }
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
                vec![Element {
                    atom: Atom::Nonterminal("bar".to_string()),
                    modifier: None
                }],
                vec![Element {
                    atom: Atom::Terminal("baz".to_string()),
                    modifier: None
                }]
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
            vec![vec![Element {
                atom: Atom::Nonterminal("bar".to_string()),
                modifier: None
            }]]
        );
        assert_eq!(result[1].name, "baz");
        assert_eq!(
            result[1].alternatives,
            vec![
                vec![Element {
                    atom: Atom::Terminal("qux".to_string()),
                    modifier: None
                }],
                vec![Element {
                    atom: Atom::Nonterminal("quux".to_string()),
                    modifier: None
                }]
            ]
        );
    }

    #[test]
    fn test_modifiers() {
        let mut input = "<foo>? \"bar\"* <baz>+";
        assert_eq!(
            parse_sequence(&mut input).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Nonterminal("foo".to_string()),
                    modifier: Some(Modifier::Optional)
                },
                Element {
                    atom: Atom::Terminal("bar".to_string()),
                    modifier: Some(Modifier::ZeroOrMore)
                },
                Element {
                    atom: Atom::Nonterminal("baz".to_string()),
                    modifier: Some(Modifier::OneOrMore)
                }
            ]]
        );

        let mut input2 = "(<a> | <b>)? \"c\"*";
        assert_eq!(
            parse_sequence(&mut input2).unwrap(),
            vec![vec![
                Element {
                    atom: Atom::Group(vec![
                        vec![Element {
                            atom: Atom::Nonterminal("a".to_string()),
                            modifier: None
                        }],
                        vec![Element {
                            atom: Atom::Nonterminal("b".to_string()),
                            modifier: None
                        }]
                    ]),
                    modifier: Some(Modifier::Optional)
                },
                Element {
                    atom: Atom::Terminal("c".to_string()),
                    modifier: Some(Modifier::ZeroOrMore)
                }
            ]]
        );
    }

    #[test]
    fn test_parse_ebnf() {
        let input = "<foo> ::= <bar> | \"baz\"";
        let diagram = parse_ebnf(input).unwrap();
        // Basic check: diagram has nodes
        assert!(!diagram.to_string().is_empty());
        // Could check for specific SVG elements, but for now, ensure no panic
    }

    #[test]
    fn test_parse_semver_ebnf() {
        let content = std::fs::read_to_string("grammars/semver2-0.ebnf").unwrap();
        let result = parse_ebnf(&content);
        // The file contains quoted terminals, so parsing should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_unquoted_nonterminal() {
        let input = "<syntax>         ::= rule>+";
        let diagram = parse_ebnf(input);
        assert!(diagram.is_err())
    }

    #[test]
    fn test_reject_unquoted_terminal() {
        let input = "<syntax>         ::= rule>+";
        let diagram = parse_ebnf(input);
        assert!(diagram.is_err())
    }

    #[test]
    fn test_accept_quoted_terminals() {
        let mut input = "<rule> ::= \"terminal\" 'another'";
        let result = parse_rule(&mut input).unwrap();
        assert_eq!(result.name, "rule");
        assert_eq!(
            result.alternatives,
            vec![vec![
                Element {
                    atom: Atom::Terminal("terminal".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("another".to_string()),
                    modifier: None
                }
            ]]
        );
    }

    #[test]
    fn test_reject_unquoted_in_sequence() {
        let input = "<rule> ::= unquoted \"quoted\"";
        assert!(parse_ebnf(input).is_err());
    }

    #[test]
    fn test_reject_unquoted_in_alternatives() {
        let input = "<rule> ::= \"quoted\" | unquoted";
        assert!(parse_ebnf(input).is_err());
    }

    #[test]
    fn test_reject_unquoted_in_group() {
        let input = "<rule> ::= (\"quoted\" unquoted)";
        assert!(parse_ebnf(input).is_err());
    }

    #[test]
    fn test_reject_nonterminal_without_angles() {
        let input = "rule ::= <valid>";
        assert!(parse_ebnf(input).is_err());
    }

    #[test]
    fn test_reject_mixed_invalid() {
        let input = "<valid> ::= <also_valid> invalid";
        assert!(parse_ebnf(input).is_err());
    }

    #[test]
    fn test_accept_mixed_quotes() {
        let mut input = "<rule> ::= \"double\" 'single' \"mixed\"";
        let result = parse_rule(&mut input).unwrap();
        assert_eq!(result.name, "rule");
        assert_eq!(
            result.alternatives,
            vec![vec![
                Element {
                    atom: Atom::Terminal("double".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("single".to_string()),
                    modifier: None
                },
                Element {
                    atom: Atom::Terminal("mixed".to_string()),
                    modifier: None
                }
            ]]
        );
    }

    #[test]
    fn test_terminal_mixed_quotes() {
        let mut input = "\"she said 'hello'\"";
        let result = terminal(&mut input).unwrap();
        assert_eq!(result.atom, Atom::Terminal("she said 'hello'".to_string()));
    }

    #[test]
    fn test_terminal_single_with_double() {
        let mut input = "'he said \"world\"'";
        let result = terminal(&mut input).unwrap();
        assert_eq!(result.atom, Atom::Terminal("he said \"world\"".to_string()));
    }
}
