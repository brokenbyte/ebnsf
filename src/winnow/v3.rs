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
BNF Grammar Parser v3

This parser implements the BNF grammar as specified in grammars/bnf.ebnf:

<syntax>         ::= <rule>+
<rule>           ::= <opt_whitespace> "<" <rule_name> ">" <opt_whitespace> "::=" <opt_whitespace> <group> <line_end>
<opt_whitespace> ::= " "*
<expression>     ::= <list> (<opt_whitespace> "|" <opt_whitespace> <expression>)?
<group>          ::= "(" <expression> ")" | <expression>
<line_end>       ::= <opt_whitespace> "\n"+
<list>           ::= <term> | <term> <opt_whitespace> <list>
<term>           ::= <literal> | "<" <rule_name> ">"
<literal>        ::= '"' <text> '"'
<text>           ::= <character>+
<character>      ::= <letter> | <digit>
<letter>         ::= "[A-Za-z]"
<digit>          ::= "[0-9]"
<rule_name>      ::= <letter> <rule_char>*
<rule_char>      ::= <letter> | <digit> | "_"
*/

// Data structures to represent the BNF grammar

#[derive(Debug, Clone)]
pub struct Syntax {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub opt_whitespace_before: String,
    pub rule_name: String,
    pub opt_whitespace_after_name: String,
    pub opt_whitespace_before_group: String,
    pub group: Group,
    pub line_end: String,
}

#[derive(Debug, Clone)]
pub enum Group {
    Parenthesized {
        opt_whitespace_before: String,
        expression: Expression,
        opt_whitespace_after: String,
    },
    Direct(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub list: List,
    pub alternatives: Vec<(String, String, String, List)>, // (opt_whitespace_before, "|", opt_whitespace_after, next_expression_list)
}

#[derive(Debug, Clone)]
pub enum List {
    Single(Term),
    Multiple {
        first: Term,
        rest: Vec<(String, Term)>, // (opt_whitespace, term)
    },
}

#[derive(Debug, Clone)]
pub enum Term {
    Literal(Literal),
    NonTerminal(String),
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub text: Text,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub characters: Vec<Character>,
}

#[derive(Debug, Clone)]
pub enum Character {
    Letter(char),
    Digit(char),
}

#[derive(Debug, Clone)]
pub enum EbnfModifier {
    Plus,     // one or more
    Star,     // zero or more
    Question, // zero or one
}

#[derive(Debug, Clone)]
pub struct EbnfTerm {
    pub term: Term,
    pub modifier: Option<EbnfModifier>,
}

// Basic character parsers

pub fn parse_letter<'a>(input: &'a mut &str) -> ModalResult<Character> {
    let c = one_of(('a'..='z', 'A'..='Z')).parse_next(input)?;
    Ok(Character::Letter(c))
}

pub fn parse_digit<'a>(input: &'a mut &str) -> ModalResult<Character> {
    let c = one_of('0'..='9').parse_next(input)?;
    Ok(Character::Digit(c))
}

pub fn parse_character<'a>(input: &'a mut &str) -> ModalResult<Character> {
    alt((parse_letter, parse_digit)).parse_next(input)
}

pub fn parse_rule_char<'a>(input: &'a mut &str) -> ModalResult<char> {
    alt((one_of(('a'..='z', 'A'..='Z', '0'..='9')), one_of('_'))).parse_next(input)
}

// Text and literal parsers

pub fn parse_text<'a>(input: &'a mut &str) -> ModalResult<Text> {
    let characters = repeat(
        1..,
        alt((
            one_of(('a'..='z', 'A'..='Z')).map(Character::Letter),
            one_of('0'..='9').map(Character::Digit),
            one_of(' ').map(Character::Letter), // Handle space
        )),
    )
    .parse_next(input)?;
    Ok(Text { characters })
}

pub fn parse_literal<'a>(input: &'a mut &str) -> ModalResult<Literal> {
    let text = delimited('"', parse_text, '"').parse_next(input)?;
    Ok(Literal { text })
}

// Rule name and nonterminal parsers

