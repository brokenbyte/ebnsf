use std::fmt::Display;
use std::str::FromStr;

use winnow::ascii::{
    alpha1, alphanumeric0, alphanumeric1, line_ending, multispace0, multispace1, newline, space0,
    space1,
};
use winnow::combinator::{
    alt, backtrack_err, delimited, dispatch, empty, fail, opt, peek, preceded, repeat, separated,
    terminated,
};
use winnow::error::InputError;
use winnow::prelude::*;
use winnow::token::{any, none_of, one_of, take_while};
use winnow::{error::ParserError, token::take_till};

/*

<rule> ::= "<" <rule_name> ">" space0 "::=" space0

<production> ::= ()

<terminals>    ::= <terminal> <space1> <terminal>*
<nonterminals> ::= <nonterminal> <space1> <nonterminal>*



*/

/*
Trim leading whitespace, then parse:
    (<terminal> | <nonterminal>)+
*/
pub fn wparse<'a>(input: &'a mut &str) -> ModalResult<Vec<Vec<String>>> {
    // Trim all leading whitespace
    {
        (multispace0).void().parse_next(input)?;
    }

    let x: Vec<Vec<String>> =
        separated(1.., list, delimited(multispace0, "|", multispace0)).parse_next(input)?;
    Ok(x)
}

// Explain how to write a function to parse one or more terminal or nonterminal, separated by spaces,
// using the Winnow rust library.
// Include support for grouping lists of terminals/nonterminals with parenthesis.
// Nonterminals are alphanumeric but must begin with a letter, and are wrapped in angle brackets.
// Terminals have the same rules but are wrapped in double quotes.
//
// Examples of valid sequences, one per line:
//      <foo> "bar" <baz>
//      <foo> <woo> <boo> "baz"
//      "honk" "bonk" "tonk" <wonk>
//      <foo> <bar> ("hello" <baz>) "world" <buz>
//

pub fn x<'a>(input: &'a mut &str) -> ModalResult<Vec<Vec<String>>> {
    // Trim all leading whitespace
    {
        (multispace0).void().parse_next(input)?;
    }

    let x: Vec<Vec<String>> =
        separated(1.., list, delimited(multispace0, "|", multispace0)).parse_next(input)?;
    Ok(x)
}

pub fn expression<'a>(input: &'a mut &str) -> ModalResult<Vec<Vec<String>>> {
    separated(1.., list, delimited(multispace0, "|", multispace0)).parse_next(input)
}

pub fn group<'a>(input: &'a mut &str) -> ModalResult<Vec<Vec<String>>> {
    alt((delimited("(", expression, ")"), expression)).parse_next(input)
}

// Space-separated list of terminal/nonterminal
pub fn list<'a>(input: &'a mut &str) -> ModalResult<Vec<String>> {
    separated(1.., alt((terminal, nonterminal)), space1).parse_next(input)
}

pub fn list2<'a>(input: &'a mut &str) -> ModalResult<String> {
    let x = separated::<_, _, Vec<String>, _, _, _, _>(1.., alt((terminal, nonterminal)), space1)
        .void()
        // .map(|x: Vec<String> | {
        // x
        // })
        .take()
        .parse_next(input)?;
    Ok(x.to_string())
}

// rules = { rule+ }
pub fn rules<'a>(input: &'a mut &str) -> ModalResult<EbnfGrammar> {
    let rules: Vec<_> = repeat(1.., delimited(multispace0, rule, multispace0)).parse_next(input)?;

    Ok(EbnfGrammar { rules })
}

// rule = { nonterminal ~ "::=" ~ sequence_list ~ NEWLINE*}
pub fn rule<'a>(input: &'a mut &str) -> ModalResult<Rule> {
    let (name, seqlist) = (
        terminated(nonterminal, (space0, "::=", space0)),
        separated(1.., old_sequence, (multispace0, '|', multispace0)),
    )
        .parse_next(input)?;

    let mut rule = Rule {
        name,
        choices: seqlist,
    };
    // rule.extend(seqlist.into_iter());

    Ok(rule)
}

// sequence_list = { sequence ~ ( "\n"* ~ "|" ~ "\n"* ~ sequence )* }
pub fn old_sequence_list<'a>(input: &'a mut &str) -> ModalResult<Vec<SequenceItem>> {
    println!("sequence_list");
    let (mut x, y) = (
        old_sequence,
        alt((
            preceded((multispace0, '|', multispace0), old_sequence_list),
            (space0, newline).map(|_| vec![]),
        )),
    )
        .parse_next(input)?;

    // let y = y.unwrap_or(vec![]);
    x.extend(y);

    Ok(x)
}