pub fn parse_rule_name<'a>(input: &'a mut &str) -> ModalResult<String> {
    let (first, rest) = (
        one_of(('a'..='z', 'A'..='Z')),
        take_while(0.., ('a'..='z', 'A'..='Z', '0'..='9', '_')),
    )
        .parse_next(input)?;

    let mut name = String::new();
    name.push(first);
    name.push_str(rest);

    Ok(name)
}

pub fn parse_nonterminal<'a>(input: &'a mut &str) -> ModalResult<Term> {
    let name = delimited('<', parse_rule_name, '>').parse_next(input)?;
    Ok(Term::NonTerminal(name))
}

// Whitespace and line ending parsers

pub fn parse_opt_whitespace<'a>(input: &'a mut &str) -> ModalResult<String> {
    take_while(0.., ' ')
        .parse_next(input)
        .map(|s| s.to_string())
}

pub fn parse_line_end<'a>(input: &'a mut &str) -> ModalResult<String> {
    let (opt_whitespace, newlines) = (
        parse_opt_whitespace,
        repeat(1.., one_of('\n')).map(|chars: Vec<char>| chars.into_iter().collect::<String>()),
    )
        .parse_next(input)?;

    Ok(format!("{}{}", opt_whitespace, newlines))
}

// Term parser

pub fn parse_term<'a>(input: &'a mut &str) -> ModalResult<Term> {
    alt((parse_literal.map(Term::Literal), parse_nonterminal)).parse_next(input)
}

// List parser (handles left recursion)

pub fn parse_list<'a>(input: &'a mut &str) -> ModalResult<List> {
    let first_term = parse_term.parse_next(input)?;

    // Use winnow's separated to parse multiple terms separated by whitespace
    let result: Result<Vec<_>, _> = separated(0.., parse_term, space1).parse_next(input);

    match result {
        Ok(additional_terms) => {
            if additional_terms.is_empty() {
                Ok(List::Single(first_term))
            } else {
                let rest = additional_terms
                    .into_iter()
                    .map(|term| (" ".to_string(), term))
                    .collect();
                Ok(List::Multiple {
                    first: first_term,
                    rest,
                })
            }
        }
        Err(_) => {
            // If separated fails, just return single term
            Ok(List::Single(first_term))
        }
    }
}

// Expression parser

pub fn parse_expression<'a>(input: &'a mut &str) -> ModalResult<Expression> {
    let list = parse_list.parse_next(input)?;

    let mut alternatives = Vec::new();
    let mut remaining = *input;

    // Try to parse alternatives
    while let Ok((opt_ws_before, opt_ws_after, next_list)) = (
        parse_opt_whitespace,
        preceded(opt(one_of('|')), parse_opt_whitespace),
        parse_list,
    )
        .parse_next(&mut remaining)
    {
        alternatives.push((opt_ws_before, "|".to_string(), opt_ws_after, next_list));
        *input = remaining;
    }

    Ok(Expression { list, alternatives })
}

// Group parser

pub fn parse_group<'a>(input: &'a mut &str) -> ModalResult<Group> {
    // Try parenthesized group first
    if let Ok((opt_ws_before, expression, opt_ws_after)) = (
        parse_opt_whitespace,
        delimited('(', parse_expression, ')'),
        parse_opt_whitespace,
    )
        .parse_next(input)
    {
        return Ok(Group::Parenthesized {
            opt_whitespace_before: opt_ws_before,
            expression,
            opt_whitespace_after: opt_ws_after,
        });
    }

    // Fall back to direct expression with possible EBNF operators
    let list = parse_list_with_operators.parse_next(input)?;
    let expression = Expression {
        list,
        alternatives: Vec::new(),
    };
    Ok(Group::Direct(expression))
}

// Parse EBNF modifier
pub fn parse_ebnf_modifier<'a>(input: &'a mut &str) -> ModalResult<Option<EbnfModifier>> {
    opt(alt((
        '+'.map(|_| EbnfModifier::Plus),
        '*'.map(|_| EbnfModifier::Star),
        '?'.map(|_| EbnfModifier::Question),
    )))
    .parse_next(input)
}

// Parse EBNF term with optional modifier
pub fn parse_ebnf_term<'a>(input: &'a mut &str) -> ModalResult<EbnfTerm> {
    let (term, modifier) = (parse_term, parse_ebnf_modifier).parse_next(input)?;
    Ok(EbnfTerm { term, modifier })
}

// Simplified list parser that handles EBNF operators
pub fn parse_list_with_operators<'a>(input: &'a mut &str) -> ModalResult<List> {
    // Try to parse EBNF term with modifier first
    if let Ok(ebnf_term) = parse_ebnf_term.parse_next(input) {
        // For now, just treat as single term - in full implementation would handle modifiers
        return Ok(List::Single(ebnf_term.term));
    }

    // Fall back to regular list parsing
    parse_list.parse_next(input)
}

// Rule parser

pub fn parse_rule<'a>(input: &'a mut &str) -> ModalResult<Rule> {
    let opt_whitespace_before = parse_opt_whitespace.parse_next(input)?;
    let rule_name = delimited('<', parse_rule_name, '>').parse_next(input)?;
    let opt_whitespace_after_name = parse_opt_whitespace.parse_next(input)?;

    // Parse ::=
    one_of(':').parse_next(input)?; // :
    one_of(':').parse_next(input)?; // :
    one_of('=').parse_next(input)?; // =

    let opt_whitespace_before_group = parse_opt_whitespace.parse_next(input)?;
    let group = parse_group.parse_next(input)?;
    let line_end = parse_line_end.parse_next(input)?;

    Ok(Rule {
        opt_whitespace_before,
        rule_name,
        opt_whitespace_after_name,
        opt_whitespace_before_group,
        group,
        line_end,
    })
}

// Syntax parser

pub fn parse_syntax<'a>(input: &'a mut &str) -> ModalResult<Syntax> {
    let rules = repeat(1.., parse_rule).parse_next(input)?;
    Ok(Syntax { rules })
}

// Public API functions

pub fn parse_bnf_grammar(input: &str) -> Result<Syntax, String> {
    let mut input = input;
    parse_syntax(&mut input).map_err(|e| format!("Parse error: {:?}", e))
}

// Display implementations

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Character::Letter(c) => write!(f, "{}", c),
            Character::Digit(c) => write!(f, "{}", c),
        }
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ch in &self.characters {
            write!(f, "{}", ch)?;
        }
        Ok(())
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Literal(lit) => write!(f, "{}", lit),
            Term::NonTerminal(name) => write!(f, "<{}>", name),
        }
    }
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Single(term) => write!(f, "{}", term),
            List::Multiple { first, rest } => {
                write!(f, "{}", first)?;
                for (ws, term) in rest {
                    write!(f, "{}{}", ws, term)?;
                }
                Ok(())
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.list)?;
        for (ws_before, pipe, ws_after, list) in &self.alternatives {
            write!(f, "{}{}{}{}", ws_before, pipe, ws_after, list)?;
        }
        Ok(())
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Group::Parenthesized {
                opt_whitespace_before,
                expression,
                opt_whitespace_after,
            } => {
                write!(
                    f,
                    "{}({}){}",
                    opt_whitespace_before, expression, opt_whitespace_after
                )
            }
            Group::Direct(expression) => write!(f, "{}", expression),
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}>{} ::= {}{}{}",
            self.opt_whitespace_before,
            self.rule_name,
            self.opt_whitespace_after_name,
            self.opt_whitespace_before_group,
            self.group,
            self.line_end
        )
    }
}

impl Display for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rule in &self.rules {
            write!(f, "{}", rule)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_letter() {
        let mut input = "a";
        let result = parse_letter(&mut input).unwrap();
        assert!(matches!(result, Character::Letter('a')));
    }

    #[test]
    fn test_parse_digit() {
        let mut input = "5";
        let result = parse_digit(&mut input).unwrap();
        assert!(matches!(result, Character::Digit('5')));
    }

    #[test]
    fn test_parse_rule_name() {
        let mut input = "test_name";
        let result = parse_rule_name(&mut input).unwrap();
        assert_eq!(result, "test_name");
    }