// sequence = { (term | group)+ }
pub fn old_sequence<'a>(input: &'a mut &str) -> ModalResult<Vec<SequenceItem>> {
    println!("sequence");
    let mut x: Vec<SequenceItem> =
        repeat(1.., terminated(alt((old_list, old_group)), space0)).parse_next(input)?;

    Ok(x)
}

// group = { "(" ~ NEWLINE* ~ (sequence_list) ~ NEWLINE* ~ ")" ~ opt_modifier}
pub fn old_group<'a>(input: &'a mut &str) -> ModalResult<SequenceItem> {
    println!("group");
    let (items, modifier) = (
        delimited(
            (multispace0, '(', multispace0),
            old_sequence_list,
            (multispace0, ')'),
        ),
        opt(modifier),
    )
        .parse_next(input)?;

    Ok(SequenceItem::Group(Group { items, modifier }))
}

// nonterminal = @{ lbrack ~ rule_name ~ rbrack }
pub fn nonterminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    delimited('<', rule_name, '>').parse_next(input)
}

// term = { (literal | nonterminal) ~ opt_modifier}
pub fn term<'a>(input: &'a mut &str) -> ModalResult<Term> {
    (
        alt((
            literal.map(|l| Token::Terminal(l)),
            nonterminal.map(|nt| Token::NonTerminal(nt)),
        )),
        opt(modifier),
    )
        .parse_next(input)
        .map(|(atom, modifier)| Term { atom, modifier })
}

pub fn terminal<'a>(input: &'a mut &str) -> ModalResult<String> {
    literal(input)
}

pub fn literal<'a>(input: &'a mut &str) -> ModalResult<String> {
    let lit = alt((
        delimited('"', take_till(1.., ('"', '\n')), '"'),
        delimited('\'', take_till(1.., ('\'', '\n')), '\''),
    ))
    .parse_next(input)?
    .to_string();

    Ok(lit)
}

/*
 * rule_name = { ASCII_ALPHA ~ (ASCII_ALPHA | ASCII_DIGIT | "_" | " " | "-")* }
 */
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

pub fn old_list<'a>(input: &'a mut &str) -> ModalResult<SequenceItem> {
    println!("list");
    let x: Vec<_> = repeat(1.., terminated(term, multispace0)).parse_next(input)?;
    let y = x.into_iter().map(|t| SequenceItem::Term(t)).collect();

    let z = SequenceItem::Group(Group {
        items: y,
        modifier: None,
    });

    Ok(z)
}

pub fn modifier<'a>(input: &'a mut &str) -> ModalResult<Modifier> {
    alt((
        '?'.map(|c| Modifier::QMark),
        '+'.map(|c| Modifier::Plus),
        '*'.map(|c| Modifier::Star),
    ))
    .parse_next(input)
}

#[derive(Debug)]
pub struct Sequence {
    nodes: Vec<Node>,
}

#[derive(Debug)]
pub enum Node {
    Term(Term),
    Group(Group),
}

#[derive(Debug)]
pub enum Token {
    Terminal(String),
    NonTerminal(String),
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Terminal(s) => write!(f, "\"{s}\""),
            Token::NonTerminal(s) => write!(f, "<{s}>"),
        }
    }
}

#[derive(Debug)]
pub enum Modifier {
    Star,
    Plus,
    QMark,
}
impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Star => write!(f, "*"),
            Self::Plus => write!(f, "+"),
            Self::QMark => write!(f, "?"),
        }
    }
}

#[derive(Debug)]
pub struct EbnfGrammar {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub choices: Vec<Vec<SequenceItem>>,
}
impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}> ::= ", self.name)?;
        for choice in &self.choices {
            for i in choice {
                write!(f, "{i}")?;
            }
            write!(f, "|")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum SequenceItem {
    Term(Term),
    Group(Group),
}
impl Display for SequenceItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Term(t) => {
                write!(f, "{t}")
            }
            Self::Group(g) => {
                write!(f, "{g}")
            }
        }
    }
}

#[derive(Debug)]
pub struct Term {
    pub atom: Token,
    pub modifier: Option<Modifier>,
}
impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.atom)?;
        if let Some(m) = &self.modifier {
            write!(f, "{}", m)
        } else {
            write!(f, "_ ")
        }
    }
}

#[derive(Debug)]
pub struct Group {
    pub items: Vec<SequenceItem>,
    pub modifier: Option<Modifier>,
}
impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for i in &self.items {
            write!(f, "{i}")?;
        }
        write!(f, ")")?;
        if let Some(m) = &self.modifier {
            write!(f, "{}", m)
        } else {
            write!(f, "_ ")
        }
    }
}