    #[test]
    fn test_parse_literal() {
        let mut input = "\"hello\"";
        let result = parse_literal(&mut input).unwrap();
        let text = result
            .text
            .characters
            .iter()
            .map(|c| match c {
                Character::Letter(l) => *l,
                Character::Digit(d) => *d,
            })
            .collect::<String>();
        assert_eq!(text, "hello");
    }

    #[test]
    fn test_parse_nonterminal() {
        let mut input = "<test_rule>";
        let result = parse_nonterminal(&mut input).unwrap();
        assert!(matches!(result, Term::NonTerminal(name) if name == "test_rule"));
    }

    #[test]
    fn test_parse_term() {
        let mut input = "\"literal\"";
        let result = parse_term(&mut input).unwrap();
        assert!(matches!(result, Term::Literal(_)));
    }

    #[test]
    fn test_parse_opt_whitespace() {
        let mut input = "   test";
        let result = parse_opt_whitespace(&mut input).unwrap();
        assert_eq!(result, "   ");

        let mut input2 = "test";
        let result2 = parse_opt_whitespace(&mut input2).unwrap();
        assert_eq!(result2, "");
    }

    #[test]
    fn test_parse_line_end() {
        let mut input = "\n\n";
        let result = parse_line_end(&mut input).unwrap();
        assert_eq!(result, "\n\n");

        let mut input2 = "  \n\n\n";
        let result2 = parse_line_end(&mut input2).unwrap();
        assert_eq!(result2, "  \n\n\n");
    }

    #[test]
    fn test_parse_bnf_file() {
        // Test with first line without EBNF operators first
        let input = "<syntax>         ::= <rule>\n";
        let result = parse_bnf_grammar(input);

        match result {
            Ok(syntax) => {
                println!(
                    "Successfully parsed simple rule with {} rules",
                    syntax.rules.len()
                );
                assert!(!syntax.rules.is_empty());
            }
            Err(e) => {
                panic!("Failed to parse simple rule: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_full_bnf_file() {
        let content =
            std::fs::read_to_string("/home/user/code/rust/ebnsf/grammars/bnf.ebnf").unwrap();
        let result = parse_bnf_grammar(&content);

        match result {
            Ok(syntax) => {
                println!(
                    "Successfully parsed BNF grammar with {} rules",
                    syntax.rules.len()
                );

                // Print first few rules to verify structure
                for (i, rule) in syntax.rules.iter().take(3).enumerate() {
                    println!("Rule {}: {}", i + 1, rule.rule_name);
                }

                // Verify we got some rules
                assert!(!syntax.rules.is_empty());

                // Check that syntax rule exists
                let syntax_rule = syntax.rules.iter().find(|r| r.rule_name == "syntax");
                assert!(syntax_rule.is_some(), "Should find 'syntax' rule");
            }
            Err(e) => {
                panic!("Failed to parse BNF file: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_simple_rule() {
        let input = r#"<test> ::= "hello"
"#;

        let result = parse_bnf_grammar(input);
        match result {
            Ok(syntax) => {
                assert_eq!(syntax.rules.len(), 1);
                let rule = &syntax.rules[0];
                assert_eq!(rule.rule_name, "test");
            }
            Err(e) => {
                panic!("Failed to parse simple rule: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_rule_with_alternatives() {
        let input = r#"<choice> ::= "a" | "b" | "c"
"#;

        let result = parse_bnf_grammar(input);
        match result {
            Ok(syntax) => {
                assert_eq!(syntax.rules.len(), 1);
                let rule = &syntax.rules[0];
                assert_eq!(rule.rule_name, "choice");
            }
            Err(e) => {
                panic!("Failed to parse rule with alternatives: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_rule_with_group() {
        let input = r#"<grouped> ::= ("a" "b") "c"
"#;

        let result = parse_bnf_grammar(input);
        match result {
            Ok(syntax) => {
                assert_eq!(syntax.rules.len(), 1);
                let rule = &syntax.rules[0];
                assert_eq!(rule.rule_name, "grouped");
            }
            Err(e) => {
                panic!("Failed to parse rule with group: {}", e);
            }
        }
    }
}
